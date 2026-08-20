# Architecture

## System shape

```text
Human / caller
     |
     v
Model / planner
     |
     | proposes Action
     v
+------------------+
|  Policy Engine   |
+------------------+
     |
     +--> DENY --------------------+
     |
     +--> REQUIRE_APPROVAL --------+--> Receipt
     |                              |
     v                              |
Approval verifier                   |
     |                              |
     v                              |
Capability executor                 |
     |                              |
     v                              |
Host / sandbox ---------------------+
```

The model is not an authority source. It is a proposal source.

## Layers

### 1. Data contracts

`model.py` defines immutable Python records for actions, approvals, policy decisions, and receipts. `schemas/` provides language-neutral JSON contracts.

### 2. Policy kernel

`policy.py` maps a proposed action to one of:

- `allow`
- `require_approval`
- `deny`

The bootstrap grants only narrow read-only actions automatically. Known effectful actions require approval. Unknown actions are denied.

### 3. Approval verifier

An approval is valid only when:

- it is affirmative;
- its `action_id` exactly equals the proposed action ID.

Future phases may add expiry, actor identity, capability scope, nonce/session binding, and cryptographic signatures.

### 4. Executors

Executors are capability-specific adapters. The bootstrap contains only a structured-argv terminal executor and it is disabled by default.

A future desktop executor must not expose raw OS input APIs directly to a model. The model proposes semantic actions; the runtime normalizes and executes them.

### 5. Receipts

Every evaluated action returns a receipt, including denials and unsupported actions. Receipt identity is content-addressed. Wall-clock time is metadata and does not determine identity.

## Why not GUI-first?

Computer-use demos are easy to make impressive and hard to make trustworthy. This project therefore builds the boring substrate first: identities, policy, approvals, receipts, replay, and bounded executors.

## Dependency direction

```text
UI / provider adapters
        |
        v
agent orchestration
        |
        v
runtime
   |         |
 policy   executors
   |
   v
contracts
```

The authority core must not import provider SDKs or GUI frameworks.
