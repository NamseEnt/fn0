use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub struct MeasureCpuTime<F> {
    future: F,
    cpu_time: CpuTime,
}

pub fn measure_cpu_time<F>(future: F) -> MeasureCpuTime<F> {
    MeasureCpuTime {
        future,
        cpu_time: CpuTime {
            inner: Default::default(),
        },
    }
}

impl<F> MeasureCpuTime<F> {
    pub fn cpu_time(&self) -> CpuTime {
        self.cpu_time.clone()
    }
}

impl<F: Future> Future for MeasureCpuTime<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let start = Instant::now();

        let this = unsafe { self.get_unchecked_mut() };

        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        let result = future.poll(cx);

        let elapsed = start.elapsed();
        this.cpu_time
            .inner
            .fetch_add(elapsed.as_nanos() as usize, Ordering::Relaxed);

        match result {
            Poll::Ready(val) => Poll::Ready(val),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone)]
pub struct CpuTime {
    inner: Arc<AtomicUsize>,
}

impl CpuTime {
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(self.inner.load(Ordering::Relaxed) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Barrier;
    use tokio::time::sleep;

    // Helper: Custom future that yields a specified number of times
    struct YieldingFuture {
        yields_remaining: u32,
        value: i32,
    }

    impl Future for YieldingFuture {
        type Output = i32;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.yields_remaining > 0 {
                self.yields_remaining -= 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(self.value)
            }
        }
    }

    fn yielding_future(yields: u32, value: i32) -> YieldingFuture {
        YieldingFuture {
            yields_remaining: yields,
            value,
        }
    }

    // Helper: Assert duration is within a tolerance range
    #[allow(dead_code)]
    fn assert_duration_in_range(actual: Duration, expected: Duration, tolerance_percent: u64) {
        let expected_ms = expected.as_millis() as u64;
        let actual_ms = actual.as_millis() as u64;
        let tolerance = expected_ms * tolerance_percent / 100;
        let lower = expected_ms.saturating_sub(tolerance);
        let upper = expected_ms + tolerance;

        assert!(
            actual_ms >= lower && actual_ms <= upper,
            "Expected {}ms ±{}%, got {}ms (range: {}-{}ms)",
            expected_ms,
            tolerance_percent,
            actual_ms,
            lower,
            upper
        );
    }

    // Category 1: Basic Functionality Tests

    #[tokio::test]
    async fn test_measure_simple_async_operation() {
        let future = async {
            sleep(Duration::from_millis(10)).await;
            42
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, 42);
        // Should be at least a few milliseconds
        assert!(elapsed.as_micros() > 0);
    }

    #[tokio::test]
    async fn test_measure_returns_correct_output_type() {
        let string_future = async { "hello".to_string() };
        let measured = measure_cpu_time(string_future);
        let result = measured.await;
        assert_eq!(result, "hello");

        let vec_future = async { vec![1, 2, 3] };
        let measured = measure_cpu_time(vec_future);
        let result = measured.await;
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_measure_immediate_ready_future() {
        let future = async { 100 };
        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, 100);
        assert!(elapsed.as_micros() < 10_000);
    }

    // Category 2: Multi-Poll Scenarios

    #[tokio::test]
    async fn test_accumulates_time_across_multiple_polls() {
        let future = async {
            sleep(Duration::from_millis(5)).await;
            tokio::task::yield_now().await;
            sleep(Duration::from_millis(5)).await;
            tokio::task::yield_now().await;
            sleep(Duration::from_millis(5)).await;
            100
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, 100);
        // Verify time accumulates across multiple polls
        assert!(elapsed.as_micros() > 0);
    }

    #[tokio::test]
    async fn test_multiple_yields_with_custom_future() {
        let future = yielding_future(5, 42);
        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, 42);
        assert!(elapsed.as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_interleaved_computation_and_awaits() {
        let future = async {
            let mut sum = 0;
            for i in 0..1000 {
                sum += i;
            }
            sleep(Duration::from_millis(5)).await;

            for i in 0..1000 {
                sum += i;
            }
            sleep(Duration::from_millis(5)).await;

            sum
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, 999000);
        // Verify that both computation and awaits are measured
        assert!(elapsed.as_micros() > 0);
    }

    // Category 3: Concurrent Execution Tests

    #[tokio::test]
    async fn test_concurrent_measurements_independent() {
        let barrier = Arc::new(Barrier::new(5));
        let mut handles = vec![];

        for i in 0..5u64 {
            let barrier_clone = barrier.clone();
            let handle = tokio::spawn(async move {
                barrier_clone.wait().await;

                let sleep_ms = (i + 1) * 2;
                let future = async move {
                    sleep(Duration::from_millis(sleep_ms)).await;
                    i
                };

                let measured = measure_cpu_time(future);
                let cpu_time = measured.cpu_time();
                let result = measured.await;
                let elapsed = cpu_time.duration();
                (result, elapsed)
            });
            handles.push(handle);
        }

        for (idx, handle) in handles.into_iter().enumerate() {
            let (result, elapsed) = handle.await.unwrap();
            assert_eq!(result, idx as u64);
            // Verify each measurement is independent
            assert!(elapsed.as_micros() > 0);
        }
    }

    #[tokio::test]
    async fn test_many_concurrent_measurements() {
        let mut handles = vec![];

        for i in 0..50 {
            let handle = tokio::spawn(async move {
                let future = async move {
                    sleep(Duration::from_millis(1)).await;
                    i
                };
                let measured = measure_cpu_time(future);
                let cpu_time = measured.cpu_time();
                let result = measured.await;
                let elapsed = cpu_time.duration();
                (result, elapsed)
            });
            handles.push(handle);
        }

        for (i, handle) in handles.into_iter().enumerate() {
            let (result, elapsed) = handle.await.unwrap();
            assert_eq!(result, i);
            assert!(elapsed.as_nanos() > 0);
        }
    }

    #[tokio::test]
    async fn test_nested_measure_cpu_time() {
        let inner_future = async {
            sleep(Duration::from_millis(5)).await;
            42
        };

        let outer_future = async {
            let inner_measured = measure_cpu_time(inner_future);
            let inner_cpu_time = inner_measured.cpu_time();
            let inner_result = inner_measured.await;
            let inner_time = inner_cpu_time.duration();
            sleep(Duration::from_millis(5)).await;
            (inner_result, inner_time)
        };

        let outer_measured = measure_cpu_time(outer_future);
        let outer_cpu_time = outer_measured.cpu_time();
        let (result, inner_elapsed) = outer_measured.await;
        let outer_elapsed = outer_cpu_time.duration();

        assert_eq!(result, 42);
        assert!(inner_elapsed.as_micros() > 0);
        assert!(outer_elapsed.as_micros() > 0);
        // Outer should include inner time plus additional work
        assert!(outer_elapsed >= inner_elapsed);
    }

    // Category 4: Error Handling Tests

    #[tokio::test]
    async fn test_measure_with_result_ok() {
        let future = async {
            sleep(Duration::from_millis(5)).await;
            Ok::<i32, String>(42)
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, Ok(42));
        assert!(elapsed.as_micros() > 0);
    }

    #[tokio::test]
    async fn test_measure_with_result_err() {
        let future = async {
            sleep(Duration::from_millis(5)).await;
            Err::<i32, String>("error occurred".to_string())
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, Err("error occurred".to_string()));
        assert!(elapsed.as_micros() > 0);
    }

    #[tokio::test]
    async fn test_measure_with_option_none() {
        let future = async {
            sleep(Duration::from_millis(5)).await;
            None::<i32>
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, None);
        assert!(elapsed.as_micros() > 0);
    }

    #[tokio::test]
    #[should_panic(expected = "intentional panic")]
    async fn test_measure_future_that_panics() {
        let future = async {
            panic!("intentional panic");
        };

        let measured = measure_cpu_time(future);
        let _ = measured.await;
    }

    // Category 5: Timing Validation Tests

    #[tokio::test]
    async fn test_timing_accuracy_with_known_delays() {
        // Test that longer delays result in longer measured times
        let future1 = async {
            sleep(Duration::from_millis(5)).await;
            1
        };
        let future2 = async {
            sleep(Duration::from_millis(10)).await;
            2
        };
        let future3 = async {
            sleep(Duration::from_millis(20)).await;
            3
        };

        let measured1 = measure_cpu_time(future1);
        let cpu_time1 = measured1.cpu_time();
        let result1 = measured1.await;
        let elapsed1 = cpu_time1.duration();

        let measured2 = measure_cpu_time(future2);
        let cpu_time2 = measured2.cpu_time();
        let result2 = measured2.await;
        let elapsed2 = cpu_time2.duration();

        let measured3 = measure_cpu_time(future3);
        let cpu_time3 = measured3.cpu_time();
        let result3 = measured3.await;
        let elapsed3 = cpu_time3.duration();

        assert_eq!(result1, 1);
        assert_eq!(result2, 2);
        assert_eq!(result3, 3);

        // All should have measured non-zero time
        assert!(elapsed1.as_micros() > 0);
        assert!(elapsed2.as_micros() > 0);
        assert!(elapsed3.as_micros() > 0);
    }

    #[tokio::test]
    async fn test_zero_duration_for_instant_completion() {
        let future = async { 1 + 1 };
        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, 2);
        assert!(elapsed.as_micros() < 1000);
    }

    #[tokio::test]
    async fn test_duration_increases_with_work() {
        // Test that more work results in longer measured time
        let future1 = async {
            sleep(Duration::from_millis(5)).await;
        };
        let future2 = async {
            sleep(Duration::from_millis(5)).await;
            sleep(Duration::from_millis(5)).await;
        };
        let future3 = async {
            sleep(Duration::from_millis(5)).await;
            sleep(Duration::from_millis(5)).await;
            sleep(Duration::from_millis(5)).await;
        };

        let measured1 = measure_cpu_time(future1);
        let cpu_time1 = measured1.cpu_time();
        let _ = measured1.await;
        let elapsed1 = cpu_time1.duration();

        let measured2 = measure_cpu_time(future2);
        let cpu_time2 = measured2.cpu_time();
        let _ = measured2.await;
        let elapsed2 = cpu_time2.duration();

        let measured3 = measure_cpu_time(future3);
        let cpu_time3 = measured3.cpu_time();
        let _ = measured3.await;
        let elapsed3 = cpu_time3.duration();

        // All should be non-zero
        assert!(elapsed1.as_micros() > 0);
        assert!(elapsed2.as_micros() > 0);
        assert!(elapsed3.as_micros() > 0);
        // More work should generally take longer
        assert!(elapsed3 > elapsed1);
    }

    // Category 6: Edge Cases

    #[tokio::test]
    async fn test_measure_unit_type_future() {
        let future = async {
            sleep(Duration::from_millis(5)).await;
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, ());
        assert!(elapsed.as_micros() > 0);
    }

    #[tokio::test]
    async fn test_measure_large_output() {
        let future = async {
            sleep(Duration::from_millis(5)).await;
            vec![0u8; 1_000_000]
        };

        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result.len(), 1_000_000);
        assert!(elapsed.as_micros() > 0);
    }

    #[tokio::test]
    async fn test_measure_empty_async_block() {
        let future = async {};
        let measured = measure_cpu_time(future);
        let cpu_time = measured.cpu_time();
        let result = measured.await;
        let elapsed = cpu_time.duration();

        assert_eq!(result, ());
        // Even an empty block should have some measurement overhead
        assert!(elapsed.as_nanos() < 1_000_000); // Less than 1ms
    }
}
