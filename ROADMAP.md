# ROADMAP

The roadmap is ordered by authority risk, not by demo appeal.

## Phase 0 — Bootstrap contracts

- [x] Establish Apache-2.0 repository.
- [x] Record independent/non-affiliation notice.
- [x] Record clean-room provenance rule.
- [x] Define action lifecycle.
- [x] Define action, approval, and receipt schemas.
- [x] Define deterministic action and receipt identities.
- [x] Implement default-deny policy kernel.
- [x] Require exact action-bound approval for effectful actions.
- [x] Add disabled-by-default structured argv executor.
- [x] Add tests and CI.

## Phase 1 — Authority kernel hardening

- [ ] Version all machine contracts explicitly.
- [ ] Add canonical fixture corpus shared across languages.
- [ ] Add receipt-chain linkage and session identity.
- [ ] Add append-only local audit storage.
- [ ] Add replay verifier that performs no effects.
- [ ] Separate proposed action from normalized executable action.
- [ ] Add argument-size and output-size limits.
- [ ] Add bounded execution time and process-tree termination.
- [ ] Add explicit environment allowlist.
- [ ] Add working-directory capability roots.
- [ ] Add policy profiles: observe-only, developer, media, custom.
- [ ] Add deny-by-default network capability class.

### Phase 1 gate

No GUI input injection or persistent autonomous loop until receipts can replay and verify offline.

## Phase 2 — Linux observation layer

- [ ] Abstract desktop backend interface.
- [ ] Wayland-first screenshot capture.
- [ ] X11 compatibility adapter only where needed.
- [ ] Multi-monitor geometry model.
- [ ] Window enumeration without control authority.
- [ ] Active-window observation.
- [ ] Cursor/location observation.
- [ ] Image identity hashing.
- [ ] Screenshot redaction hooks.
- [ ] Tests with synthetic desktop fixtures.

## Phase 3 — Controlled computer use

- [ ] Input capability broker.
- [ ] Click action with target geometry receipt.
- [ ] Text input action.
- [ ] Key/chord action.
- [ ] Application launch action.
- [ ] Per-application allowlists.
- [ ] Human confirmation for sensitive UI classes.
- [ ] Rate limits and action budgets.
- [ ] Emergency stop / revoke-all control.
- [ ] Prompt-injection boundary tests.

### Phase 3 gate

The model never talks directly to OS input APIs. All desktop effects pass through the capability broker and produce receipts.

## Phase 4 — Browser and filesystem workflows

- [ ] Prefer structured browser automation over pixel clicking.
- [ ] Isolated browser profile support.
- [ ] Download quarantine.
- [ ] Filesystem read roots.
- [ ] Filesystem write roots.
- [ ] Atomic write/replace primitives.
- [ ] Hash-before/hash-after receipts.
- [ ] Explicit delete policy separate from write policy.
- [ ] Credential broker with opaque handles rather than raw secrets.

## Phase 5 — Media workstation

- [ ] FFmpeg capability adapter.
- [ ] FFprobe inspection adapter.
- [ ] Deterministic render manifests.
- [ ] Subtitle/transcript workflow.
- [ ] Audio normalization workflow.
- [ ] Kdenlive integration where GUI interaction is genuinely useful.
- [ ] Blender integration.
- [ ] Render verification using hashes and media metadata.
- [ ] Never use GUI automation for a task that has a safer deterministic CLI path unless requested.

## Phase 6 — Provider and agent orchestration

- [ ] Provider-neutral model interface.
- [ ] OpenAI adapter.
- [ ] Local model adapter.
- [ ] Tool-result normalization.
- [ ] Bounded observe/decide/act loop.
- [ ] Per-session action budgets.
- [ ] Loop-stall detection.
- [ ] Repeated-action suppression.
- [ ] Human checkpoint protocol.
- [ ] Long-task resumable state.

## Phase 7 — Isolation and packaging

- [ ] Linux namespace/container sandbox profile.
- [ ] Optional disposable VM backend.
- [ ] Network namespace policy.
- [ ] Secretless default environment.
- [ ] Reproducible package build.
- [ ] Signed release artifacts.
- [ ] SBOM generation.
- [ ] Threat-model review before v1.0.

## Explicit non-goals before v1.0

- unrestricted autonomous root access;
- stealth or persistence mechanisms;
- bypassing OS security prompts;
- silently approving financial/legal/external-account actions;
- credential harvesting or raw-token storage;
- pretending GUI automation is deterministic when it is not.
