# Security Policy

This project is security-sensitive by design. Treat all authority expansion as potentially dangerous even when the feature appears convenient.

## Current status

The bootstrap is a development skeleton, not a hardened sandbox. Real shell execution is disabled by default. Do not run untrusted agents with `--execute` on sensitive hosts.

## Core security invariants

- Default deny for unknown capabilities.
- Effectful actions require approval.
- Approval binds to one exact action ID.
- Structured argv only for shell execution.
- No implicit network, secret, root, or filesystem authority.
- Every evaluated action produces a receipt.
- Unsupported actions do not silently execute through another path.
- Model-visible content is untrusted and may contain prompt injection.

## Threat classes

The threat model includes:

- malicious or compromised model output;
- prompt injection from webpages, documents, terminals, UI text, images, and clipboard data;
- confused-deputy behaviour;
- approval spoofing or replay;
- command/argument injection;
- capability escalation;
- secret leakage through logs or screenshots;
- unintended filesystem mutation;
- uncontrolled network effects;
- GUI state ambiguity;
- stale observations;
- infinite or runaway agent loops;
- receipt tampering.

See `docs/THREAT_MODEL.md` for the detailed model.

## Reporting

Please report security issues privately to the repository maintainers rather than publishing exploit details in a public issue before a fix is available.

## Security-development rule

A convenience feature is not accepted as a reason to bypass the policy kernel. If a capability cannot be represented, scoped, approved, executed, and receipted cleanly, it is not ready to ship.
