# Ubuntu 26.04 LTS Desktop Target

## Primary platform

The primary desktop target is:

```text
Ubuntu 26.04 LTS
GNOME
Wayland
```

The project optimizes the computer-use foundation for the security model and desktop APIs available on that stack.

## Observation strategy

### One-shot screenshots

Use the standard XDG Desktop Portal Screenshot interface through `ashpd`.

The portal remains the authority boundary for desktop capture. The application does not bypass the user/compositor permission path.

The synchronous Rust capability API runs the async portal exchange on a dedicated worker thread with its own Tokio runtime. This avoids nesting `block_on` inside an existing Tokio executor when the library is called from async applications.

### Sustained visual observation

Future continuous computer vision should use:

```text
XDG ScreenCast portal
        -> user-mediated source selection
        -> PipeWire node
        -> bounded frame broker
        -> OpenAI observation path
```

This avoids repeatedly invoking one-shot screenshot dialogs and preserves Wayland's explicit capture authority model.

## Deliberately avoided primary paths

PR #3 does not use:

- `gnome-screenshot` subprocess automation;
- `scrot` / ImageMagick screen scraping;
- `xdotool`;
- `wmctrl`;
- X11 root-window capture;
- GNOME Shell `Eval`;
- private Mutter/GNOME Shell DBus APIs;
- extension injection;
- direct compositor bypasses.

These are either brittle, X11-centric, private, or weaker authority boundaries than portals on GNOME Wayland.

## Window and cursor metadata

Global window enumeration, active-window discovery, and cursor location are **not fabricated** when the platform does not expose a stable cross-application portal contract for them.

Those roadmap items remain open until they can be implemented through a stable, reviewable Ubuntu/GNOME/Wayland API without depending on private shell internals.

## Raw screenshot handling

A screenshot can contain passwords, private messages, API keys, browser sessions, personal files, or other sensitive material even when the capture operation itself is read-only.

Therefore:

- screenshot bytes are treated as ephemeral sensitive data;
- raw bytes are never serialized into receipts or logs;
- the audit record stores content hash, byte size, dimensions, backend identity, and image format only;
- the file size is verified before allocation and screenshot bytes are read into one pre-sized zeroizing allocation, avoiding growth reallocations that could leave frame fragments in freed heap memory;
- the complete PNG chunk stream is validated through `IEND`, including chunk CRCs, before a capture can be recorded as completed;
- the portal artifact path/URI is never written into the receipt;
- parent-directory URI components are rejected;
- best-effort cleanup canonicalizes the portal path and only unlinks regular files that remain beneath canonical `/tmp`, `/var/tmp`, or the validated `/run/user/<uid>` runtime directory;
- symlink or traversal targets outside those approved roots are never deleted by cleanup.

## X11 compatibility

X11 is a future compatibility adapter only. It must not cause the Wayland implementation to weaken its security assumptions or silently fall back to unrestricted global capture.
