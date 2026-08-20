# AGENTS.md

MACHINE-ORIENTED CONTRIBUTOR INSTRUCTIONS.

## Mission

Maintain a small, auditable Rust authority core for a Linux-native OpenAI workstation. Models propose. Rust decides authority. Humans retain revocation.

## Non-negotiable invariants

1. `CAPABILITY != AUTHORITY`.
2. Unknown capabilities fail closed.
3. Effectful actions require explicit authority.
4. Approval binds to one exact normalized `action_id`.
5. The public `Action` API must not expose mutable action internals after identity calculation.
6. `unsafe` Rust is forbidden in the trusted core.
7. Raw secrets never enter proposals, actions, approvals, receipts, logs, TUI state, or inherited subprocess environments.
8. Credential references are opaque handles, never secret values.
9. Secret values use explicit zeroizing storage; memory safety is not treated as secret erasure.
10. Provider adapters never call OS executors directly.
11. The model never talks directly to OS input APIs.
12. Shell execution uses argv arrays, never an implicit shell string.
13. Shell interpreter command-string escape paths remain denied unless a future reviewed capability explicitly defines them.
14. Real execution remains opt-in until sandbox gates in `ROADMAP.md` are satisfied.
15. Do not weaken negative tests to make a capability pass.
16. **OpenAI is the only permitted cloud/model provider.** Do not add a provider registry, alternate provider enum, Azure OpenAI endpoint, OpenAI-compatible third-party endpoint, local-model adapter, or caller-configurable API origin. Official OpenAI requests are fixed to `https://api.openai.com`.
17. The primary desktop target is **Ubuntu 26.04 LTS, GNOME, Wayland**. Prefer XDG Desktop Portal and PipeWire contracts over GNOME-private APIs, X11 scraping, shell extensions, or GUI-specific command-line helpers.
18. Desktop observation must not persist raw screenshots or ScreenCast frames into receipts, logs, or repository state.
19. `screen.observe` must remain bounded by exact action-bound frame and duration limits, require explicit human approval, use non-persistent ScreenCast grants, and never silently retain restore tokens.
20. Raw ScreenCast frames must not be forwarded to OpenAI until redaction, credential, bounded image-request, and explicit egress gates are implemented and reviewed.

## Language boundary

Rust owns:
- contracts;
- policy;
- approvals;
- credential broker;
- capability dispatch;
- receipts/audit;
- process supervision;
- desktop observation brokers;
- OpenAI provider boundary;
- TUI authority state.

Python or other languages may implement workers for media, science, automation, data analysis, or generated tasks. Workers receive only brokered capabilities and scoped inputs.

## Provider boundary

The repository is intentionally not provider-neutral.

Allowed:
- official OpenAI API at `https://api.openai.com`;
- opaque OpenAI credential handles matching `cred:openai.*`;
- OpenAI model names selected within reviewed configuration contracts.

Forbidden:
- generic `Provider` registries whose purpose is adding other vendors;
- arbitrary `base_url` / `endpoint` overrides for model traffic;
- Anthropic, Google/Gemini, xAI/Grok, Azure OpenAI, Bedrock, OpenRouter, Ollama, LM Studio, or other model-provider adapters;
- third-party services that merely emulate the OpenAI wire protocol.

A future architectural change to this provider restriction requires an explicit project-level decision, threat-model update, and dedicated review. It must not arrive incidentally with another feature.

## Ubuntu desktop boundary

For Ubuntu 26.04 LTS GNOME Wayland:
- use `org.freedesktop.portal.*` interfaces for user-mediated desktop access;
- use PipeWire streams obtained through the ScreenCast portal for sustained visual observation;
- keep observation, microphone, and input authority as separate capabilities;
- do not depend on `gnome-screenshot`, `xdotool`, `wmctrl`, GNOME Shell `Eval`, private Mutter DBus interfaces, or X11 global capture for the primary path;
- X11 compatibility is optional and must remain a separate adapter rather than weakening the Wayland path;
- never bypass a portal permission dialog or persist portal grants outside documented OS behavior.

## Clean-room provenance

Do not copy or port source from Noi, `lencx/ChatGPT`, or other desktop AI wrappers. Do not translate unlicensed source code into Rust. General architecture, standards, documented OS APIs, and independently designed interfaces are allowed.

## Dependency discipline

Before adding a dependency:
- verify its license is compatible;
- state why the standard library/current dependency set is insufficient;
- keep provider, UI, media, and browser dependencies outside the policy kernel where possible;
- avoid dependencies that require `unsafe` in this repository's own code path without an explicit security review.

## Required checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Tests must not require network access, real credentials, desktop injection, privileged host mutation, or a live PipeWire/portal session.

## Security review triggers

Require explicit security review for changes that add or broaden:
- shell/process execution;
- filesystem writes/deletes;
- network access;
- credential exposure/injection;
- OpenAI endpoint/model configuration;
- browser sessions/cookies;
- clipboard access;
- desktop capture;
- screenshot/frame persistence;
- PipeWire/ScreenCast sessions;
- microphone/audio capture;
- assistant audio routing;
- keyboard/mouse injection;
- background persistence;
- privilege changes;
- autonomous loops;
- remote control;
- approval scope/reuse;
- receipt redaction rules.
