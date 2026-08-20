# ROADMAP

The roadmap is ordered by **authority risk**, not demo appeal.

## Phase 0 — Rust authority bootstrap

- [x] Establish Apache-2.0 repository and non-affiliation notice.
- [x] Record clean-room provenance rule.
- [x] Pivot trusted runtime from Python prototype to Rust.
- [x] Forbid `unsafe` Rust in the trusted core.
- [x] Add Ratatui human authority console skeleton.
- [x] Separate untrusted proposals from normalized actions.
- [x] Make normalized action internals immutable through public APIs.
- [x] Version proposal/action/approval/receipt contracts.
- [x] Define deterministic action and receipt identities.
- [x] Implement default-deny policy kernel.
- [x] Require exact action-bound approval for effectful actions.
- [x] Add disabled-by-default structured argv executor.
- [x] Deny obvious privilege escalation and shell command-string escape paths.
- [x] Clear inherited subprocess environment.
- [x] Persist output hashes/sizes rather than raw stdout/stderr in receipts.
- [x] Zeroize captured process-output buffers after evidence derivation.
- [x] Add opaque credential-handle contract.
- [x] Add zeroizing secret-store primitive and redacted debug output.
- [x] Reject common raw-secret-shaped action fields.
- [x] Refuse shell credential injection until a broker exists.
- [x] Add Rust tests and `fmt`/`clippy`/`test` CI.

## Phase 0B — OBS structured application control

- [x] Add loopback-only OBS WebSocket broker.
- [x] Bind OBS endpoint into action identity.
- [x] Bind optional OBS credential handle into action identity.
- [x] Add scene/status/recording capability vocabulary.
- [x] Keep `obs.stream.start` explicitly denied.
- [x] Add absolute protocol deadlines and bounded messages.
- [x] Hash untrusted OBS string observations in receipts.
- [x] Remove ambient environment-variable password path.
- [x] Publish OBS receipt v3 contract.

## Phase 1 — Authority kernel hardening

- [x] Enforce one authority-bearing process per Ubuntu user session.
- [x] Validate authority lock location/ownership/mode and fail closed on duplicates.
- [ ] Add canonical fixture corpus for cross-language identity verification.
- [ ] Specify canonical JSON beyond the current Rust/BTreeMap subset.
- [ ] Add session identity and receipt-chain linkage.
- [ ] Add append-only local audit storage.
- [ ] Add offline replay verifier that performs no effects.
- [ ] Add argument-size and nesting limits.
- [ ] Add output-size limits before allocation growth becomes attacker-controlled.
- [ ] Add bounded execution time.
- [ ] Add process-group/tree termination.
- [ ] Add explicit executable allowlists and executable identity receipts.
- [ ] Add working-directory capability roots.
- [ ] Add policy profiles: observe-only, developer, media, custom.
- [ ] Add deny-by-default network namespace/policy enforcement.
- [ ] Add signed/nonce/session-bound approvals with expiry.
- [ ] Make TUI revoke-all state authoritative in runtime, not just visual state.

### Phase 1 gate

No persistent autonomous loop, GUI input injection, or live OpenAI credential use until audit replay, bounded execution, network denial, and authoritative revoke-all are implemented.

## Phase 2 — OpenAI-only credential and provider boundary

- [x] Lock architecture to official OpenAI only.
- [x] Fix provider origin to `https://api.openai.com` with no arbitrary model endpoint field.
- [x] Require OpenAI credential handles to use `cred:openai.*`.
- [x] Explicitly forbid ChatGPT browser cookies/session tokens as application credentials.
- [x] Explicitly forbid multi-provider and third-party OpenAI-compatible adapters.
- [ ] Define credential-source interface: Ubuntu Secret Service / OS keyring first.
- [ ] Never store OpenAI API keys in repository config files.
- [ ] Resolve opaque handles only inside the broker at the last responsible moment.
- [ ] Zeroize temporary credential copies.
- [ ] Add OpenAI-specific credential scopes.
- [ ] Add explicit `api.openai.com` egress policy.
- [ ] Add official OpenAI Responses API adapter only after the security gates pass.
- [ ] Add tests proving model-visible and audit records contain no secret bytes.

### Phase 2 gate

The OpenAI adapter receives only the minimum credential material for the immediate request. No generic provider interface or secret-reading API exists.

## Phase 3 — Ubuntu 26.04 LTS observation layer

Primary environment: Ubuntu 26.04 LTS + GNOME + Wayland.

- [x] Abstract screenshot backend behind an internal Rust trait.
- [x] Add Wayland-first one-shot screenshot capture through XDG Desktop Portal.
- [x] Refuse model-controlled screenshot arguments and credential handles.
- [x] Validate portal result as a local `file://` URI.
- [x] Bound screenshot ingestion to 64 MiB.
- [x] Require/validate PNG for the first Ubuntu backend.
- [x] Hash screenshot identity and record dimensions without raw bytes/path.
- [x] Zeroize raw screenshot buffers after evidence derivation.
- [x] Add desktop receipt v4 while preserving receipt v2/v3 semantics.
- [x] Add synthetic desktop fixtures.
- [ ] Add screenshot redaction hooks before OpenAI image forwarding.
- [ ] Add XDG ScreenCast session broker.
- [ ] Add PipeWire bounded frame ingestion for sustained visual observation.
- [ ] Add multi-monitor/source geometry from ScreenCast stream metadata.
- [ ] Add stable active-window/window metadata only if Ubuntu exposes a suitable reviewed contract.
- [ ] Add cursor metadata only through a stable reviewed contract.
- [ ] Add X11 compatibility adapter only if useful; never silently downgrade Wayland security.

### Phase 3 gate

One-shot portal capture may produce hash-only audit evidence. Raw frames must not be forwarded to OpenAI until redaction policy, bounded OpenAI image request handling, and credential/network gates are implemented.

## Phase 4 — Controlled computer use

- [ ] Add input capability broker.
- [ ] Use XDG RemoteDesktop/InputCapture-style authority boundaries where appropriate on GNOME Wayland.
- [ ] Click action with target geometry receipt.
- [ ] Text input action.
- [ ] Key/chord action.
- [ ] Application launch action.
- [ ] Per-application allowlists.
- [ ] Sensitive UI classification and human confirmation.
- [ ] Rate limits and action budgets.
- [ ] Emergency stop that kills active effectors/process trees/sessions.
- [ ] Prompt-injection boundary tests.

### Phase 4 gate

OpenAI never talks directly to OS input APIs. Every effect passes through the Rust capability broker and produces a receipt.

## Phase 5 — Browser and filesystem workflows

- [ ] Prefer structured browser automation over pixel clicking.
- [ ] Isolated browser profile support.
- [ ] Download quarantine.
- [ ] Filesystem read roots.
- [ ] Filesystem write roots.
- [ ] Atomic write/replace primitives.
- [ ] Hash-before/hash-after receipts.
- [ ] Explicit delete authority separate from write authority.
- [ ] Browser cookies/session secrets remain broker-owned and never become OpenAI credentials.

## Phase 6 — Media workstation

- [ ] FFprobe inspection adapter.
- [ ] FFmpeg capability adapter.
- [ ] Deterministic render manifests.
- [ ] Subtitle/transcript workflow.
- [ ] Audio normalization workflow.
- [ ] Kdenlive integration only where GUI interaction adds value.
- [ ] Blender integration.
- [ ] Render verification with hashes and media metadata.
- [ ] Python worker adapter for bounded media/scientific tasks.
- [ ] Prefer safer deterministic CLI/API routes over GUI automation unless requested.

## Phase 7 — OpenAI agent orchestration

- [ ] Bounded OpenAI observe/decide/act loop.
- [ ] Per-session action budgets.
- [ ] Loop-stall detection.
- [ ] Repeated-action suppression.
- [ ] Human checkpoint protocol.
- [ ] Long-task resumable state.
- [ ] Multiple specialized roles may share one local authority daemon, but must not create multiple authority-bearing processes or provider credentials.

## Phase 8 — Isolation and release engineering

- [ ] Linux namespaces/container sandbox profile.
- [ ] Optional disposable VM backend.
- [ ] cgroup resource controls.
- [ ] seccomp policy where justified.
- [ ] Reproducible package build for Ubuntu 26.04 LTS.
- [ ] Dependency/license inventory.
- [ ] SBOM generation.
- [ ] Signed release artifacts.
- [ ] Independent threat-model review before v1.0.

## Explicit non-goals before v1.0

- unrestricted autonomous root access;
- stealth or persistence mechanisms;
- bypassing OS portal/security prompts;
- silently approving financial, legal, broadcast-start, or external-account actions;
- raw-token storage or credential harvesting;
- ChatGPT web-session-cookie automation;
- multi-provider support;
- arbitrary OpenAI-compatible model endpoints;
- multiple independent authority-bearing application instances for one user session;
- pretending GUI automation is deterministic when it is not.
