use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

const CATEGORY_COUNT: usize = 7;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub global_bytes: u64,
    pub per_tenant_bytes: u64,
    pub progress_pool_bytes: u64,
    pub max_tenants: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceError {
    InvalidLimits,
    TooManyTenants,
    CapacityExceeded,
    ArithmeticOverflow,
    SharedOwnership,
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
    pub rejected_reservations: u64,
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
    tenants: BTreeMap<u64, TenantState>,
}

struct Inner {
    limits: ResourceLimits,
    state: Mutex<State>,
}

#[derive(Clone)]
pub struct BlinkResourceManager {
    inner: Arc<Inner>,
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

impl BlinkResourceManager {
    pub fn new(limits: ResourceLimits) -> Result<Self, ResourceError> {
        if limits.global_bytes == 0
            || limits.per_tenant_bytes == 0
            || limits.max_tenants == 0
            || limits.per_tenant_bytes > limits.global_bytes
            || limits.progress_pool_bytes >= limits.global_bytes
        {
            return Err(ResourceError::InvalidLimits);
        }
        Ok(Self {
            inner: Arc::new(Inner {
                limits,
                state: Mutex::new(State::default()),
            }),
        })
    }

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
        let tenant_state = state.tenants.entry(tenant_id).or_default();
        let next_tenant_category = tenant_state.current_by_category[category_index]
            .checked_add(bytes)
            .ok_or(ResourceError::ArithmeticOverflow)?;
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
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BlinkResourceManager, ReservationClass, ResourceCategory, ResourceError, ResourceLimits,
    };

    fn manager() -> BlinkResourceManager {
        BlinkResourceManager::new(ResourceLimits {
            global_bytes: 100,
            per_tenant_bytes: 80,
            progress_pool_bytes: 20,
            max_tenants: 2,
        })
        .unwrap()
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
            progress_pool_bytes: 0,
            max_tenants: 1,
        })
        .unwrap();
        let reservation = manager
            .reserve(
                1,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                u64::MAX,
            )
            .unwrap();
        assert!(matches!(
            manager.reserve(
                1,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                1
            ),
            Err(ResourceError::ArithmeticOverflow)
        ));
        assert_eq!(manager.snapshot().unwrap().current_total, u64::MAX);
        drop(reservation);
    }

    #[test]
    fn shared_manager_serializes_competing_tenant_reservations() {
        let manager = BlinkResourceManager::new(ResourceLimits {
            global_bytes: 100,
            per_tenant_bytes: 100,
            progress_pool_bytes: 0,
            max_tenants: 2,
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
}
