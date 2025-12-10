use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl RetryConfig {
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            base_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
        }
    }
}

pub struct Retrier {
    config: RetryConfig,
}

impl Retrier {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an async operation with retry logic
    pub async fn execute_with_retry<F, Fut, T>(&self, mut operation: F) -> crate::core::Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = crate::core::Result<T>>,
    {
        let mut attempts = 0;
        loop {
            attempts += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) if err.is_retryable() && attempts < self.config.max_attempts => {
                    let delay = self.calculate_backoff(attempts);
                    sleep(delay).await;
                }
                Err(err) => return Err(err),
            }
        }
    }

    /// Calculate exponential backoff delay with capped maximum
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        // Exponential backoff: base_delay * 2^(attempt - 1)
        let delay = self.config.base_delay * 2_u32.saturating_pow(attempt - 1);
        delay.min(self.config.max_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;

    fn test_config() -> RetryConfig {
        RetryConfig {
            max_attempts: 8,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
        }
    }

    #[test]
    fn test_calculate_backoff() {
        let retrier = Retrier::new(test_config());

        assert_eq!(retrier.calculate_backoff(1), Duration::from_secs(1));
        assert_eq!(retrier.calculate_backoff(2), Duration::from_secs(2));
        assert_eq!(retrier.calculate_backoff(3), Duration::from_secs(4));
        assert_eq!(retrier.calculate_backoff(4), Duration::from_secs(8));
        assert_eq!(retrier.calculate_backoff(10), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let retrier = Retrier::new(test_config());
        let mut call_count = 0;

        let result = retrier
            .execute_with_retry(|| {
                call_count += 1;
                async move { Ok::<i32, OciError>(42) }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count, 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let retrier = Retrier::new(test_config());
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        let result = retrier
            .execute_with_retry(|| {
                let count = call_count.clone();
                async move {
                    let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if current < 3 {
                        Err(OciError::ServiceError {
                            status: 500,
                            code: "InternalError".to_string(),
                            message: "test error".to_string(),
                        })
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        let retrier = Retrier::new(test_config());
        let mut call_count = 0;

        let result = retrier
            .execute_with_retry(|| {
                call_count += 1;
                async move {
                    Err::<i32, OciError>(OciError::AuthError("Invalid credentials".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count, 1);
    }
}
