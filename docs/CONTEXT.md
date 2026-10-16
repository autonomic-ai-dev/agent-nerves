# agent-nerves — Context for agents

NATS JetStream service mesh bridge for Autonomic. Part of [agent-body](https://github.com/autonomic-ai-dev/agent-body).

| Field | Value |
|-------|-------|
| Role | Service mesh |
| Binary | `agent-nerves` |
| HTTP | `:3102` (NATS `:4222` / monitor `:8222`) |
| Version | 0.7.0+ |

## When to use

Bootstrap `AUTONOMIC` stream, connect organs to JetStream, WASM/JSON filters, multi-node cluster configs. Start after `nats-server -js`.

## Key verbs

- `agent-nerves serve`
- Stream bootstrap + reconnect backoff
- Backpressure / priority subjects

## Verification

Mesh health is operational. Record app/test verification with `agent-brain verify --` in the project under change.

## CI

Path filter on `agent-nerves/**` + Cargo/Makefile/scripts/`.github`. Docs-only PRs skip compose.

## Related

Body (start order) · all organs publishing/consuming events
