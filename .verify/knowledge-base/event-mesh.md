# Event mesh

## What it does

agent-nerves carries Autonomic events over a NATS JetStream bus. It checks broker connectivity, reports the state of the `AUTONOMIC` stream, evaluates JSON or WASM filter rules against messages, and manages multi-node NATS cluster routes.

## How it works

1. **Connectivity.** `agent-nerves ping` connects to NATS and measures round-trip time. `POST /nats/ping` does the same over HTTP. If the configured broker is unreachable, `serve` falls back to an embedded broker.
2. **Stream status.** `GET /jetstream/status` reports whether the `AUTONOMIC` stream is ready. `agent-nerves stream tail` follows `autonomic.>` subjects live.
3. **Filters.** `agent-nerves filter list` and `filter test` manage JSON and WASM rules in `~/.autonomic/filters/`. A test returns a match or no-match result for one message.
4. **Cluster.** `agent-nerves cluster init`, `status`, and `render-config` set up multi-node routes. `GET /cluster/status` reports topology and WireGuard state.
5. **Health.** `GET /health` returns status and uptime for supervisor probes.

## Where the code is

- Routes: `agent-nerves/agent-nerves/src/serve.rs` lines 60 to 65
- Filter logic: `agent-nerves/src/filter.rs`, with `filters_dir` and `evaluate_event`
- Filter rules route: `filter_rules` and `filter_test` in `serve.rs`

## How to use it

- `agent-nerves ping` to confirm the bus is reachable
- `agent-nerves stream tail` to watch live events
- `agent-nerves filter test` to check a rule before it routes real events
- `GET /jetstream/status` for stream readiness

## How it is verified

- `cargo test` for the whole crate (16 tests in the recorded baseline): config, filter, and cluster logic
- `route_drift.sh`: the README route table, including `/filter/rules` and `/filter/test`, matches the `.route()` calls in source

## Known limits

- The embedded-broker fallback can hide an unreachable NATS URL. A successful `ping` may be talking to the fallback. Check the broker URL the output reports.
- WireGuard status depends on the host. Tests must not assert a host-specific value.
- Filter rules are read from `~/.autonomic/filters/`. Tests must use a temporary directory.
- Live NATS checks need `nats-server` on the configured URL. None are in the baseline test run.
- The HTTP routes have no Hurl suite.

## Related

- Feature map: `agent-nerves/.verify/features/event-mesh.md`
- Event subscribers: [dashboard-panels](../../../agent-tui/.verify/knowledge-base/dashboard-panels.md)
