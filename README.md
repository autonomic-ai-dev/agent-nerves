# agent-nerves

**Distributed event bus — NATS connectivity, JetStream, and multi-node cluster coordination.**

`agent-nerves` is the messaging layer of the Autonomic AI ecosystem. It embeds or connects to NATS, bootstraps the shared AUTONOMIC JetStream stream, and exposes health APIs for the rest of the organism.

Works **standalone** (`agent-nerves ping`) or **integrated** (supervised by `autonomic start`, consumed by agent-muscle and agent-spine).

---

## Role in the ecosystem

| Integration | Details |
|-------------|---------|
| `autonomic start` | Starts `agent-nerves serve` first (broker + health on **3102**) |
| `agent-body-core` | Shared subjects: `autonomic.compute.job`, stream `AUTONOMIC` |
| Other organs | Publish/subscribe via `AUTONOMIC_NATS_URL` (default `nats://localhost:4222`) |

---

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-nerves/master/scripts/install.sh | bash
# or install everything:
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-body/master/scripts/install-all-organs.sh | bash
```

---

## Quick start (standalone)

```bash
agent-nerves status
agent-nerves ping
agent-nerves serve          # HTTP :3102, embedded nats-server if needed
agent-nerves stream tail    # tail JetStream on autonomic.>
```

---

## Commands

| Command | Description |
|---------|-------------|
| `serve` | HTTP daemon; embedded NATS + JetStream when unreachable |
| `ping` | Test NATS connectivity |
| `status` | Config and broker paths |
| `stream tail` | JetStream tail with core NATS fallback |
| `cluster init\|status\|render-config` | Multi-node NATS route config |
| `filter list\|test` | JSON / WASM event filters |

---

## HTTP API

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Daemon health |
| `POST /nats/ping` | NATS connectivity check |
| `GET /jetstream/status` | AUTONOMIC stream readiness |
| `GET /cluster/status` | Cluster / WireGuard status |
| `POST /filter/test` | Evaluate filter rules |

---

## Configuration

Section `[nerves]` in `~/.autonomic/config.toml` (default port **3102**).

Cluster state: `~/.autonomic/state/nerves/` · Filters: `~/.autonomic/filters/`

---

## Development

```bash
cargo test --release -p agent-nerves
cargo build --release -p agent-nerves
# WASM filters:
cargo build --release -p agent-nerves --features wasm
```

---

## License

MIT
