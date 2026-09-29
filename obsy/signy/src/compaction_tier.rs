//! Which parts a compaction pass may rewrite together, shared by the log,
//! trace and metric compactors.
//!
//! **Only parts of comparable size compact together.** A tier is a size band,
//! each [`TIER_RATIO`] times as wide as the one below it, and a pass rewrites
//! one tier of one partition. The band that a part lands in is what bounds the
//! work: a tier's parts merge into one part of the tier above, so a byte is
//! rewritten once per tier it climbs — a logarithm of the partition's size,
//! not a function of how many flushes built it.
//!
//! The compactors this replaced grouped by a fixed ceiling instead: every part
//! under it was one tier. A partition's accumulated part stays under such a
//! ceiling for most of its life, so each pass rewrote the whole of it to
//! absorb a few flushes' worth of new rows, and the bytes written over a day
//! grew with the square of the bytes ingested. Production wrote 111 MB of
//! trace parts in seven hours to store about 1.5 MB of spans.
//!
//! **A partition nobody writes to any more still gets compacted.** Once no
//! part of a partition carries a row newer than [`IDLE_PARTITION_AGE`], a tier
//! of two parts is eligible however far it is from full, so a slow tenant's
//! partition does not keep a handful of parts open forever. The age is read
//! per partition rather than per tier on purpose: the upper tiers of a busy
//! partition hold data hours older than the flush that is landing now, and
//! compacting them for their age alone is the rewrite this module exists to
//! stop.

use std::collections::BTreeMap;

/// Width of one tier. Eight parts of a tier merge into one part of the next,
/// so this is also what [`TierPolicy::min_parts`] is chosen against.
const TIER_RATIO: u64 = 8;
/// Top of tier 0, and the anchor of the whole ladder.
///
/// Every tier above it is [`TIER_RATIO`] times as wide as the one below, so
/// merging a full tier produces a part exactly one tier up whatever the part
/// size is. That holds only while the anchor sits below what one flush writes:
/// a tier 0 wide enough to swallow a merge of its own parts is the fixed
/// ceiling this module replaced, and it rewrites the merged part again.
const TIER_ZERO_MAX_BYTES: u64 = 4 * 1024;
/// How long after its newest row a partition counts as finished.
pub const IDLE_PARTITION_AGE: std::time::Duration = std::time::Duration::from_secs(60 * 60);

#[derive(Clone, Copy, Debug)]
pub struct TierPolicy {
    /// Parts this large are never inputs: rewriting them buys a part count
    /// reduction that no query notices.
    pub max_part_bytes: u64,
    pub min_parts: usize,
    pub max_parts: usize,
    /// Stored bytes one pass reads, which is what bounds its memory.
    pub max_input_bytes: u64,
}

/// One part as compaction selection sees it.
#[derive(Clone, Debug)]
pub struct TierCandidate {
    pub partition: String,
    pub bytes: u64,
    pub max_ts_ns: i64,
}

/// Which size band a part of this many bytes belongs to.
pub fn tier_of(bytes: u64) -> u32 {
    let mut tier = 0;
    let mut ceiling = TIER_ZERO_MAX_BYTES;
    while bytes >= ceiling {
        let Some(next) = ceiling.checked_mul(TIER_RATIO) else {
            return tier + 1;
        };
        ceiling = next;
        tier += 1;
    }
    tier
}

/// The candidates one pass should rewrite, as indices into `candidates`, or
/// `None` when nothing is worth rewriting.
///
/// Smallest first inside the chosen tier, so a pass retires the most parts per
/// byte it reads.
pub fn select_tier(
    candidates: &[TierCandidate],
    policy: &TierPolicy,
    now_ns: i64,
) -> Option<Vec<usize>> {
    let idle_before_ns = now_ns.saturating_sub(IDLE_PARTITION_AGE.as_nanos() as i64);
    let mut newest_by_partition: BTreeMap<&str, i64> = BTreeMap::new();
    for candidate in candidates {
        let newest = newest_by_partition
            .entry(candidate.partition.as_str())
            .or_insert(i64::MIN);
        *newest = (*newest).max(candidate.max_ts_ns);
    }
    let mut tiers: BTreeMap<(&str, u32), Vec<usize>> = BTreeMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.bytes >= policy.max_part_bytes {
            continue;
        }
        tiers
            .entry((candidate.partition.as_str(), tier_of(candidate.bytes)))
            .or_default()
            .push(index);
    }

    for ((partition, _), mut tier) in tiers {
        let idle = newest_by_partition
            .get(partition)
            .is_some_and(|newest| *newest < idle_before_ns);
        if tier.len() < policy.min_parts && !(idle && tier.len() >= 2) {
            continue;
        }
        tier.sort_by_key(|index| (candidates[*index].bytes, *index));
        let mut selected = Vec::new();
        let mut selected_bytes = 0u64;
        for index in tier {
            let bytes = candidates[index].bytes;
            if selected.len() == policy.max_parts
                || (!selected.is_empty() && selected_bytes + bytes > policy.max_input_bytes)
            {
                break;
            }
            selected_bytes += bytes;
            selected.push(index);
        }
        if selected.len() >= 2 {
            return Some(selected);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR_NS: i64 = 3_600 * 1_000_000_000;

    fn policy() -> TierPolicy {
        TierPolicy {
            max_part_bytes: 16 * 1024 * 1024,
            min_parts: 8,
            max_parts: 32,
            max_input_bytes: 16 * 1024 * 1024,
        }
    }

    fn candidate(bytes: u64, max_ts_ns: i64) -> TierCandidate {
        TierCandidate {
            partition: "2026-09-15".to_string(),
            bytes,
            max_ts_ns,
        }
    }

    /// The defect this module exists for: an accumulated part must not be
    /// rewritten every time a few flushes land beside it.
    #[test]
    fn a_large_part_is_never_rewritten_to_absorb_small_ones() {
        let now_ns = 100 * HOUR_NS;
        let mut candidates = vec![candidate(4 * 1024 * 1024, now_ns)];
        candidates.extend((0..7).map(|_| candidate(7 * 1024, now_ns)));

        assert_eq!(
            select_tier(&candidates, &policy(), now_ns),
            None,
            "seven small parts are not a tier, and the large one is not theirs"
        );

        candidates.push(candidate(7 * 1024, now_ns));
        let selected = select_tier(&candidates, &policy(), now_ns).expect("a full tier compacts");
        assert_eq!(selected.len(), 8);
        assert!(
            selected
                .iter()
                .all(|index| candidates[*index].bytes == 7 * 1024),
            "and it compacts the small parts alone"
        );
    }

    #[test]
    fn a_tier_is_promoted_only_once_it_is_full() {
        let now_ns = 100 * HOUR_NS;
        let policy = policy();
        for count in 2..8 {
            let candidates: Vec<_> = (0..count).map(|_| candidate(7 * 1024, now_ns)).collect();
            assert_eq!(
                select_tier(&candidates, &policy, now_ns),
                None,
                "{count} parts"
            );
        }
    }

    #[test]
    fn a_partition_nobody_writes_to_compacts_on_age_alone() {
        let now_ns = 100 * HOUR_NS;
        let candidates = vec![
            candidate(7 * 1024, now_ns - 3 * HOUR_NS),
            candidate(9 * 1024, now_ns - 3 * HOUR_NS),
        ];
        assert_eq!(
            select_tier(&candidates, &policy(), now_ns).map(|selected| selected.len()),
            Some(2)
        );

        let fresh = vec![candidate(7 * 1024, now_ns), candidate(9 * 1024, now_ns)];
        assert_eq!(select_tier(&fresh, &policy(), now_ns), None);
    }

    #[test]
    fn parts_of_different_partitions_never_compact_together() {
        let now_ns = 100 * HOUR_NS;
        let mut candidates: Vec<_> = (0..8).map(|_| candidate(7 * 1024, now_ns)).collect();
        for candidate in candidates.iter_mut().take(4) {
            candidate.partition = "2026-09-14".to_string();
        }
        let selected = select_tier(&candidates, &policy(), now_ns);
        assert_eq!(
            selected, None,
            "four and four are two tiers, neither of them full"
        );
    }

    /// One pass reads what its budget allows and no more.
    #[test]
    fn the_input_budget_bounds_one_pass() {
        let now_ns = 100 * HOUR_NS;
        let candidates: Vec<_> = (0..16).map(|_| candidate(32 * 1024, now_ns)).collect();
        let bounded = TierPolicy {
            max_input_bytes: 128 * 1024,
            ..policy()
        };
        let selected = select_tier(&candidates, &bounded, now_ns).expect("a full tier compacts");
        assert_eq!(selected.len(), 4);

        let counted = TierPolicy {
            max_parts: 10,
            ..policy()
        };
        assert_eq!(
            select_tier(&candidates, &counted, now_ns)
                .expect("a full tier compacts")
                .len(),
            10
        );
    }

    /// What one flush writes, what compaction writes to absorb it, and what is
    /// live at the end: the simulation the tiering exists to bound.
    struct Simulation {
        live: Vec<TierCandidate>,
        written_bytes: u64,
        ingested_bytes: u64,
        peak_live_parts: usize,
    }

    fn simulate(flushes: usize, flush_bytes: u64, policy: &TierPolicy) -> Simulation {
        let mut simulation = Simulation {
            live: Vec::new(),
            written_bytes: 0,
            ingested_bytes: 0,
            peak_live_parts: 0,
        };
        for flush in 0..flushes {
            // One flush a minute, which is the production interval.
            let now_ns = flush as i64 * 60 * 1_000_000_000;
            simulation.live.push(TierCandidate {
                partition: "2026-09-15".to_string(),
                bytes: flush_bytes,
                max_ts_ns: now_ns,
            });
            simulation.written_bytes += flush_bytes;
            simulation.ingested_bytes += flush_bytes;
            while let Some(selected) = select_tier(&simulation.live, policy, now_ns) {
                let merged_bytes: u64 = selected
                    .iter()
                    .map(|index| simulation.live[index.to_owned()].bytes)
                    .sum();
                let mut kept = Vec::new();
                for (index, candidate) in simulation.live.iter().enumerate() {
                    if !selected.contains(&index) {
                        kept.push(candidate.clone());
                    }
                }
                kept.push(TierCandidate {
                    partition: "2026-09-15".to_string(),
                    bytes: merged_bytes,
                    max_ts_ns: now_ns,
                });
                simulation.live = kept;
                simulation.written_bytes += merged_bytes;
            }
            simulation.peak_live_parts = simulation.peak_live_parts.max(simulation.live.len());
        }
        simulation
    }

    /// Bytes written per byte ingested, over a day and over four of them, at
    /// three flush sizes. The ceiling on one day is what the production
    /// incident measured at about 20x and rising; the point of four days is
    /// that it does not rise with the data, which is what "not quadratic"
    /// means here.
    #[test]
    fn compaction_writes_a_bounded_multiple_of_what_it_ingests() {
        let policy = TierPolicy {
            max_part_bytes: u64::MAX,
            max_input_bytes: u64::MAX,
            ..policy()
        };
        for flush_bytes in [7 * 1024, 40 * 1024, 1024 * 1024] {
            let day = simulate(1_440, flush_bytes, &policy);
            assert_eq!(
                day.live.iter().map(|part| part.bytes).sum::<u64>(),
                day.ingested_bytes,
                "every ingested byte is live exactly once"
            );
            // A byte is written once by the flush and once per tier it
            // climbs, so the multiple is about log8(day / flush): near 4 for
            // 7 KiB flushes, near 2 for 1 MiB ones. The production incident
            // measured about 20x on the same shape of workload.
            let day_amplification = day.written_bytes as f64 / day.ingested_bytes as f64;
            assert!(
                day_amplification < 4.5,
                "a day of {flush_bytes}-byte flushes wrote {day_amplification:.2}x what it ingested"
            );

            let four_days = simulate(4 * 1_440, flush_bytes, &policy);
            let four_day_amplification =
                four_days.written_bytes as f64 / four_days.ingested_bytes as f64;
            // Four times the data through a quadratic compactor writes about
            // four times the multiple; through this one it writes one more
            // tier's worth.
            assert!(
                four_day_amplification < day_amplification * 1.5,
                "four days of {flush_bytes}-byte flushes wrote {four_day_amplification:.2}x \
against one day's {day_amplification:.2}x"
            );
            assert!(
                four_days.peak_live_parts <= 40,
                "the live part count stays bounded: {}",
                four_days.peak_live_parts
            );
        }
    }
}
