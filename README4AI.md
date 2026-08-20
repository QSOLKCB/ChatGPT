# README4AI

## Identity

Project: `QSOLKCB/ChatGPT`
Purpose: Ubuntu 26.04 LTS / GNOME / Wayland OpenAI workstation, capability broker, and human authority console.
Trusted implementation language: Rust.
Human control plane: Ratatui TUI.
Model/provider boundary: **OpenAI only**.
Official API origin: `https://api.openai.com`.
License: Apache-2.0.
Affiliation: independent community project; not affiliated with or endorsed by OpenAI.

## Prime invariant

```text
CAPABILITY != AUTHORITY
```

OpenAI/model output is a proposal source, never an authority source.

## Provider invariant

Do not introduce a generic provider abstraction.

Allowed:
- official OpenAI API only;
- opaque `cred:openai.*` handles;
- reviewed OpenAI model identifiers.

Forbidden:
- arbitrary model `base_url` or endpoint overrides;
- Azure OpenAI;
- Anthropic/Claude;
- Google/Gemini;
- xAI/Grok;
- OpenRouter;
- Bedrock;
- Ollama/LM Studio/local model adapters;
- third-party OpenAI-compatible endpoints;
- ChatGPT browser cookies/session/access tokens as application credentials.

## Primary platform invariant

Target:

```text
Ubuntu 26.04 LTS
GNOME
Wayland
```

Primary desktop access uses XDG Desktop Portal. Stable portal/compositor contracts are preferred over GNOME-private APIs and X11 tools.

## Authority-instance invariant

At most one non-denied `--execute` process may hold local authority for the current Ubuntu user session.

Lock:

```text
/run/user/<uid>/qsol-chatgpt-authority.lock
```

The lock directory must match the expected `XDG_RUNTIME_DIR`, be owned by the current UID, and have no group/other permission bits. Existing/stale lock => fail closed.

This reduces local credential/session sharing risk. It does not claim to determine whether a credential was stolen or resold.

## Desktop observation

`screen.capture`:

- accepts no model-controlled arguments;
- accepts no credential handles;
- is user/compositor mediated through `org.freedesktop.portal.Screenshot`;
- accepts only a local `file://` portal result;
- ingests at most 64 MiB;
- currently accepts PNG only;
- derives SHA-256, byte count, width and height;
- serializes only `qsol-chatgpt-receipt/4` evidence;
- never serializes screenshot URI/path/raw bytes;
- stores raw screenshot bytes in a zeroizing buffer;
- removes temporary `/run/user`, `/tmp`, or `/var/tmp` portal artifacts best-effort after ingestion.

Continuous future visual observation MUST use XDG ScreenCast + PipeWire through a separately reviewed capability. Do not implement screenshot polling as an ersatz video stream.

## Desktop APIs explicitly not used for the primary path

- `gnome-screenshot` subprocess automation;
- `xdotool`;
- `wmctrl`;
- X11 root capture;
- GNOME Shell `Eval`;
- private Mutter/GNOME Shell DBus APIs.

## Receipt versions

- process: `qsol-chatgpt-receipt/2`
- OBS: `qsol-chatgpt-receipt/3`
- desktop screenshot: `qsol-chatgpt-receipt/4`

Existing v2/v3 semantics must not be weakened when extending v4.

## Credential rules

Raw credentials MUST NOT appear in:
- proposal/action JSON;
- action identity material;
- receipts;
- logs;
- TUI state;
- CLI arguments;
- ambient environment-variable credential paths;
- repository config/fixtures.

Future OpenAI credential resolution uses an OS keyring/Secret Service broker at the last responsible moment.

## Rust / dependency rules

- Rust MSRV: 1.87+.
- `unsafe_code = "forbid"`.
- `ashpd` is used narrowly for XDG Screenshot portal support.
- `tokio` exists only to drive async portal calls in this phase.
- no provider/network SDK is introduced by the OpenAI-only configuration contract itself.

## Source-of-truth order

1. executable Rust tests;
2. Rust contracts/policy/runtime;
3. JSON Schemas;
4. `AGENTS.md` security invariants;
5. architecture/platform docs;
6. README prose.

On disagreement, fail closed and repair the inconsistency.
