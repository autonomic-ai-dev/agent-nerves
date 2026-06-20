# Changelog

## [Unreleased]

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
