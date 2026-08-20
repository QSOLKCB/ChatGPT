# XDG ScreenCast + PipeWire Observation

## Scope

PR #4 adds bounded sustained visual observation for the primary platform:

```text
Ubuntu 26.04 LTS
GNOME
Wayland
```

The capability is named `screen.observe`.

It is **not** an autonomous camera into the desktop. Each action is bounded, exact-approval-gated, and user-mediated by the XDG ScreenCast portal.

## Authority flow

```text
OpenAI/model proposal
        |
        v
screen.observe
max_frames + max_duration_ms
        |
        v
Rust policy
        |
 exact human approval
        |
        v
XDG ScreenCast portal
        |
 user selects one monitor/window
        |
        v
PipeWire remote FD + node
        |
        v
bounded frame hashing
        |
        v
receipt/5
```

The approved action binds both observation limits into the normalized `action_id`:

- `max_frames`: 1..=300
- `max_duration_ms`: 500..=30000

Changing either limit invalidates an existing approval.

## Portal contract

The broker:

- requests `Monitor | Window` sources;
- allows exactly one selected source;
- hides the cursor in the video stream;
- uses `PersistMode::DoNot`;
- does not store a restore token;
- does not bypass the portal's source-selection or permission UI;
- keeps the portal session alive only for the bounded observation operation.

## PipeWire contract

The portal's PipeWire remote FD and selected node ID remain internal to the broker.

The PipeWire stream negotiates bounded raw video formats and validates negotiated dimensions before recording evidence. Frame buffers are processed inside PipeWire callbacks.

Raw frame payload is **not copied into a growing application-owned frame archive**. The broker hashes mapped payload in place, plane by plane, and derives an order-sensitive frame chain:

```text
frame_digest_i = SHA256(plane metadata || mapped frame bytes)
chain = SHA256(frame_index || payload_size || frame_digest_i ...)
```

The receipt stores only the final chain hash and bounded metadata.

## Receipt v5

`qsol-chatgpt-receipt/5` can record:

- frame-chain SHA-256;
- frames observed;
- total payload bytes hashed;
- observed duration;
- negotiated width/height;
- negotiated framerate;
- selected source category;
- optional compositor source position and displayed size.

It does **not** record:

- raw pixels;
- a portal restore token;
- PipeWire file descriptors;
- PipeWire node IDs;
- source/window names;
- application titles;
- portal object paths;
- image files.

## OpenAI boundary

PR #4 does not forward ScreenCast frames to OpenAI.

The sustained observation broker is local infrastructure only until all of these are separately implemented and reviewed:

1. screenshot/frame redaction policy;
2. bounded official OpenAI image request handling;
3. Ubuntu Secret Service OpenAI credential broker;
4. explicit `api.openai.com` egress policy;
5. audit rules for what image-derived model observations may be retained.

This keeps `screen.observe` from becoming an accidental raw-desktop exfiltration capability.

## System dependency

The Rust `pipewire` crate links to the system PipeWire development libraries. Ubuntu builds require:

```bash
sudo apt install libpipewire-0.3-dev
```

CI installs this package explicitly.

## Future relationship to input control

ScreenCast grants observation only. It does not grant keyboard or mouse authority.

Controlled input remains a later capability using separately reviewed RemoteDesktop/InputCapture-style portal boundaries. Observation authority and input authority must remain independently revocable.
