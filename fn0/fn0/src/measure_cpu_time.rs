use std::future::Future;
use std::ops::Sub;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub trait Clock: Clone + Send + Sync + 'static {
    type Instant: Sub<Output = Duration> + Copy + Send + Sync + 'static;
    fn now(&self) -> Self::Instant;
}

#[derive(Clone, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    type Instant = Instant;

    fn now(&self) -> Self::Instant {
        Instant::now()
    }
}

pub struct MeasureCpuTime<F, C: Clock> {
    future: F,
    tracker: TimeTracker<C>,
    clock: C,
}

pub fn measure_cpu_time<F, C: Clock>(tracker: TimeTracker<C>, future: F) -> MeasureCpuTime<F, C> {
    let clock = (*tracker.clock).clone();
    MeasureCpuTime {
        future,
        tracker,
        clock,
    }
}

impl<F: Future, C: Clock> Future for MeasureCpuTime<F, C> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let start = this.clock.now();
        {
            this.tracker.last_start.lock().unwrap().replace(start);
        }

        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        let result = future.poll(cx);

        let end = this.clock.now();
        let elapsed = end - start;
        {
            this.tracker.last_start.lock().unwrap().take();
            this.tracker
                .acc
                .fetch_add(elapsed.as_nanos() as usize, Ordering::Relaxed);
        }

        match result {
            Poll::Ready(val) => Poll::Ready(val),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone)]
pub struct TimeTracker<C: Clock> {
    acc: Arc<AtomicUsize>,
    last_start: Arc<Mutex<Option<C::Instant>>>,
    clock: Arc<C>,
}

impl<C: Clock> TimeTracker<C> {
    pub fn new(clock: C) -> Self {
        Self {
            acc: Default::default(),
            last_start: Arc::new(Mutex::new(None)),
            clock: Arc::new(clock),
        }
    }
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(self.acc.load(Ordering::Relaxed) as u64)
            + self
                .last_start
                .lock()
                .unwrap()
                .as_ref()
                .map(|last_start| self.clock.now() - *last_start)
                .unwrap_or_default()
    }
}

impl<C: Clock + Default> Default for TimeTracker<C> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

#[cfg(test)]
mod tests;
