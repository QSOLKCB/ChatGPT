# Ubuntu 26.04 LTS Desktop Target

## Primary platform

```text
Ubuntu 26.04 LTS
GNOME
Wayland
```

The project optimizes the computer-use foundation for the security model and desktop APIs available on that stack.

## Observation strategy

### One-shot screenshots

Use the standard XDG Desktop Portal Screenshot interface through `ashpd`.

The portal remains the authority boundary for desktop capture. The application does not bypass user/compositor permission paths. The synchronous Rust capability API performs the async portal exchange on a dedicated worker thread so callers already inside Tokio cannot trigger nested-runtime panics.

### Sustained visual observation

PR #4 implements the Ubuntu-native sustained path:

```text
screen.observe
        -> exact frame/duration bounds
        -> exact human approval
        -> XDG ScreenCast portal
        -> user-mediated monitor/window selection
        -> PipeWire remote + node
        -> bounded mapped-frame hashing
        -> receipt/5
```

The ScreenCast grant is non-persistent. The broker uses `PersistMode::DoNot`, does not retain restore tokens, selects one source, and does not expose the PipeWire node/FD outside the adapter.

Frame payload is hashed directly from mapped PipeWire buffers. Raw video is not accumulated into an application-owned recording and is not forwarded to OpenAI in this phase.

## Deliberately avoided primary paths

The Ubuntu primary path does not use:

- `gnome-screenshot` subprocess automation;
- `scrot` / ImageMagick screen scraping;
- `xdotool`;
- `wmctrl`;
- X11 root-window capture;
- GNOME Shell `Eval`;
- private Mutter/GNOME Shell DBus APIs;
- extension injection;
- direct compositor bypasses.

These are brittle, X11-centric, private, or weaker authority boundaries than portals on GNOME Wayland.

## Window, source, and cursor metadata

ScreenCast receipts may record the selected source category and compositor position/displayed-size metadata supplied by the portal. They do not store source/window names.

Global window enumeration and active-window discovery are **not fabricated** when the platform lacks a stable cross-application portal contract. Cursor capture remains hidden for `screen.observe` in PR #4. Richer cursor metadata requires a separately reviewed stable contract.

## Raw visual-data handling

A screenshot or video frame can contain passwords, private messages, API keys, browser sessions, personal files, or other sensitive material even when capture itself is read-only.

Therefore:

- raw screenshot/frame data is treated as ephemeral sensitive material;
- raw pixels are never serialized into receipts or logs;
- one-shot screenshots use pre-sized zeroizing buffers and complete PNG validation;
- ScreenCast buffers are hashed in place rather than copied into a growing frame archive;
- screenshot paths/URIs are never written into receipts;
- ScreenCast receipts omit PipeWire FD/node IDs, portal object paths, restore tokens, and source titles;
- OpenAI image forwarding remains disabled until redaction, credential, bounded request, and egress gates are complete.

## System build dependency

PipeWire Rust bindings link to Ubuntu's PipeWire development libraries:

```bash
sudo apt install libpipewire-0.3-dev
```

CI installs this package explicitly.

## X11 compatibility

X11 is a future compatibility adapter only. It must not cause the Wayland implementation to weaken its security assumptions or silently fall back to unrestricted global capture.
