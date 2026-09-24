use std::collections::BTreeMap;
use std::fmt;

use dodb_core::{Error, Result as CoreResult};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrashInjected {
    pub point: String,
    pub hit: u64,
}

impl fmt::Display for CrashInjected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "simulated crash at {} hit {}",
            self.point, self.hit
        )
    }
}

impl std::error::Error for CrashInjected {}

struct CrashTarget {
    point: String,
    nth_hit: u64,
}

/// Deterministic named crash-point injector.
///
/// Hit counts are one-based. `at("point", 1)` fires on the first matching
/// hit; a disabled injector never fires. This is a simulated error only and
/// does not terminate the process.
#[derive(Default)]
pub struct CrashInjector {
    target: Option<CrashTarget>,
    hits: BTreeMap<String, u64>,
    fired: bool,
}

impl CrashInjector {
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn at(point: impl Into<String>, nth_hit: u64) -> Self {
        Self {
            target: Some(CrashTarget {
                point: point.into(),
                nth_hit: nth_hit.max(1),
            }),
            ..Self::default()
        }
    }

    pub fn hit(&mut self, point: &str) -> Result<(), CrashInjected> {
        let count = self.hits.entry(point.to_owned()).or_insert(0);
        *count += 1;
        if !self.fired
            && self
                .target
                .as_ref()
                .is_some_and(|target| target.point == point && target.nth_hit == *count)
        {
            self.fired = true;
            return Err(CrashInjected {
                point: point.to_owned(),
                hit: *count,
            });
        }
        Ok(())
    }

    pub fn hit_count(&self, point: &str) -> u64 {
        self.hits.get(point).copied().unwrap_or(0)
    }
}

impl dodb_storage::FaultInjector for CrashInjector {
    fn hit(&mut self, point: &str) -> CoreResult<()> {
        CrashInjector::hit(self, point).map_err(|crash| Error::recovery(crash.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_nth_hit_is_reproducible() {
        let mut injector = CrashInjector::at("after_page_publish", 2);
        assert!(injector.hit("before_page_publish").is_ok());
        assert!(injector.hit("after_page_publish").is_ok());
        assert_eq!(
            injector.hit("after_page_publish"),
            Err(CrashInjected {
                point: "after_page_publish".to_owned(),
                hit: 2,
            })
        );
        assert!(injector.hit("after_page_publish").is_ok());
        assert_eq!(injector.hit_count("after_page_publish"), 3);
    }
}
