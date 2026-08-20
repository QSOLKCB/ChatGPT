# Architecture

## System shape

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
OpenAI proposal ---> Rust authority core ---> capability broker ---> Ubuntu 26.04 / OBS
                           |                        |
                           |                        +-> XDG Screenshot portal
                           |                        +-> OBS loopback WebSocket
                           |                        +-> bounded process executor
                           |
                           +-> single-instance guard
                           +-> secret broker
                           +-> deterministic receipts
```

The OpenAI model is a proposal source, not an authority source. The TUI is a human control plane, not a bypass around runtime policy.

## Trusted computing base

The intended trusted core remains small:
- `contracts.rs`
- `policy.rs`
- `runtime.rs`
- `receipts.rs`
- `secrets.rs`
- `instance.rs`
- capability-specific brokers/executors.

Provider/model output, webpages, clipboard contents, OCR, downloaded files, Python workers, and future browser automation are outside the trust boundary.

## OpenAI-only provider architecture

There is deliberately no generic provider registry.

```text
OpenAIConfig
  credential: cred:openai.*
  model:      reviewed identifier
  origin:     https://api.openai.com   (compile-time fixed)
```

No arbitrary provider host/base URL exists in the configuration type. See `docs/OPENAI_ONLY.md`.

## Single authority process

Effectful execution acquires an atomic local lock under validated `/run/user/<uid>`. Multiple TUI/client views may eventually talk to one authority daemon, but multiple independent executors/credential brokers are not permitted for the same user session.

## Ubuntu 26.04 desktop observation

The primary desktop stack is GNOME Wayland.

One-shot observation:

```text
screen.capture
   -> policy validates empty args / no credentials
   -> XDG Desktop Portal Screenshot
   -> local file URI validation
   -> bounded PNG ingestion
   -> SHA-256 + dimensions
   -> desktop receipt v4
   -> raw bytes zeroized
```

The screenshot URI/path and raw pixels are not audit fields.

Continuous future observation should use XDG ScreenCast + PipeWire. The project does not treat repeated screenshot polling as a substitute for a real bounded stream broker.

Window enumeration, active-window metadata, and cursor metadata remain unimplemented until a stable Ubuntu/GNOME/Wayland contract is selected. The project will not silently adopt GNOME-private APIs to make roadmap checkboxes turn green.

## Capability executors

Executors receive normalized actions only after policy/approval gates.

Current structured adapters:
- shell/process executor;
- OBS loopback broker;
- XDG Screenshot portal observation.

Future input injection must remain behind a separate broker and portal/compositor authority boundary.

## Receipts

Receipt families are versioned by capability semantics:
- v2 process evidence;
- v3 OBS evidence;
- v4 desktop screenshot evidence.

Adding a new receipt family must not loosen older schema branches.

## Credentials

Machine-visible contracts contain only opaque credential handles. Secret values live in non-serializable broker storage.

Future OpenAI credential storage/resolution uses Ubuntu Secret Service / OS keyring. ChatGPT browser cookies/session tokens and ambient `OPENAI_API_KEY`-style environment sourcing are outside the credential contract.

## Dependency direction

```text
OpenAI adapter / TUI / workers
          |
          v
   proposal interface
          |
          v
 contracts -> policy -> runtime -> capability brokers -> OS/OBS
                        |
                        +-> receipt/audit
                        +-> secret broker
                        +-> instance guard
```

No arrow points from policy upward into an OpenAI SDK or arbitrary provider interface.
