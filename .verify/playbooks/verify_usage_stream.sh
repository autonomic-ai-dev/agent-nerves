#!/usr/bin/env bash
set -euo pipefail

# Playbook: Verify JetStream Usage Stream
# Verifies stream configuration, usage subjects, and deduplication publishing logic.

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$DIR/../.." && pwd)"

echo "==> [Playbook: Usage Stream] Running jetstream::tests..."
cd "$REPO_ROOT"
cargo test jetstream::tests

echo "==> [Playbook: Usage Stream] Passed cleanly (exit 0)."
