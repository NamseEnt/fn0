use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::*;

const LAUNCH_LOCK_TIMEOUT: Duration = Duration::from_secs(300);

struct LaunchLock {
    started_at: Instant,
    count_at_lock: usize,
}

pub struct OciScaler {
    launch_lock: Mutex<Option<LaunchLock>>,
}

pub enum ScaleAction {
    ScaleOut(usize),
    ScaleIn(usize),
    Locked,
    NoOp,
}

impl OciScaler {
    pub fn new() -> Self {
        Self {
            launch_lock: Mutex::new(None),
        }
    }

    pub async fn decide(&self, current_count: usize, target: usize) -> ScaleAction {
        let mut lock = self.launch_lock.lock().await;

        if let Some(launch_lock) = lock.as_ref() {
            if current_count > launch_lock.count_at_lock {
                info!(
                    prev = launch_lock.count_at_lock,
                    current = current_count,
                    "Launch lock released: new instances appeared"
                );
                *lock = None;
            } else if launch_lock.started_at.elapsed() > LAUNCH_LOCK_TIMEOUT {
                warn!(
                    elapsed_secs = launch_lock.started_at.elapsed().as_secs(),
                    "Launch lock timed out: instances may have failed to provision"
                );
                *lock = None;
            } else {
                return ScaleAction::Locked;
            }
        }

        if current_count < target {
            let deficit = target - current_count;
            *lock = Some(LaunchLock {
                started_at: Instant::now(),
                count_at_lock: current_count,
            });
            ScaleAction::ScaleOut(deficit)
        } else if current_count > target {
            ScaleAction::ScaleIn(current_count - target)
        } else {
            ScaleAction::NoOp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scale_out_when_below_target() {
        let scaler = OciScaler::new();
        match scaler.decide(0, 2).await {
            ScaleAction::ScaleOut(n) => assert_eq!(n, 2),
            _ => panic!("Expected ScaleOut"),
        }
    }

    #[tokio::test]
    async fn scale_in_when_above_target() {
        let scaler = OciScaler::new();
        match scaler.decide(5, 2).await {
            ScaleAction::ScaleIn(n) => assert_eq!(n, 3),
            _ => panic!("Expected ScaleIn"),
        }
    }

    #[tokio::test]
    async fn noop_when_at_target() {
        let scaler = OciScaler::new();
        assert!(matches!(scaler.decide(2, 2).await, ScaleAction::NoOp));
    }

    #[tokio::test]
    async fn locked_after_scale_out() {
        let scaler = OciScaler::new();

        match scaler.decide(0, 2).await {
            ScaleAction::ScaleOut(2) => {}
            _ => panic!("Expected ScaleOut(2)"),
        }

        assert!(matches!(scaler.decide(0, 2).await, ScaleAction::Locked));
    }

    #[tokio::test]
    async fn lock_released_when_count_increases() {
        let scaler = OciScaler::new();

        match scaler.decide(0, 2).await {
            ScaleAction::ScaleOut(2) => {}
            _ => panic!("Expected ScaleOut(2)"),
        }

        assert!(matches!(scaler.decide(0, 2).await, ScaleAction::Locked));

        match scaler.decide(1, 2).await {
            ScaleAction::ScaleOut(1) => {}
            _ => panic!("Expected ScaleOut(1) after lock release"),
        }
    }

    #[tokio::test]
    async fn lock_released_on_timeout() {
        let scaler = OciScaler::new();

        match scaler.decide(0, 2).await {
            ScaleAction::ScaleOut(2) => {}
            _ => panic!("Expected ScaleOut(2)"),
        }

        {
            let mut lock = scaler.launch_lock.lock().await;
            if let Some(ref mut l) = *lock {
                l.started_at = Instant::now() - Duration::from_secs(301);
            }
        }

        match scaler.decide(0, 2).await {
            ScaleAction::ScaleOut(2) => {}
            _ => panic!("Expected ScaleOut(2) after timeout"),
        }
    }

    #[tokio::test]
    async fn lock_not_set_on_scale_in() {
        let scaler = OciScaler::new();

        match scaler.decide(5, 2).await {
            ScaleAction::ScaleIn(3) => {}
            _ => panic!("Expected ScaleIn(3)"),
        }

        match scaler.decide(0, 2).await {
            ScaleAction::ScaleOut(2) => {}
            _ => panic!("Expected ScaleOut(2), lock should not be set after scale in"),
        }
    }

    #[tokio::test]
    async fn lock_not_set_on_noop() {
        let scaler = OciScaler::new();

        assert!(matches!(scaler.decide(2, 2).await, ScaleAction::NoOp));

        match scaler.decide(0, 2).await {
            ScaleAction::ScaleOut(2) => {}
            _ => panic!("Expected ScaleOut(2), lock should not be set after noop"),
        }
    }

    #[tokio::test]
    async fn scale_out_partial_after_partial_provision() {
        let scaler = OciScaler::new();

        match scaler.decide(0, 3).await {
            ScaleAction::ScaleOut(3) => {}
            _ => panic!("Expected ScaleOut(3)"),
        }

        // count increased 0→1, lock releases, launches remaining 2
        match scaler.decide(1, 3).await {
            ScaleAction::ScaleOut(2) => {}
            _ => panic!("Expected ScaleOut(2)"),
        }

        // locked again after second scale out
        assert!(matches!(scaler.decide(1, 3).await, ScaleAction::Locked));

        // count increased 1→2, lock releases, launches remaining 1
        match scaler.decide(2, 3).await {
            ScaleAction::ScaleOut(1) => {}
            _ => panic!("Expected ScaleOut(1)"),
        }
    }
}
