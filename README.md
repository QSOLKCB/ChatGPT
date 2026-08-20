# ChatGPT

A clean-room, **Ubuntu 26.04 LTS / GNOME / Wayland** workstation for giving OpenAI models controlled, inspectable access to local computer capabilities.

> **Independent community project.** Not affiliated with or endorsed by OpenAI. ChatGPT is a trademark of OpenAI.

## Prime invariant

```text
CAPABILITY != AUTHORITY
```

The model proposes. The local Rust runtime decides what authority exists. The human can refuse or revoke it.

## Deliberate scope

This project is intentionally narrow:

- **OpenAI-only model/provider boundary**;
- official OpenAI API origin fixed to `https://api.openai.com`;
- Ubuntu 26.04 LTS is the primary desktop target;
- GNOME Wayland uses XDG Desktop Portal rather than X11 scraping or private GNOME APIs;
- one authority-bearing process per Ubuntu user session;
- OBS is controlled through its loopback WebSocket API, not GUI clicking;
- input injection and persistent autonomous control are still gated behind later security phases.

There is no generic provider registry, arbitrary model `base_url`, Azure OpenAI adapter, Anthropic/Gemini/xAI adapter, OpenRouter adapter, Ollama/LM Studio adapter, or OpenAI-compatible third-party endpoint mode.

See `docs/OPENAI_ONLY.md`.

## Architecture

```text
                         HUMAN
                           |
                    +------+-------+
                    | Ratatui TUI  |
                    | control plane|
                    +------+-------+
                           |
                    grant / revoke
                           |
                           v
OpenAI proposal ---> Rust authority core ---> capability broker ---> Ubuntu / OBS
                           |                        |
                           |                        +-> XDG Portal screenshot
                           |                        +-> OBS loopback websocket
                           |                        +-> bounded process executor
                           |
                           +-> secret broker
                           +-> deterministic receipts
```

## Single authority instance

Effectful execution acquires an atomic per-user lock at:

```text
/run/user/<uid>/qsol-chatgpt-authority.lock
```

A second effectful instance fails closed. The runtime validates that the lock directory is the expected Ubuntu `XDG_RUNTIME_DIR`, owned by the current UID, and not accessible to group/other users.

This is not a claim that the application can identify a stolen or resold credential from its bytes. Instead it prevents this workstation from becoming a convenient multi-instance credential-sharing host.

A crash may leave a stale lock. That intentionally fails closed rather than guessing that a lock is safe to steal.

## Ubuntu 26.04 observation

`screen.capture` now has a real Ubuntu Wayland backend:

```text
screen.capture
    -> Rust policy
    -> authority-instance guard
    -> org.freedesktop.portal.Screenshot
    -> bounded local PNG ingestion
    -> SHA-256 + dimensions
    -> receipt/4
    -> raw screenshot buffer zeroized
```

The screenshot path, URI, and raw image bytes are never serialized into receipts.

The portal artifact is accepted only as a local `file://` URI. Capture is capped at 64 MiB and must be a PNG. Temporary artifacts under `/run/user`, `/tmp`, or `/var/tmp` are removed on a best-effort basis after ingestion.

For sustained future vision, the planned Ubuntu-native path is **XDG ScreenCast + PipeWire**, not repeated screenshot polling.

## OBS control

The OBS adapter remains loopback-only and authority-gated. Supported operations include scene/status inspection, scene switching, recording start/stop, and stream stop. `obs.stream.start` remains explicitly denied until a stronger broadcast approval class exists.

## OpenAI credentials

PR #3 defines the OpenAI-only provider configuration contract but does **not** yet add live OpenAI API credential use.

Future credentials must:

- use opaque `cred:openai.*` handles;
- resolve through an OS keyring/Secret Service broker;
- never be accepted as ChatGPT web cookies/session tokens;
- never arrive through command-line arguments, ambient environment variables, receipts, logs, or repository files;
- never be redirected to a caller-configurable provider endpoint.

## Build

Rust **1.87+** is required.

```bash
git clone https://github.com/QSOLKCB/ChatGPT.git
cd ChatGPT
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Launch the TUI:

```bash
cargo run -- tui
```

Simulate screenshot authority without touching the desktop:

```bash
cargo run -- run \
  '{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture"}'
```

Use the Ubuntu portal for a real one-shot screenshot:

```bash
cargo run -- run \
  '{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture"}' \
  --execute
```

The desktop may present its normal portal permission UI. The project does not bypass it.

## Repository map

```text
src/contracts.rs       normalized authority contracts
src/instance.rs        single authority-instance guard
src/openai.rs          OpenAI-only provider configuration boundary
src/desktop.rs         Ubuntu XDG Portal screenshot broker
src/obs/               bounded OBS loopback broker
src/policy.rs          default-deny authority decisions
src/runtime.rs         approval + execution lifecycle
src/receipts.rs        v2 process, v3 OBS, v4 desktop receipts
src/secrets.rs         zeroizing in-memory secret primitives
src/tui.rs             human authority console
schemas/               language-neutral machine contracts
docs/                  security/architecture/platform decisions
```

## Current limits

This is not yet a production autonomous computer-use system. In particular:

- no keyboard/mouse injection;
- no persistent observe/act loop;
- no live OpenAI API credential broker;
- no ScreenCast/PipeWire continuous frame stream yet;
- no stable GNOME/Wayland global window enumeration or active-window implementation;
- no comprehensive shell network namespace containment yet.

See `ROADMAP.md` for the gates before those capabilities are enabled.

## License

Apache-2.0. See `LICENSE`.
