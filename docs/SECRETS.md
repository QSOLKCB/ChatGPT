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

## Why environment variables are not the primary design

Environment variables are convenient but ambient: descendants may inherit them, diagnostics may dump them, and unrelated subprocesses can receive authority accidentally. The long-term design therefore uses a broker and opaque handles. Environment injection, if ever needed, must be explicit, scoped, cleared immediately, and covered by tests.

## TUI contract

The TUI may display:
- credential handle;
- provider/account label safe for display;
- requested capability;
- scope;
- approval state.

The TUI must never display the raw credential value.
