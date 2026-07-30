pub use std::time::Duration;

use crate::bindings::wasi::clocks::monotonic_clock;

pub async fn sleep(duration: Duration) {
    let nanos = duration.as_nanos();
    let how_long = if nanos > u64::MAX as u128 {
        u64::MAX
    } else {
        nanos as u64
    };
    monotonic_clock::wait_for(how_long).await;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    nanos: u64,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            nanos: monotonic_clock::now(),
        }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        Duration::from_nanos(self.nanos.saturating_sub(earlier.nanos))
    }

    pub fn elapsed(&self) -> Duration {
        Self::now().duration_since(*self)
    }
}
