//! Latency series, percentiles that refuse to be computed, and the backlog
//! trend the flush gate is built on.

use serde_json::{Value, json};

/// The smallest sample count from which `p` is distinguishable from the
/// maximum.
///
/// With `n` samples the reported index is `ceil((n - 1) * p)`, so `p95` over
/// twenty samples *is* the largest sample and `p95` over eight is the largest
/// twice over. The published numbers included a p95 over eight samples and a
/// p99 over 167 (`docs/LOAD_RESULTS.md`, retirement header); requiring
/// `ceil((2 - p) / (1 - p))` puts at least one sample above the quantile, so
/// the number reported is a quantile rather than a maximum wearing its name.
pub fn min_samples_for(quantile: f64) -> usize {
    if quantile <= 0.0 || quantile >= 1.0 {
        return 1;
    }
    ((2.0 - quantile) / (1.0 - quantile)).ceil() as usize
}

const REPORTED: [(&str, f64); 3] = [("p50", 0.50), ("p95", 0.95), ("p99", 0.99)];

#[derive(Default, Clone)]
pub struct Series {
    samples: Vec<f64>,
    sorted: bool,
}

impl Series {
    pub fn push(&mut self, value: f64) {
        self.samples.push(value);
        self.sorted = false;
    }

    pub fn extend(&mut self, other: &Series) {
        self.samples.extend_from_slice(&other.samples);
        self.sorted = false;
    }

    fn sort(&mut self) {
        if !self.sorted {
            self.samples.sort_by(f64::total_cmp);
            self.sorted = true;
        }
    }

    /// `None` when the sample count cannot support the quantile. A caller that
    /// wants a number anyway has to say what it will do with a `null`.
    pub fn quantile(&mut self, quantile: f64) -> Option<f64> {
        if self.samples.len() < min_samples_for(quantile) {
            return None;
        }
        self.sort();
        let index = (((self.samples.len() - 1) as f64) * quantile).ceil() as usize;
        self.samples.get(index.min(self.samples.len() - 1)).copied()
    }

    pub fn mean(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        Some(self.samples.iter().sum::<f64>() / self.samples.len() as f64)
    }

    /// The count sits beside every percentile, and a percentile the count
    /// cannot support is `null` with the reason next to it.
    pub fn summary(&mut self) -> Value {
        let count = self.samples.len();
        let mut summary = json!({ "count": count });
        let mut unsupported = Vec::new();
        for (name, quantile) in REPORTED {
            match self.quantile(quantile) {
                Some(value) => summary[format!("{name}_ms")] = json!(value),
                None => {
                    summary[format!("{name}_ms")] = Value::Null;
                    unsupported.push(format!(
                        "{name} needs {} samples, has {count}",
                        min_samples_for(quantile)
                    ));
                }
            }
        }
        self.sort();
        summary["min_ms"] = json!(self.samples.first().copied());
        summary["max_ms"] = json!(self.samples.last().copied());
        summary["mean_ms"] = json!(self.mean());
        summary["unsupported"] = json!(unsupported);
        summary
    }
}

/// A latency pair per request.
///
/// `service` starts when the bytes went out; `response` starts when the pacer
/// intended them to. When the server slows and the client cannot issue on
/// schedule, `service` keeps looking fine and `response` does not — the old
/// harness reported only the first (`bin/load.rs:195` before the rewrite),
/// which is why `offered_eps: 3000` could sit beside `achieved_eps: 478` with
/// healthy percentiles between them.
#[derive(Default, Clone)]
pub struct LatencyPair {
    pub service: Series,
    pub response: Series,
    pub queueing: Series,
}

impl LatencyPair {
    pub fn record(&mut self, intended_to_sent_ms: f64, service_ms: f64) {
        self.queueing.push(intended_to_sent_ms);
        self.service.push(service_ms);
        self.response.push(intended_to_sent_ms + service_ms);
    }

    pub fn extend(&mut self, other: &LatencyPair) {
        self.service.extend(&other.service);
        self.response.extend(&other.response);
        self.queueing.extend(&other.queueing);
    }

    pub fn summary(&mut self) -> Value {
        json!({
            "service": self.service.summary(),
            "response": self.response.summary(),
            "queueing_delay": self.queueing.summary(),
            "note": "service time starts at the actual send, response time at the intended send. \
        Their difference is the delay the offered rate could not be issued at; a large gap means the \
        offered rate was not achievable and the service percentiles describe a rate nobody asked for.",
        })
    }
}

/// A sampled gauge with the shape kept, not just its endpoints.
#[derive(Default)]
pub struct GaugeSeries {
    /// `(seconds since the run started, value)`.
    pub samples: Vec<(f64, u64)>,
}

impl GaugeSeries {
    pub fn push(&mut self, elapsed_seconds: f64, value: u64) {
        self.samples.push((elapsed_seconds, value));
    }

    pub fn peak(&self) -> u64 {
        self.samples
            .iter()
            .map(|(_, value)| *value)
            .max()
            .unwrap_or(0)
    }

    pub fn trough(&self) -> u64 {
        self.samples
            .iter()
            .map(|(_, value)| *value)
            .min()
            .unwrap_or(0)
    }

    /// Least-squares slope in units per second. Zero when there is nothing to
    /// fit.
    pub fn slope_per_second(&self) -> f64 {
        let n = self.samples.len() as f64;
        if self.samples.len() < 2 {
            return 0.0;
        }
        let mean_t = self.samples.iter().map(|(t, _)| t).sum::<f64>() / n;
        let mean_v = self.samples.iter().map(|(_, v)| *v as f64).sum::<f64>() / n;
        let mut covariance = 0.0;
        let mut variance = 0.0;
        for (t, value) in &self.samples {
            covariance += (t - mean_t) * (*value as f64 - mean_v);
            variance += (t - mean_t) * (t - mean_t);
        }
        if variance == 0.0 {
            return 0.0;
        }
        covariance / variance
    }

    /// The lowest value seen at or after the peak.
    pub fn post_peak_min(&self) -> u64 {
        let Some(peak_index) = self
            .samples
            .iter()
            .enumerate()
            .max_by_key(|(_, (_, value))| *value)
            .map(|(index, _)| index)
        else {
            return 0;
        };
        self.samples[peak_index..]
            .iter()
            .map(|(_, value)| *value)
            .min()
            .unwrap_or(0)
    }

    pub fn summary(&self) -> Value {
        json!({
            "samples": self.samples.len(),
            "first": self.samples.first().map(|(_, value)| *value),
            "last": self.samples.last().map(|(_, value)| *value),
            "peak": self.peak(),
            "trough": self.trough(),
            "post_peak_min": self.post_peak_min(),
            "slope_per_second": self.slope_per_second(),
        })
    }

    pub fn points(&self) -> Value {
        json!(
            self.samples
                .iter()
                .map(|(t, value)| json!([t, value]))
                .collect::<Vec<_>>()
        )
    }
}

pub struct DrainVerdict {
    pub decided: bool,
    pub drained: bool,
    pub reason: String,
}

/// Whether flush is draining the WAL, judged on the series rather than on a
/// ratio.
///
/// The check this replaces was `trough <= peak * 0.5` (`bin/load.rs:452`
/// before the rewrite), which is order-blind: a backlog that dipped once early
/// and then climbed without ever coming down again satisfies it, and so does
/// any oscillation whatever its trend. Drainage is an ordered property — the
/// backlog has to come down *after* it went up — so it is tested that way,
/// with a non-positive slope accepted as the other way a run can show flush
/// keeping up over its whole length.
pub fn wal_backlog_drains(
    series: &GaugeSeries,
    ceiling_bytes: u64,
    minimum_samples: usize,
) -> DrainVerdict {
    if series.samples.len() < minimum_samples {
        return DrainVerdict {
            decided: false,
            drained: false,
            reason: format!(
                "{} backlog samples is too few to read a trend from; {minimum_samples} required",
                series.samples.len()
            ),
        };
    }
    let peak = series.peak();
    if peak > ceiling_bytes {
        return DrainVerdict {
            decided: true,
            drained: false,
            reason: format!(
                "backlog peaked at {peak} bytes, past the engine's own {ceiling_bytes}"
            ),
        };
    }
    // A peak this small is the sawtooth of a flush loop that never fell
    // behind at all: at 20k eps the compressed WAL accrues megabytes per
    // second, so a run whose worst backlog is under this floor was drained
    // continuously. The halving test below is scale-free and flips on such a
    // flat sawtooth — the verdict that rejected the run whose backlog peaked
    // at 2 MB while accepting the one that peaked at 200 was reading noise.
    const TRIVIALLY_HEALTHY_BYTES: u64 = 16 * 1024 * 1024;
    if peak < TRIVIALLY_HEALTHY_BYTES {
        return DrainVerdict {
            decided: true,
            drained: true,
            reason: format!(
                "backlog never exceeded {peak} bytes, below the {TRIVIALLY_HEALTHY_BYTES}-byte \
floor a falling-behind flush would dwarf"
            ),
        };
    }
    let slope = series.slope_per_second();
    let post_peak_min = series.post_peak_min();
    // Halving is the evidence that a flush cycle completed against this
    // backlog rather than that the sample landed between two writes.
    let came_down = post_peak_min * 2 <= peak;
    if slope <= 0.0 {
        DrainVerdict {
            decided: true,
            drained: true,
            reason: format!("backlog trend is {slope:.1} bytes/s over the run"),
        }
    } else if came_down {
        DrainVerdict {
            decided: true,
            drained: true,
            reason: format!(
                "backlog is still ramping ({slope:.1} bytes/s) but fell to {post_peak_min} \
after its peak of {peak}"
            ),
        }
    } else {
        DrainVerdict {
            decided: true,
            drained: false,
            reason: format!(
                "backlog rises at {slope:.1} bytes/s and never fell below half its peak of \
{peak} after reaching it"
            ),
        }
    }
}

/// A measured value against its ceiling. `measured: null` is neither a pass
/// nor a failure to be inherited — `pass` is `null` too, and the caller has to
/// decide, which is how an unmeasurable gate stops silently passing.
pub fn target_row(measured: Option<f64>, target: f64, note: &str) -> Value {
    let mut row = json!({
        "measured": measured,
        "target": target,
        "pass": measured.map(|value| value <= target),
    });
    if !note.is_empty() {
        row["note"] = json!(note);
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_percentile_that_would_land_on_the_maximum_is_refused() {
        assert_eq!(min_samples_for(0.50), 3);
        assert_eq!(min_samples_for(0.95), 21);
        assert_eq!(min_samples_for(0.99), 101);

        let mut eight = Series::default();
        for value in 0..8 {
            eight.push(value as f64);
        }
        assert_eq!(eight.quantile(0.50), Some(4.0));
        assert_eq!(
            eight.quantile(0.95),
            None,
            "a p95 over eight is the maximum"
        );
        assert_eq!(eight.quantile(0.99), None);

        let mut hundred_sixty_seven = Series::default();
        for value in 0..167 {
            hundred_sixty_seven.push(value as f64);
        }
        assert!(hundred_sixty_seven.quantile(0.95).is_some());
        assert!(hundred_sixty_seven.quantile(0.99).is_some());
    }

    #[test]
    fn a_refused_percentile_reports_null_with_its_reason() {
        let mut series = Series::default();
        for value in 0..8 {
            series.push(value as f64);
        }
        let summary = series.summary();
        assert_eq!(summary["count"], 8);
        assert!(summary["p50_ms"].is_number());
        assert!(summary["p95_ms"].is_null());
        let unsupported = summary["unsupported"].as_array().expect("array").len();
        assert_eq!(unsupported, 2);
    }

    fn series_of(values: &[u64]) -> GaugeSeries {
        let mut series = GaugeSeries::default();
        for (index, value) in values.iter().enumerate() {
            series.push(index as f64, *value);
        }
        series
    }

    #[test]
    fn a_backlog_that_only_grows_does_not_drain() {
        // Megabyte-scale values: a peak under the trivially-healthy floor is
        // a flush loop that never fell behind, not a climb worth failing.
        let growing = series_of(&(0..40).map(|step| step * 1_000_000).collect::<Vec<_>>());
        let verdict = wal_backlog_drains(&growing, u64::MAX, 8);
        assert!(verdict.decided);
        assert!(!verdict.drained, "{}", verdict.reason);
    }

    /// The compressed WAL made a healthy run's whole sawtooth fit under two
    /// megabytes, and the scale-free halving test read that noise as a
    /// failure. Below the floor the backlog is trivially healthy.
    #[test]
    fn a_tiny_backlog_is_healthy_whatever_its_shape() {
        let flat = series_of(
            &(0..40)
                .map(|step| 1_000_000 + (step % 3) * 400_000)
                .collect::<Vec<_>>(),
        );
        let verdict = wal_backlog_drains(&flat, u64::MAX, 8);
        assert!(verdict.decided);
        assert!(verdict.drained, "{}", verdict.reason);
    }

    /// The failure the ratio check could not see: one early dip satisfies
    /// `trough <= peak * 0.5` for the whole rest of a monotone climb.
    #[test]
    fn an_early_dip_does_not_excuse_a_climb() {
        let mut values = vec![50_000_000, 1];
        values.extend((0..40).map(|step| 10_000_000 + step * 5_000_000));
        let dipping = series_of(&values);
        assert!(
            dipping.trough() * 2 <= dipping.peak(),
            "the old check passes this"
        );
        let verdict = wal_backlog_drains(&dipping, u64::MAX, 8);
        assert!(!verdict.drained, "{}", verdict.reason);
    }

    /// The shape a ten-minute merge-on run actually produced: oscillating
    /// between 6 and 140 MB with a slightly negative trend.
    #[test]
    fn a_sawtooth_with_a_falling_trend_drains() {
        let values: Vec<u64> = (0..40)
            .map(|step| {
                let oscillation = if step % 4 == 3 {
                    6_000_000
                } else {
                    140_000_000
                };
                oscillation - step * 100_000
            })
            .collect();
        let verdict = wal_backlog_drains(&series_of(&values), u64::MAX, 8);
        assert!(verdict.drained, "{}", verdict.reason);
    }

    /// A short run is all ramp, so the slope is positive; what makes it
    /// healthy is that flush emptied the backlog after the peak.
    #[test]
    fn a_short_ramp_that_came_down_after_its_peak_drains() {
        let values = vec![
            0, 10_000, 20_000, 40_000, 80_000, 2_000, 20_000, 60_000, 5_000, 30_000,
        ];
        let verdict = wal_backlog_drains(&series_of(&values), u64::MAX, 8);
        assert!(verdict.drained, "{}", verdict.reason);
    }

    #[test]
    fn too_few_samples_decides_nothing() {
        let verdict = wal_backlog_drains(&series_of(&[1, 2, 3]), u64::MAX, 8);
        assert!(!verdict.decided);
        assert!(!verdict.drained);
    }

    #[test]
    fn a_backlog_past_the_engines_ceiling_fails_however_it_moved() {
        let values: Vec<u64> = (0..40)
            .map(|step| if step % 2 == 0 { 1 } else { 10_000_000 })
            .collect();
        let verdict = wal_backlog_drains(&series_of(&values), 1_000_000, 8);
        assert!(verdict.decided);
        assert!(!verdict.drained, "{}", verdict.reason);
    }

    #[test]
    fn an_unmeasured_target_neither_passes_nor_fails() {
        let row = target_row(None, 10.0, "");
        assert!(row["pass"].is_null());
        assert!(row["measured"].is_null());
        assert_eq!(
            target_row(Some(9.0), 10.0, "")["pass"],
            serde_json::json!(true)
        );
        assert_eq!(
            target_row(Some(11.0), 10.0, "")["pass"],
            serde_json::json!(false)
        );
    }
}
