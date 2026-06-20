#!/usr/bin/env bash
# Build agent-nerves release and adhoc-sign on macOS.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo build --release -p agent-nerves
"$ROOT/scripts/sign-macos.sh"
