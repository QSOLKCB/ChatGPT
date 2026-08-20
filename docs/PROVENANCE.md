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

Rust dependencies are normal upstream crates selected for narrow functions such as CLI parsing, terminal rendering, serialization, hashing, errors and zeroization. Dependencies must be license-reviewed before release packaging.

## Naming

This repository is an independent community project and is not affiliated with or endorsed by OpenAI. ChatGPT is a trademark of OpenAI.
