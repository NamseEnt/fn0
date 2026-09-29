use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The wall clock, as something a test can move.
///
/// `tokio::time::pause` virtualizes the *monotonic* clock — sleeps, intervals,
/// `tokio::time::Instant` — which covers every loop cadence and backoff in this
/// engine. It does not touch `SystemTime`, and `SystemTime` is what decides the
/// things a user actually sees: which timestamps ingest accepts, what range a
/// query defaults to, when a tenant's retention cutoff falls.
///
/// Those were tested by constructing inputs relative to the real `now`, which
/// makes the boundary itself untestable — you can assert "an hour ago is
/// accepted" but not "exactly the window edge is accepted and one nanosecond
/// past it is not". The interesting cases live at the edges.
pub struct Clock {
    fixed_ns: Option<AtomicI64>,
}

impl Clock {
    pub fn system() -> Arc<Self> {
        Arc::new(Self { fixed_ns: None })
    }

    /// Nanoseconds since the UNIX epoch, saturated into `i64`.
    ///
    /// Every timestamp in this engine is an `i64` nanosecond count, so the
    /// conversion belongs here rather than being repeated — and repeated
    /// slightly differently — at each call site.
    pub fn now_ns(&self) -> i64 {
        match &self.fixed_ns {
            Some(fixed) => fixed.load(Ordering::Relaxed),
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos().min(i64::MAX as u128) as i64)
                // A clock before the epoch is a broken machine, not a state
                // worth propagating: zero makes every window reject rather
                // than accept.
                .unwrap_or(0),
        }
    }

    pub fn now(&self) -> SystemTime {
        match &self.fixed_ns {
            Some(_) => UNIX_EPOCH + Duration::from_nanos(self.now_ns().max(0) as u64),
            None => SystemTime::now(),
        }
    }

    /// A clock parked at an instant, for tests that care about a boundary.
    pub fn fixed(now_ns: i64) -> Arc<Self> {
        Arc::new(Self {
            fixed_ns: Some(AtomicI64::new(now_ns)),
        })
    }

    /// Move a fixed clock. A no-op on the system clock, which is the honest
    /// behaviour: production code holding a `Clock` cannot move real time, and
    /// a panic here would make the type unusable outside tests.
    pub fn advance(&self, by: Duration) {
        if let Some(fixed) = &self.fixed_ns {
            fixed.fetch_add(
                by.as_nanos().min(i64::MAX as u128) as i64,
                Ordering::Relaxed,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixed_clock_stands_still_until_it_is_moved() {
        let clock = Clock::fixed(1_000);
        assert_eq!(clock.now_ns(), 1_000);
        assert_eq!(clock.now_ns(), 1_000);

        clock.advance(Duration::from_secs(1));
        assert_eq!(clock.now_ns(), 1_000 + 1_000_000_000);
        assert_eq!(
            clock.now(),
            UNIX_EPOCH + Duration::from_nanos(1_000 + 1_000_000_000)
        );
    }

    #[test]
    fn the_system_clock_moves_on_its_own_and_ignores_advance() {
        let clock = Clock::system();
        let before = clock.now_ns();
        assert!(before > 1_700_000_000_000_000_000, "epoch nanoseconds");
        // Advancing the system clock is meaningless rather than an error: the
        // same call site runs in production holding this exact type.
        clock.advance(Duration::from_secs(86_400));
        assert!(clock.now_ns() - before < 60_000_000_000);
    }
}
