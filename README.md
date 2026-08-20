# ChatGPT

A clean-room, **Ubuntu 26.04 LTS / GNOME / Wayland** workstation for giving OpenAI models controlled, inspectable access to local computer capabilities.

> **Independent community project.** Not affiliated with or endorsed by OpenAI. ChatGPT is a trademark of OpenAI.

## Prime invariant

```text
CAPABILITY != AUTHORITY
```

The model proposes. The local Rust runtime decides what authority exists. The human can refuse or revoke it.

## Deliberate scope

- **OpenAI-only model/provider boundary**;
- official OpenAI API origin fixed to `https://api.openai.com`;
- Ubuntu 26.04 LTS + GNOME + Wayland first;
- XDG Desktop Portal + PipeWire for desktop observation;
- one authority-bearing process per Ubuntu user session;
- OBS through its loopback WebSocket API, not GUI clicking;
- input injection and persistent autonomous control remain later gated phases.

There is no generic provider registry, arbitrary model `base_url`, Azure OpenAI adapter, Anthropic/Gemini/xAI adapter, OpenRouter adapter, Ollama/LM Studio adapter, or OpenAI-compatible third-party endpoint mode.

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
                           |                        +-> XDG Screenshot
                           |                        +-> XDG ScreenCast -> PipeWire
                           |                        +-> OBS loopback websocket
                           |                        +-> bounded process executor
                           |
                           +-> secret broker
                           +-> deterministic receipts
```

## Single authority instance

Effectful runtime construction acquires:

```text
/run/user/<uid>/qsol-chatgpt-authority.lock
```

A second authority-bearing runtime fails closed. This does not claim to identify stolen/resold credentials from their bytes; it prevents this workstation from becoming a convenient multi-instance credential-sharing host.

## Ubuntu observation

### One-shot

`screen.capture` uses `org.freedesktop.portal.Screenshot`, validates a bounded local PNG, records SHA-256/dimensions in receipt v4, and never serializes raw image bytes or the portal path.

### Sustained

`screen.observe` uses **XDG ScreenCast + PipeWire**:

```text
screen.observe
  max_frames + max_duration_ms
        |
        v
 exact human approval
        |
        v
 XDG ScreenCast portal
        |
 user chooses monitor/window
        |
        v
 PipeWire mapped buffers
        |
 in-place frame hashing
        |
        v
 receipt/5
```

The action must bind:

- `max_frames`: 1..=300
- `max_duration_ms`: 500..=30000

The ScreenCast broker selects one source, uses `PersistMode::DoNot`, retains no restore token, and records no raw pixels, PipeWire FD/node ID, source title, or portal object path. Frame payload is hashed in place into an order-sensitive chain.

**PR #4 does not forward raw frames to OpenAI.** That remains gated behind redaction, Ubuntu Secret Service credentials, bounded official OpenAI image requests, and explicit `api.openai.com` egress.

See `docs/SCREENCAST_PIPEWIRE.md`.

## OBS control

OBS remains loopback-only and authority-gated. Supported operations include scene/status inspection, scene switching, recording start/stop, and stream stop. `obs.stream.start` remains explicitly denied until a stronger public-broadcast approval class exists.

## OpenAI credentials

Live OpenAI credential use is still gated. Future credentials must:

- use opaque `cred:openai.*` handles;
- resolve through Ubuntu Secret Service / OS keyring;
- never be ChatGPT web cookies/session tokens;
- never arrive through argv, ambient environment variables, receipts, logs, or repository files;
- never be redirected to a caller-configurable provider endpoint.

## Build

Rust **1.87+** is required. Ubuntu builds with PipeWire support also need:

```bash
sudo apt install libpipewire-0.3-dev
```

Then:

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

One-shot screenshot:

```bash
cargo run -- run \
  '{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.capture"}' \
  --execute
```

Bounded sustained observation:

```bash
cargo run -- run \
  '{"schema_version":"qsol-chatgpt-proposal/1","kind":"screen.observe","args":{"max_frames":60,"max_duration_ms":5000}}' \
  --approve --execute
```

The desktop may present normal portal permission/source-selection UI. The project does not bypass it.

## Repository map

```text
src/contracts.rs       normalized authority contracts
src/instance.rs        single authority-instance guard
src/openai.rs          OpenAI-only provider configuration boundary
src/desktop.rs         one-shot XDG Screenshot broker
src/screencast.rs      bounded XDG ScreenCast + PipeWire observer
src/obs/               bounded OBS loopback broker
src/policy.rs          default-deny authority decisions
src/runtime.rs         approval + execution lifecycle
src/receipts.rs        v2 process, v3 OBS, v4 screenshot, v5 ScreenCast receipts
src/secrets.rs         zeroizing secret primitives
src/tui.rs             human authority console
schemas/               language-neutral machine contracts
docs/                  security/architecture/platform decisions
```

## Current limits

This is not yet a production autonomous computer-use system. In particular:

- no keyboard/mouse injection;
- no persistent autonomous observe/act loop;
- no live OpenAI API credential broker;
- no raw screenshot/ScreenCast forwarding to OpenAI;
- no microphone/voice broker yet;
- no stable GNOME/Wayland global window enumeration or active-window implementation;
- no comprehensive shell network namespace containment yet.

See `ROADMAP.md` for the authority gates and planned voice + OBS co-host stages.

## License

Apache-2.0. See `LICENSE`.
