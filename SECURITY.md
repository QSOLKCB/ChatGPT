# Security Policy

This project is security-sensitive because its purpose is to mediate AI access to a real operating system.

## Current status

The repository is an early bootstrap, not a production sandbox. Do not attach untrusted autonomous models to `--execute`, privileged accounts, production machines, financial accounts, or valuable credentials.

## Core invariants

- capability is not authority;
- default deny;
- exact action-bound approval;
- immutable normalized actions through the public API;
- no `unsafe` Rust in the trusted core;
- no raw secret values in machine-visible contracts;
- no inherited host environment in child processes;
- no raw command-string shell contract;
- no model-direct OS executor access;
- deterministic, secret-free receipts;
- explicit human revocation path.

## Secrets

Rust memory safety prevents broad classes of memory corruption but does not guarantee that secret bytes disappear immediately after use. Secret material therefore requires explicit lifetime control and zeroization.

The bootstrap:
- stores secret values in `Zeroizing<String>` wrappers;
- redacts secret `Debug` output;
- represents credentials in contracts with opaque `cred:*` handles;
- rejects common raw-secret-shaped argument keys during action normalization;
- refuses credential injection into shell commands;
- clears subprocess environment inheritance;
- stores only hashes and sizes of stdout/stderr in receipts and zeroizes captured buffers afterward.

See `docs/SECRETS.md`.

## Reporting vulnerabilities

Please report vulnerabilities privately through GitHub's security reporting facilities when available. Avoid publishing live credentials, exploit transcripts against third-party systems, or sensitive host data in a public issue.

Useful reports include:
- authority bypass;
- approval replay/substitution;
- action identity mismatch;
- secret leakage;
- output/log leakage;
- executor escape;
- unsafe path handling;
- prompt-injection boundary failure;
- TUI state/approval confusion;
- sandbox breakout;
- network-policy bypass.
