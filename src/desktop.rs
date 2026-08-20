use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};
use std::thread;

use ashpd::desktop::screenshot::Screenshot;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::runtime::Builder;
use zeroize::{Zeroize, Zeroizing};

use crate::instance;
use crate::receipts::DesktopEvidence;

const MAX_SCREENSHOT_BYTES: usize = 64 * 1024 * 1024;
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const PNG_IHDR_LENGTH: usize = 13;

struct DesktopFrame {
    bytes: Zeroizing<Vec<u8>>,
}

impl DesktopFrame {
    fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl std::fmt::Debug for DesktopFrame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DesktopFrame")
            .field("bytes", &"[REDACTED]")
            .field("length", &self.bytes.len())
            .finish()
    }
}

trait DesktopBackend {
    fn capture(&mut self) -> Result<DesktopFrame, DesktopError>;
}

struct XdgPortalScreenshotBackend;

impl DesktopBackend for XdgPortalScreenshotBackend {
    fn capture(&mut self) -> Result<DesktopFrame, DesktopError> {
        let portal_thread = thread::Builder::new()
            .name("qsol-xdg-screenshot".to_owned())
            .spawn(|| {
                let runtime = Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|_| DesktopError::RuntimeUnavailable)?;

                runtime.block_on(async {
                    let request = Screenshot::request()
                        .interactive(false)
                        .modal(false)
                        .send()
                        .await
                        .map_err(|_| DesktopError::PortalFailed)?;
                    let response = request.response().map_err(|_| DesktopError::PortalDenied)?;
                    Ok::<String, DesktopError>(response.uri().as_str().to_owned())
                })
            })
            .map_err(|_| DesktopError::PortalThreadFailed)?;

        let uri = portal_thread
            .join()
            .map_err(|_| DesktopError::PortalThreadFailed)??;
        let path = file_uri_to_path(&uri)?;
        let result = read_bounded_png(&path);
        cleanup_ephemeral_portal_file(&path);
        result.map(|bytes| DesktopFrame { bytes })
    }
}

pub(crate) fn capture_live() -> Result<DesktopEvidence, DesktopError> {
    let mut backend = XdgPortalScreenshotBackend;
    capture_with_backend(&mut backend)
}

fn capture_with_backend(backend: &mut dyn DesktopBackend) -> Result<DesktopEvidence, DesktopError> {
    let frame = backend.capture()?;
    let (width, height) = validate_png(frame.bytes())?;
    Ok(DesktopEvidence {
        backend: "xdg_desktop_portal_screenshot".to_owned(),
        image_sha256: format!("{:x}", Sha256::digest(frame.bytes())),
        image_bytes: frame.bytes().len(),
        width,
        height,
        image_format: "png".to_owned(),
    })
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum DesktopError {
    #[error("Tokio runtime is unavailable")]
    RuntimeUnavailable,
    #[error("desktop portal worker thread is unavailable")]
    PortalThreadFailed,
    #[error("XDG screenshot portal request failed")]
    PortalFailed,
    #[error("XDG screenshot portal denied or cancelled capture")]
    PortalDenied,
    #[error("portal returned a non-local or malformed file URI")]
    InvalidPortalUri,
    #[error("portal screenshot could not be opened")]
    ScreenshotOpenFailed,
    #[error("portal screenshot exceeded the 64 MiB bound or changed while being read")]
    ScreenshotTooLarge,
    #[error("portal screenshot is not a complete valid PNG")]
    InvalidPng,
}

fn read_bounded_png(path: &Path) -> Result<Zeroizing<Vec<u8>>, DesktopError> {
    let mut file = File::open(path).map_err(|_| DesktopError::ScreenshotOpenFailed)?;
    let metadata = file
        .metadata()
        .map_err(|_| DesktopError::ScreenshotOpenFailed)?;
    let file_bytes = metadata.len();
    if file_bytes == 0 || file_bytes > MAX_SCREENSHOT_BYTES as u64 {
        return Err(DesktopError::ScreenshotTooLarge);
    }
    let length = usize::try_from(file_bytes).map_err(|_| DesktopError::ScreenshotTooLarge)?;

    let mut bytes = Zeroizing::new(vec![0u8; length]);
    file.read_exact(bytes.as_mut_slice())
        .map_err(|_| DesktopError::ScreenshotOpenFailed)?;

    let mut extra = [0u8; 1];
    let extra_result = file.read(&mut extra);
    let extra_count = match extra_result {
        Ok(value) => value,
        Err(_) => {
            extra.zeroize();
            return Err(DesktopError::ScreenshotOpenFailed);
        }
    };
    extra.zeroize();
    if extra_count != 0 {
        return Err(DesktopError::ScreenshotTooLarge);
    }

    validate_png(bytes.as_slice())?;
    Ok(bytes)
}

fn validate_png(bytes: &[u8]) -> Result<(u32, u32), DesktopError> {
    if bytes.len() < PNG_SIGNATURE.len() || !bytes.starts_with(PNG_SIGNATURE) {
        return Err(DesktopError::InvalidPng);
    }

    let mut offset = PNG_SIGNATURE.len();
    let mut dimensions = None;
    let mut seen_idat = false;

    while offset < bytes.len() {
        if bytes.len().saturating_sub(offset) < 12 {
            return Err(DesktopError::InvalidPng);
        }

        let length = read_u32(&bytes[offset..offset + 4])? as usize;
        let type_start = offset + 4;
        let data_start = offset + 8;
        let data_end = data_start
            .checked_add(length)
            .ok_or(DesktopError::InvalidPng)?;
        let crc_end = data_end.checked_add(4).ok_or(DesktopError::InvalidPng)?;
        if crc_end > bytes.len() {
            return Err(DesktopError::InvalidPng);
        }

        let chunk_type = &bytes[type_start..data_start];
        let data = &bytes[data_start..data_end];
        let expected_crc = read_u32(&bytes[data_end..crc_end])?;
        if png_crc32(chunk_type, data) != expected_crc {
            return Err(DesktopError::InvalidPng);
        }

        match chunk_type {
            b"IHDR" => {
                if offset != PNG_SIGNATURE.len()
                    || length != PNG_IHDR_LENGTH
                    || dimensions.is_some()
                {
                    return Err(DesktopError::InvalidPng);
                }
                let width = read_u32(&data[0..4])?;
                let height = read_u32(&data[4..8])?;
                if width == 0
                    || height == 0
                    || !valid_png_color_depth(data[8], data[9])
                    || data[10] != 0
                    || data[11] != 0
                    || data[12] > 1
                {
                    return Err(DesktopError::InvalidPng);
                }
                dimensions = Some((width, height));
            }
            b"PLTE" => {
                if dimensions.is_none() || seen_idat {
                    return Err(DesktopError::InvalidPng);
                }
            }
            b"IDAT" => {
                if dimensions.is_none() {
                    return Err(DesktopError::InvalidPng);
                }
                seen_idat = true;
            }
            b"IEND" => {
                if dimensions.is_none() || !seen_idat || length != 0 || crc_end != bytes.len() {
                    return Err(DesktopError::InvalidPng);
                }
                return dimensions.ok_or(DesktopError::InvalidPng);
            }
            _ => {
                if dimensions.is_none() || is_unknown_critical_chunk(chunk_type) {
                    return Err(DesktopError::InvalidPng);
                }
            }
        }

        offset = crc_end;
    }

    Err(DesktopError::InvalidPng)
}

fn read_u32(bytes: &[u8]) -> Result<u32, DesktopError> {
    let array: [u8; 4] = bytes.try_into().map_err(|_| DesktopError::InvalidPng)?;
    Ok(u32::from_be_bytes(array))
}

fn valid_png_color_depth(bit_depth: u8, color_type: u8) -> bool {
    match color_type {
        0 => matches!(bit_depth, 1 | 2 | 4 | 8 | 16),
        2 => matches!(bit_depth, 8 | 16),
        3 => matches!(bit_depth, 1 | 2 | 4 | 8),
        4 | 6 => matches!(bit_depth, 8 | 16),
        _ => false,
    }
}

fn is_unknown_critical_chunk(chunk_type: &[u8]) -> bool {
    chunk_type.len() != 4
        || (chunk_type[0].is_ascii_uppercase()
            && !matches!(chunk_type, b"IHDR" | b"PLTE" | b"IDAT" | b"IEND"))
}

fn png_crc32(chunk_type: &[u8], data: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in chunk_type.iter().chain(data.iter()) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn file_uri_to_path(uri: &str) -> Result<PathBuf, DesktopError> {
    let encoded = uri
        .strip_prefix("file://")
        .ok_or(DesktopError::InvalidPortalUri)?;
    if !encoded.starts_with('/') || encoded.contains('?') || encoded.contains('#') {
        return Err(DesktopError::InvalidPortalUri);
    }

    let decoded = percent_decode(encoded.as_bytes())?;
    if decoded.contains(&0) {
        return Err(DesktopError::InvalidPortalUri);
    }
    let path = PathBuf::from(OsString::from_vec(decoded));
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(DesktopError::InvalidPortalUri);
    }
    Ok(path)
}

fn percent_decode(input: &[u8]) -> Result<Vec<u8>, DesktopError> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'%' {
            if index + 2 >= input.len() {
                return Err(DesktopError::InvalidPortalUri);
            }
            let high = hex_value(input[index + 1]).ok_or(DesktopError::InvalidPortalUri)?;
            let low = hex_value(input[index + 2]).ok_or(DesktopError::InvalidPortalUri)?;
            output.push((high << 4) | low);
            index += 3;
        } else {
            output.push(input[index]);
            index += 1;
        }
    }
    Ok(output)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn cleanup_ephemeral_portal_file(path: &Path) {
    let Ok(original_metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if !original_metadata.is_file() || original_metadata.file_type().is_symlink() {
        return;
    }

    let Ok(canonical_path) = fs::canonicalize(path) else {
        return;
    };
    let mut roots = vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")];
    if let Ok(runtime_root) = instance::validated_runtime_directory() {
        roots.push(runtime_root);
    }

    for root in roots {
        let Ok(canonical_root) = fs::canonicalize(root) else {
            continue;
        };
        if canonical_path != canonical_root && canonical_path.starts_with(&canonical_root) {
            let _ = fs::remove_file(path);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::os::unix::fs::symlink;

    use super::*;

    struct FakeBackend {
        bytes: Vec<u8>,
    }

    impl DesktopBackend for FakeBackend {
        fn capture(&mut self) -> Result<DesktopFrame, DesktopError> {
            Ok(DesktopFrame {
                bytes: Zeroizing::new(self.bytes.clone()),
            })
        }
    }

    fn one_pixel_png() -> Vec<u8> {
        hex_bytes("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000b49444154789c6360000200000500017a5eab3f0000000049454e44ae426082")
    }

    fn hex_bytes(hex: &str) -> Vec<u8> {
        let mut output = Vec::with_capacity(hex.len() / 2);
        let bytes = hex.as_bytes();
        let mut index = 0;
        while index + 1 < bytes.len() {
            let high = hex_value(bytes[index]).unwrap_or(0);
            let low = hex_value(bytes[index + 1]).unwrap_or(0);
            output.push((high << 4) | low);
            index += 2;
        }
        output
    }

    #[test]
    fn synthetic_frame_produces_secret_free_evidence() {
        let mut backend = FakeBackend {
            bytes: one_pixel_png(),
        };
        let capture = capture_with_backend(&mut backend);
        match capture {
            Ok(evidence) => {
                assert_eq!(evidence.width, 1);
                assert_eq!(evidence.height, 1);
                assert_eq!(evidence.image_format, "png");
                assert_eq!(evidence.image_sha256.len(), 64);
                assert_eq!(evidence.image_bytes, 68);
            }
            Err(error) => panic!("synthetic capture failed: {error}"),
        }
    }

    #[test]
    fn truncated_png_is_rejected() {
        let mut bytes = one_pixel_png();
        bytes.truncate(40);
        assert!(matches!(validate_png(&bytes), Err(DesktopError::InvalidPng)));
    }

    #[test]
    fn png_with_corrupt_crc_is_rejected() {
        let mut bytes = one_pixel_png();
        bytes[29] ^= 1;
        assert!(matches!(validate_png(&bytes), Err(DesktopError::InvalidPng)));
    }

    #[test]
    fn file_uri_parser_rejects_remote_authority() {
        assert!(matches!(
            file_uri_to_path("file://example.com/tmp/a.png"),
            Err(DesktopError::InvalidPortalUri)
        ));
    }

    #[test]
    fn file_uri_parser_rejects_parent_traversal() {
        assert!(matches!(
            file_uri_to_path("file:///tmp/../home/user/a.png"),
            Err(DesktopError::InvalidPortalUri)
        ));
    }

    #[test]
    fn file_uri_parser_decodes_percent_encoded_paths() {
        let path = file_uri_to_path("file:///tmp/a%20b.png");
        assert_eq!(path.ok(), Some(PathBuf::from("/tmp/a b.png")));
    }

    #[test]
    fn cleanup_does_not_follow_symlink_outside_approved_roots() {
        let fixture_name = format!("qsol-desktop-cleanup-{}", std::process::id());
        let target = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(format!("{fixture_name}.target"));
        let link = env::temp_dir().join(format!("{fixture_name}.link"));
        let _ = fs::remove_file(&link);
        let _ = fs::remove_file(&target);
        if fs::write(&target, b"keep").is_err() || symlink(&target, &link).is_err() {
            let _ = fs::remove_file(&link);
            let _ = fs::remove_file(&target);
            panic!("failed to create cleanup fixture");
        }

        cleanup_ephemeral_portal_file(&link);
        assert!(target.exists());

        let _ = fs::remove_file(&link);
        let _ = fs::remove_file(&target);
    }

    #[test]
    fn bounded_reader_uses_verified_file_size_without_reallocation_growth() {
        let fixture = env::temp_dir().join(format!(
            "qsol-desktop-png-{}.png",
            std::process::id()
        ));
        let bytes = one_pixel_png();
        if fs::write(&fixture, &bytes).is_err() {
            panic!("failed to create PNG fixture");
        }
        let result = read_bounded_png(&fixture);
        let _ = fs::remove_file(&fixture);
        assert_eq!(result.ok().map(|value| value.len()), Some(bytes.len()));
    }
}
