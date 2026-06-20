# Changelog

## [Unreleased]

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
