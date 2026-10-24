# event-mesh

Summary: agent-nerves carries Autonomic events over a NATS JetStream bus, exposes connectivity and stream status over HTTP and MCP, and evaluates JSON or WASM filter rules against messages.

## Sub-features

- `nats-connectivity`: Checks the broker and reports latency.
  - Intent: Let operators confirm the bus is reachable before depending on it.
  - Behaviour: `agent-nerves ping` connects to NATS and measures round-trip time. `POST /nats/ping` does the same over HTTP.
  - Outcome: A latency value on success. Falls back to an embedded broker when the configured one is unreachable, as `serve` documents.
- `jetstream-stream`: Reports readiness of the `AUTONOMIC` stream.
  - Intent: Show that events are being persisted.
  - Behaviour: `GET /jetstream/status` reports stream state. `agent-nerves stream tail` follows `autonomic.>` subjects.
  - Outcome: Stream state in the response and live messages in the tail.
- `filter-rules`: Lists and tests event filters from the command line.
  - Intent: Let operators check a rule before it routes real events.
  - Behaviour: `agent-nerves filter list` and `filter test` manage JSON and WASM rules from `~/.autonomic/filters/`.
  - Outcome: A match or no-match result for the message. The README also lists `POST /filter/test`, but no such route exists in `agent-nerves/src/serve.rs`, so the filter is CLI-only.
- `cluster-topology`: Renders and reports multi-node NATS routes.
  - Intent: Configure a cluster without hand-editing route lists.
  - Behaviour: `agent-nerves cluster init`, `status`, and `render-config`. `GET /cluster/status` reports topology and WireGuard state.
  - Outcome: A rendered config and a topology report.
- `daemon-health`: Reports daemon health and uptime.
  - Intent: Give supervisors a probe.
  - Behaviour: `GET /health`.
  - Outcome: A JSON status with uptime.

## Code

- `agent-nerves`

## How to get to it (user POV)

- CLI: `agent-nerves serve`, `serve-mcp`, `ping`, `status`, `stream tail`, `cluster …`, `filter …`
- HTTP: `GET /health`, `POST /nats/ping`, `GET /jetstream/status`, `GET /cluster/status` (the four routes in `agent-nerves/src/serve.rs` lines 60-63)

## Verification Suites (Targeted Test Commands)

- **Unit Tests:** `cargo test` (whole crate, 16 tests in the baseline)
- **Functional API Tests (Hurl):** not yet written for the HTTP routes
- **Allowlisted Verification:** entries in `.verify/verify.toml`
- **Preconditions:** Rust toolchain. Live NATS checks need `nats-server` reachable on the configured URL.

### Verification Steps

1. **nerves-unit-tests** (sub-features: all)
   - Request: `cargo test`
   - Why: Proves config, filter, and cluster logic.
   - Expected: 16 passed, 0 failed (baseline).
   - Impact: A failure means routing or filter behaviour changed. Fix the cause.

## Gotchas

- The embedded-broker fallback hides an unreachable NATS URL. A `ping` that succeeds may be talking to the fallback, not the configured broker. Check the reported broker URL.
- WireGuard status in `GET /cluster/status` depends on the host. Tests must not assert a host-specific value.
- Filter rules are read from the user's `~/.autonomic/filters/` directory. Tests must use a temporary directory.
