# Computer-Use Contract

Computer use is modeled as a sequence of explicit capabilities rather than unrestricted desktop possession.

## Intended loop

```text
observe -> propose -> validate -> authorize -> execute -> receipt -> observe again
```

## Observation capabilities

Planned examples:

- capture screen;
- enumerate windows;
- inspect active application identity;
- read bounded filesystem content;
- inspect media metadata.

Observation is not automatically harmless. Screenshots and file reads may expose secrets, so future policy profiles must scope them.

## Effect capabilities

Planned examples:

- click at validated geometry;
- type text;
- send bounded key chords;
- launch an allowlisted application;
- execute structured command argv;
- write within a scoped filesystem root.

## Preferred-control rule

Use the most structured and deterministic interface available:

```text
native API / file format
        > dedicated CLI
        > accessibility / automation API
        > GUI coordinate control
```

For media work, prefer FFmpeg/FFprobe and deterministic project files before pixel-level timeline manipulation.

## No direct model-to-OS path

A provider adapter MUST NOT expose unrestricted mouse, keyboard, process, filesystem, network, or credential APIs directly to model output. Every effect crosses the policy and receipt boundary.

## Stale-state rule

After a meaningful effect, re-observe before assuming success. A successful API call is not proof that the intended semantic state now exists.
