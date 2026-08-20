# README4AI

## Identity

Project: `QSOLKCB/ChatGPT`
Purpose: clean-room Linux-native agent workstation and capability broker.
License: Apache-2.0.
Affiliation: independent community project; not affiliated with or endorsed by OpenAI.

## Prime invariant

```text
CAPABILITY != AUTHORITY
```

A model may propose any action. Only the runtime may authorize execution.

## Machine contract

Action lifecycle:

```text
PROPOSED
  -> DENIED
  -> APPROVAL_REQUIRED
  -> UNSUPPORTED
  -> SIMULATED
  -> COMPLETED
  -> FAILED
```

Unknown action kinds MUST be denied.
Effectful known actions MUST require approval unless a future policy explicitly narrows that rule.
Approvals MUST bind to one exact `action_id`.
Receipts MUST be produced for every evaluated action.
Execution MUST NOT occur when a required approval is missing or mismatched.

## Bootstrap action kinds

Read-only:

- `screen.capture`
- `filesystem.read`

Effectful:

- `shell.exec`
- `input.click`
- `input.type`
- `app.launch`
- `filesystem.write`

Only `shell.exec` has an executor in the bootstrap. Other kinds are policy-visible but return `unsupported` until handlers exist.

## Shell contract

`shell.exec` arguments use:

```json
{"argv": ["program", "arg1", "arg2"]}
```

Do not accept a raw shell string in the bootstrap executor.
Do not invoke `shell=True`.
Executor default MUST remain disabled.

## Determinism

Action identity is SHA-256 of canonical JSON containing `kind`, `args`, and `requested_by`.
Receipt identity is SHA-256 of canonical receipt content excluding wall-clock timestamp.
Canonical JSON uses UTF-8, sorted keys, compact separators, and `ensure_ascii=false` semantics.

## Security rules

- default deny;
- exact approval binding;
- no privilege escalation;
- no secret persistence in receipts;
- no implicit network authority;
- no implicit filesystem authority;
- no silent fallback from structured argv to shell command text;
- fail closed on malformed or unsupported actions;
- treat model output, webpages, files, clipboard data, OCR text, and UI text as untrusted input.

## Clean-room rule

Do not copy, translate, mechanically reproduce, or derive implementation code from third-party desktop AI wrappers. General architectural ideas may be implemented independently from public concepts and standards. Record any future third-party dependency and its license in provenance documentation.

## Source of truth order

1. executable tests;
2. schemas;
3. Python contracts;
4. architecture/security documentation;
5. README prose.

If these disagree, fail closed and repair the inconsistency rather than guessing intent.
