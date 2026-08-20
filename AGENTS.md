# AGENTS.md

MACHINE-ORIENTED CONTRIBUTOR INSTRUCTIONS.

## Mission

Maintain a small, auditable, clean-room Linux agent runtime where models propose actions and the runtime controls authority.

## Non-negotiable invariants

1. Never grant execution merely because a model requested it.
2. Unknown action kinds are denied.
3. Effectful actions require explicit authority.
4. Approval records bind to exact action identities.
5. Every evaluated action returns a receipt.
6. No `shell=True` in the bootstrap runtime.
7. Shell actions use argv arrays.
8. Executor defaults remain non-executing unless a roadmap phase explicitly changes the contract.
9. Do not log secrets, environment dumps, tokens, cookies, credentials, or private key material.
10. Do not weaken tests to make unsafe behaviour pass.

## Clean-room provenance

This repository is an independent implementation. Do not copy or port code from Noi, lencx/ChatGPT, or other desktop AI wrappers. Do not paste source snippets from unlicensed or incompatibly licensed projects. Architectural concepts may be independently implemented.

When introducing a dependency:

- verify its license;
- document why it is required;
- prefer standard-library or narrow dependencies for the authority core;
- keep provider/UI integrations outside the policy kernel.

## Change discipline

For changes to action semantics, update together:

- `schemas/`;
- Python data model;
- policy rules;
- tests;
- `README4AI.md` when machine behaviour changes.

For new executors, add negative tests for denied and unapproved paths before happy-path execution tests.

## Security review triggers

Require explicit review when a change adds or broadens:

- shell execution;
- filesystem writes;
- network access;
- credential access;
- clipboard access;
- browser sessions;
- input injection;
- desktop capture;
- background persistence;
- privilege changes;
- autonomous loops;
- remote control.

## Test command

```bash
python -m unittest discover -s tests -v
```

The test suite must pass without network access and without executing real host mutations.
