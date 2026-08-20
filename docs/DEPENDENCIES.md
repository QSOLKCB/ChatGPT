# Direct Dependency Review

Review date: 2026-08-20

This file records the review required by `AGENTS.md` before a direct dependency enters the project. Licenses below are the upstream SPDX/license terms published by the corresponding projects. All current direct licenses are compatible with this repository's Apache-2.0 distribution model.

| Crate | Declared range | Upstream license | Why it is present | Trust-boundary placement |
| --- | --- | --- | --- | --- |
| `ashpd` | `0.13`, `default-features = false`, `features = ["screenshot", "screencast", "tokio"]` | MIT | Ubuntu 26.04 GNOME Wayland observation uses standard XDG Screenshot and ScreenCast portals instead of GNOME-private or X11-specific capture. Hand-writing the DBus lifecycle would enlarge the trusted protocol surface. | Ubuntu desktop observation adapter only; no provider or policy authority decisions. |
| `base64` | `0.22` | MIT OR Apache-2.0 | obs-websocket 5.x authentication requires base64 encoding of SHA-256 challenge material. | OBS authentication adapter only. |
| `clap` | `4.5` | MIT OR Apache-2.0 | Provides typed CLI parsing rather than hand-written argument dispatch. | CLI adapter only. |
| `crossterm` | `0.28` | MIT | Provides terminal raw mode, input events, alternate screen, and cursor control. | TUI adapter only. |
| `pipewire` | `0.10` | MIT | The ScreenCast portal yields a PipeWire remote FD and node. The standard library has no PipeWire media-graph or mapped-buffer API. | Sustained desktop observation only. The crate provides safe Rust bindings over the system PipeWire library; this repository continues to forbid `unsafe` in its own code. |
| `ratatui` | `0.29` | MIT | Provides the human authority console without a heavyweight GUI framework. | TUI adapter only. |
| `serde` | `1` | MIT OR Apache-2.0 | Machine contracts require testable serialization/deserialization. | Contracts and receipt boundary. |
| `serde_json` | `1` | MIT OR Apache-2.0 | JSON is the published interchange and identity input format. | Contracts, CLI input, receipts. |
| `sha2` | `0.10` | MIT OR Apache-2.0 | Action/receipt identities and secret-free observation fingerprints require SHA-256. | Contracts, receipts, OBS and desktop observation fingerprints. |
| `thiserror` | `2` | MIT OR Apache-2.0 | Declarative error types avoid duplicated manual implementations. | Error representation only. |
| `tokio` | `1.51`, `default-features = false`, `features = ["rt-multi-thread"]` | MIT | `ashpd` portal requests are async. A dedicated portal worker bridges them to the synchronous authority core without nesting a runtime in caller threads. | XDG portal adapter runtime only. |
| `tungstenite` | `0.30`, `default-features = false`, `features = ["handshake"]` | MIT OR Apache-2.0 | Implements the bounded loopback WebSocket protocol used by OBS. | OBS loopback transport only. |
| `zeroize` | `1` | MIT OR Apache-2.0 | Rust memory safety does not erase secret or screenshot bytes. | Secret store, OBS auth intermediates, one-shot frames, and process-output cleanup. |

## Upstream sources checked

- `ashpd`: https://github.com/bilelmoussaoui/ashpd — current 0.13 line declares MIT and Rust 1.87.
- `pipewire`: https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/ and https://docs.rs/pipewire/0.10.0 — 0.10.0 declares MIT and provides safe Rust bindings over system PipeWire.
- `base64`: https://github.com/marshallpierce/rust-base64
- `clap`: https://github.com/clap-rs/clap
- `crossterm`: https://github.com/crossterm-rs/crossterm
- `ratatui`: https://github.com/ratatui/ratatui
- `serde`: https://github.com/serde-rs/serde
- `serde_json`: https://github.com/serde-rs/json
- `sha2`: https://github.com/RustCrypto/hashes
- `thiserror`: https://github.com/dtolnay/thiserror
- `tokio`: https://github.com/tokio-rs/tokio
- `tungstenite`: https://github.com/snapview/tungstenite-rs
- `zeroize`: https://github.com/RustCrypto/utils

## Rust MSRV

Rust **1.87** remains the application MSRV because `ashpd` 0.13 requires it. PR #4 does not raise the MSRV further.

## Ubuntu portal and PipeWire constraints

The sustained observation path is deliberately narrow:

1. `ashpd` enables only `screenshot`, `screencast`, and `tokio`; RemoteDesktop/InputCapture remain disabled.
2. ScreenCast source selection remains user-mediated by the portal.
3. Exactly one monitor or window is selected per bounded observation action.
4. `PersistMode::DoNot` is used and restore tokens are not retained.
5. The portal-provided PipeWire FD/node remain internal and never appear in receipts.
6. Mapped frame payload is hashed in place and is not copied into an application frame archive.
7. Each `screen.observe` action binds maximum frame count and maximum duration into its `action_id`.
8. Raw frame forwarding to OpenAI remains disabled until redaction, credential, and egress gates are implemented.
9. Ubuntu builds require the system development package `libpipewire-0.3-dev`; CI installs it explicitly.

## OBS network dependency constraints

`tungstenite` remains restricted to the OBS loopback path: internally constructed `127.0.0.1`, no arbitrary URL, bounded messages, absolute deadlines, and no exported raw WebSocket object.

## Release gate

Direct review is necessary but not sufficient. Before a release artifact is produced:

1. generate and commit a deterministic `Cargo.lock`;
2. enumerate the complete transitive dependency graph;
3. run automated license-policy and vulnerability/advisory checks;
4. retain required third-party notices;
5. fail release on an unreviewed or incompatible license.
