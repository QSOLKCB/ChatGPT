# README4AI

## Identity

Project: `QSOLKCB/ChatGPT`
Purpose: Ubuntu 26.04 LTS / GNOME / Wayland OpenAI workstation, capability broker, and human authority console.
Trusted implementation language: Rust.
Human control plane: Ratatui TUI.
Model/provider boundary: **OpenAI only**.
Official API origin: `https://api.openai.com`.
License: Apache-2.0.

## Prime invariant

```text
CAPABILITY != AUTHORITY
```

OpenAI/model output proposes actions. Rust policy and human authority decide whether they may execute.

## Provider invariant

Allowed: official OpenAI API only, opaque `cred:openai.*` handles, reviewed OpenAI model identifiers.

Forbidden: generic provider registries, arbitrary model endpoints/base URLs, Azure OpenAI, Claude, Gemini, Grok, OpenRouter, Bedrock, Ollama/LM Studio, other local models, third-party OpenAI-compatible endpoints, or ChatGPT browser-session tokens.

## Primary platform

```text
Ubuntu 26.04 LTS
GNOME
Wayland
```

Use XDG Desktop Portal and PipeWire for the primary desktop path. Do not replace them with X11 scraping, GNOME-private APIs, `xdotool`, `wmctrl`, or shell helpers.

## Authority instance

`Runtime::effectful*` owns a single per-user authority lease at:

```text
/run/user/<uid>/qsol-chatgpt-authority.lock
```

Duplicate/stale lock => fail closed. Simulated/denied actions do not require effect authority.

## Observation capabilities

### `screen.capture`

- no args;
- no credentials;
- policy disposition `allow`;
- XDG Screenshot portal;
- complete bounded PNG validation;
- raw bytes are zeroizing/ephemeral;
- receipt v4 contains only hash/size/dimensions/backend/format.

### `screen.observe`

- args exactly `max_frames` + `max_duration_ms`;
- `max_frames`: 1..=300;
- `max_duration_ms`: 500..=30000;
- no credentials;
- policy disposition `approval_required`;
- approval binds to exact limits through `action_id`;
- XDG ScreenCast portal, one user-selected monitor/window;
- cursor hidden;
- `PersistMode::DoNot`;
- no restore token retention;
- portal PipeWire FD/node remain internal;
- mapped frame payload is hashed in place;
- no application-owned raw-frame archive;
- receipt v5 stores a frame-chain hash + bounded video/source geometry metadata;
- no raw pixels, source titles, node IDs, FDs, restore tokens, or portal paths in receipts.

PR #4 MUST NOT forward raw frames to OpenAI. Forwarding requires later redaction, Secret Service credential, bounded official image request, and `api.openai.com` egress gates.

## Receipt versions

- process: `qsol-chatgpt-receipt/2`
- OBS: `qsol-chatgpt-receipt/3`
- one-shot screenshot: `qsol-chatgpt-receipt/4`
- sustained ScreenCast: `qsol-chatgpt-receipt/5`

Earlier receipt semantics must remain stable when newer receipt kinds are added.

## OBS boundary

OBS transport is loopback-only and typed. `obs.stream.start` remains denied. No raw OBS request escape exists.

## Credential rules

Raw credentials never enter action JSON, receipts, logs, TUI state, CLI arguments, ambient env credential paths, or repository fixtures. Long-term OpenAI credential resolution must use Ubuntu Secret Service / OS keyring.

## Rust / dependency rules

- Rust MSRV: 1.87+.
- `unsafe_code = "forbid"` in this repository.
- `ashpd`: Screenshot + ScreenCast portal client.
- `pipewire`: safe Rust API over system PipeWire for sustained observation.
- Ubuntu build dependency: `libpipewire-0.3-dev`.
- tests must not require live credentials, portal interaction, PipeWire server, desktop injection, or privileged mutation.

## Future authority separation

Keep these independently granted/revoked:

```text
desktop observation
keyboard/mouse input
microphone observation
assistant voice output
OBS local recording
OBS public broadcast
```

Microphone permission never implies recording. Local recording never implies public broadcast.

## Source-of-truth order

1. executable Rust tests;
2. Rust contracts/policy/runtime;
3. JSON Schemas;
4. `AGENTS.md` security invariants;
5. architecture/platform docs;
6. README prose.

On disagreement, fail closed and repair the inconsistency.
