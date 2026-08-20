use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Read, Take};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use ashpd::desktop::screenshot::Screenshot;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::runtime::Builder;
use zeroize::Zeroizing;

use crate::receipts::DesktopEvidence;

const MAX_SCREENSHOT_BYTES: usize = 64 * 1024 * 1024;
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

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
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| DesktopError::RuntimeUnavailable)?;

        let uri = runtime.block_on(async {
            let request = Screenshot::request()
                .interactive(false)
                .modal(false)
                .send()
                .await
                .map_err(|_| DesktopError::PortalFailed)?;
            let response = request.response().map_err(|_| DesktopError::PortalDenied)?;
            Ok::<String, DesktopError>(response.uri().as_str().to_owned())
        })?;

        let path = file_uri_to_path(&uri)?;
        let result = read_bounded_png(&path);
        if is_ephemeral_portal_path(&path) {
            let _ = fs::remove_file(&path);
        }
        result.map(|bytes| DesktopFrame { bytes })
    }
}

pub(crate) fn capture_live() -> Result<DesktopEvidence, DesktopError> {
    let mut backend = XdgPortalScreenshotBackend;
    capture_with_backend(&mut backend)
}

fn capture_with_backend(backend: &mut dyn DesktopBackend) -> Result<DesktopEvidence, DesktopError> {
    let frame = backend.capture()?;
    let (width, height) = png_dimensions(frame.bytes())?;
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
    #[error("XDG screenshot portal request failed")]
    PortalFailed,
    #[error("XDG screenshot portal denied or cancelled capture")]
    PortalDenied,
    #[error("portal returned a non-local or malformed file URI")]
    InvalidPortalUri,
    #[error("portal screenshot could not be opened")]
    ScreenshotOpenFailed,
    #[error("portal screenshot exceeded the 64 MiB bound")]
    ScreenshotTooLarge,
    #[error("portal screenshot is not a valid PNG")]
    InvalidPng,
}

fn read_bounded_png(path: &Path) -> Result<Zeroizing<Vec<u8>>, DesktopError> {
    let file = File::open(path).map_err(|_| DesktopError::ScreenshotOpenFailed)?;
    let mut reader: Take<File> = file.take((MAX_SCREENSHOT_BYTES + 1) as u64);
    let mut bytes = Zeroizing::new(Vec::new());
    reader
        .read_to_end(&mut bytes)
        .map_err(|_| DesktopError::ScreenshotOpenFailed)?;
    if bytes.len() > MAX_SCREENSHOT_BYTES {
        return Err(DesktopError::ScreenshotTooLarge);
    }
    if !bytes.starts_with(PNG_SIGNATURE) {
        return Err(DesktopError::InvalidPng);
    }
    Ok(bytes)
}

fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32), DesktopError> {
    if bytes.len() < 24 || !bytes.starts_with(PNG_SIGNATURE) || &bytes[12..16] != b"IHDR" {
        return Err(DesktopError::InvalidPng);
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width == 0 || height == 0 {
        return Err(DesktopError::InvalidPng);
    }
    Ok((width, height))
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
    if !path.is_absolute() {
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

fn is_ephemeral_portal_path(path: &Path) -> bool {
    path.starts_with("/run/user/") || path.starts_with("/tmp/") || path.starts_with("/var/tmp/")
}

#[cfg(test)]
mod tests {
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
        let mut bytes = Vec::from(PNG_SIGNATURE.as_slice());
        bytes.extend_from_slice(&[0, 0, 0, 13]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes
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
                assert_eq!(evidence.image_bytes, 29);
            }
            Err(error) => panic!("synthetic capture failed: {error}"),
        }
    }

    #[test]
    fn file_uri_parser_rejects_remote_authority() {
        assert!(matches!(
            file_uri_to_path("file://example.com/tmp/a.png"),
            Err(DesktopError::InvalidPortalUri)
        ));
    }

    #[test]
    fn file_uri_parser_decodes_percent_encoded_paths() {
        let path = file_uri_to_path("file:///tmp/a%20b.png");
        assert_eq!(path.ok(), Some(PathBuf::from("/tmp/a b.png")));
    }
}
