# Contributing

Contributions are welcome when they preserve the project's authority and provenance boundaries.

## Before coding

Read:

- `README4AI.md`
- `AGENTS.md`
- `docs/ARCHITECTURE.md`
- `docs/THREAT_MODEL.md`

## Clean-room requirement

Do not contribute copied, translated, mechanically reproduced, or source-derived implementation code from third-party AI desktop wrappers. If you studied another implementation's source to reproduce a particular internal mechanism, disclose that fact before contributing so maintainers can assess provenance risk.

General ideas, standards, documented protocols, operating-system APIs, and independently designed interfaces may be implemented normally.

## Tests

Run:

```bash
python -m unittest discover -s tests -v
```

New effectful capabilities need at minimum:

1. denial test for malformed or forbidden input;
2. missing-approval test;
3. mismatched-approval test;
4. happy-path test using a fake or disabled executor;
5. receipt test.

## Pull requests

Keep the authority core small. Separate policy changes from UI polish where practical. Explain any new capability, why it is needed, its abuse cases, and its revocation story.
