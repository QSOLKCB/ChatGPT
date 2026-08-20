# Secret Handling

## Principle

OpenAI may need to *use* a credential, but the model and machine-action contracts must never receive the credential value.

```text
model/action sees:  cred:openai.default
Ubuntu broker sees: OpenAI API secret
request adapter:    minimum scoped lifetime
receipt sees:        no raw secret
```

## Provider restriction

The model/provider boundary is OpenAI-only.

Accepted future credential handles:

```text
cred:openai.*
```

The application does not accept ChatGPT web session cookies/access tokens, credentials for other model vendors, or arbitrary OpenAI-compatible endpoints.

## Rules

1. Raw API keys, bearer tokens, passwords, cookies, private keys, and OAuth refresh tokens do not belong in action JSON.
2. Raw secrets do not belong in receipts, logs, TUI state, crash messages, CLI history, repository config, fixtures, or tests.
3. OpenAI contracts refer to credentials only by opaque `cred:openai.*` handles.
4. Secret values are non-serializable and use explicit zeroizing storage.
5. `Debug` implementations for secret containers are redacted.
6. Subprocesses do not inherit the host environment.
7. Credentials are not injected into shell actions.
8. The OpenAI adapter must receive only the minimum credential scope for an immediate official OpenAI request.
9. Long-term storage uses Ubuntu Secret Service / an OS keyring rather than application-managed secret files.
10. Rust memory safety is not treated as secret erasure.
11. The CLI must not load OpenAI or OBS passwords from ambient environment variables.
12. A screenshot is secret-adjacent data even though it is not a credential; raw screenshot bytes follow the same non-logging/minimized-lifetime discipline.

## Ambient environment is not a credential channel

Environment variables are ambient: descendants may inherit them, diagnostics may dump them, and unrelated process inspection can expose them. The runtime therefore does not treat environment variables as an approved OpenAI credential source.

## OBS bootstrap protected-input channel

Until a general OS credential broker lands, authenticated OBS may receive a password through `--obs-password-stdin` with non-interactive stdin. Interactive terminal input is refused so the application never presents an echoing password prompt.

This exception is OBS-specific and does not establish stdin as the final OpenAI credential design.

## Single authority instance

Effectful execution is limited to one authority-bearing process per Ubuntu user session. This reduces accidental or deliberate local credential sharing among multiple independent application processes.

It does not prove the provenance of a valid credential and cannot prevent use by unrelated programs or other machines.

## TUI contract

The TUI may display:
- credential handle;
- safe account/project label supplied by a trusted broker;
- requested capability;
- scope;
- approval state.

The TUI must never display raw credential values.
