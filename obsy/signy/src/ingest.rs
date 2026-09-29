//! Input bounds every log ingest answers to, whatever the transport.
//!
//! The Loki push handlers lived here until the bed moved to OTLP and the
//! endpoint was removed (`todo.md`, "Next — OTLP only"). What survived the
//! removal is exactly what was never about that wire: the timestamp window
//! exists to keep a timestamp from landing in a partition retention has
//! already swept, and `otlp_log`'s normalization is checked against it in
//! `log_ingest.rs` the same way the push decoder was.

use crate::config::Config;

/// Accepted timestamp band around the server clock, resolved once per request.
/// Timestamps outside it are rejected: a far-past entry lands in a partition
/// retention already swept, and a far-future entry lands in one whose
/// `max_ts_ns` never falls behind the retention cutoff, so it is never expired.
pub struct TimestampWindow {
    oldest_ns: Option<i64>,
    newest_ns: Option<i64>,
}

impl TimestampWindow {
    pub fn from_config(config: &Config, clock: &crate::clock::Clock) -> Self {
        let now_ns = clock.now_ns();
        Self {
            oldest_ns: config
                .max_timestamp_age
                .map(|age| now_ns.saturating_sub(duration_to_ns(age))),
            newest_ns: config
                .max_timestamp_skew
                .map(|skew| now_ns.saturating_add(duration_to_ns(skew))),
        }
    }

    pub fn validate(&self, timestamp_ns: i64) -> Result<(), String> {
        if self.oldest_ns.is_some_and(|oldest| timestamp_ns < oldest) {
            return Err(format!(
                "entry timestamp {timestamp_ns} is older than the accepted window; \
raise SIGNY_MAX_TIMESTAMP_AGE or disable it with 'off'"
            ));
        }
        if self.newest_ns.is_some_and(|newest| timestamp_ns > newest) {
            return Err(format!(
                "entry timestamp {timestamp_ns} is further in the future than the accepted window; \
check the client clock and timestamp units, or raise SIGNY_MAX_TIMESTAMP_SKEW"
            ));
        }
        Ok(())
    }
}

fn duration_to_ns(duration: std::time::Duration) -> i64 {
    duration.as_nanos().min(i64::MAX as u128) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// The window's whole content is two boundaries, and every test until now
    /// approached them from far away — "a day old is rejected", "an hour old is
    /// accepted" — which never exercises the edge itself. A unit mix-up lands
    /// *just* outside; an ordinary clock skew lands *just* inside.
    #[test]
    fn the_timestamp_window_accepts_its_edge_and_rejects_one_nanosecond_past_it() {
        let now_ns = 1_800_000_000_000_000_000i64;
        let age = Duration::from_secs(3600);
        let config = Config {
            max_timestamp_age: Some(age),
            max_timestamp_skew: Some(age),
            ..Config::default()
        };
        let clock = crate::clock::Clock::fixed(now_ns);

        let window = TimestampWindow::from_config(&config, &clock);
        let oldest = now_ns - age.as_nanos() as i64;
        let newest = now_ns + age.as_nanos() as i64;

        assert!(window.validate(oldest).is_ok(), "the oldest edge is inside");
        assert!(window.validate(newest).is_ok(), "the newest edge is inside");
        assert!(
            window.validate(oldest - 1).is_err(),
            "one nanosecond older must be refused"
        );
        assert!(
            window.validate(newest + 1).is_err(),
            "one nanosecond newer must be refused"
        );

        // And the window travels with the clock rather than being fixed at
        // startup: a long-running process must not drift into refusing the
        // present.
        clock.advance(Duration::from_secs(7200));
        let later = TimestampWindow::from_config(&config, &clock);
        assert!(
            later.validate(oldest).is_err(),
            "what was the edge two hours ago is now outside"
        );
        assert!(later.validate(newest + 1).is_ok());
    }

    #[test]
    fn a_disabled_window_accepts_any_timestamp() {
        let config = Config {
            max_timestamp_age: None,
            max_timestamp_skew: None,
            ..Config::default()
        };
        let window = TimestampWindow::from_config(&config, &crate::clock::Clock::fixed(0));
        assert!(window.validate(i64::MIN).is_ok());
        assert!(window.validate(i64::MAX).is_ok());
    }
}
