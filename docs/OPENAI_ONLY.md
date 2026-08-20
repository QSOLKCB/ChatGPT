# OpenAI-Only Provider Boundary

## Decision

This project is intentionally **OpenAI-only**.

The model/provider side of the application may integrate with the official OpenAI API at:

```text
https://api.openai.com
```

The Responses endpoint is fixed to:

```text
https://api.openai.com/v1/responses
```

There is no generic provider registry and no caller-configurable model API origin.

## Why

This workstation is a security-sensitive authority broker. Supporting arbitrary model providers or OpenAI-compatible endpoints would broaden:

- credential classes;
- network destinations;
- authentication semantics;
- data-handling policies;
- provider-specific tool behavior;
- threat-model and supply-chain surface;
- endpoint-substitution risks.

The project deliberately gives up provider portability to keep this boundary narrow and inspectable.

## Contract

OpenAI provider configuration may contain:

- an opaque `cred:openai.*` credential handle;
- an OpenAI model identifier;
- reviewed request options that do not alter the API origin.

Provider configuration must not contain:

- `provider` / `vendor` selection;
- arbitrary `base_url`, `endpoint`, `host`, or proxy-to-model-service fields;
- third-party API credentials;
- local model socket paths;
- compatibility mode for OpenAI-shaped third-party APIs.

## Explicitly unsupported

- Anthropic / Claude;
- Google Gemini;
- xAI / Grok;
- Azure OpenAI;
- Amazon Bedrock;
- OpenRouter;
- Ollama;
- LM Studio;
- arbitrary OpenAI-compatible servers;
- generic multi-provider abstraction layers.

This restriction is architectural, not a temporary missing-feature list.

## Future change rule

Changing the provider boundary requires an explicit project-level decision, dedicated threat-model update, credential/network redesign, and focused review. It must never arrive incidentally inside an unrelated PR.
