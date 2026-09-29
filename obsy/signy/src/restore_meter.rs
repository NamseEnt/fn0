//! What a restore would cost if the download applied the read path's
//! selection, and what the whole-object download buys by not applying it.
//!
//! "Add Parquet range reads" is an item whose sign is not obvious, and the
//! byte meter in [`crate::object_storage::counting_store`] measured only one
//! side of it. A whole-part restore over-fetches — 15.2 of 16 tenants per part
//! and every row group, since the download applies none of the selection the
//! scan then applies. But the bytes it wastes are the axis R2 does not bill
//! (`docs/ARCHITECTURE.md` — this design is dominated by Class A *requests*),
//! and the copy it leaves on disk is what serves the next query locally. So
//! range reads trade a free resource for a billed one, and cancel an
//! amortisation. Two numbers decide which way that lands, and both are
//! properties of this code rather than of a deployment:
//!
//! 1. **What the selection would cost in requests.** A row group's column
//!    chunks are contiguous in a Parquet file and the log path projects every
//!    column, so a maximal run of selected row groups is one byte range. The
//!    request count a selective download would issue is therefore
//!    `selected_runs + 1` (the footer) per part, against the one GET a whole
//!    restore issues today. `selected / present` is what it would save in
//!    bytes. Counted twice: over every query scan, and over the **first scan
//!    after each restore** alone. The second is the one that decides the
//!    trade, because that is the query the download happens for — the rig
//!    mixes query shapes and a recent-window scan's selectivity is not the
//!    lookback scan's.
//! 2. **What the whole copy earns.** `restored_scans / restores` is how many
//!    query scans one restored body serves before eviction takes it. At 1 the
//!    over-fetch is pure waste; above it, part of the waste is prepaid work
//!    for later queries — including other tenants' queries, since they share
//!    the object.
//!
//! Both are counted for query scans only. A merge or retention rewrite passes
//! a row-group window and is not a query, so it is excluded: it reads the part
//! it is rewriting whole by construction, and counting it would report the
//! rewrite's own layout as a query's selectivity.
//!
//! The resident set holds one path per restored body and drops it on
//! eviction. A body that a merge retires instead leaks its entry; part
//! directories are unique per part, so the leak cannot corrupt a later
//! reading, and it is bounded by the parts one process restores.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use crate::tenant::TenantId;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    /// Query scans that opened a part body.
    pub part_scans: u64,
    /// Row groups those parts hold, all tenants.
    pub row_groups_present: u64,
    /// Row groups the querying tenant's segment spans.
    pub row_groups_tenant: u64,
    /// Row groups the scan actually read.
    pub row_groups_selected: u64,
    /// Maximal contiguous runs among the selected — one byte range each.
    pub selected_runs: u64,
    /// Bodies downloaded whole after eviction.
    pub restores: u64,
    /// Query scans served by a restored body still on disk.
    pub restored_scans: u64,
    /// Restored bodies that eviction has since taken.
    pub restored_retired: u64,
    /// The first scan after each restore — the query that paid for the
    /// download, and therefore the one whose selection a selective download
    /// would have applied.
    pub first_scan_parts: u64,
    pub first_scan_row_groups_present: u64,
    pub first_scan_row_groups_selected: u64,
    pub first_scan_runs: u64,
    /// Distinct (restored body, querying tenant) pairs. A selective download
    /// serves one tenant's slice, so this is how many downloads the same work
    /// would have taken; a whole restore takes one whatever this reaches.
    pub restored_tenant_slices: u64,
}

#[derive(Debug, Default)]
pub struct RestoreMeter {
    part_scans: AtomicU64,
    row_groups_present: AtomicU64,
    row_groups_tenant: AtomicU64,
    row_groups_selected: AtomicU64,
    selected_runs: AtomicU64,
    restores: AtomicU64,
    restored_scans: AtomicU64,
    restored_retired: AtomicU64,
    first_scan_parts: AtomicU64,
    first_scan_row_groups_present: AtomicU64,
    first_scan_row_groups_selected: AtomicU64,
    first_scan_runs: AtomicU64,
    restored_tenant_slices: AtomicU64,
    /// Restored bodies still on disk, against the tenants that have read one
    /// since the download.
    resident: Mutex<HashMap<PathBuf, HashSet<TenantId>>>,
}

/// Maximal runs of consecutive values in an ascending list.
///
/// One run is one byte range, which is the whole point of counting them: a
/// scan that selects groups 3..9 would issue one request, not six.
fn runs(selected: &[u32]) -> u64 {
    let mut runs = 0u64;
    let mut previous: Option<u32> = None;
    for &group in selected {
        if previous.is_none_or(|last| group != last.saturating_add(1)) {
            runs += 1;
        }
        previous = Some(group);
    }
    runs
}

impl RestoreMeter {
    pub fn note_query_scan(
        &self,
        dir: &Path,
        tenant: &TenantId,
        row_groups_present: u32,
        tenant_groups: u32,
        selected: &[u32],
    ) {
        self.part_scans.fetch_add(1, Ordering::Relaxed);
        self.row_groups_present
            .fetch_add(u64::from(row_groups_present), Ordering::Relaxed);
        self.row_groups_tenant
            .fetch_add(u64::from(tenant_groups), Ordering::Relaxed);
        self.row_groups_selected
            .fetch_add(selected.len() as u64, Ordering::Relaxed);
        self.selected_runs
            .fetch_add(runs(selected), Ordering::Relaxed);

        let Ok(mut resident) = self.resident.lock() else {
            return;
        };
        let Some(tenants) = resident.get_mut(dir) else {
            return;
        };
        self.restored_scans.fetch_add(1, Ordering::Relaxed);
        let first_for_tenant = tenants.insert(tenant.clone());
        if first_for_tenant {
            self.restored_tenant_slices.fetch_add(1, Ordering::Relaxed);
        }
        if tenants.len() == 1 && first_for_tenant {
            self.first_scan_parts.fetch_add(1, Ordering::Relaxed);
            self.first_scan_row_groups_present
                .fetch_add(u64::from(row_groups_present), Ordering::Relaxed);
            self.first_scan_row_groups_selected
                .fetch_add(selected.len() as u64, Ordering::Relaxed);
            self.first_scan_runs
                .fetch_add(runs(selected), Ordering::Relaxed);
        }
    }

    pub fn note_restore(&self, dir: &Path) {
        self.restores.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut resident) = self.resident.lock() {
            resident.insert(dir.to_path_buf(), HashSet::new());
        }
    }

    pub fn note_evict(&self, dir: &Path) {
        let was_restored = self
            .resident
            .lock()
            .map(|mut resident| resident.remove(dir).is_some())
            .unwrap_or(false);
        if was_restored {
            self.restored_retired.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            part_scans: self.part_scans.load(Ordering::Relaxed),
            row_groups_present: self.row_groups_present.load(Ordering::Relaxed),
            row_groups_tenant: self.row_groups_tenant.load(Ordering::Relaxed),
            row_groups_selected: self.row_groups_selected.load(Ordering::Relaxed),
            selected_runs: self.selected_runs.load(Ordering::Relaxed),
            restores: self.restores.load(Ordering::Relaxed),
            restored_scans: self.restored_scans.load(Ordering::Relaxed),
            restored_retired: self.restored_retired.load(Ordering::Relaxed),
            first_scan_parts: self.first_scan_parts.load(Ordering::Relaxed),
            first_scan_row_groups_present: self
                .first_scan_row_groups_present
                .load(Ordering::Relaxed),
            first_scan_row_groups_selected: self
                .first_scan_row_groups_selected
                .load(Ordering::Relaxed),
            first_scan_runs: self.first_scan_runs.load(Ordering::Relaxed),
            restored_tenant_slices: self.restored_tenant_slices.load(Ordering::Relaxed),
        }
    }
}

static GLOBAL: LazyLock<RestoreMeter> = LazyLock::new(RestoreMeter::default);

/// Process-wide, like the bloom and row-group cache budgets: the producers are
/// a part reader, a cache eviction and a download, and threading a handle from
/// the app state to all three would touch every constructor between them to
/// report one measurement.
pub fn global() -> &'static RestoreMeter {
    &GLOBAL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_counts_byte_ranges_not_row_groups() {
        assert_eq!(runs(&[]), 0);
        assert_eq!(runs(&[7]), 1);
        assert_eq!(runs(&[3, 4, 5, 6, 7, 8]), 1);
        assert_eq!(runs(&[0, 2, 4]), 3);
        assert_eq!(runs(&[0, 1, 5, 6, 7, 12]), 3);
    }

    #[test]
    fn a_scan_counts_as_reuse_only_between_restore_and_eviction() {
        let meter = RestoreMeter::default();
        let dir = Path::new("/parts/2026-08-18/abc");
        let one = TenantId::parse("tenant-1").unwrap();
        let two = TenantId::parse("tenant-2").unwrap();

        // Before any restore: scanned, but nothing was downloaded for it.
        meter.note_query_scan(dir, &one, 8, 4, &[1, 2]);
        assert_eq!(meter.snapshot().restored_scans, 0);

        meter.note_restore(dir);
        meter.note_query_scan(dir, &one, 8, 4, &[1, 2]);
        meter.note_query_scan(dir, &two, 8, 4, &[3]);
        assert_eq!(meter.snapshot().restored_scans, 2);

        // Only the first of the two paid for the download, and the two tenants
        // are two slices a selective download would have fetched separately.
        assert_eq!(meter.snapshot().first_scan_parts, 1);
        assert_eq!(meter.snapshot().first_scan_row_groups_selected, 2);
        assert_eq!(meter.snapshot().restored_tenant_slices, 2);

        meter.note_evict(dir);
        meter.note_query_scan(dir, &one, 8, 4, &[1]);
        let snapshot = meter.snapshot();
        assert_eq!(snapshot.restored_scans, 2);
        assert_eq!(snapshot.restores, 1);
        assert_eq!(snapshot.restored_retired, 1);
        assert_eq!(snapshot.part_scans, 4);
        // Two singletons and two adjacent pairs.
        assert_eq!(snapshot.selected_runs, 4);
        assert_eq!(snapshot.row_groups_selected, 6);
        assert_eq!(snapshot.row_groups_present, 32);
        assert_eq!(snapshot.row_groups_tenant, 16);
    }

    #[test]
    fn evicting_a_body_that_was_never_restored_is_not_a_retirement() {
        let meter = RestoreMeter::default();
        meter.note_evict(Path::new("/parts/2026-08-18/fresh"));
        assert_eq!(meter.snapshot().restored_retired, 0);
    }
}
