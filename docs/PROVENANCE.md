# Provenance

## Clean-room origin

`QSOLKCB/ChatGPT` is an independent implementation. The project was conceived after discussing the general idea of Linux AI desktop wrappers and agent-controlled workstations, but no source code from Noi, `lencx/ChatGPT`, or another desktop AI wrapper is copied, translated, mechanically reproduced, or used as an implementation base.

The architecture is independently defined around general concepts:
- capability-based security;
- default-deny policy;
- human approval gates;
- content-addressed records;
- terminal user interfaces;
- OS process and desktop APIs;
- opaque credential brokers;
- deterministic audit receipts.

## Dependency provenance

Direct Rust dependency review is recorded in `docs/DEPENDENCIES.md`. A dependency must not be added to `Cargo.toml` until its license compatibility, role, trust-boundary placement, and standard-library/current-dependency rationale are recorded there.

The current direct dependency set is limited to permissively licensed upstream crates. Transitive dependencies must additionally be locked and reviewed by automated license/advisory tooling before release packaging.

## Naming

This repository is an independent community project and is not affiliated with or endorsed by OpenAI. ChatGPT is a trademark of OpenAI.
