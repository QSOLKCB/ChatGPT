# Computer Use Contract

Computer use is a future capability layer on top of the Rust authority core, not a direct model-to-desktop API.

## Loop

```text
capture -> hash/redact -> model observes -> model proposes semantic action
        -> Rust policy -> human approval when required -> OS broker executes
        -> receipt -> next observation
```

## Rules

- Wayland-first observation/control boundaries where practical.
- Screen capture and input injection are separate capabilities.
- The model never receives a raw OS input handle.
- Coordinates, target geometry, active window and application identity become part of effect receipts where relevant.
- Sensitive UI classes require stronger confirmation.
- Action/rate budgets are enforced locally.
- Emergency revoke-all is local and provider-independent.
- Prompt text visible on the desktop is untrusted input.
- Prefer deterministic API/CLI operations over pixel interaction when they provide a narrower authority surface.

## Media implication

For video/audio workflows, FFmpeg/FFprobe adapters should handle deterministic operations first. Kdenlive/Blender GUI control is used where visual editing is genuinely necessary, always through the same broker and approval model.
