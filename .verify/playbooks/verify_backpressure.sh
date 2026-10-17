#!/usr/bin/env bash
set -euo pipefail

# Playbook: Verify Backpressure & Priority Queues
# Tests packet classification and token bucket parking.

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$DIR/../.." && pwd)"

echo "==> [Playbook: Backpressure] Running backpressure::tests..."
cd "$REPO_ROOT"
cargo test backpressure::tests

echo "==> [Playbook: Backpressure] Passed cleanly (exit 0)."
