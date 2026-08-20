# Threat Model

## Assets

Protect:
- user files and filesystem integrity;
- API keys, OAuth/session credentials and cookies;
- authenticated accounts;
- shell/process authority;
- desktop input authority;
- browser state;
- audit integrity;
- approval meaning;
- user attention and ability to revoke control.

## Adversaries / hostile inputs

Assume any of the following may be malicious or compromised:
- model output;
- a provider response;
- webpage/document text;
- downloaded files;
- repository contents;
- clipboard data;
- OCR/UI text;
- worker-script output;
- tool output;
- prompt-injection content embedded in otherwise legitimate data.

## Primary failure modes

### Confused deputy
A model convinces the runtime to use authority the model itself does not possess.

Mitigation: capability broker, default deny, explicit approvals, scoped future credential broker.

### Action substitution after approval
An action changes after the human approved it.

Mitigation: normalize before approval, immutable public action API, approval bound to content-derived `action_id`.

### Approval replay
A previously approved action is reused in another session or later context.

Current state: exact action binding exists. Session/nonce/expiry binding is Phase 1 and remains required before autonomous operation.

### Secret injection into model-visible data
Raw credentials appear in action arguments, receipts, logs, CLI text, or TUI state.

Mitigation: opaque handles, secret-shaped key rejection, non-serializable secret store, redacted debug, explicit documentation/test requirements.

### Secret residue in memory
A key remains in freed/reused memory.

Mitigation: zeroizing containers and minimized secret lifetime. Rust memory safety alone is explicitly not considered sufficient.

### Ambient environment leakage
A child command inherits `OPENAI_API_KEY`, SSH agent variables, cloud tokens, cookies or other host environment data.

Mitigation: bootstrap executor calls `env_clear()` and supplies only a minimal fixed environment. Future credential injection is explicit and scoped.

### Output exfiltration through receipts
A command prints credentials or private data and the audit log persists it.

Mitigation: receipts store SHA-256 and byte counts, not raw stdout/stderr; captured buffers are zeroized after evidence derivation.

### Shell escape
Structured argv is converted back into unrestricted shell text.

Mitigation: no raw shell-string contract; shell interpreter `-c` is denied in bootstrap policy; no `shell=true` equivalent.

### Privilege escalation
The agent invokes sudo/doas/su or privileged helpers.

Mitigation: common escalation commands are denied now; OS-level sandbox and privilege boundary are still required.

### Network-policy bypass
A shell program opens arbitrary sockets even though network authority was not intended.

Current state: not solved comprehensively by command-name filtering. Real untrusted execution is forbidden until network namespace/policy enforcement lands in Phase 1.

### TUI spoof/confusion
Model-controlled text tricks the human into granting unrelated authority.

Mitigation target: render normalized action fields from trusted structures, visually separate untrusted content, exact action identity, explicit risk/capability labels, no model-controlled keybindings.

### Kill-switch failure
The user requests revoke-all but running processes/effectors continue.

Current state: bootstrap TUI displays revocation state only. Phase 1 requires runtime-authoritative revoke-all and process-tree termination before autonomous loops.

## Security posture

The bootstrap demonstrates the authority model. It does not claim containment. Until the roadmap gates pass, `--execute` is a local developer tool only.
