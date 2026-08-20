# Single Authority Instance

## Goal

Prevent one Ubuntu login session from running multiple independent QSOL ChatGPT processes that each hold effectful machine authority or independently consume the same brokered OpenAI credential.

This is a local anti-sharing and authority-integrity control. It does **not** claim that a valid OpenAI credential can be classified as legitimate, stolen, resold, or otherwise by inspecting the credential bytes.

## Lock contract

Before any non-denied `--execute` path proceeds, the process atomically creates:

```text
/run/user/<uid>/qsol-chatgpt-authority.lock
```

The process validates:

1. current UID from `/proc/self/status`;
2. runtime directory is exactly `/run/user/<uid>` (or matching `XDG_RUNTIME_DIR`);
3. runtime directory is owned by the current UID;
4. runtime directory has no group/other permission bits;
5. lock file is created with create-new semantics and mode `0600`.

If the lock already exists, execution fails closed.

## Why not silently delete a stale lock?

Automatically stealing a lock after a crash introduces a race: a second process could incorrectly decide that the legitimate authority owner is dead. This bootstrap therefore prefers inconvenience to duplicate authority.

A later recovery command may inspect boot identity, process start identity, PID and executable identity before offering a human-confirmed stale-lock cleanup path. Until then, stale cleanup is manual and must occur only after confirming no authority process is active.

## What the lock does not do

It cannot stop:

- another unrelated program using the same OpenAI API credential;
- a credential being used on another machine;
- an attacker who already controls the user's Linux account;
- OpenAI account abuse outside this application.

Those concerns require server-side account/project controls plus safe credential issuance and storage.

## Future daemon model

The intended mature architecture is one local authority daemon per user session. Multiple UI views or specialized OpenAI agent roles may connect to that daemon, but they do not each gain an independent credential broker, OS executor, or authority lease.
