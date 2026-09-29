//! The one place a log query meets its rows.
//!
//! Lifted out of `query/execution.rs` because nothing under `benches/` could
//! reach it there. The shape [`docs/COMPARISON.md`](../docs/COMPARISON.md)
//! measured this engine losing on — `| json | field=` with a small limit over a
//! window far larger than the limit — is a property of the *executor* rather
//! than of any single part scan, so it was the one hot path with no bench.
//! `benches/query.rs` measures this module.
//!
//! What it owns: the memtable and the parts are read through one funnel, in one
//! direction, with one limit, and the deletion mask is applied in one place.
//! `query/execution.rs` supplies the mask and the stats; everything else about
//! how a log query is answered is here.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::logql::LogQuery;
use crate::memtable::{Labels, LogEntry, MemTable, SharedLabels, StreamResult};
use crate::part::{ExactFieldPruning, QueryTimeRange, RowSink, TopKRows};
use crate::part_registry::PartRegistry;
use crate::tenant::TenantId;

/// Whether a row is hidden from this tenant by a delete request.
///
/// A predicate rather than the mask itself: the mask lives behind the delete
/// API and carries a metric this module has no business bumping, and the only
/// thing a scan needs to know is yes or no.
pub type HiddenRow<'a> = &'a dyn Fn(&Labels, &LogEntry) -> bool;

pub struct LogScanResult {
    pub results: Vec<StreamResult>,
    pub scanned_rows: u64,
    pub scanned_bytes: u64,
    /// What the pipeline actually materialized, for the caller to reconcile
    /// against the estimate it was admitted on. Charged for every row that
    /// survived the pipeline rather than for the rows the sink kept, so it is
    /// the peak the query passed through and not its result's size.
    pub materialized_bytes: u64,
}

/// One log query, with every bound that applies to it.
///
/// A struct rather than a fifteen-argument function: every caller sets the
/// first five and a different subset of the rest, and the arguments were
/// already threaded through four wrappers that existed only to default them.
pub struct LogScan<'a> {
    tenant: &'a TenantId,
    query: &'a LogQuery,
    range: QueryTimeRange,
    limit: usize,
    forward: bool,
    /// Rows the scan may read before it must refuse, rather than rows it may
    /// return. Exceeding it is an error and not a truncation, because a
    /// silently short answer to a range query reads as an absence of logs.
    scan_budget: Option<usize>,
    max_scan_bytes: Option<u64>,
    max_memory_bytes: Option<u64>,
    cancellation: Option<&'a AtomicBool>,
    hidden: Option<HiddenRow<'a>>,
    columns: crate::part::ColumnSet,
}

impl<'a> LogScan<'a> {
    pub fn new(
        tenant: &'a TenantId,
        query: &'a LogQuery,
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
    ) -> Self {
        Self {
            tenant,
            query,
            range,
            limit,
            forward,
            scan_budget: None,
            max_scan_bytes: None,
            max_memory_bytes: None,
            cancellation: None,
            hidden: None,
            columns: crate::part::ColumnSet::all(),
        }
    }

    pub fn columns(mut self, columns: crate::part::ColumnSet) -> Self {
        self.columns = columns;
        self
    }

    pub fn scan_budget(mut self, rows: Option<usize>) -> Self {
        self.scan_budget = rows;
        self
    }

    pub fn max_scan_bytes(mut self, bytes: Option<u64>) -> Self {
        self.max_scan_bytes = bytes;
        self
    }

    pub fn max_memory_bytes(mut self, bytes: Option<u64>) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    pub fn cancellation(mut self, cancellation: Option<&'a AtomicBool>) -> Self {
        self.cancellation = cancellation;
        self
    }

    pub fn hidden(mut self, hidden: HiddenRow<'a>) -> Self {
        self.hidden = Some(hidden);
        self
    }

    fn cancelled(&self) -> bool {
        self.cancellation
            .is_some_and(|flag| flag.load(Ordering::Acquire))
    }

    /// The memtable and then the parts, both streaming into one bounded sink.
    ///
    /// There is no `normal_scan_limit` any more. It existed because the limit
    /// had to be handed to the storage scan *before* the pipeline could say
    /// which rows survived, so any `| json` had to be given `usize::MAX` and the
    /// whole window was materialized. The sink inverts that: the pipeline runs
    /// as each row arrives, the sink holds the best `limit` survivors, and the
    /// scan reads until it has them rather than until it has read everything.
    ///
    /// The memtable is scanned first in both directions. It holds the newest
    /// rows, so a backward query fills the sink from it and prunes most parts on
    /// their metadata; a forward query fills the sink with rows the parts then
    /// displace, which costs the sink's own bookkeeping and nothing else.
    pub fn run(&self, memtable: &MemTable, parts: &PartRegistry) -> Result<LogScanResult, String> {
        let mut sink = PipelineSink {
            query: self.query,
            hidden: self.hidden,
            cancellation: self.cancellation,
            max_memory_bytes: self.max_memory_bytes,
            materialized_memory_bytes: 0,
            rows: TopKRows::new(self.limit, self.forward),
        };
        let (scanned_rows, scanned_bytes) = self.run_into(memtable, parts, &mut sink)?;
        let materialized_bytes = sink.materialized_memory_bytes;
        Ok(LogScanResult {
            results: sink.rows.into_stream_results(),
            scanned_rows,
            scanned_bytes,
            materialized_bytes,
        })
    }

    /// The same orchestration into a caller-supplied sink — the memtable first,
    /// then the parts, budgets checked between — for consumers that aggregate
    /// rows instead of holding them.
    pub fn run_into(
        &self,
        memtable: &MemTable,
        parts: &PartRegistry,
        sink: &mut dyn crate::part::RowSink,
    ) -> Result<(u64, u64), String> {
        let mut scanned_rows = 0u64;
        let mut scanned_bytes = 0u64;
        let scan_limit = self.scan_budget.map(|budget| budget.saturating_add(1));

        scanned_rows = scanned_rows.saturating_add(memtable.scan_into(
            self.tenant,
            &self.query.line_filters,
            self.range,
            self.forward,
            scan_limit,
            self.cancellation,
            sink,
        )? as u64);

        if self.cancelled() {
            return Err("query timed out".to_string());
        }
        self.check_scan_budget(scanned_rows)?;

        let exact_fields = self.query.exact_field_predicates();
        let part_scan_limit = scan_limit.map(|budget| budget.saturating_sub(scanned_rows as usize));
        let part_scan_bytes_limit = self
            .max_scan_bytes
            .map(|budget| budget.saturating_sub(scanned_bytes));
        let part_stats = parts.scan_into(
            self.tenant,
            ExactFieldPruning::new(&self.query.line_filters, &exact_fields),
            self.range,
            self.forward,
            part_scan_limit,
            part_scan_bytes_limit,
            self.cancellation,
            &self.columns,
            sink,
        )?;
        scanned_rows = scanned_rows.saturating_add(part_stats.scanned_rows as u64);
        scanned_bytes = scanned_bytes.saturating_add(part_stats.scanned_bytes);

        if self.cancelled() {
            return Err("query timed out".to_string());
        }
        self.check_scan_budget(scanned_rows)?;
        if let Some(budget) = self.max_scan_bytes
            && scanned_bytes > budget
        {
            return Err(format!(
                "query exceeds the maximum of {budget} scanned bytes"
            ));
        }

        Ok((scanned_rows, scanned_bytes))
    }

    fn check_scan_budget(&self, scanned_rows: u64) -> Result<(), String> {
        if let Some(budget) = self.scan_budget
            && scanned_rows > budget as u64
        {
            return Err(format!(
                "query exceeds the maximum of {budget} scanned rows"
            ));
        }
        Ok(())
    }
}

/// The executor's sink: the pipeline, the deletion mask and the bounded result.
///
/// This is where the read path's three materializations collapsed into one. The
/// reader and the registry hand rows straight through, and the only rows that
/// are held anywhere are the `limit` in `rows`.
struct PipelineSink<'a> {
    query: &'a LogQuery,
    hidden: Option<HiddenRow<'a>>,
    cancellation: Option<&'a AtomicBool>,
    max_memory_bytes: Option<u64>,
    /// Charged for every row that survives the pipeline, not for the rows the
    /// sink kept. The bound is on what the query materialized on its way to an
    /// answer, and lowering it to what it *held* would loosen a limit — which is
    /// the metering work, not this (`todo.md`, M10 honest metering).
    materialized_memory_bytes: u64,
    rows: TopKRows,
}

impl RowSink for PipelineSink<'_> {
    fn accept_extracted(
        &mut self,
        labels: &SharedLabels,
        entry: LogEntry,
        extracted_json: Option<std::collections::BTreeMap<String, String>>,
    ) -> Result<(), String> {
        self.accept_inner(labels, entry, extracted_json)
    }

    fn accept(&mut self, labels: &SharedLabels, entry: LogEntry) -> Result<(), String> {
        self.accept_inner(labels, entry, None)
    }

    fn frontier_ns(&self) -> Option<i64> {
        self.rows.frontier_ns()
    }

    fn remaining(&self) -> Option<usize> {
        self.rows.remaining()
    }

    fn is_closed(&self) -> bool {
        self.rows.is_closed()
    }
}

impl PipelineSink<'_> {
    fn accept_inner(
        &mut self,
        labels: &SharedLabels,
        mut entry: LogEntry,
        extracted_json: Option<std::collections::BTreeMap<String, String>>,
    ) -> Result<(), String> {
        if self
            .cancellation
            .is_some_and(|flag| flag.load(Ordering::Acquire))
        {
            return Err("query timed out".to_string());
        }
        // Before the pipeline runs: a delete selector matches the line as it was
        // written, and `line_format` would have rewritten it.
        if self.hidden.is_some_and(|hidden| hidden(labels, &entry)) {
            return Ok(());
        }
        if self.query.stages.is_empty() && self.query.matchers.is_empty() {
            // A matcherless, stage-less pipeline accepts every row and changes
            // nothing about it, so the full evaluation — which builds a
            // `BTreeMap` field map per row — is skipped. A selector cannot
            // take this path: it is evaluated against the row's attributes
            // inside the pipeline now that no stream exists to match first.
        } else if !self.query.process_entry_with_precomputed_json(
            labels,
            &mut entry,
            self.cancellation,
            extracted_json.as_ref(),
        )? {
            return Ok(());
        }
        self.materialized_memory_bytes = self
            .materialized_memory_bytes
            .checked_add(estimated_log_entry_memory_bytes(labels, &entry))
            .ok_or_else(|| "query materialized memory accounting overflowed".to_string())?;
        if let Some(max) = self.max_memory_bytes
            && self.materialized_memory_bytes > max
        {
            return Err(format!(
                "query exceeds the maximum of {max} materialized bytes"
            ));
        }
        self.rows.offer(labels, entry);
        Ok(())
    }
}

/// What one returned row is charged against `max_query_memory_bytes`.
///
/// Still charges every row for the label bytes it now shares through
/// `SharedLabels`, so it over-counts by the rows-per-stream factor. That makes
/// the ceiling stricter than the memory really held, which is the safe
/// direction; correcting it loosens a limit and belongs with the rest of the
/// metering (`todo.md`, M10 honest metering).
pub(crate) fn estimated_log_entry_memory_bytes(labels: &Labels, entry: &LogEntry) -> u64 {
    let labels = labels
        .iter()
        .map(|(name, value)| name.len().saturating_add(value.len()))
        .sum::<usize>();
    let metadata = entry
        .structured_metadata
        .iter()
        .map(|(name, value)| name.len().saturating_add(value.len()))
        .sum::<usize>();
    labels
        .saturating_add(metadata)
        .saturating_add(entry.line.len())
        .saturating_add(std::mem::size_of::<LogEntry>()) as u64
}

/// The metric fast path's sink: `sum(rate(...))`-shaped questions never need
/// the rows, only how many of them (or how many line bytes) land in each
/// evaluation window — so this accumulates a difference array over the
/// evaluation grid and materializes nothing. A row at `ts` contributes to
/// every point `t` with `t - range < ts <= t`, which on a sorted grid is one
/// contiguous index range: two array updates per row where the general path
/// built a `LogEntry`, grouped it, sorted it and prefix-summed it.
pub struct CountingSink<'a> {
    pub query: &'a LogQuery,
    pub hidden: Option<HiddenRow<'a>>,
    pub cancellation: Option<&'a AtomicBool>,
    /// Sorted evaluation instants.
    pub times: &'a [i64],
    pub range_ns: i64,
    /// Count line bytes instead of rows (`bytes_over_time`).
    pub bytes: bool,
    /// `times.len() + 1` slots; prefix-summing it yields the per-point totals.
    pub diff: Vec<f64>,
    pub rows: u64,
}

impl RowSink for CountingSink<'_> {
    fn accept_extracted(
        &mut self,
        labels: &SharedLabels,
        entry: LogEntry,
        extracted_json: Option<std::collections::BTreeMap<String, String>>,
    ) -> Result<(), String> {
        self.accept_inner(labels, entry, extracted_json)
    }

    fn accept(&mut self, labels: &SharedLabels, entry: LogEntry) -> Result<(), String> {
        self.accept_inner(labels, entry, None)
    }

    fn frontier_ns(&self) -> Option<i64> {
        None
    }

    fn remaining(&self) -> Option<usize> {
        None
    }

    fn is_closed(&self) -> bool {
        false
    }
}

impl CountingSink<'_> {
    fn accept_inner(
        &mut self,
        labels: &SharedLabels,
        mut entry: LogEntry,
        extracted_json: Option<std::collections::BTreeMap<String, String>>,
    ) -> Result<(), String> {
        if self
            .cancellation
            .is_some_and(|flag| flag.load(Ordering::Acquire))
        {
            return Err("query timed out".to_string());
        }
        if self.hidden.is_some_and(|hidden| hidden(labels, &entry)) {
            return Ok(());
        }
        // Same rule as `PipelineSink`: matchers and stages run per row (a
        // selector or field filter must still reject), and a matcherless,
        // stage-less pipeline accepts everything — its effects touch only
        // fields nothing here reads.
        if (!self.query.stages.is_empty() || !self.query.matchers.is_empty())
            && !self.query.process_entry_with_precomputed_json(
                labels,
                &mut entry,
                self.cancellation,
                extracted_json.as_ref(),
            )?
        {
            return Ok(());
        }
        let start = self.times.partition_point(|&t| t < entry.timestamp_ns);
        let end = self
            .times
            .partition_point(|&t| t < entry.timestamp_ns.saturating_add(self.range_ns));
        if start < end {
            let value = if self.bytes {
                entry.line.len() as f64
            } else {
                1.0
            };
            self.diff[start] += value;
            self.diff[end] -= value;
        }
        self.rows += 1;
        Ok(())
    }
}
