# Contributing

Contributions are welcome when they preserve the project's authority, secret-handling, and clean-room boundaries.

## Before coding

Read:
- `README4AI.md`
- `AGENTS.md`
- `docs/ARCHITECTURE.md`
- `docs/THREAT_MODEL.md`
- `docs/SECRETS.md`

## Clean-room requirement

Do not contribute copied, translated, mechanically reproduced, or source-derived implementation code from third-party AI desktop wrappers. If provenance is uncertain, disclose it before contribution.

General ideas, standards, documented protocols, operating-system APIs, and independently designed interfaces may be implemented normally.

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

New effectful capabilities need, at minimum:
1. malformed-input denial test;
2. forbidden-input denial test;
3. missing-approval test;
4. mismatched-approval test;
5. simulation/fake-executor happy-path test;
6. secret/non-leakage test when credentials or output are involved;
7. receipt test.

## Pull requests

Keep the authority core small. Separate capability-policy changes from UI polish where practical. Explain the capability, abuse cases, authority source, revocation mechanism, credential implications, receipt semantics, and sandbox boundary.
