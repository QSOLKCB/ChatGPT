# ChatGPT

A clean-room, Linux-native agent workstation built around a **Rust authority core** and a **terminal human-control plane**.

> **Independent community project.** Not affiliated with or endorsed by OpenAI. ChatGPT is a trademark of OpenAI.

## Mission

Give AI systems useful access to a computer without confusing intelligence with authority.

```text
model/provider -> proposes action
                    |
                    v
             Rust authority core
                    |
              policy decision
              /     |      \
           deny  approve   allow
                    |
                    v
             capability broker
                    |
                    v
               host/sandbox
                    |
                    v
                  receipt

Human <---------- Ratatui TUI ----------> authority core
```

The prime invariant is:

```text
CAPABILITY != AUTHORITY
```

The model may reason about an action. Only the local runtime may authorize and execute it.

## Why Rust + TUI

This application sits between models and operating-system capabilities, eventually including API credentials, authenticated sessions, filesystem access, input injection, browser state, media tools, and long-running processes.

Rust is the trusted implementation language because memory safety, strong types, ownership, and explicit state transitions are valuable properties at that boundary. A TUI keeps the human control plane small, inspectable, fast, SSH-friendly, and free of a heavyweight GUI stack.

Rust memory safety is **not** secret erasure. The bootstrap therefore also uses zeroizing secret containers, redacted debug output, opaque credential handles, cleared subprocess environments, and a rule that raw secrets never enter action JSON, receipts, logs, or TUI state.

Python remains welcome for scientific, media, automation, and generated worker tasks. It runs *behind* the Rust capability broker rather than guarding authority itself.

## Current bootstrap

The Rust skeleton now includes:

- immutable normalized actions with content-derived identities;
- separate untrusted proposals and normalized executable actions;
- a default-deny policy kernel;
- exact action-bound approval records;
- hard denial of unknown capabilities and obvious shell escapes/privilege escalation;
- structured argv execution only, disabled unless `--execute` is supplied;
- cleared child-process environment with a minimal fixed environment;
- hashed output evidence instead of stdout/stderr persistence;
- zeroization of captured subprocess output after evidence is derived;
- opaque `cred:*` handles and an in-memory zeroizing secret store;
- raw secret-shaped action fields rejected during normalization;
- deterministic receipts without wall-clock data in their identity;
- a Ratatui authority console with an emergency revoke state;
- language-neutral JSON Schemas;
- Rust unit/integration tests and CI with `fmt`, `clippy`, and `test`.

This is **not yet a production sandbox or autonomous desktop-control product**. See `ROADMAP.md`.

## Build and test

```bash
git clone https://github.com/QSOLKCB/ChatGPT.git
cd ChatGPT
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Launch the authority console:

```bash
cargo run -- tui
```

Inspect policy without executing anything:

```bash
cargo run -- policy \
  '{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","hello\\n"]}}'
```

Simulate an approved action:

```bash
cargo run -- run \
  '{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","hello\\n"]}}' \
  --approve
```

Real host effects are deliberately opt-in during the bootstrap:

```bash
cargo run -- run \
  '{"schema_version":"qsol-chatgpt-proposal/1","kind":"shell.exec","args":{"argv":["printf","hello\\n"]}}' \
  --approve --execute
```

Do **not** use `--execute` with an untrusted model or privileged/production environment yet.

## Repository map

```text
src/contracts.rs       proposal, action, approval and credential-handle contracts
src/policy.rs          default-deny authority decisions
src/runtime.rs         approval gate and execution lifecycle
src/executor.rs        disabled-by-default structured command executor
src/receipts.rs        deterministic, secret-free receipts
src/secrets.rs         zeroizing in-memory secret primitives
src/tui.rs             Ratatui human authority console
schemas/               language-neutral contracts
docs/                  architecture, threat model, computer-use and secret rules
tests/                 cross-module contract tests
README4AI.md            machine-oriented source map
AGENTS.md               machine contributor invariants
ROADMAP.md              authority-risk-ordered implementation plan
```

## Clean-room provenance

No source code from Noi, `lencx/ChatGPT`, or another desktop AI wrapper is used by this repository. General publicly known architectural concepts may be independently implemented. See `docs/PROVENANCE.md`.

## License

Apache-2.0. See `LICENSE`.
