#!/usr/bin/env bash
set -euo pipefail

# agent-nerves verification master entrypoint
# Executed by automated verification runners or developer testing

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$DIR/.." && pwd)"

echo "==> [agent-nerves] Starting verification suite..."
cd "$REPO_ROOT"

echo "==> Checking cargo clippy with -D warnings..."
cargo clippy -- -D warnings

echo "==> Checking cargo check..."
cargo check

echo "==> Running Playbook: Usage Stream..."
"$DIR/playbooks/verify_usage_stream.sh"

echo "==> Running Playbook: Backpressure..."
"$DIR/playbooks/verify_backpressure.sh"

echo "==> Running Playbook: Full Suite..."
"$DIR/playbooks/verify_full_suite.sh"

echo "==> [agent-nerves] All verification playbooks passed cleanly (exit 0)."
