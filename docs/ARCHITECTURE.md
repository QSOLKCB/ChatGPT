# Architecture

## System shape

```text
                         HUMAN
                           |
                           v
                    +--------------+
                    | Ratatui TUI  |
                    | control plane|
                    +------+-------+
                           |
                    grant / revoke
                           |
UNTRUSTED                  v                         EFFECTS
model/provider ---> ProposedAction ---> +--------------------------+
                                       | Rust authority core       |
                                       | normalize -> policy       |
                                       | -> approval -> dispatch   |
                                       +-----+---------------+-----+
                                             |               |
                                             v               v
                                       secret broker     capability adapter
                                             |               |
                                             |               v
                                             |          host / sandbox
                                             |               |
                                             +-------+-------+
                                                     v
                                                  Receipt
```

The model is a proposal source, not an authority source. The TUI is a human control plane, not a bypass around runtime policy.

## Trusted computing base

The intended trusted core is deliberately small:
- `contracts.rs`
- `policy.rs`
- `runtime.rs`
- `receipts.rs`
- `secrets.rs`
- capability-specific brokers/executors

Provider SDKs, model clients, browser automation, media tools, Python workers, webpages, clipboard contents, OCR, downloaded files, and model output are outside the trust boundary.

## Proposal vs normalized action

Untrusted JSON enters as `ProposedAction`. Normalization validates contract version, rejects raw-secret-shaped fields, validates/deduplicates credential handles, and derives a content identity. The resulting `Action` exposes only immutable borrows through its public API.

This removes the Python prototype failure mode where a mutable dictionary could theoretically change after identity/approval calculation.

## Policy and approval

Policy returns one of:
- `allow`;
- `approval_required`;
- `deny`.

Unknown capabilities deny by default. An approval authorizes exactly one normalized `action_id`; action substitution therefore changes identity and invalidates the approval.

Future approvals add session binding, nonces, expiry and signatures.

## Capability executors

Executors receive normalized actions only after policy/approval gates. The bootstrap shell executor:
- accepts structured argv only;
- denies shell-interpreter `-c` escapes at policy level;
- clears inherited environment;
- injects no credentials;
- records output hashes/sizes rather than raw output;
- zeroizes captured stdout/stderr buffers after hashing.

Network/filesystem/process isolation remains incomplete, so real execution is explicitly a developer bootstrap capability rather than a production sandbox.

## Credentials

Machine-visible contracts contain only opaque `CredentialHandle` values. Secret values live in a non-serializable broker/store. A provider or executor receives a secret only through a future narrowly scoped broker operation.

The TUI never owns or renders raw secret values.

## Why a TUI first

The authority console needs reliability and inspectability more than decorative chrome. Ratatui provides a small local control surface that works in terminals and remote shells while keeping the GUI/toolkit attack surface outside the authority kernel.

Future GUI frontends may speak a narrow local protocol to the same Rust core.

## Dependency direction

```text
frontends/providers/workers
          |
          v
   proposal interface
          |
          v
 contracts -> policy -> runtime -> capability broker -> OS/sandbox
                        |
                        +-> receipt/audit
                        +-> secret broker
```

No arrow points from the policy core upward into a provider SDK.
