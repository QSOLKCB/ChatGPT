# Threat Model

## Assets

The runtime may eventually gain access to:

- user files;
- desktop contents;
- browser sessions;
- terminals and processes;
- network services;
- creative applications;
- credentials through brokers;
- source repositories;
- media and research data.

The policy kernel, approval records, and receipt history are themselves security assets.

## Adversaries and failure sources

### Malicious external content

A webpage, file, repository, terminal output, subtitle, image, document, or UI element may contain instructions designed to redirect the agent.

Rule: **observed content is data, not authority**.

### Compromised or mistaken model

The model may hallucinate state, misunderstand a task, choose a destructive command, or intentionally attempt escalation.

Rule: model intent does not bypass policy.

### Confused deputy

A legitimate capability may be used for a different purpose than the user authorized.

Mitigation direction: narrow action schemas, exact approval binding, capability roots, session budgets, and receipts.

### Approval replay

An old approval could be reused against a new action.

Bootstrap mitigation: approvals bind to the deterministic `action_id`. Future mitigation: session binding, expiry, nonce, and signature.

### Shell injection

Raw shell strings make authority boundaries ambiguous.

Bootstrap mitigation: argv arrays, `shell=False`, default-disabled executor, policy inspection.

### Receipt tampering

Audit records may be altered after execution.

Bootstrap mitigation: content-addressed receipt IDs. Future mitigation: append-only chained storage and replay verification.

### Secret leakage

Secrets can leak through environment variables, stdout/stderr, screenshots, logs, browser pages, or generated receipts.

Bootstrap rule: do not intentionally capture or persist secrets. Future work requires redaction, opaque secret handles, and brokered credentials.

### GUI ambiguity

A screenshot can be stale, controls can move, focus can change, and visual interpretation can be wrong.

Future mitigation: observation/action sequence numbers, geometry receipts, application identity, accessibility APIs where possible, and re-observation after effects.

### Runaway autonomy

A long-running agent may repeat actions, consume resources, or compound an early error.

Future mitigation: action budgets, wall-clock budgets, repeated-action suppression, checkpoints, revocation, and emergency stop.

## Trust boundaries

```text
UNTRUSTED: model text, web content, files, UI text, OCR, tool output
BOUNDARY:  parser -> schema -> policy -> approval verifier
TRUSTED TCB: minimal policy kernel + executor broker + receipt writer
EFFECT DOMAIN: host OS / sandbox / network / applications
```

Keep the trusted computing base small.

## Bootstrap limitations

The destructive-command filter is deliberately described as a **floor**, not a complete command security system. It catches a few obvious catastrophic forms but cannot reason about arbitrary programs. Real safety must come from OS isolation and narrow capability executors, not an ever-growing blacklist.
