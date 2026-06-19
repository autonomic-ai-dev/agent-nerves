# agent-nerves

**Distributed event bus for AI agents — asynchronous pub/sub messaging across multi-node clusters using nats.rs.**

agent-nerves handles asynchronous messaging across the organism. Built entirely in Rust on top of `nats.rs`, it allows the ecosystem to scale from a single laptop to a massively distributed cluster of nodes without changing a single line of orchestrator code.

Rust is the nervous system; NATS is the synapse.

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-nerves/master/scripts/install.sh | bash -s -- --global
agent-nerves broker start --daemon
```

**MCP is live immediately** — the agent can subscribe to channels to listen for background job completions.

---

## Why agent-nerves?

Monolithic AI agents usually run entirely in a single Python `while` loop. This design breaks completely at scale.

1. **Synchronous Blocking:** If the agent needs to compile a 50GB C++ project, the entire `while` loop freezes for an hour waiting for the API to return.
2. **State Loss:** If the machine restarts mid-workflow, all context is lost.
3. **Single Machine Limit:** A laptop cannot fine-tune an LLM while simultaneously running a 50-node Selenium test suite.

**agent-nerves fixes this with an embedded message broker:**

| Problem | agent-nerves answer |
|---------|-------------------|
| "Synchronous REST APIs freeze the agent" | **Pub/Sub Communication** — replaces blocking HTTP calls with resilient, decoupled event streams. |
| "I want to run heavy compute on a different server" | **Multi-Node Clusters** — allows `agent-muscle` to run remotely while `agent-spine` orchestrates from your laptop. |
| "A crash lost my workflow state" | **Event Replay** — uses native JetStream to keep a short-term buffer of state changes for immediate failure recovery. |

---

## Architectural Deep Dive

`agent-nerves` embeds the NATS server directly into a Rust binary. You do not need to install Docker or Java to run the message broker.

### 1. Zero-Config Cluster Discovery
When `agent-nerves` boots on your laptop, it acts as the primary seed node.
- If you start `agent-muscle` on a remote GPU server and pass it the `AGENT_NERVES_SEED_URL`, they instantly form a cluster.
- The orchestrator (`agent-spine`) simply publishes an event: `workload.compile.cxx`. It has no idea *where* that compute happens.

### 2. JetStream Persistence
Unlike Redis Pub/Sub which drops messages if a consumer goes offline, `agent-nerves` persists events.
- If the agent laptop closes its lid (sleeps), remote compilation jobs finish and queue their success events.
- When the laptop wakes up, `agent-spine` consumes the backlog and resumes instantly.

---

## Complete Setup (Copy & Paste)

### 1. Install the binary

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-nerves/master/scripts/install.sh | bash -s -- --global
```

### 2. Configuration (`~/.agent_nerves/config.yaml`)

```yaml
broker:
  port: 4222
  jetstream: true
  max_memory_mb: 256

cluster:
  name: "autonomic-local"
  tls_required: true
```

### 3. Verify

```bash
agent-nerves version
agent-nerves status
```

---

## Commands

| Command | Description |
|---------|-------------|
| `agent-nerves broker start` | Start the local NATS message broker |
| `agent-nerves stream tail` | Listen to live agent workflow events (CLI Tail) |
| `agent-nerves cluster info` | View connected organs/nodes |

---

## Development

```bash
cargo test --release -p agent-nerves
cargo build --release -p agent-nerves
```

## License
MIT
