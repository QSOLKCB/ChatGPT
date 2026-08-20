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
- [x] Deny obvious privilege escalation and shell `-c` escape paths.
- [x] Clear inherited subprocess environment.
- [x] Persist output hashes/sizes rather than raw stdout/stderr in receipts.
- [x] Zeroize captured process-output buffers after evidence derivation.
- [x] Add opaque credential-handle contract.
- [x] Add zeroizing secret-store primitive and redacted debug output.
- [x] Reject common raw-secret-shaped action fields.
- [x] Refuse shell credential injection until a broker exists.
- [x] Add Rust tests and `fmt`/`clippy`/`test` CI.

## Phase 1 — Authority kernel hardening

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

No persistent autonomous loop, GUI input injection, or live provider credential use until audit replay, bounded execution, network denial, and authoritative revoke-all are implemented.

## Phase 2 — Credential broker and provider boundary

- [ ] Define credential-source interface: OS keyring/secret service first.
- [ ] Never store provider API keys in repository config files.
- [ ] Resolve opaque handles only inside the broker at the last responsible moment.
- [ ] Zeroize temporary credential copies.
- [ ] Add per-provider credential scopes.
- [ ] Add explicit network destinations/egress policy.
- [ ] Add provider-neutral request/response interface.
- [ ] Add OpenAI adapter only after the above gates pass.
- [ ] Add local-model adapter that requires no cloud credential.
- [ ] Add tests proving provider/model-visible records contain no secret bytes.

### Phase 2 gate

No provider gets a general-purpose secret-reading API. Provider adapters receive only the minimum scoped credential material needed for the immediate request.

## Phase 3 — Linux observation layer

- [ ] Abstract desktop backend interface.
- [ ] Wayland-first screenshot capture via appropriate portal/compositor boundary.
- [ ] X11 compatibility adapter only where needed.
- [ ] Multi-monitor geometry model.
- [ ] Window enumeration without control authority.
- [ ] Active-window observation.
- [ ] Cursor/location observation.
- [ ] Image identity hashing.
- [ ] Screenshot redaction hooks.
- [ ] Synthetic desktop fixtures.

## Phase 4 — Controlled computer use

- [ ] Input capability broker.
- [ ] Click action with target geometry receipt.
- [ ] Text input action.
- [ ] Key/chord action.
- [ ] Application launch action.
- [ ] Per-application allowlists.
- [ ] Sensitive UI classification and human confirmation.
- [ ] Rate limits and action budgets.
- [ ] Emergency stop that kills active effectors/process trees.
- [ ] Prompt-injection boundary tests.

### Phase 4 gate

The model never talks directly to OS input APIs. Every effect passes through the Rust capability broker and produces a receipt.

## Phase 5 — Browser and filesystem workflows

- [ ] Prefer structured browser automation over pixel clicking.
- [ ] Isolated browser profile support.
- [ ] Download quarantine.
- [ ] Filesystem read roots.
- [ ] Filesystem write roots.
- [ ] Atomic write/replace primitives.
- [ ] Hash-before/hash-after receipts.
- [ ] Explicit delete authority separate from write authority.
- [ ] Browser cookies/session secrets remain broker-owned.

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

## Phase 7 — Agent orchestration

- [ ] Bounded observe/decide/act loop.
- [ ] Per-session action budgets.
- [ ] Loop-stall detection.
- [ ] Repeated-action suppression.
- [ ] Human checkpoint protocol.
- [ ] Long-task resumable state.
- [ ] Multiple specialized agents without shared ambient authority.

## Phase 8 — Isolation and release engineering

- [ ] Linux namespaces/container sandbox profile.
- [ ] Optional disposable VM backend.
- [ ] cgroup resource controls.
- [ ] seccomp policy where justified.
- [ ] Reproducible package build.
- [ ] Dependency/license inventory.
- [ ] SBOM generation.
- [ ] Signed release artifacts.
- [ ] Independent threat-model review before v1.0.

## Explicit non-goals before v1.0

- unrestricted autonomous root access;
- stealth or persistence mechanisms;
- bypassing OS security prompts;
- silently approving financial, legal, or external-account actions;
- raw-token storage or credential harvesting;
- direct model access to secret stores;
- pretending GUI automation is deterministic when it is not.
