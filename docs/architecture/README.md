# agent-nerves architecture documentation

## Design goals

agent-nerves provides the **async messaging backbone** for the Autonomic ecosystem. Every organ that needs non-blocking communication (muscle computing jobs, immune scanning results, spine domain events) goes through nerves.

### Why not direct NATS access?

Every organ *could* connect to NATS directly. But that would mean:

- Each organ needs NATS client configuration (URL, credentials, TLS)
- Reconnection logic is duplicated across organs
- Stream creation is manual (subjects must exist before publishing)
- No visibility into who is publishing what

agent-nerves centralizes all of this behind an HTTP API, so organs only need to know the nerves URL (`http://localhost:3102`).

### Key design decisions

| Decision | Rationale |
|----------|-----------|
| **HTTP API, not NATS client lib** | Organs speak HTTP; nerves translates to NATS. This decouples organs from NATS protocol changes. |
| **Embedded NATS fallback** | If `nats-server` is not running, nerves starts an embedded NATS server. Dev laptops work without installing NATS. |
| **JetStream, not core NATS** | At-least-once delivery guarantees prevent message loss during organ restarts. |
| **WASM event filters** | JSON rules cover most filtering needs; WASM handles complex routing logic without modifying nerves. |

### Stream topology

```
AUTONOMIC stream (JetStream):
  subjects:
    autonomic.compute.job.>      # muscle compute requests
    autonomic.scan.request.>     # immune scan requests
    autonomic.scan.result.>      # immune scan results
    autonomic.spine.event.>      # spine domain events
    autonomic.muscle.train.>     # training requests
```

### Cluster mode

For multi-machine deployments, nerves generates NATS route configurations with WireGuard tunnel templates:

```
agent-nerves cluster init --peers 3
  → Generates:
    - nats-routes.conf  (NATS route definitions)
    - wireguard.conf    (WireGuard interface config)
    - cluster.json      (Cluster state for operator review)
```

### Alternatives considered

| Option | Why rejected |
|--------|-------------|
| **Direct NATS from every organ** | Duplicated config, reconnection logic, no centralized observability |
| **RabbitMQ instead of NATS** | NATS is simpler (no exchange/binding model), faster, and has JetStream for durability |
| **Redis pub/sub** | No durability, no wildcard subjects, no consumer groups |
| **gRPC streams** | Requires generated protobufs per message type; NATS is schemaless |
