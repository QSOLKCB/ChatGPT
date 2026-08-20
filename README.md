# ChatGPT

A clean-room, Linux-native agent workstation for giving AI systems **controlled, inspectable access** to computers, terminals, applications, and complex workflows.

> **Independent community project.** Not affiliated with or endorsed by OpenAI. ChatGPT is a trademark of OpenAI.

## Mission

Build a local agent runtime where intelligence and authority are deliberately separate:

```text
model proposes -> policy evaluates -> approval gates -> executor acts -> receipt records -> model observes
```

The project begins with Linux and a small Python core. GUI automation, media workflows, browser control, and richer provider adapters come later, only after the authority and audit layers are trustworthy.

## Design principles

1. **Capability != authority.** A capable model does not automatically receive permission to act.
2. **Default deny.** Unknown actions are rejected.
3. **Observation before mutation.** Read-only capabilities are easier to grant than effectful ones.
4. **Explicit approval for effects.** Shell execution, input injection, application launch, and writes require approval.
5. **Receipts, not vibes.** Every evaluated action yields a canonical receipt.
6. **Deterministic identities.** Action and receipt identities are hashes of canonical JSON payloads.
7. **No hidden shell.** The first executor accepts argv arrays, not shell strings.
8. **Fail closed.** Unsupported or malformed actions do not silently degrade into execution.
9. **Clean-room implementation.** No source code is copied from third-party AI desktop wrappers.
10. **Keep the core boring.** Small contracts and testable state transitions beat GUI spectacle.

## Current bootstrap

The first implementation skeleton provides:

- typed action, approval, policy-decision, and receipt records;
- canonical JSON hashing for action and receipt identity;
- a default-deny policy engine;
- hard rejection of a small set of obviously destructive shell invocations;
- explicit approval binding to one exact action identity;
- a shell executor that is **disabled by default**;
- a runtime that records denied, approval-required, simulated, completed, failed, and unsupported outcomes;
- JSON Schemas for actions, approvals, and receipts;
- standard-library unit tests;
- CI on supported Python versions.

This is intentionally **not yet** a desktop-control product. See `ROADMAP.md`.

## Quick start

```bash
git clone https://github.com/QSOLKCB/ChatGPT.git
cd ChatGPT
python -m venv .venv
source .venv/bin/activate
python -m pip install -e .
python -m unittest discover -s tests -v
```

Inspect a proposed action without executing it:

```bash
qsol-chatgpt policy '{"kind":"shell.exec","args":{"argv":["printf","hello\\n"]}}'
```

Run through the full runtime in simulation mode:

```bash
qsol-chatgpt run '{"kind":"shell.exec","args":{"argv":["printf","hello\\n"]}}' --approve
```

Real command execution is opt-in:

```bash
qsol-chatgpt run '{"kind":"shell.exec","args":{"argv":["printf","hello\\n"]}}' --approve --execute
```

## Repository map

```text
docs/                  architecture, threat model, computer-use contract, provenance
schemas/               language-neutral machine contracts
src/qsol_chatgpt/      first runtime implementation
tests/                 contract and policy tests
.github/workflows/     CI
README4AI.md            machine-oriented project summary
AGENTS.md               machine contributor rules
ROADMAP.md              staged implementation plan
SECURITY.md             security policy and invariants
CONTRIBUTING.md         contribution and clean-room rules
```

## Safety boundary

The bootstrap policy is a **minimum safety floor, not a complete sandbox**. Do not expose the executor to untrusted models, secrets, privileged accounts, or production machines. Later phases must add OS-level isolation, scoped filesystem roots, network policy, credential brokers, desktop capability brokers, and stronger approval semantics before broad autonomous use.

## License

Apache-2.0. See `LICENSE`.
