# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.8] - 2026-06-27

### Changed

- README refactored to document technical architecture and cloud-native platform messaging

## [0.6.7] - 2026-06-23

### Added

- Global `--progress` CLI flag for structured ProgressTree output (also `AGENT_PROGRESS=1`)

## [0.6.6] - 2026-06-21

### Fixed

- CI builds use git `agent-body-core` dependency instead of a local path

## [0.6.5] - 2026-06-21

### Changed

- Embedded broker uses secure `nats-server.conf` from `~/.autonomic/broker/` when credentials exist
- NATS ping/stream tail use `agent_body_core::connect_nats()` for per-organ ACL auth
- `async-nats` 0.39 aligned with `agent-body-core` 0.3.3

## [0.6.4] - 2026-06-21

### Added

- `agent-nerves update [--force]` — self-update subcommand that checks GitHub releases, compares versions, and downloads the latest binary

## [0.6.3] - 2026-06-21

### Added

- `agent-nerves log <name> [--follow] [--list]` — read daemon logs from the supervisor log directory

## [0.6.2] - 2026-06-21

### Fixed

- agent-spine registration is now non-fatal — daemon starts even without spine available

## [0.6.1] - 2026-06-20

### Added

- `--version` CLI flag (`2eea97b`)
- Mermaid architecture charts in README (`386e3d4`)

### Changed

- Professional README with standalone and integrated usage (`f533d41`)

## [0.6.0] - 2026-06-20

### Added

- **Cluster coordination** — `cluster init|status|render-config`, leader election state at `~/.autonomic/state/nerves/cluster.json`
- **NATS cluster routes** — embedded broker writes `nats-cluster.conf` with JetStream + route peers when `cluster.enabled`
- **WireGuard probe** — cluster status reports whether the configured tunnel interface is up
- **Event filters** — JSON rules + optional WASM modules (`filter(i32,i32)->i32`) under `~/.autonomic/filters`
- **CLI** — `filter list|test`, `stream tail --filter`, HTTP `/cluster/status`, `/filter/test`, `/filter/rules`

## [0.5.1] - 2026-06-20

### Added

- **`stream tail`** — JetStream tail on AUTONOMIC with `--from new|all`, stream stats, and core NATS fallback

## [0.5.0] - 2026-06-20

### Added

- **JetStream bootstrap** — ensures AUTONOMIC stream with explicit ACK consumers and Msg-Id dedup
- **Embedded broker** — auto-starts `nats-server -js` when NATS is unreachable (`nats.embedded`, default on)
- **`/jetstream/status`** — reports stream readiness and dedup/ack settings

### Changed

- Version bumped from `0.4.0` to `0.5.0`

## [0.4.0] - 2026-06-20

### Added

- **Unified config** — loads from `~/.autonomic/config.toml` via `agent-body-core::organ_config::load("nerves")`
- **Global broker dir** — NATS/JetStream persistence defaults to `~/.autonomic/broker/`

### Changed

- Version bumped from `0.3.0` to `0.4.0`

## [0.3.0] - 2026-06-20

### Added

- **Stream tailer** — `agent-nerves stream <subject>` subscribes to a NATS subject and prints formatted messages to stdout (like `tail -f` for NATS)
- **Raw mode** — `--raw` flag prints unformatted message payloads

### Changed

- Version bumped from `0.2.0` to `0.3.0`
- Spine capabilities include `nats:stream`

## [0.2.0] - 2026-06-20

### Added

- **HTTP daemon** — `agent-nerves serve` now starts an axum HTTP server with `/health` and `/nats/ping` endpoints
- **Agent-spine integration** — registers with agent-spine event bus on startup, heartbeats every 30s
- **Config extended** — `server.port` (default 3102) and `spine.url` (default `http://localhost:3100`) settings

### Changed

- Version bumped from `0.1.0` to `0.2.0`

## [0.1.0] - 2026-06-20

### Added

- **Initial project scaffold** — workspace, crate, config with auto-created config.yaml
- **NATS connectivity** — async-nats client with ping/connectivity check
- **CLI** — `agent-nerves serve` (daemon placeholder), `ping` (test NATS connection), `status` (config info)
- **CI pipeline** — test + build + release workflows
