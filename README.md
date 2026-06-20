# agent-nerves

**Distributed event bus for autonomic agents — NATS connectivity and messaging.**

agent-nerves provides NATS connectivity for the autonomic-ai-dev ecosystem. It tests broker connectivity and serves as the event bus for inter-agent communication.

---

## Why agent-nerves?

| Problem | agent-nerves answer |
|---------|-------------------|
| "Is my NATS broker running?" | **NATS ping** — tests connectivity and returns server info |
| "How do agents discover each other?" | **Event bus** — pub/sub messaging via NATS |

## Commands

| Command | Description |
|---------|-------------|
| `agent-nerves serve` | Start daemon (future: embedded broker) |
| `agent-nerves ping` | Test NATS server connectivity |
| `agent-nerves status` | Show config and NATS URL |

---

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-nerves/master/scripts/install.sh | bash
```

## Development

```bash
cargo build --release -p agent-nerves
cargo test --release -p agent-nerves
```

## License

MIT
