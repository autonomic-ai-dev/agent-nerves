# agent-nerves Verification Suite

Deterministic playbook suite testing NATS JetStream event streaming, token bucket backpressure, JSON/WASM filters, and the **AUTONOMIC_USAGE Telemetry Pipeline**.

## Running Verification

```bash
# Master entrypoint:
.verify/run.sh

# Individual playbooks:
.verify/playbooks/verify_usage_stream.sh
.verify/playbooks/verify_backpressure.sh
.verify/playbooks/verify_full_suite.sh
```

## Playbook Matrix

| Playbook | Target Subsystem | Invariants Verified |
| :--- | :--- | :--- |
| `verify_usage_stream.sh` | JetStream Telemetry | `AUTONOMIC_USAGE` stream constants, `events.heart.>` subject hierarchy, and dedup publishing |
| `verify_backpressure.sh` | Priority Queue & Throttle | Priority class tagging, excess packet parking, and rate-limiting |
| `verify_full_suite.sh` | Complete agent-nerves | 100% unit, integration, and doc tests pass with 0 warnings |
