# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0-beta.3.2] - 2026-10-18

### Added
- **Cloud Broker Leaf Bridge & Global NGS Mesh (ABH-14)**: Added `[broker.cloud]` configuration in `~/.autonomic/config.toml` and connection builder for remote JetStream cloud leaves with TLS and JWT/NKey authentication.
- **CI Skip Path Optimizations**: Enhanced `.github/workflows/ci.yml` to skip pipeline execution when changes only affect `CHANGELOG.md`, `CONTEXT.md`, and markdown documentation files.

### Changed
- Bumped version to `v1.0.0-beta.3.2`.

## [1.0.0-beta.3.1] - 2026-10-17

### Added
- **`events.heart.usage` JetStream Stream** — Added dedicated file-backed JetStream stream for asynchronous model usage telemetry broadcasts.
- **Model Usage Publisher & Subscriber Helpers** — Provided type-safe helpers for publishing and streaming `UsageRecordedEvent` payloads across the mesh.
- **Verification Playbook Suite** — Added `.verify/run.sh` test suite validating stream initialization and message propagation.

### Changed
- Bumped version to `v1.0.0-beta.3.1`.

## [1.0.0-beta.3] - 2026-10-17

### Changed
- Bumped version to `v1.0.0-beta.3` and updated `agent-body-core` dependency to `v1.0.0-beta.3`.
- Relocated comprehensive `CONTEXT.md` to repository root and aligned documentation hub links.

## [1.0.0-beta.2] - 2026-10-16

### Added
- Official Homebrew tap formula support (`brew install agent-nerves`).

### Changed
- Consolidated architecture documentation into centralized ecosystem hub.

## [1.0.0-beta.1] - 2026-10-16

### Changed
- Unified ecosystem versioning to 1.0.0-beta.1
- Pinned `agent-body-core` dependency to `v1.0.0-beta.1`
- Preserved CI Developer ID signatures in `scripts/install.sh` (clears quarantine without re-signing)
- Dynamic GitHub pre-release flag in release workflows for beta/rc/alpha tags
- CI strictly arm64 macOS native runners with Developer ID code signing
- Aligned documentation and CI path filters with agent-brain

## [0.7.0] - 2026-07-05

### Added
- **Phase 7 mesh hardening** — `backpressure.rs` token-bucket JetStream consumer with RT/BE/IDLE subject priority
- **WASM filter cache** — `wasm_filter.rs` uses `agent-body-core::WasmEngine` shared module cache
- Bench: `wasm_filter_cached_vs_cold`

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.8] - 2026-06-27

### Added

- **Dual-Mode MCP support** — `serve-mcp` subcommand starts MCP server only (no HTTP daemon)
- **Nerves MCP tools** — `nerves_ping`, `nerves_health`, `nerves_jetstream_status`, `nerves_cluster_status`, `nerves_filter_list`, `nerves_filter_test` for NATS/event bridge operations via MCP

## [0.6.7] - 2026-06-27

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
