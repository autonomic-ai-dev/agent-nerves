# agent-nerves

`agent-nerves` connects the Autonomic daemons to a shared NATS JetStream broker. It creates the `AUTONOMIC` stream on startup, reports broker health, and applies event filters before messages are published.

Docs: [autonomic-ai-dev.vercel.app](https://autonomic-ai-dev.vercel.app)

## Quick start

```bash
brew tap autonomic-ai-dev/tap && brew install agent-nerves
agent-nerves ping
agent-nerves serve
```

## Commands

| Command | What it does |
|---------|--------------|
| `agent-nerves serve` | Runs the HTTP daemon on port 3102 and creates the `AUTONOMIC` stream |
| `agent-nerves serve-mcp` | Runs the MCP stdio server only |
| `agent-nerves ping` | Checks the NATS connection and latency |
| `agent-nerves status` | Shows the broker URL, cluster state, and filter registry |
| `agent-nerves stream tail` | Tails messages on `autonomic.>` |
| `agent-nerves cluster init\|status\|render-config` | Sets up a multi-node NATS cluster |
| `agent-nerves filter list\|test` | Lists filter rules and tests one against a subject and payload |

## HTTP API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Daemon health and uptime |
| `POST` | `/nats/ping` | NATS connectivity check |
| `GET` | `/jetstream/status` | `AUTONOMIC` stream readiness |
| `GET` | `/cluster/status` | Cluster topology and WireGuard status |
| `GET` | `/filter/rules` | Loaded filter rules |
| `POST` | `/filter/test` | Evaluates a subject and payload against the filter rules |

## Configuration

Settings live in the `[nerves]` section of `~/.autonomic/config.toml`. The default port is 3102. Filter rules live in `~/.autonomic/filters/`, and NATS persistence lives in `~/.autonomic/broker/`.

## Develop

```bash
cargo test -p agent-nerves
cargo clippy -p agent-nerves -- -D warnings
```

Feature map and knowledge base: `.verify/features/` and `.verify/knowledge-base/`.
