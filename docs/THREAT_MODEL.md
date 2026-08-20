# Threat Model

## Assets

Protect:
- OpenAI API credentials and account/project authority;
- user files and filesystem integrity;
- OBS credentials and broadcast/recording state;
- shell/process authority;
- desktop observation and future input authority;
- browser state;
- audit integrity;
- approval meaning;
- user attention and ability to revoke control.

## Adversaries / hostile inputs

Assume any of the following may be malicious or compromised:
- OpenAI/model output;
- webpage/document text;
- downloaded files;
- repository contents;
- clipboard data;
- OCR/UI text;
- worker-script output;
- tool output;
- prompt-injection content embedded in otherwise legitimate data.

The Ubuntu desktop portal/compositor and Linux kernel are part of the host trust base for desktop capture.

## Provider substitution

Risk: a configuration change redirects prompts, screenshots, or credentials to a third-party model endpoint while still looking “OpenAI-compatible.”

Mitigation:
- project is intentionally OpenAI-only;
- official origin fixed to `https://api.openai.com`;
- no arbitrary model `base_url` field;
- no generic provider registry;
- no Azure/OpenRouter/local/OpenAI-compatible adapters.

## Stolen/resold/shared credentials

Risk: a valid credential obtained elsewhere is reused across multiple local application instances or the app accepts browser/session artifacts designed for account sharing.

Important limitation: the runtime cannot infer whether a cryptographically valid credential is stolen or resold merely from its bytes.

Mitigation:
- ChatGPT browser cookies/session/access tokens are not accepted as application credentials;
- future OpenAI credentials use opaque `cred:openai.*` handles resolved by the OS secret broker;
- no ambient environment-variable credential channel;
- no raw credential CLI/config file path;
- one authority-bearing process per Ubuntu user session;
- future UI/agent roles share one authority daemon rather than spawning independent credential consumers.

Server-side OpenAI account/project controls remain outside this local runtime and are still required for abuse outside this application.

## Duplicate authority process

Risk: two local application instances independently execute OS actions or consume the same brokered credential.

Mitigation: atomic `0600` authority lock under validated `/run/user/<uid>`. Existing or stale lock fails closed.

## Confused deputy

Risk: OpenAI/model output convinces the runtime to use authority the model itself does not possess.

Mitigation: capability broker, default deny, explicit approvals, scoped credential broker.

## Action substitution after approval

Risk: an action changes after the human approved it.

Mitigation: normalize before approval, immutable public action API, approval bound to content-derived `action_id`. OBS endpoint identity is included in the action.

## Approval replay

Risk: a previously approved action is reused in another session or later context.

Current state: exact action binding exists. Session/nonce/expiry binding remains required before autonomous operation.

## Screenshot disclosure

Risk: a read-only screenshot contains passwords, private messages, API keys, browser sessions, personal files, or other sensitive data.

Mitigation in PR #3:
- capture through the user-mediated XDG Screenshot portal on Ubuntu GNOME Wayland;
- no model-controlled capture target/path arguments;
- no credentials on `screen.capture` actions;
- portal result must be a local `file://` URI;
- screenshot input capped at 64 MiB and validated as PNG;
- receipts contain hash/size/dimensions only;
- raw screenshot bytes use zeroizing memory and are dropped after evidence derivation;
- portal URI/path is never serialized into receipts;
- temporary portal artifacts are deleted best-effort only when under known runtime/temp locations.

Raw screenshot forwarding to OpenAI remains gated until redaction and OpenAI credential/network controls land.

## Desktop API bypass

Risk: a computer-use feature bypasses Wayland portal authority using private GNOME APIs, X11 global scraping, or shell helpers.

Mitigation: Ubuntu 26.04 primary path is XDG Desktop Portal. GNOME Shell Eval/private Mutter APIs, `xdotool`, `wmctrl`, and X11 root capture are not primary mechanisms.

## Secret injection into model-visible data

Risk: raw credentials appear in action arguments, receipts, logs, CLI text, or TUI state.

Mitigation: opaque handles, secret-shaped key rejection, non-serializable secret store, redacted debug, explicit tests/documentation.

## Secret residue in memory

Risk: a credential or screenshot remains in freed/reused memory.

Mitigation: zeroizing containers and minimized lifetime. Rust memory safety alone is explicitly not considered secret erasure.

## Ambient environment leakage

Risk: a child command inherits `OPENAI_API_KEY`, SSH agent variables, cloud tokens, cookies, or other host environment data.

Mitigation: process executor uses `env_clear()` and supplies only a minimal fixed environment. OpenAI credentials will not be sourced from ambient environment variables.

## Output exfiltration through receipts

Risk: a process, OBS server, or desktop backend returns credentials/private data and the audit log persists it.

Mitigation:
- process receipts store hashes/byte counts rather than stdout/stderr;
- OBS arbitrary strings are hashed before receipts;
- desktop receipts store image hashes/metadata only;
- raw payloads are not audit evidence.

## Shell escape

Risk: structured argv is converted back into unrestricted shell text.

Mitigation: no raw shell-string contract; known interpreter command-string forms are denied; no `shell=true` equivalent.

## Privilege escalation

Risk: the agent invokes sudo/doas/su or privileged helpers.

Mitigation: common escalation commands are denied now; OS-level sandbox and privilege boundary remain required.

## Network-policy bypass

Risk: a shell program opens arbitrary sockets even though network authority was not intended.

Current state: not solved comprehensively by command-name filtering. Real untrusted shell execution remains gated until network namespace/policy enforcement lands.

OBS is a narrow exception: its adapter constructs IPv4 loopback endpoints internally and does not accept arbitrary hosts.

Future OpenAI egress will be separately restricted to the official OpenAI origin.

## TUI spoof/confusion

Risk: model-controlled text tricks the human into granting unrelated authority.

Mitigation target: render normalized action fields from trusted structures, visually separate untrusted content, exact action identity, explicit risk/capability labels, no model-controlled keybindings.

## Kill-switch failure

Risk: the user requests revoke-all but running processes/effectors continue.

Current state: TUI revocation is not yet fully authoritative over every active effector. Runtime-authoritative revoke-all and process/session termination remain gates before autonomous loops.

## Security posture

The current system establishes authority contracts, OBS structured control, single-instance local authority, OpenAI-only provider constraints, and one-shot Ubuntu Wayland screenshot observation. It does not yet claim full containment or autonomous computer-use readiness.
