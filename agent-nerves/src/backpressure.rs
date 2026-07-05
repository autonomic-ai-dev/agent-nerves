//! Token-bucket backpressure for JetStream consumers (TX queue analogue).

use std::time::{Duration, Instant};

/// Linux-style scheduling class for NATS subjects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectPriority {
    /// Real-time: spine state, sandbox execute, explicit `.rt.` segment.
    Rt,
    /// Best-effort: default autonomic traffic.
    Be,
    /// Idle: debug / background `.idle.` traffic.
    Idle,
}

/// Map a subject to RT / BE / IDLE priority.
pub fn classify_subject(subject: &str) -> SubjectPriority {
    let s = subject.to_ascii_lowercase();
    if s.contains(".rt.")
        || s.starts_with("autonomic.spine.")
        || s.starts_with("autonomic.execute.")
    {
        SubjectPriority::Rt
    } else if s.contains(".idle.") || s.starts_with("autonomic.debug.") {
        SubjectPriority::Idle
    } else {
        SubjectPriority::Be
    }
}

impl SubjectPriority {
    pub fn bucket_limits(self) -> (f64, f64) {
        match self {
            SubjectPriority::Rt => (100.0, 500.0),
            SubjectPriority::Be => (50.0, 100.0),
            SubjectPriority::Idle => (10.0, 20.0),
        }
    }
}

pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_per_sec: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_per_sec,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed > 0.0 && self.refill_per_sec > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
            self.last_refill = now;
        }
    }

    pub fn try_consume(&mut self, cost: f64) -> bool {
        self.refill();
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }

    /// Duration until `cost` tokens are available (parks excess load).
    pub fn park_duration(&mut self, cost: f64) -> Duration {
        self.refill();
        if self.tokens >= cost {
            return Duration::ZERO;
        }
        if self.refill_per_sec <= 0.0 {
            return Duration::from_secs(3600);
        }
        let deficit = cost - self.tokens;
        Duration::from_secs_f64((deficit / self.refill_per_sec).max(0.001))
    }

    pub async fn acquire(&mut self, cost: f64) {
        loop {
            if self.try_consume(cost) {
                return;
            }
            let wait = self.park_duration(cost).min(Duration::from_millis(250));
            tokio::time::sleep(wait).await;
        }
    }
}

pub struct PriorityConsumer {
    rt: TokenBucket,
    be: TokenBucket,
    idle: TokenBucket,
}

impl Default for PriorityConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityConsumer {
    pub fn new() -> Self {
        let (cap, rate) = SubjectPriority::Rt.bucket_limits();
        let rt = TokenBucket::new(cap, rate);
        let (cap, rate) = SubjectPriority::Be.bucket_limits();
        let be = TokenBucket::new(cap, rate);
        let (cap, rate) = SubjectPriority::Idle.bucket_limits();
        let idle = TokenBucket::new(cap, rate);
        Self { rt, be, idle }
    }

    pub async fn acquire(&mut self, subject: &str) {
        match classify_subject(subject) {
            SubjectPriority::Rt => self.rt.acquire(1.0).await,
            SubjectPriority::Be => self.be.acquire(1.0).await,
            SubjectPriority::Idle => self.idle.acquire(1.0).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_bucket_parks_excess() {
        let mut bucket = TokenBucket::new(2.0, 1.0);
        assert!(bucket.try_consume(2.0));
        assert!(!bucket.try_consume(1.0));
        let park = bucket.park_duration(1.0);
        assert!(park >= Duration::from_millis(1));
        assert!(park <= Duration::from_secs(2));
    }

    #[test]
    fn classify_priority_classes() {
        assert_eq!(
            classify_subject("autonomic.spine.state"),
            SubjectPriority::Rt
        );
        assert_eq!(
            classify_subject("autonomic.execute.sandbox"),
            SubjectPriority::Rt
        );
        assert_eq!(
            classify_subject("autonomic.compute.job"),
            SubjectPriority::Be
        );
        assert_eq!(
            classify_subject("autonomic.debug.trace"),
            SubjectPriority::Idle
        );
    }
}
