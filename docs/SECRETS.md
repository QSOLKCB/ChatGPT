# Secret Handling

## Principle

A model should be able to *request use of a credential* without ever receiving the credential itself.

```text
model sees:      cred:openai.default
broker resolves: sk-...        (not serialized)
provider uses:   minimum scoped lifetime
broker zeroizes: temporary secret material
receipt sees:    credential handle / operation identity only when safe
```

## Rules

1. Raw API keys, bearer tokens, passwords, cookies, private keys and OAuth refresh tokens do not belong in action JSON.
2. Raw secrets do not belong in receipts, logs, TUI state, crash messages, CLI history, repository config, fixtures or tests.
3. Contracts refer to credentials by opaque `cred:*` handles.
4. Secret values are non-serializable and use explicit zeroizing storage.
5. `Debug` implementations for secret containers are redacted.
6. Subprocesses do not inherit the host environment.
7. Credentials are not injected into shell actions during the bootstrap.
8. Future provider adapters receive the minimum secret scope for the immediate request, not a general secret-store interface.
9. Prefer OS keyring/secret-service integration to application-managed long-term secret files.
10. Never claim Rust memory safety alone erases secrets.
11. The CLI must not load OBS or provider passwords from ambient environment variables.

## Ambient environment is not a credential channel

Environment variables are ambient: descendants may inherit them, diagnostics may dump them, and unrelated process inspection can expose them. The runtime therefore does not treat environment variables as an approved credential source.

Long-term provider and OBS authentication should resolve opaque handles through an OS keyring/Secret Service broker at the last responsible moment.

## OBS bootstrap protected-input channel

Until the OS credential broker lands, authenticated OBS can receive a password only through `--obs-password-stdin` with **non-interactive stdin**. The CLI refuses this option when stdin is a terminal so a password cannot be typed into an echoing prompt.

The input reader:

- accepts at most 4096 UTF-8 password bytes;
- permits one transport line ending (`\n` or `\r\n`) and removes it before authentication;
- zeroizes the original byte buffer after constructing the broker-owned secret value;
- never serializes the password into an action, approval, receipt, log, or CLI argument.

This is a bootstrap channel, not long-term credential storage. Prefer piping directly from an OS secret manager or another protected producer rather than staging a password in shell variables or files.

## TUI contract

The TUI may display:
- credential handle;
- provider/account label safe for display;
- requested capability;
- scope;
- approval state.

The TUI must never display the raw credential value.
