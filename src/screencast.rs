use std::cell::RefCell;
use std::io::Cursor;
use std::os::fd::OwnedFd;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use ashpd::desktop::{
    PersistMode,
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType},
};
use pipewire as pw;
use pw::{properties::properties, spa};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::runtime::Builder;

use crate::contracts::Action;
use crate::receipts::ScreencastEvidence;

const MIN_OBSERVE_DURATION_MS: u64 = 500;
const MAX_OBSERVE_DURATION_MS: u64 = 30_000;
const MIN_OBSERVE_FRAMES: u32 = 1;
const MAX_OBSERVE_FRAMES: u32 = 300;
const MAX_FRAME_PAYLOAD_BYTES: u64 = 64 * 1024 * 1024;
const MAX_VIDEO_DIMENSION: u32 = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObservationBounds {
    max_frames: u32,
    max_duration_ms: u64,
}

impl ObservationBounds {
    fn from_action(action: &Action) -> Result<Self, ScreencastError> {
        if action.kind() != "screen.observe" || !action.credential_handles().is_empty() {
            return Err(ScreencastError::InvalidAction);
        }
        if action.args().len() != 2 {
            return Err(ScreencastError::InvalidAction);
        }
        let max_frames = action
            .args()
            .get("max_frames")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| (MIN_OBSERVE_FRAMES..=MAX_OBSERVE_FRAMES).contains(value))
            .ok_or(ScreencastError::InvalidAction)?;
        let max_duration_ms = action
            .args()
            .get("max_duration_ms")
            .and_then(Value::as_u64)
            .filter(|value| {
                (MIN_OBSERVE_DURATION_MS..=MAX_OBSERVE_DURATION_MS).contains(value)
            })
            .ok_or(ScreencastError::InvalidAction)?;

        Ok(Self {
            max_frames,
            max_duration_ms,
        })
    }
}

#[derive(Debug, Clone)]
struct PortalStreamMetadata {
    node_id: u32,
    source_kind: &'static str,
    position_x: Option<i32>,
    position_y: Option<i32>,
    portal_width: Option<u32>,
    portal_height: Option<u32>,
}

struct PortalGrant {
    metadata: PortalStreamMetadata,
    fd: OwnedFd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NegotiatedFormat {
    width: u32,
    height: u32,
    framerate_num: u32,
    framerate_denom: u32,
}

struct ObserverState {
    started: Instant,
    deadline: Instant,
    ended: Option<Instant>,
    max_frames: u32,
    frames_observed: u32,
    payload_bytes_hashed: u64,
    frame_chain: Sha256,
    format: Option<NegotiatedFormat>,
    error: Option<ScreencastError>,
}

impl ObserverState {
    fn new(bounds: ObservationBounds) -> Result<Self, ScreencastError> {
        let started = Instant::now();
        let deadline = started
            .checked_add(Duration::from_millis(bounds.max_duration_ms))
            .ok_or(ScreencastError::DeadlineExceeded)?;
        Ok(Self {
            started,
            deadline,
            ended: None,
            max_frames: bounds.max_frames,
            frames_observed: 0,
            payload_bytes_hashed: 0,
            frame_chain: Sha256::new(),
            format: None,
            error: None,
        })
    }

    fn set_format(&mut self, format: NegotiatedFormat) -> Result<(), ScreencastError> {
        match self.format {
            None => {
                self.format = Some(format);
                Ok(())
            }
            Some(existing) if existing == format => Ok(()),
            Some(_) => Err(ScreencastError::FormatChanged),
        }
    }

    fn remaining(&self) -> Result<Duration, ScreencastError> {
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            Err(ScreencastError::DeadlineExceeded)
        } else {
            Ok(remaining)
        }
    }

    fn deadline_reached(&mut self) -> bool {
        if Instant::now() >= self.deadline {
            self.ended = Some(self.deadline);
            true
        } else {
            false
        }
    }

    fn mark_deadline(&mut self) {
        self.ended = Some(self.deadline);
    }

    fn record_frame_digest(&mut self, digest: &[u8], payload_bytes: u64) -> bool {
        let next_index = u64::from(self.frames_observed) + 1;
        let Some(total) = self.payload_bytes_hashed.checked_add(payload_bytes) else {
            self.error = Some(ScreencastError::FrameBoundsExceeded);
            return true;
        };
        self.payload_bytes_hashed = total;
        self.frame_chain.update(next_index.to_be_bytes());
        self.frame_chain.update(payload_bytes.to_be_bytes());
        self.frame_chain.update(digest);
        self.frames_observed += 1;
        if self.frames_observed >= self.max_frames {
            let now = Instant::now();
            self.ended = Some(if now > self.deadline {
                self.deadline
            } else {
                now
            });
            true
        } else {
            false
        }
    }

    fn duration_ms(&self) -> Result<u64, ScreencastError> {
        let ended = self.ended.ok_or(ScreencastError::PipeWireFailed)?;
        u64::try_from(ended.duration_since(self.started).as_millis())
            .map_err(|_| ScreencastError::DeadlineExceeded)
    }
}

fn next_frame_payload_bytes(current: u64, plane_size: usize) -> Result<u64, ScreencastError> {
    let plane_size = u64::try_from(plane_size).map_err(|_| ScreencastError::FrameBoundsExceeded)?;
    current
        .checked_add(plane_size)
        .filter(|total| *total <= MAX_FRAME_PAYLOAD_BYTES)
        .ok_or(ScreencastError::FrameBoundsExceeded)
}

pub(crate) fn observe_live(action: &Action) -> Result<ScreencastEvidence, ScreencastError> {
    let bounds = ObservationBounds::from_action(action)?;
    let worker = thread::Builder::new()
        .name("qsol-xdg-screencast".to_owned())
        .spawn(move || observe_worker(bounds))
        .map_err(|_| ScreencastError::WorkerThreadFailed)?;
    worker
        .join()
        .map_err(|_| ScreencastError::WorkerThreadFailed)?
}

fn observe_worker(bounds: ObservationBounds) -> Result<ScreencastEvidence, ScreencastError> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .map_err(|_| ScreencastError::RuntimeUnavailable)?;

    let (proxy, session, grant) = runtime.block_on(async {
        let proxy = Screencast::new()
            .await
            .map_err(|_| ScreencastError::PortalFailed)?;
        let session = proxy
            .create_session(Default::default())
            .await
            .map_err(|_| ScreencastError::PortalFailed)?;

        proxy
            .select_sources(
                &session,
                SelectSourcesOptions::default()
                    .set_cursor_mode(CursorMode::Hidden)
                    .set_sources(SourceType::Monitor | SourceType::Window)
                    .set_multiple(false)
                    .set_restore_token(None)
                    .set_persist_mode(PersistMode::DoNot),
            )
            .await
            .map_err(|_| ScreencastError::PortalFailed)?;

        let response = proxy
            .start(&session, None, Default::default())
            .await
            .map_err(|_| ScreencastError::PortalFailed)?
            .response()
            .map_err(|_| ScreencastError::PortalDenied)?;

        if response.streams().len() != 1 {
            return Err(ScreencastError::UnexpectedStreamCount);
        }
        let stream = response
            .streams()
            .first()
            .cloned()
            .ok_or(ScreencastError::UnexpectedStreamCount)?;
        let fd = proxy
            .open_pipe_wire_remote(&session, Default::default())
            .await
            .map_err(|_| ScreencastError::PortalFailed)?;

        let size = stream.size().and_then(|(width, height)| {
            let width = u32::try_from(width).ok()?;
            let height = u32::try_from(height).ok()?;
            (width > 0 && height > 0).then_some((width, height))
        });
        let (position_x, position_y) = stream
            .position()
            .map_or((None, None), |(x, y)| (Some(x), Some(y)));
        let source_kind = match stream.source_type() {
            Some(SourceType::Monitor) => "monitor",
            Some(SourceType::Window) => "window",
            Some(SourceType::Virtual) => "virtual",
            None => "unknown",
        };
        let metadata = PortalStreamMetadata {
            node_id: stream.pipe_wire_node_id(),
            source_kind,
            position_x,
            position_y,
            portal_width: size.map(|value| value.0),
            portal_height: size.map(|value| value.1),
        };
        Ok::<_, ScreencastError>((proxy, session, PortalGrant { metadata, fd }))
    })?;

    let result = observe_pipewire(grant, bounds);
    drop(session);
    drop(proxy);
    drop(runtime);
    result
}

fn observe_pipewire(
    grant: PortalGrant,
    bounds: ObservationBounds,
) -> Result<ScreencastEvidence, ScreencastError> {
    pw::init();

    let mainloop = pw::main_loop::MainLoopRc::new(None)
        .map_err(|_| ScreencastError::PipeWireFailed)?;
    let context = pw::context::ContextRc::new(&mainloop, None)
        .map_err(|_| ScreencastError::PipeWireFailed)?;
    let core = context
        .connect_fd_rc(grant.fd, None)
        .map_err(|_| ScreencastError::PipeWireFailed)?;
    let stream = pw::stream::StreamBox::new(
        &core,
        "qsol-chatgpt-screen-observer",
        properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Screen",
        },
    )
    .map_err(|_| ScreencastError::PipeWireFailed)?;

    let state = Rc::new(RefCell::new(ObserverState::new(bounds)?));

    let format_state = Rc::clone(&state);
    let format_loop = mainloop.clone();
    let process_state = Rc::clone(&state);
    let process_loop = mainloop.clone();

    let _listener = stream
        .add_local_listener_with_user_data(())
        .param_changed(move |_, _, id, param| {
            let Some(param) = param else {
                return;
            };
            if id != spa::param::ParamType::Format.as_raw() {
                return;
            }
            let (media_type, media_subtype) = match spa::param::format_utils::parse_format(param) {
                Ok(value) => value,
                Err(_) => {
                    format_state.borrow_mut().error = Some(ScreencastError::FormatUnavailable);
                    format_loop.quit();
                    return;
                }
            };
            if media_type != spa::param::format::MediaType::Video
                || media_subtype != spa::param::format::MediaSubtype::Raw
            {
                format_state.borrow_mut().error = Some(ScreencastError::FormatUnavailable);
                format_loop.quit();
                return;
            }

            let mut info: spa::param::video::VideoInfoRaw = Default::default();
            if info.parse(param).is_err() {
                format_state.borrow_mut().error = Some(ScreencastError::FormatUnavailable);
                format_loop.quit();
                return;
            }
            let size = info.size();
            let framerate = info.framerate();
            if size.width == 0
                || size.height == 0
                || size.width > MAX_VIDEO_DIMENSION
                || size.height > MAX_VIDEO_DIMENSION
                || framerate.denom == 0
            {
                format_state.borrow_mut().error = Some(ScreencastError::FormatUnavailable);
                format_loop.quit();
                return;
            }
            let negotiated = NegotiatedFormat {
                width: size.width,
                height: size.height,
                framerate_num: framerate.num,
                framerate_denom: framerate.denom,
            };
            if let Err(error) = format_state.borrow_mut().set_format(negotiated) {
                format_state.borrow_mut().error = Some(error);
                format_loop.quit();
            }
        })
        .process(move |stream, _| {
            if process_state.borrow_mut().deadline_reached() {
                process_loop.quit();
                return;
            }

            let Some(mut buffer) = stream.dequeue_buffer() else {
                return;
            };
            let datas = buffer.datas_mut();
            if datas.is_empty() {
                return;
            }

            let mut frame_hasher = Sha256::new();
            let mut frame_bytes = 0u64;
            for (plane_index, data) in datas.iter_mut().enumerate() {
                if data
                    .chunk()
                    .flags()
                    .contains(spa::buffer::ChunkFlags::CORRUPTED)
                {
                    process_state.borrow_mut().error = Some(ScreencastError::CorruptedFrame);
                    process_loop.quit();
                    return;
                }

                let offset = match usize::try_from(data.chunk().offset()) {
                    Ok(value) => value,
                    Err(_) => {
                        process_state.borrow_mut().error = Some(ScreencastError::FrameBoundsExceeded);
                        process_loop.quit();
                        return;
                    }
                };
                let size = match usize::try_from(data.chunk().size()) {
                    Ok(value) => value,
                    Err(_) => {
                        process_state.borrow_mut().error = Some(ScreencastError::FrameBoundsExceeded);
                        process_loop.quit();
                        return;
                    }
                };
                if size == 0 {
                    continue;
                }

                let next_frame_bytes = match next_frame_payload_bytes(frame_bytes, size) {
                    Ok(value) => value,
                    Err(error) => {
                        process_state.borrow_mut().error = Some(error);
                        process_loop.quit();
                        return;
                    }
                };

                let Some(slice) = data.data() else {
                    process_state.borrow_mut().error = Some(ScreencastError::UnmappableFrame);
                    process_loop.quit();
                    return;
                };
                if slice.is_empty() || size > slice.len() {
                    process_state.borrow_mut().error = Some(ScreencastError::FrameBoundsExceeded);
                    process_loop.quit();
                    return;
                }

                let start = offset % slice.len();
                let first_len = size.min(slice.len() - start);
                let second_len = size - first_len;
                frame_hasher.update(u64::try_from(plane_index).unwrap_or(u64::MAX).to_be_bytes());
                frame_hasher.update(u64::try_from(size).unwrap_or(u64::MAX).to_be_bytes());
                frame_hasher.update(&slice[start..start + first_len]);
                if second_len > 0 {
                    frame_hasher.update(&slice[..second_len]);
                }
                frame_bytes = next_frame_bytes;
            }
            if frame_bytes == 0 {
                return;
            }

            let digest = frame_hasher.finalize();
            if process_state
                .borrow_mut()
                .record_frame_digest(digest.as_ref(), frame_bytes)
            {
                process_loop.quit();
            }
        })
        .register()
        .map_err(|_| ScreencastError::PipeWireFailed)?;

    let obj = spa::pod::object!(
        spa::utils::SpaTypes::ObjectParamFormat,
        spa::param::ParamType::EnumFormat,
        spa::pod::property!(
            spa::param::format::FormatProperties::MediaType,
            Id,
            spa::param::format::MediaType::Video
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::MediaSubtype,
            Id,
            spa::param::format::MediaSubtype::Raw
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            spa::param::video::VideoFormat::RGBx,
            spa::param::video::VideoFormat::RGBx,
            spa::param::video::VideoFormat::BGRx,
            spa::param::video::VideoFormat::RGBA,
            spa::param::video::VideoFormat::RGB,
            spa::param::video::VideoFormat::YUY2,
            spa::param::video::VideoFormat::I420,
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            spa::utils::Rectangle {
                width: 1920,
                height: 1080
            },
            spa::utils::Rectangle {
                width: 1,
                height: 1
            },
            spa::utils::Rectangle {
                width: MAX_VIDEO_DIMENSION,
                height: MAX_VIDEO_DIMENSION
            }
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            spa::utils::Fraction { num: 30, denom: 1 },
            spa::utils::Fraction { num: 1, denom: 1 },
            spa::utils::Fraction { num: 120, denom: 1 }
        ),
    );
    let serialized = spa::pod::serialize::PodSerializer::serialize(
        Cursor::new(Vec::new()),
        &spa::pod::Value::Object(obj),
    )
    .map_err(|_| ScreencastError::PipeWireFailed)?;
    let values = serialized.0.into_inner();
    let pod = spa::pod::Pod::from_bytes(&values).map_err(|_| ScreencastError::PipeWireFailed)?;
    let mut params = [pod];

    stream
        .connect(
            spa::utils::Direction::Input,
            Some(grant.metadata.node_id),
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut params,
        )
        .map_err(|_| ScreencastError::PipeWireFailed)?;

    let remaining = state.borrow().remaining()?;
    let timer_state = Rc::clone(&state);
    let timer_loop = mainloop.clone();
    let timer = mainloop.loop_().add_timer(move |_| {
        timer_state.borrow_mut().mark_deadline();
        timer_loop.quit();
    });
    timer
        .update_timer(Some(remaining), None)
        .into_result()
        .map_err(|_| ScreencastError::PipeWireFailed)?;

    mainloop.run();

    let mut state = state.borrow_mut();
    if let Some(error) = state.error.take() {
        return Err(error);
    }
    if state.frames_observed == 0 {
        return Err(ScreencastError::NoFramesObserved);
    }
    let format = state.format.ok_or(ScreencastError::FormatUnavailable)?;
    let duration_ms = state.duration_ms()?;
    let frame_chain = std::mem::take(&mut state.frame_chain).finalize();

    Ok(ScreencastEvidence {
        backend: "xdg_screencast_pipewire".to_owned(),
        frame_chain_sha256: format!("{frame_chain:x}"),
        frames_observed: state.frames_observed,
        payload_bytes_hashed: state.payload_bytes_hashed,
        duration_ms,
        width: format.width,
        height: format.height,
        framerate_num: format.framerate_num,
        framerate_denom: format.framerate_denom,
        source_kind: grant.metadata.source_kind.to_owned(),
        position_x: grant.metadata.position_x,
        position_y: grant.metadata.position_y,
        portal_width: grant.metadata.portal_width,
        portal_height: grant.metadata.portal_height,
    })
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum ScreencastError {
    #[error("screen.observe action does not satisfy the bounded contract")]
    InvalidAction,
    #[error("ScreenCast worker thread could not be created or joined")]
    WorkerThreadFailed,
    #[error("Tokio runtime for XDG ScreenCast is unavailable")]
    RuntimeUnavailable,
    #[error("XDG ScreenCast portal request failed")]
    PortalFailed,
    #[error("XDG ScreenCast portal was denied or cancelled")]
    PortalDenied,
    #[error("XDG ScreenCast returned an unexpected stream count")]
    UnexpectedStreamCount,
    #[error("PipeWire setup or streaming failed")]
    PipeWireFailed,
    #[error("PipeWire did not provide a supported raw video format")]
    FormatUnavailable,
    #[error("PipeWire renegotiated the video format during one observation")]
    FormatChanged,
    #[error("PipeWire supplied a corrupted frame chunk")]
    CorruptedFrame,
    #[error("PipeWire frame could not be mapped")]
    UnmappableFrame,
    #[error("PipeWire frame exceeded a bounded payload contract")]
    FrameBoundsExceeded,
    #[error("bounded ScreenCast observation exceeded its approved deadline")]
    DeadlineExceeded,
    #[error("bounded ScreenCast session ended before any frame was observed")]
    NoFramesObserved,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use crate::contracts::{ProposedAction, PROPOSAL_SCHEMA_VERSION};

    use super::*;

    fn action(max_frames: u32, max_duration_ms: u64) -> Action {
        let mut args = BTreeMap::new();
        args.insert("max_frames".to_owned(), json!(max_frames));
        args.insert("max_duration_ms".to_owned(), json!(max_duration_ms));
        let proposal = ProposedAction {
            schema_version: PROPOSAL_SCHEMA_VERSION.to_owned(),
            kind: "screen.observe".to_owned(),
            args,
            requested_by: "agent".to_owned(),
            credential_handles: Vec::new(),
        };
        match proposal.normalize() {
            Ok(value) => value,
            Err(error) => panic!("fixture normalization failed: {error}"),
        }
    }

    fn format(width: u32, height: u32) -> NegotiatedFormat {
        NegotiatedFormat {
            width,
            height,
            framerate_num: 30,
            framerate_denom: 1,
        }
    }

    #[test]
    fn bounds_are_part_of_the_action_identity() {
        let first = action(60, 5_000);
        let second = action(61, 5_000);
        assert_ne!(first.id(), second.id());
    }

    #[test]
    fn bounds_fail_closed_outside_the_contract() {
        let too_many = action(MAX_OBSERVE_FRAMES + 1, 5_000);
        assert!(matches!(
            ObservationBounds::from_action(&too_many),
            Err(ScreencastError::InvalidAction)
        ));
        let too_long = action(60, MAX_OBSERVE_DURATION_MS + 1);
        assert!(matches!(
            ObservationBounds::from_action(&too_long),
            Err(ScreencastError::InvalidAction)
        ));
    }

    #[test]
    fn frame_chain_is_order_sensitive_and_bounded() {
        let bounds = ObservationBounds {
            max_frames: 2,
            max_duration_ms: 1_000,
        };
        let mut left = match ObserverState::new(bounds) {
            Ok(value) => value,
            Err(error) => panic!("observer state failed: {error}"),
        };
        let mut right = match ObserverState::new(bounds) {
            Ok(value) => value,
            Err(error) => panic!("observer state failed: {error}"),
        };
        let a = Sha256::digest(b"frame-a");
        let b = Sha256::digest(b"frame-b");
        assert!(!left.record_frame_digest(a.as_ref(), 7));
        assert!(left.record_frame_digest(b.as_ref(), 7));
        assert!(!right.record_frame_digest(b.as_ref(), 7));
        assert!(right.record_frame_digest(a.as_ref(), 7));
        let left_hash = std::mem::take(&mut left.frame_chain).finalize();
        let right_hash = std::mem::take(&mut right.frame_chain).finalize();
        assert_ne!(left_hash.as_ref(), right_hash.as_ref());
    }

    #[test]
    fn format_transition_fails_closed() {
        let bounds = ObservationBounds {
            max_frames: 2,
            max_duration_ms: 1_000,
        };
        let mut state = match ObserverState::new(bounds) {
            Ok(value) => value,
            Err(error) => panic!("observer state failed: {error}"),
        };
        assert!(state.set_format(format(1920, 1080)).is_ok());
        assert!(state.set_format(format(1920, 1080)).is_ok());
        assert!(matches!(
            state.set_format(format(1280, 720)),
            Err(ScreencastError::FormatChanged)
        ));
    }

    #[test]
    fn prospective_payload_bound_is_checked_before_hashing() {
        let almost_full = MAX_FRAME_PAYLOAD_BYTES - 1;
        assert_eq!(
            next_frame_payload_bytes(almost_full, 1).ok(),
            Some(MAX_FRAME_PAYLOAD_BYTES)
        );
        assert!(matches!(
            next_frame_payload_bytes(almost_full, 2),
            Err(ScreencastError::FrameBoundsExceeded)
        ));
    }

    #[test]
    fn deadline_and_receipt_clock_share_one_window() {
        let bounds = ObservationBounds {
            max_frames: 2,
            max_duration_ms: 1_000,
        };
        let mut state = match ObserverState::new(bounds) {
            Ok(value) => value,
            Err(error) => panic!("observer state failed: {error}"),
        };
        assert_eq!(
            state.deadline.duration_since(state.started),
            Duration::from_millis(bounds.max_duration_ms)
        );
        state.mark_deadline();
        assert_eq!(state.duration_ms().ok(), Some(bounds.max_duration_ms));
    }
}
