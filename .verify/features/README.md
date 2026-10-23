# agent-nerves features

## Baseline preconditions

- Rust toolchain matching `Cargo.lock`.
- Live checks need `nats-server` reachable on the broker URL from config.
- Filter tests must use a temporary `~/.autonomic/filters/` directory.

## Driving conventions

- Unit tests run through `cargo test`. Logs go to `.verify/artifacts/<RUN_ID>/`.

## Proof standards

- A feature is proven by its unit test and, for HTTP routes, a request that returns the documented status and body.

## Feature index

| Feature file | Sub-features | Primary surface |
|---|---|---|
| `event-mesh.md` | 5 | `serve`, `ping`, `stream tail`, `filter`, `cluster`, HTTP `/nats/ping`, `/jetstream/status`, `/cluster/status` (no HTTP filter route exists) |
