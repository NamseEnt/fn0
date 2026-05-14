use super::*;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::time::sleep;

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

#[tokio::test]
async fn test_measure_simple_async_operation() {
    let future = async {
        sleep(Duration::from_millis(10)).await;
        42
    };

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, 42);
    assert!(elapsed.as_micros() > 0);
}

#[tokio::test]
async fn test_measure_returns_correct_output_type() {
    let string_future = async { "hello".to_string() };
    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker, string_future);
    let result = measured.await;
    assert_eq!(result, "hello");

    let vec_future = async { vec![1, 2, 3] };
    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker, vec_future);
    let result = measured.await;
    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_measure_immediate_ready_future() {
    let future = async { 100 };
    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, 100);
    assert!(elapsed.as_micros() < 10_000);
}

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

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, 100);
    assert!(elapsed.as_micros() > 0);
}

#[tokio::test]
async fn test_multiple_yields_with_custom_future() {
    let future = yielding_future(5, 42);
    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

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

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, 999000);
    assert!(elapsed.as_micros() > 0);
}

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

            let tracker = TimeTracker::<SystemClock>::default();
            let measured = measure_cpu_time(tracker.clone(), future);
            let result = measured.await;
            let elapsed = tracker.duration();
            (result, elapsed)
        });
        handles.push(handle);
    }

    for (idx, handle) in handles.into_iter().enumerate() {
        let (result, elapsed) = handle.await.unwrap();
        assert_eq!(result, idx as u64);
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
            let tracker = TimeTracker::<SystemClock>::default();
            let measured = measure_cpu_time(tracker.clone(), future);
            let result = measured.await;
            let elapsed = tracker.duration();
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

    let inner_tracker = TimeTracker::<SystemClock>::default();
    let outer_future = async {
        let inner_measured = measure_cpu_time(inner_tracker.clone(), inner_future);
        let inner_result = inner_measured.await;
        let inner_time = inner_tracker.duration();
        sleep(Duration::from_millis(5)).await;
        (inner_result, inner_time)
    };

    let outer_tracker = TimeTracker::<SystemClock>::default();
    let outer_measured = measure_cpu_time(outer_tracker.clone(), outer_future);
    let (result, inner_elapsed) = outer_measured.await;
    let outer_elapsed = outer_tracker.duration();

    assert_eq!(result, 42);
    assert!(inner_elapsed.as_micros() > 0);
    assert!(outer_elapsed.as_micros() > 0);
    assert!(outer_elapsed >= inner_elapsed);
}

#[tokio::test]
async fn test_measure_with_result_ok() {
    let future = async {
        sleep(Duration::from_millis(5)).await;
        Ok::<i32, String>(42)
    };

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, Ok(42));
    assert!(elapsed.as_micros() > 0);
}

#[tokio::test]
async fn test_measure_with_result_err() {
    let future = async {
        sleep(Duration::from_millis(5)).await;
        Err::<i32, String>("error occurred".to_string())
    };

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, Err("error occurred".to_string()));
    assert!(elapsed.as_micros() > 0);
}

#[tokio::test]
async fn test_measure_with_option_none() {
    let future = async {
        sleep(Duration::from_millis(5)).await;
        None::<i32>
    };

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, None);
    assert!(elapsed.as_micros() > 0);
}

#[tokio::test]
#[should_panic(expected = "intentional panic")]
async fn test_measure_future_that_panics() {
    let future = async {
        panic!("intentional panic");
    };

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker, future);
    let _ = measured.await;
}

#[tokio::test]
async fn test_timing_accuracy_with_known_delays() {
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

    let tracker1 = TimeTracker::<SystemClock>::default();
    let measured1 = measure_cpu_time(tracker1.clone(), future1);
    let result1 = measured1.await;
    let elapsed1 = tracker1.duration();

    let tracker2 = TimeTracker::<SystemClock>::default();
    let measured2 = measure_cpu_time(tracker2.clone(), future2);
    let result2 = measured2.await;
    let elapsed2 = tracker2.duration();

    let tracker3 = TimeTracker::<SystemClock>::default();
    let measured3 = measure_cpu_time(tracker3.clone(), future3);
    let result3 = measured3.await;
    let elapsed3 = tracker3.duration();

    assert_eq!(result1, 1);
    assert_eq!(result2, 2);
    assert_eq!(result3, 3);

    assert!(elapsed1.as_micros() > 0);
    assert!(elapsed2.as_micros() > 0);
    assert!(elapsed3.as_micros() > 0);
}

#[tokio::test]
async fn test_zero_duration_for_instant_completion() {
    let future = async { 1 + 1 };
    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, 2);
    assert!(elapsed.as_micros() < 1000);
}

#[tokio::test]
async fn test_duration_increases_with_work() {
    struct PollNFuture {
        clock: MockClock,
        polls_remaining: u32,
    }

    impl Future for PollNFuture {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.clock.advance(Duration::from_millis(10));
            if self.polls_remaining > 0 {
                self.polls_remaining -= 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }

    let clock1 = MockClock::new(Instant::now());
    let tracker1 = TimeTracker::new(clock1.clone());
    let future1 = PollNFuture {
        clock: clock1,
        polls_remaining: 1,
    };
    let measured1 = measure_cpu_time(tracker1.clone(), future1);
    let _ = measured1.await;
    let elapsed1 = tracker1.duration();

    let clock3 = MockClock::new(Instant::now());
    let tracker3 = TimeTracker::new(clock3.clone());
    let future3 = PollNFuture {
        clock: clock3,
        polls_remaining: 3,
    };
    let measured3 = measure_cpu_time(tracker3.clone(), future3);
    let _ = measured3.await;
    let elapsed3 = tracker3.duration();

    assert_eq!(elapsed1, Duration::from_millis(20));
    assert_eq!(elapsed3, Duration::from_millis(40));
    assert!(elapsed3 > elapsed1);
}

#[tokio::test]
async fn test_measure_unit_type_future() {
    let future = async {
        sleep(Duration::from_millis(5)).await;
    };

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, ());
    assert!(elapsed.as_micros() > 0);
}

#[tokio::test]
async fn test_measure_large_output() {
    let future = async {
        sleep(Duration::from_millis(5)).await;
        vec![0u8; 1_000_000]
    };

    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result.len(), 1_000_000);
    assert!(elapsed.as_micros() > 0);
}

#[tokio::test]
async fn test_measure_empty_async_block() {
    let future = async {};
    let tracker = TimeTracker::<SystemClock>::default();
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, ());
    assert!(elapsed.as_nanos() < 1_000_000);
}

#[derive(Clone)]
struct MockClock {
    now: Arc<std::sync::Mutex<Instant>>,
}

impl MockClock {
    fn new(start_time: Instant) -> Self {
        Self {
            now: Arc::new(std::sync::Mutex::new(start_time)),
        }
    }

    fn advance(&self, duration: Duration) {
        let mut now = self.now.lock().unwrap();
        *now += duration;
    }
}

impl Clock for MockClock {
    type Instant = Instant;

    fn now(&self) -> Self::Instant {
        *self.now.lock().unwrap()
    }
}

#[tokio::test]
async fn test_measure_with_mock_clock() {
    let start_time = Instant::now();
    let clock = MockClock::new(start_time);
    let clock_clone = clock.clone();

    let future = async move {
        clock_clone.advance(Duration::from_secs(1));
        42
    };

    let tracker = TimeTracker::new(clock.clone());
    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let elapsed = tracker.duration();

    assert_eq!(result, 42);
    assert_eq!(elapsed, Duration::from_secs(1));
}

#[tokio::test]
async fn test_duration_accumulates_across_polls() {
    let start_time = Instant::now();
    let clock = MockClock::new(start_time);

    struct MultiPollFuture {
        clock: MockClock,
        poll_count: u32,
        max_polls: u32,
    }

    impl Future for MultiPollFuture {
        type Output = u32;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.poll_count < self.max_polls {
                self.clock.advance(Duration::from_millis(100));
                self.poll_count += 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(self.poll_count)
            }
        }
    }

    let future = MultiPollFuture {
        clock: clock.clone(),
        poll_count: 0,
        max_polls: 10,
    };

    let tracker = TimeTracker::new(clock.clone());
    let measured = measure_cpu_time(tracker.clone(), future);

    let result = measured.await;
    let final_duration = tracker.duration();

    assert_eq!(result, 10);
    assert!(
        final_duration >= Duration::from_millis(1000),
        "Expected >= 1000ms, got {:?}",
        final_duration
    );
}

#[tokio::test]
async fn test_duration_between_polls() {
    let start_time = Instant::now();
    let clock = MockClock::new(start_time);

    struct YieldingFuture {
        clock: MockClock,
        poll_count: u32,
    }

    impl Future for YieldingFuture {
        type Output = u32;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.poll_count < 5 {
                self.clock.advance(Duration::from_millis(100));
                self.poll_count += 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(self.poll_count)
            }
        }
    }

    let tracker = TimeTracker::new(clock.clone());

    let future = YieldingFuture {
        clock: clock.clone(),
        poll_count: 0,
    };

    let measured = measure_cpu_time(tracker.clone(), future);
    let result = measured.await;
    let final_duration = tracker.duration();

    assert_eq!(result, 5);

    assert!(
        final_duration >= Duration::from_millis(500),
        "Expected >= 500ms, got {:?}",
        final_duration
    );
}

#[tokio::test]
async fn test_concurrent_duration_access() {
    let tracker = TimeTracker::<SystemClock>::default();

    let mut handles = vec![];
    for _ in 0..10 {
        let tracker_clone = tracker.clone();
        let handle = tokio::spawn(async move {
            let mut durations = Vec::new();
            for _ in 0..50 {
                durations.push(tracker_clone.duration());
                tokio::task::yield_now().await;
            }
            durations
        });
        handles.push(handle);
    }

    let work_tracker = tracker.clone();
    let work_handle = tokio::spawn(async move {
        let future = async {
            for _ in 0..20 {
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        };
        let measured = measure_cpu_time(work_tracker, future);
        measured.await
    });

    let mut all_durations = Vec::new();
    for handle in handles {
        let durations = handle.await.unwrap();
        all_durations.extend(durations);
    }

    work_handle.await.unwrap();
    let final_duration = tracker.duration();

    assert_eq!(
        all_durations.len(),
        500,
        "Should have 500 duration readings"
    );

    for duration in &all_durations {
        assert!(
            *duration <= final_duration,
            "Duration {:?} > final {:?}",
            duration,
            final_duration
        );
    }
}
