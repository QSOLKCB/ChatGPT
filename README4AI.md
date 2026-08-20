# README4AI

## Identity

Project: `QSOLKCB/ChatGPT`
Purpose: clean-room Linux-native AI workstation, capability broker, and human authority console.
Trusted implementation language: Rust.
Human control plane: Ratatui TUI.
License: Apache-2.0.
Affiliation: independent community project; not affiliated with or endorsed by OpenAI.

## Prime invariant

```text
CAPABILITY != AUTHORITY
```

Models/providers are proposal sources, never authority sources.

## Trust boundary

```text
UNTRUSTED: model output, provider SDKs, webpages, files, OCR, clipboard, UI text, worker scripts
TRUSTED:   Rust contracts -> policy -> approval verifier -> capability broker -> receipt builder
HUMAN:     Ratatui TUI supplies/revokes authority; it does not bypass policy
```

No provider may call an executor directly.
No executor may accept a model object directly.

## Action lifecycle

```text
ProposedAction
  -> normalize + reject raw-secret-shaped fields
  -> Action (content-addressed, immutable through public API)
  -> policy
     -> deny
     -> approval_required -> exact action_id verification
     -> allow
  -> executor or simulation
  -> Receipt
```

Unknown action kinds MUST be denied.
Effectful known actions MUST require approval unless a future policy explicitly narrows the rule.
Approval MUST bind to the exact normalized `action_id`.

## Bootstrap capability classes

Read-only policy-visible:
- `screen.capture`
- `filesystem.read`

Effectful policy-visible:
- `shell.exec`
- `input.click`
- `input.type`
- `app.launch`
- `filesystem.write`

Only `shell.exec` has a bootstrap executor. Real effects are disabled unless the local human explicitly starts the runtime with execution enabled.

## Secret contract

Raw secrets MUST NOT appear in:
- action/proposal JSON;
- action identity material;
- approvals;
- receipts;
- logs;
- TUI application state;
- CLI arguments created by this project;
- child-process environment by inheritance.

Machine contracts refer to credentials only as opaque handles matching `cred:*`.
In-process secret values use zeroizing storage and redacted `Debug` output.
The bootstrap shell executor refuses actions that request credential handles; credential injection does not exist yet.

Rust memory safety MUST NOT be described as automatic secret erasure. Secret lifetime/zeroization is a separate requirement.

## Shell contract

`shell.exec` accepts structured `argv` only. No raw shell command-string contract exists.
Shell interpreter `-c` escapes are denied.
Privilege-escalation commands are denied.
Child environment is cleared and replaced with a minimal fixed environment.
Stdout/stderr are hashed and byte-counted for receipts, then zeroized; raw output is not persisted in receipts.

## Rust rules

- `unsafe_code = "forbid"`.
- authority-core public APIs expose immutable borrows, not mutable action internals;
- avoid `unwrap`/`expect` in production and test targets;
- provider/UI dependencies do not enter the policy kernel;
- Python is an external worker language, never the trusted authority implementation.

## Contract versions

- proposal: `qsol-chatgpt-proposal/1`
- normalized action: `qsol-chatgpt-action/2`
- approval: `qsol-chatgpt-approval/2`
- receipt: `qsol-chatgpt-receipt/2`

## Source-of-truth order

1. executable Rust tests;
2. Rust contracts/policy/runtime;
3. JSON Schemas and canonical fixtures;
4. architecture/security documentation;
5. README prose.

When sources disagree, fail closed and repair the inconsistency.

## Clean-room rule

Do not copy, translate, mechanically reproduce, or derive implementation code from Noi, `lencx/ChatGPT`, or other third-party desktop AI wrappers. Independently implement general concepts from standards and public architectural ideas. Record third-party dependencies and licenses in provenance documentation.
