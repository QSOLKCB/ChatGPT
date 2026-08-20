# Direct Dependency Review

Review date: 2026-08-20

This file records the review required by `AGENTS.md` before a direct dependency enters the project. Licenses below are the upstream SPDX/license terms published by the corresponding projects. All current direct licenses are compatible with this repository's Apache-2.0 distribution model.

| Crate | Declared range | Upstream license | Why it is present | Trust-boundary placement |
| --- | --- | --- | --- | --- |
| `clap` | `4.5` | MIT OR Apache-2.0 | The standard library exposes raw argv but no typed subcommand/flag parser. `clap` keeps CLI parsing declarative and rejects malformed command shapes before dispatch. | CLI adapter only; not imported by policy/contracts. |
| `crossterm` | `0.28` | MIT | The standard library has no raw terminal mode, alternate-screen, keyboard-event, or cursor-control API. | TUI adapter only. |
| `ratatui` | `0.29` | MIT | The standard library has no terminal layout/widget renderer. It provides the human authority console without a heavyweight GUI framework. | TUI adapter only. |
| `serde` | `1` | MIT OR Apache-2.0 | Language-neutral machine contracts require explicit, testable serialization/deserialization. Hand-written JSON conversion would enlarge the security-sensitive parser surface. | Contracts and receipt boundary. |
| `serde_json` | `1` | MIT OR Apache-2.0 | JSON is the published interchange format and canonical identity input. The standard library has no JSON parser/serializer. | Contracts, CLI input, receipts. |
| `sha2` | `0.10` | MIT OR Apache-2.0 | Action and receipt identities require SHA-256; the standard library deliberately provides no cryptographic hash implementation. | Contracts and receipt identity only. |
| `thiserror` | `2` | MIT OR Apache-2.0 | The standard library provides `Error`/`Display` traits but no derive support. `thiserror` keeps error text declarative and avoids duplicated manual implementations across security boundary errors. | Error representation only; no authority decisions. |
| `zeroize` | `1` | MIT OR Apache-2.0 | Rust memory safety does not erase secret bytes. `zeroize` provides explicit best-effort memory clearing that the standard library does not guarantee. | Secret store and captured process-output cleanup. |

## Upstream sources checked

- `clap`: https://github.com/clap-rs/clap
- `crossterm`: https://github.com/crossterm-rs/crossterm
- `ratatui`: https://github.com/ratatui/ratatui
- `serde`: https://github.com/serde-rs/serde
- `serde_json`: https://github.com/serde-rs/json
- `sha2`: https://github.com/RustCrypto/hashes
- `thiserror`: https://github.com/dtolnay/thiserror
- `zeroize`: https://github.com/RustCrypto/utils

## Release gate

Direct review is necessary but not sufficient. Before a release artifact is produced:

1. generate and commit a deterministic `Cargo.lock` for the application;
2. enumerate the complete transitive dependency graph;
3. run automated license-policy and vulnerability/advisory checks;
4. retain required third-party notices in release artifacts;
5. fail the release on an unreviewed or incompatible license.
