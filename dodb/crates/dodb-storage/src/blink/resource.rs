use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::Notify;

const CATEGORY_COUNT: usize = 7;
const DISK_CATEGORY_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(usize)]
pub enum ResourceCategory {
    DecodedPayload = 0,
    Metadata = 1,
    LoadReconstruction = 2,
    QueuedForeground = 3,
    WritePreparation = 4,
    DirtyImages = 5,
    CheckpointValidation = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationClass {
    Foreground,
    Progress,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(usize)]
pub enum DiskCategory {
    Wal = 0,
    CheckpointOutput = 1,
    VersionIndex = 2,
    Scratch = 3,
    RetainedSource = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiskResourceLimits {
    pub global_bytes: u64,
    pub per_tenant_bytes: u64,
    pub max_tenants: usize,
    pub global_category_bytes: [u64; DISK_CATEGORY_COUNT],
    pub per_tenant_category_bytes: [u64; DISK_CATEGORY_COUNT],
}

impl DiskResourceLimits {
    fn unbounded(max_tenants: usize) -> Self {
        Self {
            global_bytes: u64::MAX,
            per_tenant_bytes: u64::MAX,
            max_tenants,
            global_category_bytes: [u64::MAX; DISK_CATEGORY_COUNT],
            per_tenant_category_bytes: [u64::MAX; DISK_CATEGORY_COUNT],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub global_bytes: u64,
    pub per_tenant_bytes: u64,
    pub progress_pool_bytes: u64,
    pub max_tenants: usize,
    pub max_pending_waiters: usize,
    pub global_category_bytes: [u64; CATEGORY_COUNT],
    pub per_tenant_category_bytes: [u64; CATEGORY_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceError {
    InvalidLimits,
    TooManyTenants,
    TooManyWaiters,
    CapacityExceeded,
    ArithmeticOverflow,
    SharedOwnership,
    WaitTimedOut,
    Poisoned,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResourceSnapshot {
    pub current_by_category: [u64; CATEGORY_COUNT],
    pub peak_by_category: [u64; CATEGORY_COUNT],
    pub current_total: u64,
    pub peak_total: u64,
    pub current_progress: u64,
    pub peak_progress: u64,
    pub active_tenants: usize,
    pub pending_waiters: usize,
    pub rejected_reservations: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiskResourceSnapshot {
    pub current_by_category: [u64; DISK_CATEGORY_COUNT],
    pub peak_by_category: [u64; DISK_CATEGORY_COUNT],
    pub current_total: u64,
    pub peak_total: u64,
    pub active_tenants: usize,
    pub pending_waiters: usize,
    pub rejected_reservations: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantDiskResourceSnapshot {
    pub current_by_category: [u64; DISK_CATEGORY_COUNT],
    pub peak_by_category: [u64; DISK_CATEGORY_COUNT],
    pub current_total: u64,
    pub peak_total: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantResourceSnapshot {
    pub current_by_category: [u64; CATEGORY_COUNT],
    pub peak_by_category: [u64; CATEGORY_COUNT],
    pub current_total: u64,
    pub peak_total: u64,
}

#[derive(Default)]
struct TenantState {
    current_by_category: [u64; CATEGORY_COUNT],
    peak_by_category: [u64; CATEGORY_COUNT],
    current_total: u64,
    peak_total: u64,
}

#[derive(Default)]
struct State {
    current_by_category: [u64; CATEGORY_COUNT],
    peak_by_category: [u64; CATEGORY_COUNT],
    current_total: u64,
    peak_total: u64,
    current_progress: u64,
    peak_progress: u64,
    rejected_reservations: u64,
    pending_waiters: usize,
    tenants: BTreeMap<u64, TenantState>,
}

#[derive(Default)]
struct DiskTenantState {
    current_by_category: [u64; DISK_CATEGORY_COUNT],
    peak_by_category: [u64; DISK_CATEGORY_COUNT],
    current_total: u64,
    peak_total: u64,
}

#[derive(Default)]
struct DiskState {
    current_by_category: [u64; DISK_CATEGORY_COUNT],
    peak_by_category: [u64; DISK_CATEGORY_COUNT],
    current_total: u64,
    peak_total: u64,
    rejected_reservations: u64,
    pending_waiters: usize,
    tenants: BTreeMap<u64, DiskTenantState>,
    persistent_usage: BTreeMap<(u64, DiskCategory, String), u64>,
}

struct Inner {
    limits: ResourceLimits,
    state: Mutex<State>,
    capacity_changed: Notify,
}

struct DiskInner {
    limits: DiskResourceLimits,
    state: Mutex<DiskState>,
    capacity_changed: Notify,
}

struct WaiterRegistration {
    inner: Arc<Inner>,
}

struct DiskWaiterRegistration {
    inner: Arc<DiskInner>,
}

#[derive(Clone)]
pub struct BlinkResourceManager {
    inner: Arc<Inner>,
    disk: BlinkDiskResourceManager,
}

#[derive(Clone)]
pub struct BlinkDiskResourceManager {
    inner: Arc<DiskInner>,
}

struct Charge {
    inner: Arc<Inner>,
    tenant_id: u64,
    category: ResourceCategory,
    class: ReservationClass,
    bytes: u64,
}

pub struct Reservation {
    charge: Arc<Charge>,
}

struct DiskCharge {
    inner: Arc<DiskInner>,
    tenant_id: u64,
    category: DiskCategory,
    bytes: u64,
}

pub struct DiskReservation {
    charge: Arc<DiskCharge>,
}

impl BlinkResourceManager {
    pub fn new(limits: ResourceLimits) -> Result<Self, ResourceError> {
        let disk_limits = DiskResourceLimits::unbounded(limits.max_tenants);
        Self::new_with_disk_limits(limits, disk_limits)
    }

    pub fn new_with_disk_limits(
        limits: ResourceLimits,
        disk_limits: DiskResourceLimits,
    ) -> Result<Self, ResourceError> {
        if limits.global_bytes == 0
            || limits.per_tenant_bytes == 0
            || limits.max_tenants == 0
            || limits.max_pending_waiters == 0
            || limits.per_tenant_bytes > limits.global_bytes
            || limits.progress_pool_bytes == 0
            || limits.progress_pool_bytes >= limits.global_bytes
            || limits
                .global_category_bytes
                .iter()
                .any(|limit| *limit == 0 || *limit > limits.global_bytes)
            || limits
                .per_tenant_category_bytes
                .iter()
                .any(|limit| *limit == 0 || *limit > limits.per_tenant_bytes)
        {
            return Err(ResourceError::InvalidLimits);
        }
        if disk_limits.global_bytes == 0
            || disk_limits.per_tenant_bytes == 0
            || disk_limits.max_tenants == 0
            || disk_limits.per_tenant_bytes > disk_limits.global_bytes
            || disk_limits
                .global_category_bytes
                .iter()
                .any(|limit| *limit == 0 || *limit > disk_limits.global_bytes)
            || disk_limits
                .per_tenant_category_bytes
                .iter()
                .any(|limit| *limit == 0 || *limit > disk_limits.per_tenant_bytes)
        {
            return Err(ResourceError::InvalidLimits);
        }
        Ok(Self {
            inner: Arc::new(Inner {
                limits,
                state: Mutex::new(State::default()),
                capacity_changed: Notify::new(),
            }),
            disk: BlinkDiskResourceManager {
                inner: Arc::new(DiskInner {
                    limits: disk_limits,
                    state: Mutex::new(DiskState::default()),
                    capacity_changed: Notify::new(),
                }),
            },
        })
    }

    pub fn disk(&self) -> BlinkDiskResourceManager {
        self.disk.clone()
    }
}

impl BlinkDiskResourceManager {
    pub async fn reserve_wait(
        &self,
        tenant_id: u64,
        category: DiskCategory,
        bytes: u64,
        wait_limit: Duration,
    ) -> Result<DiskReservation, ResourceError> {
        let category_index = category as usize;
        if bytes > self.inner.limits.global_bytes
            || bytes > self.inner.limits.per_tenant_bytes
            || bytes > self.inner.limits.global_category_bytes[category_index]
            || bytes > self.inner.limits.per_tenant_category_bytes[category_index]
        {
            return Err(ResourceError::CapacityExceeded);
        }
        let _waiter_registration = self.register_waiter()?;
        let deadline = tokio::time::Instant::now() + wait_limit;
        loop {
            let capacity_changed = self.inner.capacity_changed.notified();
            tokio::pin!(capacity_changed);
            capacity_changed.as_mut().enable();
            match self.reserve(tenant_id, category, bytes) {
                Ok(reservation) => return Ok(reservation),
                Err(ResourceError::CapacityExceeded | ResourceError::TooManyTenants) => {}
                Err(error) => return Err(error),
            }
            tokio::time::timeout_at(deadline, capacity_changed)
                .await
                .map_err(|_| ResourceError::WaitTimedOut)?;
        }
    }

    fn register_waiter(&self) -> Result<DiskWaiterRegistration, ResourceError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        if state.pending_waiters == self.inner.limits.max_tenants {
            return Err(ResourceError::TooManyWaiters);
        }
        state.pending_waiters += 1;
        Ok(DiskWaiterRegistration {
            inner: Arc::clone(&self.inner),
        })
    }

    pub fn reconcile_persistent_usage(
        &self,
        tenant_id: u64,
        category: DiskCategory,
        key: impl Into<String>,
        bytes: u64,
    ) -> Result<(), ResourceError> {
        self.update_persistent_usage(tenant_id, category, key.into(), bytes)
    }

    pub fn replace_persistent_usage_from_reservation(
        &self,
        mut reservation: DiskReservation,
        key: impl Into<String>,
        bytes: u64,
    ) -> Result<(), ResourceError> {
        let charge = Arc::get_mut(&mut reservation.charge).ok_or(ResourceError::SharedOwnership)?;
        if !Arc::ptr_eq(&charge.inner, &self.inner) {
            return Err(ResourceError::SharedOwnership);
        }
        let key = (charge.tenant_id, charge.category, key.into());
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let previous = state.persistent_usage.get(&key).copied().unwrap_or(0);
        let removed = previous
            .checked_add(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_total = state
            .current_total
            .checked_sub(removed)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let category_index = charge.category as usize;
        let next_category = state.current_by_category[category_index]
            .checked_sub(removed)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_state = state
            .tenants
            .get(&charge.tenant_id)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_tenant_total = tenant_state
            .current_total
            .checked_sub(removed)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_tenant_category = tenant_state.current_by_category[category_index]
            .checked_sub(removed)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if next_total > self.inner.limits.global_bytes
            || next_category > self.inner.limits.global_category_bytes[category_index]
            || next_tenant_total > self.inner.limits.per_tenant_bytes
            || next_tenant_category > self.inner.limits.per_tenant_category_bytes[category_index]
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::CapacityExceeded);
        }
        let tenant_state = state
            .tenants
            .get_mut(&charge.tenant_id)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        tenant_state.current_total = next_tenant_total;
        tenant_state.current_by_category[category_index] = next_tenant_category;
        tenant_state.peak_total = tenant_state.peak_total.max(next_tenant_total);
        tenant_state.peak_by_category[category_index] =
            tenant_state.peak_by_category[category_index].max(next_tenant_category);
        state.current_total = next_total;
        state.current_by_category[category_index] = next_category;
        state.peak_total = state.peak_total.max(next_total);
        state.peak_by_category[category_index] =
            state.peak_by_category[category_index].max(next_category);
        if bytes == 0 {
            state.persistent_usage.remove(&key);
        } else {
            state.persistent_usage.insert(key, bytes);
        }
        charge.bytes = 0;
        if next_tenant_total == 0 {
            state.tenants.remove(&charge.tenant_id);
        }
        drop(state);
        self.inner.capacity_changed.notify_waiters();
        Ok(())
    }

    pub fn increase_persistent_usage_from_reservation(
        &self,
        mut reservation: DiskReservation,
        key: impl Into<String>,
        bytes: u64,
    ) -> Result<(), ResourceError> {
        let charge = Arc::get_mut(&mut reservation.charge).ok_or(ResourceError::SharedOwnership)?;
        if !Arc::ptr_eq(&charge.inner, &self.inner) {
            return Err(ResourceError::SharedOwnership);
        }
        let key = (charge.tenant_id, charge.category, key.into());
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let previous = state.persistent_usage.get(&key).copied().unwrap_or(0);
        let next_persistent = previous
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_total = state
            .current_total
            .checked_sub(charge.bytes)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let category_index = charge.category as usize;
        let next_category = state.current_by_category[category_index]
            .checked_sub(charge.bytes)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_state = state
            .tenants
            .get(&charge.tenant_id)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_tenant_total = tenant_state
            .current_total
            .checked_sub(charge.bytes)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_tenant_category = tenant_state.current_by_category[category_index]
            .checked_sub(charge.bytes)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if next_total > self.inner.limits.global_bytes
            || next_category > self.inner.limits.global_category_bytes[category_index]
            || next_tenant_total > self.inner.limits.per_tenant_bytes
            || next_tenant_category > self.inner.limits.per_tenant_category_bytes[category_index]
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::CapacityExceeded);
        }
        let tenant_state = state
            .tenants
            .get_mut(&charge.tenant_id)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        tenant_state.current_total = next_tenant_total;
        tenant_state.current_by_category[category_index] = next_tenant_category;
        tenant_state.peak_total = tenant_state.peak_total.max(next_tenant_total);
        tenant_state.peak_by_category[category_index] =
            tenant_state.peak_by_category[category_index].max(next_tenant_category);
        state.current_total = next_total;
        state.current_by_category[category_index] = next_category;
        state.peak_total = state.peak_total.max(next_total);
        state.peak_by_category[category_index] =
            state.peak_by_category[category_index].max(next_category);
        if next_persistent == 0 {
            state.persistent_usage.remove(&key);
        } else {
            state.persistent_usage.insert(key, next_persistent);
        }
        charge.bytes = 0;
        if next_tenant_total == 0 {
            state.tenants.remove(&charge.tenant_id);
        }
        drop(state);
        self.inner.capacity_changed.notify_waiters();
        Ok(())
    }

    fn update_persistent_usage(
        &self,
        tenant_id: u64,
        category: DiskCategory,
        key: String,
        bytes: u64,
    ) -> Result<(), ResourceError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let persistent_key = (tenant_id, category, key);
        let previous = state
            .persistent_usage
            .get(&persistent_key)
            .copied()
            .unwrap_or(0);
        let category_index = category as usize;
        let next_total = state
            .current_total
            .checked_sub(previous)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_category = state.current_by_category[category_index]
            .checked_sub(previous)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_total = state
            .tenants
            .get(&tenant_id)
            .map_or(0, |entry| entry.current_total);
        let tenant_category = state
            .tenants
            .get(&tenant_id)
            .map_or(0, |entry| entry.current_by_category[category_index]);
        let next_tenant_total = tenant_total
            .checked_sub(previous)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_tenant_category = tenant_category
            .checked_sub(previous)
            .and_then(|current| current.checked_add(bytes))
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if next_total > self.inner.limits.global_bytes
            || next_category > self.inner.limits.global_category_bytes[category_index]
            || next_tenant_total > self.inner.limits.per_tenant_bytes
            || next_tenant_category > self.inner.limits.per_tenant_category_bytes[category_index]
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::CapacityExceeded);
        }
        if !state.tenants.contains_key(&tenant_id)
            && state.tenants.len() == self.inner.limits.max_tenants
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::TooManyTenants);
        }
        let tenant_state = state.tenants.entry(tenant_id).or_default();
        tenant_state.current_total = next_tenant_total;
        tenant_state.current_by_category[category_index] = next_tenant_category;
        tenant_state.peak_total = tenant_state.peak_total.max(next_tenant_total);
        tenant_state.peak_by_category[category_index] =
            tenant_state.peak_by_category[category_index].max(next_tenant_category);
        state.current_total = next_total;
        state.current_by_category[category_index] = next_category;
        state.peak_total = state.peak_total.max(next_total);
        state.peak_by_category[category_index] =
            state.peak_by_category[category_index].max(next_category);
        if bytes == 0 {
            state.persistent_usage.remove(&persistent_key);
        } else {
            state.persistent_usage.insert(persistent_key, bytes);
        }
        if next_tenant_total == 0 {
            state.tenants.remove(&tenant_id);
        }
        drop(state);
        if bytes < previous {
            self.inner.capacity_changed.notify_waiters();
        }
        Ok(())
    }

    pub fn reserve(
        &self,
        tenant_id: u64,
        category: DiskCategory,
        bytes: u64,
    ) -> Result<DiskReservation, ResourceError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let category_index = category as usize;
        let next_total = state
            .current_total
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_state = state.tenants.get(&tenant_id);
        let tenant_total = tenant_state.map_or(0, |entry| entry.current_total);
        let next_tenant_total = tenant_total
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let current_category = state.current_by_category[category_index];
        let next_category = current_category
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_category =
            tenant_state.map_or(0, |entry| entry.current_by_category[category_index]);
        let next_tenant_category = tenant_category
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if next_total > self.inner.limits.global_bytes
            || next_tenant_total > self.inner.limits.per_tenant_bytes
            || next_category > self.inner.limits.global_category_bytes[category_index]
            || next_tenant_category > self.inner.limits.per_tenant_category_bytes[category_index]
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::CapacityExceeded);
        }
        if !state.tenants.contains_key(&tenant_id)
            && state.tenants.len() == self.inner.limits.max_tenants
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::TooManyTenants);
        }
        let tenant_state = state.tenants.entry(tenant_id).or_default();
        tenant_state.current_by_category[category_index] = next_tenant_category;
        tenant_state.peak_by_category[category_index] =
            tenant_state.peak_by_category[category_index].max(next_tenant_category);
        tenant_state.current_total = next_tenant_total;
        tenant_state.peak_total = tenant_state.peak_total.max(next_tenant_total);
        state.current_by_category[category_index] = next_category;
        state.peak_by_category[category_index] =
            state.peak_by_category[category_index].max(next_category);
        state.current_total = next_total;
        state.peak_total = state.peak_total.max(next_total);
        Ok(DiskReservation {
            charge: Arc::new(DiskCharge {
                inner: Arc::clone(&self.inner),
                tenant_id,
                category,
                bytes,
            }),
        })
    }

    pub fn snapshot(&self) -> Result<DiskResourceSnapshot, ResourceError> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        Ok(DiskResourceSnapshot {
            current_by_category: state.current_by_category,
            peak_by_category: state.peak_by_category,
            current_total: state.current_total,
            peak_total: state.peak_total,
            active_tenants: state.tenants.len(),
            pending_waiters: state.pending_waiters,
            rejected_reservations: state.rejected_reservations,
        })
    }

    pub fn tenant_snapshot(
        &self,
        tenant_id: u64,
    ) -> Result<Option<TenantDiskResourceSnapshot>, ResourceError> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        Ok(state
            .tenants
            .get(&tenant_id)
            .map(|tenant_state| TenantDiskResourceSnapshot {
                current_by_category: tenant_state.current_by_category,
                peak_by_category: tenant_state.peak_by_category,
                current_total: tenant_state.current_total,
                peak_total: tenant_state.peak_total,
            }))
    }
}

impl Drop for DiskWaiterRegistration {
    fn drop(&mut self) {
        if let Ok(mut state) = self.inner.state.lock() {
            state.pending_waiters = state.pending_waiters.saturating_sub(1);
        }
    }
}

impl BlinkResourceManager {
    pub fn reserve(
        &self,
        tenant_id: u64,
        category: ResourceCategory,
        class: ReservationClass,
        bytes: u64,
    ) -> Result<Reservation, ResourceError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let next_total = state
            .current_total
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_total = state
            .tenants
            .get(&tenant_id)
            .map_or(0, |tenant_state| tenant_state.current_total);
        let next_tenant_total = tenant_total
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let next_progress = if class == ReservationClass::Progress {
            Some(
                state
                    .current_progress
                    .checked_add(bytes)
                    .ok_or(ResourceError::ArithmeticOverflow)?,
            )
        } else {
            None
        };
        let foreground_limit =
            self.inner.limits.global_bytes - self.inner.limits.progress_pool_bytes;
        let class_capacity_exceeded = match class {
            ReservationClass::Foreground => next_total > foreground_limit,
            ReservationClass::Progress => {
                next_total > self.inner.limits.global_bytes
                    || next_progress.unwrap_or(0) > self.inner.limits.progress_pool_bytes
            }
        };
        if class_capacity_exceeded || next_tenant_total > self.inner.limits.per_tenant_bytes {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::CapacityExceeded);
        }
        if !state.tenants.contains_key(&tenant_id)
            && state.tenants.len() == self.inner.limits.max_tenants
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::TooManyTenants);
        }
        let category_index = category as usize;
        let next_category = state.current_by_category[category_index]
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_category = state.tenants.get(&tenant_id).map_or(0, |tenant_state| {
            tenant_state.current_by_category[category_index]
        });
        let next_tenant_category = tenant_category
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if next_category > self.inner.limits.global_category_bytes[category_index]
            || next_tenant_category > self.inner.limits.per_tenant_category_bytes[category_index]
        {
            state.rejected_reservations = state.rejected_reservations.saturating_add(1);
            return Err(ResourceError::CapacityExceeded);
        }
        let tenant_state = state.tenants.entry(tenant_id).or_default();
        tenant_state.current_by_category[category_index] = next_tenant_category;
        tenant_state.peak_by_category[category_index] =
            tenant_state.peak_by_category[category_index].max(next_tenant_category);
        tenant_state.current_total = next_tenant_total;
        tenant_state.peak_total = tenant_state.peak_total.max(next_tenant_total);
        state.current_by_category[category_index] = next_category;
        state.peak_by_category[category_index] =
            state.peak_by_category[category_index].max(next_category);
        state.current_total = next_total;
        state.peak_total = state.peak_total.max(next_total);
        if let Some(next_progress) = next_progress {
            state.current_progress = next_progress;
            state.peak_progress = state.peak_progress.max(next_progress);
        }
        Ok(Reservation {
            charge: Arc::new(Charge {
                inner: Arc::clone(&self.inner),
                tenant_id,
                category,
                class,
                bytes,
            }),
        })
    }

    pub async fn reserve_wait(
        &self,
        tenant_id: u64,
        category: ResourceCategory,
        class: ReservationClass,
        bytes: u64,
        wait_limit: Duration,
    ) -> Result<Reservation, ResourceError> {
        let _waiter_registration = self.register_waiter()?;
        let deadline = tokio::time::Instant::now() + wait_limit;
        loop {
            let capacity_changed = self.inner.capacity_changed.notified();
            tokio::pin!(capacity_changed);
            capacity_changed.as_mut().enable();
            match self.reserve(tenant_id, category, class, bytes) {
                Ok(reservation) => return Ok(reservation),
                Err(ResourceError::CapacityExceeded | ResourceError::TooManyTenants) => {}
                Err(error) => return Err(error),
            }
            tokio::time::timeout_at(deadline, capacity_changed)
                .await
                .map_err(|_| ResourceError::WaitTimedOut)?;
        }
    }

    pub fn snapshot(&self) -> Result<ResourceSnapshot, ResourceError> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        Ok(ResourceSnapshot {
            current_by_category: state.current_by_category,
            peak_by_category: state.peak_by_category,
            current_total: state.current_total,
            peak_total: state.peak_total,
            current_progress: state.current_progress,
            peak_progress: state.peak_progress,
            active_tenants: state.tenants.len(),
            rejected_reservations: state.rejected_reservations,
            pending_waiters: state.pending_waiters,
        })
    }

    pub fn tenant_snapshot(
        &self,
        tenant_id: u64,
    ) -> Result<Option<TenantResourceSnapshot>, ResourceError> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        Ok(state
            .tenants
            .get(&tenant_id)
            .map(|tenant_state| TenantResourceSnapshot {
                current_by_category: tenant_state.current_by_category,
                peak_by_category: tenant_state.peak_by_category,
                current_total: tenant_state.current_total,
                peak_total: tenant_state.peak_total,
            }))
    }

    fn register_waiter(&self) -> Result<WaiterRegistration, ResourceError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        if state.pending_waiters == self.inner.limits.max_pending_waiters {
            return Err(ResourceError::TooManyWaiters);
        }
        state.pending_waiters += 1;
        Ok(WaiterRegistration {
            inner: Arc::clone(&self.inner),
        })
    }
}

impl Drop for WaiterRegistration {
    fn drop(&mut self) {
        if let Ok(mut state) = self.inner.state.lock() {
            state.pending_waiters = state.pending_waiters.saturating_sub(1);
        }
    }
}

impl Reservation {
    pub fn bytes(&self) -> u64 {
        self.charge.bytes
    }

    pub fn category(&self) -> ResourceCategory {
        self.charge.category
    }

    pub fn share(&self) -> Self {
        Self {
            charge: Arc::clone(&self.charge),
        }
    }

    pub fn transfer_to(&mut self, category: ResourceCategory) -> Result<(), ResourceError> {
        if self.charge.category == category {
            return Ok(());
        }
        let charge = Arc::get_mut(&mut self.charge).ok_or(ResourceError::SharedOwnership)?;
        let mut state = charge
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let old_index = charge.category as usize;
        let new_index = category as usize;
        let new_global = state.current_by_category[new_index]
            .checked_add(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_state = state
            .tenants
            .get_mut(&charge.tenant_id)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let new_tenant = tenant_state.current_by_category[new_index]
            .checked_add(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if new_global > charge.inner.limits.global_category_bytes[new_index]
            || new_tenant > charge.inner.limits.per_tenant_category_bytes[new_index]
        {
            return Err(ResourceError::CapacityExceeded);
        }
        tenant_state.current_by_category[old_index] = tenant_state.current_by_category[old_index]
            .checked_sub(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        tenant_state.current_by_category[new_index] = new_tenant;
        tenant_state.peak_by_category[new_index] =
            tenant_state.peak_by_category[new_index].max(new_tenant);
        state.current_by_category[old_index] = state.current_by_category[old_index]
            .checked_sub(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        state.current_by_category[new_index] = new_global;
        state.peak_by_category[new_index] = state.peak_by_category[new_index].max(new_global);
        charge.category = category;
        drop(state);
        charge.inner.capacity_changed.notify_waiters();
        Ok(())
    }
}

impl DiskReservation {
    pub fn bytes(&self) -> u64 {
        self.charge.bytes
    }

    pub fn category(&self) -> DiskCategory {
        self.charge.category
    }

    pub fn share(&self) -> Self {
        Self {
            charge: Arc::clone(&self.charge),
        }
    }

    pub fn grow(&mut self, bytes: u64) -> Result<(), ResourceError> {
        if bytes == 0 {
            return Ok(());
        }
        let charge = Arc::get_mut(&mut self.charge).ok_or(ResourceError::SharedOwnership)?;
        let manager = BlinkDiskResourceManager {
            inner: Arc::clone(&charge.inner),
        };
        let additional = manager.reserve(charge.tenant_id, charge.category, bytes)?;
        self.merge(additional)
    }

    pub fn shrink(&mut self, bytes: u64) -> Result<(), ResourceError> {
        if bytes == 0 {
            return Ok(());
        }
        let charge = Arc::get_mut(&mut self.charge).ok_or(ResourceError::SharedOwnership)?;
        if bytes > charge.bytes {
            return Err(ResourceError::ArithmeticOverflow);
        }
        let mut state = charge
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let category_index = charge.category as usize;
        state.current_by_category[category_index] = state.current_by_category[category_index]
            .checked_sub(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        state.current_total = state
            .current_total
            .checked_sub(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_state = state
            .tenants
            .get_mut(&charge.tenant_id)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        tenant_state.current_by_category[category_index] = tenant_state.current_by_category
            [category_index]
            .checked_sub(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        tenant_state.current_total = tenant_state
            .current_total
            .checked_sub(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let remove_tenant = tenant_state.current_total == 0;
        charge.bytes = charge
            .bytes
            .checked_sub(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if remove_tenant {
            state.tenants.remove(&charge.tenant_id);
        }
        drop(state);
        charge.inner.capacity_changed.notify_waiters();
        Ok(())
    }

    pub fn merge(&mut self, mut additional: Self) -> Result<(), ResourceError> {
        if !Arc::ptr_eq(&self.charge.inner, &additional.charge.inner)
            || self.charge.tenant_id != additional.charge.tenant_id
            || self.charge.category != additional.charge.category
        {
            return Err(ResourceError::SharedOwnership);
        }
        let charge = Arc::get_mut(&mut self.charge).ok_or(ResourceError::SharedOwnership)?;
        let additional_charge =
            Arc::get_mut(&mut additional.charge).ok_or(ResourceError::SharedOwnership)?;
        charge.bytes = charge
            .bytes
            .checked_add(additional_charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        additional_charge.bytes = 0;
        Ok(())
    }

    pub fn transfer_to(&mut self, category: DiskCategory) -> Result<(), ResourceError> {
        if self.charge.category == category {
            return Ok(());
        }
        let charge = Arc::get_mut(&mut self.charge).ok_or(ResourceError::SharedOwnership)?;
        let mut state = charge
            .inner
            .state
            .lock()
            .map_err(|_| ResourceError::Poisoned)?;
        let old_index = charge.category as usize;
        let new_index = category as usize;
        let new_global = state.current_by_category[new_index]
            .checked_add(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let tenant_state = state
            .tenants
            .get_mut(&charge.tenant_id)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        let new_tenant = tenant_state.current_by_category[new_index]
            .checked_add(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        if new_global > charge.inner.limits.global_category_bytes[new_index]
            || new_tenant > charge.inner.limits.per_tenant_category_bytes[new_index]
        {
            return Err(ResourceError::CapacityExceeded);
        }
        tenant_state.current_by_category[old_index] = tenant_state.current_by_category[old_index]
            .checked_sub(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        tenant_state.current_by_category[new_index] = new_tenant;
        tenant_state.peak_by_category[new_index] =
            tenant_state.peak_by_category[new_index].max(new_tenant);
        state.current_by_category[old_index] = state.current_by_category[old_index]
            .checked_sub(charge.bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
        state.current_by_category[new_index] = new_global;
        state.peak_by_category[new_index] = state.peak_by_category[new_index].max(new_global);
        charge.category = category;
        Ok(())
    }
}

impl Drop for Charge {
    fn drop(&mut self) {
        let Ok(mut state) = self.inner.state.lock() else {
            return;
        };
        let category_index = self.category as usize;
        state.current_by_category[category_index] =
            state.current_by_category[category_index].saturating_sub(self.bytes);
        state.current_total = state.current_total.saturating_sub(self.bytes);
        if self.class == ReservationClass::Progress {
            state.current_progress = state.current_progress.saturating_sub(self.bytes);
        }
        let remove_tenant = if let Some(tenant_state) = state.tenants.get_mut(&self.tenant_id) {
            tenant_state.current_by_category[category_index] =
                tenant_state.current_by_category[category_index].saturating_sub(self.bytes);
            tenant_state.current_total = tenant_state.current_total.saturating_sub(self.bytes);
            tenant_state.current_total == 0
        } else {
            false
        };
        if remove_tenant {
            state.tenants.remove(&self.tenant_id);
        }
        drop(state);
        self.inner.capacity_changed.notify_waiters();
    }
}

impl Drop for DiskCharge {
    fn drop(&mut self) {
        let Ok(mut state) = self.inner.state.lock() else {
            return;
        };
        let category_index = self.category as usize;
        state.current_by_category[category_index] =
            state.current_by_category[category_index].saturating_sub(self.bytes);
        state.current_total = state.current_total.saturating_sub(self.bytes);
        let remove_tenant = if let Some(tenant_state) = state.tenants.get_mut(&self.tenant_id) {
            tenant_state.current_by_category[category_index] =
                tenant_state.current_by_category[category_index].saturating_sub(self.bytes);
            tenant_state.current_total = tenant_state.current_total.saturating_sub(self.bytes);
            tenant_state.current_total == 0
        } else {
            false
        };
        if remove_tenant {
            state.tenants.remove(&self.tenant_id);
        }
        drop(state);
        self.inner.capacity_changed.notify_waiters();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BlinkResourceManager, CATEGORY_COUNT, DISK_CATEGORY_COUNT, DiskCategory,
        DiskResourceLimits, ReservationClass, ResourceCategory, ResourceError, ResourceLimits,
    };
    use std::time::Duration;

    fn manager() -> BlinkResourceManager {
        BlinkResourceManager::new(ResourceLimits {
            global_bytes: 100,
            per_tenant_bytes: 80,
            progress_pool_bytes: 20,
            max_tenants: 2,
            max_pending_waiters: 2,
            global_category_bytes: [100; CATEGORY_COUNT],
            per_tenant_category_bytes: [80; CATEGORY_COUNT],
        })
        .unwrap()
    }

    fn disk_manager() -> BlinkResourceManager {
        BlinkResourceManager::new_with_disk_limits(
            ResourceLimits {
                global_bytes: 100,
                per_tenant_bytes: 80,
                progress_pool_bytes: 20,
                max_tenants: 2,
                max_pending_waiters: 2,
                global_category_bytes: [100; CATEGORY_COUNT],
                per_tenant_category_bytes: [80; CATEGORY_COUNT],
            },
            DiskResourceLimits {
                global_bytes: 100,
                per_tenant_bytes: 80,
                max_tenants: 2,
                global_category_bytes: [100; DISK_CATEGORY_COUNT],
                per_tenant_category_bytes: [80; DISK_CATEGORY_COUNT],
            },
        )
        .unwrap()
    }

    #[test]
    fn disk_reservations_enforce_global_tenant_and_category_limits() {
        let manager = disk_manager().disk();
        let retained = manager
            .reserve(1, DiskCategory::RetainedSource, 60)
            .unwrap();
        assert!(manager.reserve(1, DiskCategory::Wal, 21).is_err());
        assert!(manager.reserve(2, DiskCategory::Wal, 41).is_err());
        assert_eq!(retained.bytes(), 60);
        let snapshot = manager.snapshot().unwrap();
        assert_eq!(snapshot.current_total, 60);
        assert_eq!(snapshot.rejected_reservations, 2);
        drop(retained);
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
        let later_reservation = manager.reserve(2, DiskCategory::Wal, 40).unwrap();
        assert_eq!(later_reservation.bytes(), 40);
    }

    #[tokio::test]
    async fn disk_capacity_waiter_resumes_after_retained_source_release() {
        let manager = disk_manager().disk();
        manager
            .reconcile_persistent_usage(1, DiskCategory::RetainedSource, "/tenant-1", 60)
            .unwrap();
        let waiting_manager = manager.clone();
        let waiter = tokio::spawn(async move {
            waiting_manager
                .reserve_wait(
                    2,
                    DiskCategory::CheckpointOutput,
                    50,
                    Duration::from_secs(1),
                )
                .await
        });
        tokio::task::yield_now().await;
        assert_eq!(manager.snapshot().unwrap().pending_waiters, 1);
        manager
            .reconcile_persistent_usage(1, DiskCategory::RetainedSource, "/tenant-1", 40)
            .unwrap();
        let reservation = waiter.await.unwrap().unwrap();
        assert_eq!(reservation.bytes(), 50);
        let snapshot = manager.snapshot().unwrap();
        assert_eq!(snapshot.current_total, 90);
        assert_eq!(snapshot.pending_waiters, 0);
    }

    #[tokio::test]
    async fn disk_capacity_waiters_are_bounded_and_cancellation_releases_a_slot() {
        let manager = disk_manager().disk();
        manager
            .reconcile_persistent_usage(1, DiskCategory::RetainedSource, "/tenant-1", 80)
            .unwrap();
        let first_manager = manager.clone();
        let first_waiter = tokio::spawn(async move {
            first_manager
                .reserve_wait(
                    2,
                    DiskCategory::CheckpointOutput,
                    30,
                    Duration::from_secs(1),
                )
                .await
        });
        let second_manager = manager.clone();
        let second_waiter = tokio::spawn(async move {
            second_manager
                .reserve_wait(
                    2,
                    DiskCategory::CheckpointOutput,
                    30,
                    Duration::from_secs(1),
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if manager.snapshot().unwrap().pending_waiters == 2 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(matches!(
            manager
                .reserve_wait(3, DiskCategory::CheckpointOutput, 1, Duration::from_secs(1),)
                .await,
            Err(ResourceError::TooManyWaiters)
        ));
        first_waiter.abort();
        assert!(first_waiter.await.is_err_and(|error| error.is_cancelled()));
        assert_eq!(manager.snapshot().unwrap().pending_waiters, 1);
        manager
            .reconcile_persistent_usage(1, DiskCategory::RetainedSource, "/tenant-1", 0)
            .unwrap();
        assert_eq!(second_waiter.await.unwrap().unwrap().bytes(), 30);
        assert_eq!(manager.snapshot().unwrap().pending_waiters, 0);
    }

    #[test]
    fn disk_reservations_transfer_grow_shrink_and_merge_charges() {
        let manager = disk_manager().disk();
        let mut output = manager
            .reserve(7, DiskCategory::CheckpointOutput, 20)
            .unwrap();
        output.grow(10).unwrap();
        output.transfer_to(DiskCategory::RetainedSource).unwrap();
        let extra = manager
            .reserve(7, DiskCategory::RetainedSource, 15)
            .unwrap();
        output.merge(extra).unwrap();
        assert_eq!(output.bytes(), 45);
        output.shrink(5).unwrap();
        let snapshot = manager.tenant_snapshot(7).unwrap().unwrap();
        assert_eq!(snapshot.current_total, 40);
        assert_eq!(
            snapshot.current_by_category[DiskCategory::RetainedSource as usize],
            40
        );
        drop(output);
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[test]
    fn persistent_disk_usage_survives_owner_drop_and_reconciles_idempotently() {
        let manager = disk_manager().disk();
        manager
            .reconcile_persistent_usage(7, DiskCategory::RetainedSource, "/store", 60)
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 60);
        manager
            .reconcile_persistent_usage(7, DiskCategory::RetainedSource, "/store", 60)
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 60);
        manager
            .reconcile_persistent_usage(7, DiskCategory::RetainedSource, "/store", 35)
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 35);
        manager
            .reconcile_persistent_usage(7, DiskCategory::RetainedSource, "/store", 0)
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[test]
    fn disk_reservation_can_be_atomically_converted_to_persistent_usage() {
        let manager = disk_manager().disk();
        manager
            .reconcile_persistent_usage(7, DiskCategory::RetainedSource, "/store", 40)
            .unwrap();
        let output = manager
            .reserve(7, DiskCategory::RetainedSource, 25)
            .unwrap();
        manager
            .replace_persistent_usage_from_reservation(output, "/store", 65)
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 65);
        manager
            .reconcile_persistent_usage(7, DiskCategory::RetainedSource, "/store", 50)
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 50);
    }

    #[test]
    fn foreground_reservation_accepts_exact_limit_and_releases_on_drop() {
        let manager = manager();
        let reservation = manager
            .reserve(
                7,
                ResourceCategory::DecodedPayload,
                ReservationClass::Foreground,
                80,
            )
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 80);
        assert_eq!(reservation.bytes(), 80);
        drop(reservation);
        let snapshot = manager.snapshot().unwrap();
        assert_eq!(snapshot.current_total, 0);
        assert_eq!(snapshot.active_tenants, 0);
    }

    #[test]
    fn foreground_cannot_borrow_progress_pool() {
        let manager = manager();
        let foreground = manager
            .reserve(
                7,
                ResourceCategory::QueuedForeground,
                ReservationClass::Foreground,
                80,
            )
            .unwrap();
        assert!(matches!(
            manager.reserve(
                7,
                ResourceCategory::QueuedForeground,
                ReservationClass::Foreground,
                1
            ),
            Err(ResourceError::CapacityExceeded)
        ));
        let progress = manager
            .reserve(
                8,
                ResourceCategory::CheckpointValidation,
                ReservationClass::Progress,
                20,
            )
            .unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 100);
        drop((foreground, progress));
    }

    #[test]
    fn transfer_changes_categories_without_changing_total() {
        let manager = manager();
        let mut reservation = manager
            .reserve(
                7,
                ResourceCategory::LoadReconstruction,
                ReservationClass::Foreground,
                12,
            )
            .unwrap();
        reservation
            .transfer_to(ResourceCategory::DecodedPayload)
            .unwrap();
        let tenant_snapshot = manager.tenant_snapshot(7).unwrap().unwrap();
        assert_eq!(tenant_snapshot.current_total, 12);
        assert_eq!(tenant_snapshot.current_by_category[2], 0);
        assert_eq!(tenant_snapshot.current_by_category[0], 12);
        assert_eq!(reservation.category(), ResourceCategory::DecodedPayload);
    }

    #[test]
    fn shared_charge_releases_only_after_last_owner_drops() {
        let manager = manager();
        let reservation = manager
            .reserve(
                7,
                ResourceCategory::DecodedPayload,
                ReservationClass::Foreground,
                12,
            )
            .unwrap();
        let shared_owner = reservation.share();
        drop(reservation);
        assert_eq!(manager.snapshot().unwrap().current_total, 12);
        drop(shared_owner);
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[test]
    fn tenant_limit_and_shared_global_limit_are_enforced() {
        let manager = manager();
        let first_tenant = manager
            .reserve(
                7,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                20,
            )
            .unwrap();
        assert!(matches!(
            manager.reserve(
                7,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                61
            ),
            Err(ResourceError::CapacityExceeded)
        ));
        let second_tenant = manager
            .reserve(
                8,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                10,
            )
            .unwrap();
        assert!(matches!(
            manager.reserve(
                9,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                1
            ),
            Err(ResourceError::TooManyTenants)
        ));
        drop((first_tenant, second_tenant));
    }

    #[test]
    fn rejects_progress_pool_that_consumes_global_capacity() {
        assert_eq!(
            BlinkResourceManager::new(ResourceLimits {
                global_bytes: 10,
                per_tenant_bytes: 10,
                progress_pool_bytes: 10,
                max_tenants: 1,
                max_pending_waiters: 2,
                global_category_bytes: [10; CATEGORY_COUNT],
                per_tenant_category_bytes: [10; CATEGORY_COUNT],
            })
            .err(),
            Some(ResourceError::InvalidLimits)
        );
    }

    #[test]
    fn reservation_drop_releases_charge_after_unwind() {
        let manager = manager();
        let unwind_manager = manager.clone();
        let panic_result = std::panic::catch_unwind(move || {
            let _reservation = unwind_manager
                .reserve(
                    7,
                    ResourceCategory::WritePreparation,
                    ReservationClass::Foreground,
                    15,
                )
                .unwrap();
            assert_eq!(unwind_manager.snapshot().unwrap().current_total, 15);
            panic!("reservation owner unwound");
        });
        assert!(panic_result.is_err());
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[test]
    fn checked_arithmetic_rejects_overflow_without_changing_usage() {
        let manager = BlinkResourceManager::new(ResourceLimits {
            global_bytes: u64::MAX,
            per_tenant_bytes: u64::MAX,
            progress_pool_bytes: 1,
            max_tenants: 1,
            max_pending_waiters: 2,
            global_category_bytes: [u64::MAX; CATEGORY_COUNT],
            per_tenant_category_bytes: [u64::MAX; CATEGORY_COUNT],
        })
        .unwrap();
        let reservation = manager
            .reserve(
                1,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                u64::MAX - 1,
            )
            .unwrap();
        assert!(matches!(
            manager.reserve(
                1,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                2
            ),
            Err(ResourceError::ArithmeticOverflow)
        ));
        assert_eq!(manager.snapshot().unwrap().current_total, u64::MAX - 1);
        drop(reservation);
    }

    #[test]
    fn shared_manager_serializes_competing_tenant_reservations() {
        let manager = BlinkResourceManager::new(ResourceLimits {
            global_bytes: 100,
            per_tenant_bytes: 100,
            progress_pool_bytes: 1,
            max_tenants: 2,
            max_pending_waiters: 2,
            global_category_bytes: [100; CATEGORY_COUNT],
            per_tenant_category_bytes: [100; CATEGORY_COUNT],
        })
        .unwrap();
        let first_manager = manager.clone();
        let second_manager = manager.clone();
        let start_barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let first_barrier = std::sync::Arc::clone(&start_barrier);
        let second_barrier = std::sync::Arc::clone(&start_barrier);
        let first_task = std::thread::spawn(move || {
            first_barrier.wait();
            first_manager.reserve(
                1,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                60,
            )
        });
        let second_task = std::thread::spawn(move || {
            second_barrier.wait();
            second_manager.reserve(
                2,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                60,
            )
        });
        start_barrier.wait();
        let first_result = first_task.join().unwrap();
        let second_result = second_task.join().unwrap();
        assert_ne!(first_result.is_ok(), second_result.is_ok());
        let snapshot = manager.snapshot().unwrap();
        assert_eq!(snapshot.current_total, 60);
        assert_eq!(snapshot.peak_total, 60);
        drop((first_result, second_result));
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[test]
    fn category_limits_reject_before_accounting_changes() {
        let manager = BlinkResourceManager::new(ResourceLimits {
            global_bytes: 100,
            per_tenant_bytes: 100,
            progress_pool_bytes: 10,
            max_tenants: 1,
            max_pending_waiters: 2,
            global_category_bytes: [10, 100, 100, 100, 100, 100, 100],
            per_tenant_category_bytes: [8, 100, 100, 100, 100, 100, 100],
        })
        .unwrap();
        let first_charge = manager
            .reserve(
                1,
                ResourceCategory::DecodedPayload,
                ReservationClass::Foreground,
                8,
            )
            .unwrap();
        assert!(matches!(
            manager.reserve(
                1,
                ResourceCategory::DecodedPayload,
                ReservationClass::Foreground,
                1
            ),
            Err(ResourceError::CapacityExceeded)
        ));
        assert_eq!(manager.snapshot().unwrap().current_total, 8);
        drop(first_charge);
    }

    #[tokio::test]
    async fn bounded_wait_resumes_after_capacity_is_released() {
        let manager = BlinkResourceManager::new(ResourceLimits {
            global_bytes: 100,
            per_tenant_bytes: 100,
            progress_pool_bytes: 10,
            max_tenants: 1,
            max_pending_waiters: 2,
            global_category_bytes: [100; CATEGORY_COUNT],
            per_tenant_category_bytes: [100; CATEGORY_COUNT],
        })
        .unwrap();
        let held_charge = manager
            .reserve(
                1,
                ResourceCategory::WritePreparation,
                ReservationClass::Foreground,
                90,
            )
            .unwrap();
        let waiting_manager = manager.clone();
        let waiter = tokio::spawn(async move {
            waiting_manager
                .reserve_wait(
                    1,
                    ResourceCategory::WritePreparation,
                    ReservationClass::Foreground,
                    90,
                    Duration::from_secs(1),
                )
                .await
        });
        tokio::task::yield_now().await;
        drop(held_charge);
        let acquired_charge = waiter.await.unwrap().unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 90);
        drop(acquired_charge);
    }

    #[tokio::test]
    async fn bounded_wait_times_out_and_cancellation_leaves_no_charge() {
        let manager = BlinkResourceManager::new(ResourceLimits {
            global_bytes: 100,
            per_tenant_bytes: 100,
            progress_pool_bytes: 1,
            max_tenants: 1,
            max_pending_waiters: 2,
            global_category_bytes: [100; CATEGORY_COUNT],
            per_tenant_category_bytes: [100; CATEGORY_COUNT],
        })
        .unwrap();
        let held_charge = manager
            .reserve(
                1,
                ResourceCategory::WritePreparation,
                ReservationClass::Foreground,
                99,
            )
            .unwrap();
        assert!(matches!(
            manager
                .reserve_wait(
                    1,
                    ResourceCategory::WritePreparation,
                    ReservationClass::Foreground,
                    1,
                    Duration::from_millis(1),
                )
                .await,
            Err(ResourceError::WaitTimedOut)
        ));
        let waiting_manager = manager.clone();
        let waiter = tokio::spawn(async move {
            waiting_manager
                .reserve_wait(
                    1,
                    ResourceCategory::WritePreparation,
                    ReservationClass::Foreground,
                    1,
                    Duration::from_secs(1),
                )
                .await
        });
        tokio::task::yield_now().await;
        waiter.abort();
        drop(held_charge);
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test]
    async fn pending_waiters_are_bounded_and_cancellation_releases_slot() {
        let manager = BlinkResourceManager::new(ResourceLimits {
            global_bytes: 10,
            per_tenant_bytes: 10,
            progress_pool_bytes: 1,
            max_tenants: 1,
            max_pending_waiters: 1,
            global_category_bytes: [10; CATEGORY_COUNT],
            per_tenant_category_bytes: [10; CATEGORY_COUNT],
        })
        .unwrap();
        let held_charge = manager
            .reserve(
                1,
                ResourceCategory::QueuedForeground,
                ReservationClass::Foreground,
                9,
            )
            .unwrap();
        let waiting_manager = manager.clone();
        let waiter = tokio::spawn(async move {
            waiting_manager
                .reserve_wait(
                    1,
                    ResourceCategory::QueuedForeground,
                    ReservationClass::Foreground,
                    1,
                    Duration::from_secs(1),
                )
                .await
        });
        tokio::task::yield_now().await;
        assert_eq!(manager.snapshot().unwrap().pending_waiters, 1);
        assert!(matches!(
            manager
                .reserve_wait(
                    1,
                    ResourceCategory::QueuedForeground,
                    ReservationClass::Foreground,
                    1,
                    Duration::from_secs(1),
                )
                .await,
            Err(ResourceError::TooManyWaiters)
        ));
        waiter.abort();
        tokio::task::yield_now().await;
        assert_eq!(manager.snapshot().unwrap().pending_waiters, 0);
        drop(held_charge);
    }
}
