//! One number for the memory this process has committed to work in flight.
//!
//! Before this there were two pools and neither could answer "how much is in
//! use". The query pool was a semaphore handed out eight mebibytes at a time
//! *during* a scan, so a query that had already read for seconds could die
//! halfway through and leave a `429` behind; the background pool was a second,
//! separate counter that flush and metric compaction shared with nobody. The
//! process therefore had two ceilings, no total, and a refusal that arrived
//! after the work rather than before it.
//!
//! This is the write path's shape applied to the read path.
//! [`crate::backpressure::IngestGate::admit_body`] already counted request
//! bodies at admission, refused with `429` when the sum would pass a ceiling,
//! and released on `Drop`. A query now does the same: it computes what it will
//! need before it scans anything, charges that against one shared account, and
//! is refused **on arrival** or not at all. Flush and metric compaction charge
//! the same account and, having no client to refuse, wait for the next tick.
//!
//! Two refusals, deliberately distinct:
//!
//! * [`EXHAUSTED_PREFIX`] — the account is spoken for right now. Temporary by
//!   construction, so the client is told `429` and retrying is honest advice.
//! * [`OVER_BUDGET_PREFIX`] — the request alone wants more than the whole
//!   account. No amount of waiting helps, so the client is told `400` and the
//!   message names the number to narrow.
//!
//! The scan path reports `String`, so a status code is decided by reading the
//! message — the same reason [`crate::query::TENANT_QUOTA_PREFIX`] exists. A
//! literal typed twice is a refusal that silently becomes a `500` again the
//! day someone rewords it.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// "No room right now." Classified as `429`.
pub const EXHAUSTED_PREFIX: &str = "memory account of";

/// "No room here, ever." Classified as `400`.
pub const OVER_BUDGET_PREFIX: &str = "request needs";

/// The bytes this process has promised to work that has not finished.
///
/// Every consumer declares what it will hold *before* it holds it, and the sum
/// of those declarations is the one figure an operator reads. It is an
/// accounted total, not a measurement of the heap: allocator retention and the
/// caches with their own ceilings are outside it on purpose, because bytes
/// that a `Drop` cannot return are bytes an admission decision must not wait
/// on — gate on retained memory and a freed reservation never comes back,
/// which is a refusal that never lifts.
#[derive(Debug)]
pub struct MemoryAccount {
    budget_bytes: u64,
    in_use_bytes: AtomicU64,
    /// Requests refused because the account was spoken for.
    ///
    /// Counted apart from every other refusal because it means something the
    /// others do not: not that a tenant asked for more than it was sold
    /// (`query_quota_rejected`), and not that a query was too broad (a `400`),
    /// but that **this instance ran out of room for work it was willing to
    /// do**. It is the read side's `ingest_throttled` — the signal an operator
    /// scales or re-budgets on — and without it the event is invisible: as a
    /// `500` it hides among faults, and as a bare `429` it hides among healthy
    /// throttling.
    exhausted: AtomicU64,
    /// Background passes that found no room and left their input alone.
    ///
    /// Flush and compaction have no client to refuse, so their version of
    /// `exhausted` is a postponement. A number that climbs without falling is
    /// the shape of compaction debt that never drains.
    deferred: AtomicU64,
}

/// One admitted consumer's charge against the account.
///
/// Held for as long as the memory is — through the scan, the aggregation and
/// the response body for a query, through the whole rewrite for a merge —
/// because releasing it while the bytes it paid for are still resident is
/// exactly how the account would start lying. `Drop` is the release, so an
/// early `?` cannot leak the charge.
#[derive(Debug)]
pub struct MemoryCharge {
    account: Arc<MemoryAccount>,
    bytes: AtomicU64,
}

impl MemoryAccount {
    pub fn new(budget_bytes: u64) -> Self {
        Self {
            budget_bytes,
            in_use_bytes: AtomicU64::new(0),
            exhausted: AtomicU64::new(0),
            deferred: AtomicU64::new(0),
        }
    }

    pub fn budget_bytes(&self) -> u64 {
        self.budget_bytes
    }

    /// Bytes charged and not yet released: what work in flight has declared it
    /// holds right now. This is the figure an admission decision is made
    /// against and the one `/metrics` publishes.
    pub fn in_use_bytes(&self) -> u64 {
        self.in_use_bytes.load(Ordering::Relaxed)
    }

    /// Requests refused for want of room. See [`Self::exhausted`].
    pub fn exhausted(&self) -> u64 {
        self.exhausted.load(Ordering::Relaxed)
    }

    /// Background passes postponed for want of room. See [`Self::deferred`].
    pub fn deferred(&self) -> u64 {
        self.deferred.load(Ordering::Relaxed)
    }

    /// Admit a request that has already computed what it needs, or say why
    /// not in words the status classifier reads.
    pub fn admit(self: &Arc<Self>, bytes: u64) -> Result<MemoryCharge, String> {
        if bytes > self.budget_bytes {
            // Not a throttle. Waiting cannot make this fit, and telling a
            // client to retry a request that can never succeed is worse than
            // refusing it: it turns one bad query into an infinite loop of
            // them.
            return Err(format!(
                "{OVER_BUDGET_PREFIX} {bytes} bytes, more than the whole instance memory \
account of {} bytes (SIGNY_MEMORY_ACCOUNT_BYTES) — narrow the query",
                self.budget_bytes
            ));
        }
        match self.take(bytes) {
            Some(charge) => Ok(charge),
            None => {
                self.exhausted.fetch_add(1, Ordering::Relaxed);
                Err(format!(
                    "{EXHAUSTED_PREFIX} {} bytes holds {} in use and this request needs {bytes}",
                    self.budget_bytes,
                    self.in_use_bytes()
                ))
            }
        }
    }

    /// Admit background work, or tell it to come back next tick.
    ///
    /// No error string: flush and compaction have nobody to hand one to. The
    /// caller's contract is that a `None` leaves its input untouched, so the
    /// next pass re-selects the same work rather than half-doing it now.
    pub fn try_admit(self: &Arc<Self>, bytes: u64) -> Option<MemoryCharge> {
        let charge = self.take(bytes);
        if charge.is_none() {
            self.deferred.fetch_add(1, Ordering::Relaxed);
        }
        charge
    }

    fn take(self: &Arc<Self>, bytes: u64) -> Option<MemoryCharge> {
        self.in_use_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                let next = current.saturating_add(bytes);
                (next <= self.budget_bytes).then_some(next)
            })
            .ok()?;
        Some(MemoryCharge {
            account: self.clone(),
            bytes: AtomicU64::new(bytes),
        })
    }
}

impl MemoryCharge {
    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Acquire)
    }

    /// Replace the estimate with what the work actually holds.
    ///
    /// Called once, when the peak has passed and what remains is the result
    /// the response will be built from — so the number that stands for the
    /// rest of the charge's life is a measurement rather than a prediction.
    /// It moves in both directions on purpose. Upward, because an estimate
    /// that undershot must stop hiding real bytes from everyone else: this
    /// deliberately may push the account past its budget, and the next arrival
    /// is then refused on a figure that is true. Downward, because a
    /// pessimistic admission that a trace lookup did not use is memory a peer
    /// should not be kept waiting for.
    pub fn reconcile(&self, resident_bytes: u64) {
        let held = self.bytes.swap(resident_bytes, Ordering::AcqRel);
        let _ = self.account.in_use_bytes.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |current| Some(current.saturating_sub(held).saturating_add(resident_bytes)),
        );
    }
}

impl Drop for MemoryCharge {
    fn drop(&mut self) {
        let bytes = self.bytes.load(Ordering::Acquire);
        // Saturating: a decrement below zero would mean the counter was reset
        // under a live charge, and a wrapped counter refuses everything
        // forever after. Losing the charge is the harmless direction.
        let _ = self.account.in_use_bytes.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |current| Some(current.saturating_sub(bytes)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_charge_holds_until_it_drops_and_then_returns_every_byte() {
        let account = Arc::new(MemoryAccount::new(100));
        let charge = account.admit(60).expect("fits");
        assert_eq!(account.in_use_bytes(), 60);
        let error = account
            .admit(60)
            .expect_err("does not fit beside the first");
        assert!(error.starts_with(EXHAUSTED_PREFIX), "{error}");
        assert_eq!(account.exhausted(), 1);
        drop(charge);
        assert_eq!(account.in_use_bytes(), 0);
        account.admit(60).expect("fits once the first released");
        assert_eq!(
            account.exhausted(),
            1,
            "a success must not count as a refusal"
        );
    }

    #[test]
    fn a_request_larger_than_the_account_is_refused_as_permanent() {
        let account = Arc::new(MemoryAccount::new(100));
        let error = account.admit(101).expect_err("cannot ever fit");
        // The distinction the status classifier turns into 400 vs 429: this
        // one is not a throttle, and telling the client to retry would loop
        // it forever on a request that can never be served.
        assert!(error.starts_with(OVER_BUDGET_PREFIX), "{error}");
        assert!(!error.starts_with(EXHAUSTED_PREFIX), "{error}");
        assert_eq!(
            account.exhausted(),
            0,
            "a query too big for any instance is not evidence the instance is full"
        );
        assert_eq!(account.in_use_bytes(), 0, "a refusal charges nothing");
    }

    #[test]
    fn background_work_is_postponed_rather_than_refused() {
        let account = Arc::new(MemoryAccount::new(100));
        let held = account.try_admit(100).expect("first pass admitted");
        assert!(account.try_admit(1).is_none());
        assert_eq!(account.deferred(), 1);
        assert_eq!(
            account.exhausted(),
            0,
            "a postponement is not a client refusal"
        );
        drop(held);
        assert!(account.try_admit(100).is_some(), "the next tick finds room");
    }

    #[test]
    fn reconcile_tells_the_truth_even_when_it_passes_the_budget() {
        let account = Arc::new(MemoryAccount::new(100));
        let charge = account.admit(10).expect("the estimate fit");
        // The scan materialized more than it estimated. The bytes exist
        // whether or not the account admits it, so the account admits it.
        charge.reconcile(120);
        assert_eq!(account.in_use_bytes(), 120);
        let error = account
            .admit(1)
            .expect_err("the overrun is visible to peers");
        assert!(error.starts_with(EXHAUSTED_PREFIX), "{error}");
        // And an estimate that overshot gives the difference back, so a peer
        // is not kept waiting on memory nobody holds.
        charge.reconcile(5);
        assert_eq!(account.in_use_bytes(), 5);
        let peer = account
            .admit(90)
            .expect("the returned bytes are available again");
        drop(charge);
        assert_eq!(account.in_use_bytes(), 90, "only this charge went back");
        drop(peer);
        assert_eq!(account.in_use_bytes(), 0);
    }
}
