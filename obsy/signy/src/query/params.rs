/// The first-party API's shared parameter grammar (`docs/QUERY_API.md`).
///
/// One hand-written parser instead of a serde extractor, for two reasons that
/// are the same reason: axum's `Query<T>` cannot collect repeated keys, and a
/// refusal here must teach — name the offending input, list what the endpoint
/// accepts, show a correct form — which is the contract that makes the API
/// usable by an agent that only sees the error text.
pub const LOGS_PARAMS: &[&str] = &[
    "start",
    "end",
    "attr",
    "contains",
    "not_contains",
    "regex",
    "not_regex",
    "parse",
    "limit",
    "direction",
];

pub(crate) const HISTOGRAM_PARAMS: &[&str] = &[
    "start",
    "end",
    "attr",
    "contains",
    "not_contains",
    "regex",
    "not_regex",
    "parse",
    "bucket",
];

/// No `end` and no `direction`: a tail follows now, forward, by definition.
pub(crate) const TAIL_PARAMS: &[&str] = &[
    "start",
    "attr",
    "contains",
    "not_contains",
    "regex",
    "not_regex",
    "parse",
    "limit",
    "delay",
];

/// What a delete request may select by: filters that name rows that exist.
/// `parse=` is absent deliberately — deleting "the lines whose parsed status
/// was 500" would change meaning whenever the parser does.
pub(crate) const DELETE_PARAMS: &[&str] = &[
    "start",
    "end",
    "attr",
    "contains",
    "not_contains",
    "regex",
    "not_regex",
];

/// The persisted form of a delete selector: `DELETE_PARAMS` minus the time
/// bounds, which are stored as their own fields.
pub(crate) const DELETE_FILTER_PARAMS: &[&str] =
    &["attr", "contains", "not_contains", "regex", "not_regex"];

pub(crate) const ATTRIBUTE_KEYS_PARAMS: &[&str] = &["start", "end"];

/// Line filters are deliberately absent: this endpoint samples metadata
/// without evaluating line content, and answering as though it did would be
/// the silent approximation this engine refuses elsewhere. The unknown-
/// parameter refusal is what says so.
pub(crate) const ATTRIBUTE_VALUES_PARAMS: &[&str] = &["start", "end", "attr"];

/// Line filters and `parse` are deliberately absent: a span is not a line,
/// and every trace predicate goes through `attr` — including the duration
/// comparisons only this surface accepts.
pub(crate) const TRACE_SEARCH_PARAMS: &[&str] = &["start", "end", "attr", "limit"];

pub(crate) const TRACE_ATTRIBUTE_KEYS_PARAMS: &[&str] = &["start", "end"];

/// `attr` narrows the values to traces the already-placed filters match, so a
/// dropdown offers only values whose click still returns something.
pub(crate) const TRACE_ATTRIBUTE_VALUES_PARAMS: &[&str] = &["start", "end", "attr"];

/// One operation per request, no expression language: an optional per-series
/// `func` with its `range`, then an optional one-level `agg` grouped `by` —
/// ratios are two requests composed client-side, and the refusals say so.
pub(crate) const METRIC_QUERY_PARAMS: &[&str] = &[
    "metric", "attr", "start", "end", "step", "func", "range", "agg", "by", "lookback", "limit",
];

/// The alert evaluation: the range grammar at a single instant, `at`.
pub(crate) const METRIC_INSTANT_PARAMS: &[&str] = &[
    "metric", "attr", "at", "func", "range", "agg", "by", "lookback", "limit",
];

/// `metric` names the *base* histogram — the engine selects `<metric>_bucket`
/// and groups by labels-minus-`le` — and `range` is the per-bucket increase
/// window, required because a bucket count without a window is a lifetime
/// total.
pub(crate) const METRIC_QUANTILE_PARAMS: &[&str] = &[
    "metric", "q", "attr", "start", "end", "step", "range", "by", "limit",
];

pub(crate) const METRIC_NAMES_PARAMS: &[&str] = &["start", "end"];

pub(crate) const METRIC_LABELS_PARAMS: &[&str] = &["start", "end", "metric", "attr"];

pub(crate) const METRIC_LABEL_VALUES_PARAMS: &[&str] = &["start", "end", "metric", "attr"];

pub(crate) const METRIC_SERIES_PARAMS: &[&str] = &["metric", "attr", "start", "end", "limit"];

/// The first-party routes, listed by the router fallback so one wrong request
/// teaches the whole surface. Grows with each endpoint that lands.
pub(crate) const ROUTES: &[&str] = &[
    "/signy/api/v1/logs",
    "/signy/api/v1/logs/histogram",
    "/signy/api/v1/logs/attributes",
    "/signy/api/v1/logs/attributes/{key}/values",
    "/signy/api/v1/logs/tail",
    "/signy/api/v1/logs/delete",
    "/signy/api/v1/traces",
    "/signy/api/v1/traces/{trace_id}",
    "/signy/api/v1/traces/attributes",
    "/signy/api/v1/traces/attributes/{key}/values",
    "/signy/api/v1/metrics/query",
    "/signy/api/v1/metrics/instant",
    "/signy/api/v1/metrics/quantile",
    "/signy/api/v1/metrics/names",
    "/signy/api/v1/metrics/labels",
    "/signy/api/v1/metrics/labels/{key}/values",
    "/signy/api/v1/metrics/series",
];

#[derive(Debug)]
pub struct FilterParams {
    pub start_ns: Option<i64>,
    pub end_ns: Option<i64>,
    pub query: logql::LogQuery,
    pub limit: Option<usize>,
    pub forward: bool,
    pub bucket_ns: Option<i64>,
    pub delay_seconds: Option<u64>,
}

/// Everything the one parsing loop collects, before an endpoint family's
/// finisher shapes it: `build_log_query` for the log endpoints,
/// `build_trace_filters` for the trace ones. The unknown-parameter refusal
/// lives in the loop, so both surfaces refuse with the same sentence.
#[derive(Default)]
struct RawParams {
    start_ns: Option<i64>,
    end_ns: Option<i64>,
    limit: Option<usize>,
    direction: Option<String>,
    bucket_ns: Option<i64>,
    delay_seconds: Option<u64>,
    attrs: Vec<(String, AttrOp, String)>,
    line_filters: Vec<logql::LineFilter>,
    parse_stages: Vec<logql::PipelineStage>,
    metric: Option<String>,
    at_ns: Option<i64>,
    step_ns: Option<i64>,
    func: Option<MetricFunc>,
    range_ns: Option<i64>,
    agg: Option<MetricAgg>,
    by: Vec<String>,
    lookback_ns: Option<i64>,
    q: Option<f64>,
}

pub fn parse_filter_params(
    raw: &str,
    now_ns: i64,
    allowed: &'static [&'static str],
) -> Result<FilterParams, String> {
    let raw = parse_raw_params(raw, now_ns, allowed)?;
    let forward = parse_direction(&raw.direction)?;
    let query = build_log_query(raw.attrs, raw.line_filters, raw.parse_stages)?;
    Ok(FilterParams {
        start_ns: raw.start_ns,
        end_ns: raw.end_ns,
        query,
        limit: raw.limit,
        forward,
        bucket_ns: raw.bucket_ns,
        delay_seconds: raw.delay_seconds,
    })
}

fn parse_raw_params(
    raw: &str,
    now_ns: i64,
    allowed: &'static [&'static str],
) -> Result<RawParams, String> {
    let mut start_ns = None;
    let mut end_ns = None;
    let mut limit = None;
    let mut direction: Option<String> = None;
    let mut bucket_ns = None;
    let mut delay_seconds = None;
    let mut attrs: Vec<(String, AttrOp, String)> = Vec::new();
    let mut line_filters: Vec<logql::LineFilter> = Vec::new();
    let mut parse_stages: Vec<logql::PipelineStage> = Vec::new();
    let mut metric: Option<String> = None;
    let mut at_ns: Option<i64> = None;
    let mut step_ns: Option<i64> = None;
    let mut func: Option<MetricFunc> = None;
    let mut range_ns: Option<i64> = None;
    let mut agg: Option<MetricAgg> = None;
    let mut by: Vec<String> = Vec::new();
    let mut lookback_ns: Option<i64> = None;
    let mut q: Option<f64> = None;

    for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
        let key = key.as_ref();
        let value = value.into_owned();
        if !allowed.contains(&key) {
            return Err(format!(
                "unknown parameter '{key}': this endpoint accepts {} — see docs/QUERY_API.md",
                allowed.join(", ")
            ));
        }
        match key {
            "start" => set_once("start", &mut start_ns, parse_time_or_relative_ns(&value, now_ns)?)?,
            "end" => set_once("end", &mut end_ns, parse_time_or_relative_ns(&value, now_ns)?)?,
            "limit" => set_once(
                "limit",
                &mut limit,
                value.parse::<usize>().map_err(|_| {
                    format!("invalid limit '{value}': expected a non-negative integer")
                })?,
            )?,
            "direction" => set_once("direction", &mut direction, value)?,
            "bucket" => {
                let parsed = logql::parse_duration_ns(&value)
                    .map_err(|error| format!("invalid bucket '{value}': {error}"))?;
                if parsed <= 0 {
                    return Err(format!(
                        "invalid bucket '{value}': the width must be positive, like bucket=30s"
                    ));
                }
                set_once("bucket", &mut bucket_ns, parsed)?
            }
            "delay" => set_once(
                "delay",
                &mut delay_seconds,
                value.parse::<u64>().map_err(|_| {
                    format!("invalid delay '{value}': expected whole seconds, like delay=2")
                })?,
            )?,
            "attr" => attrs.push(split_attr(&value)?),
            "contains" => line_filters.push(logql::LineFilter::Contains(value)),
            "not_contains" => line_filters.push(logql::LineFilter::NotContains(value)),
            "regex" => line_filters.push(logql::LineFilter::Regex(line_regex("regex", &value)?)),
            "not_regex" => {
                line_filters.push(logql::LineFilter::NotRegex(line_regex("not_regex", &value)?))
            }
            "parse" => {
                let stage = match value.as_str() {
                    "json" => logql::PipelineStage::Json,
                    "logfmt" => logql::PipelineStage::Logfmt,
                    other => {
                        return Err(format!(
                            "invalid parse value '{other}': expected json or logfmt"
                        ));
                    }
                };
                if parse_stages
                    .iter()
                    .any(|existing| std::mem::discriminant(existing) == std::mem::discriminant(&stage))
                {
                    return Err(format!("parse={value} was given more than once"));
                }
                parse_stages.push(stage);
            }
            "metric" => set_once("metric", &mut metric, value)?,
            "at" => set_once("at", &mut at_ns, parse_time_or_relative_ns(&value, now_ns)?)?,
            "step" => set_once(
                "step",
                &mut step_ns,
                positive_duration_param("step", &value)?,
            )?,
            "range" => set_once(
                "range",
                &mut range_ns,
                positive_duration_param("range", &value)?,
            )?,
            "lookback" => set_once(
                "lookback",
                &mut lookback_ns,
                positive_duration_param("lookback", &value)?,
            )?,
            "func" => set_once(
                "func",
                &mut func,
                match value.as_str() {
                    "rate" => MetricFunc::Rate,
                    "increase" => MetricFunc::Increase,
                    other => {
                        return Err(format!(
                            "invalid func '{other}': expected rate or increase — see \
docs/QUERY_API.md"
                        ));
                    }
                },
            )?,
            "agg" => set_once(
                "agg",
                &mut agg,
                match value.as_str() {
                    "sum" => MetricAgg::Sum,
                    "avg" => MetricAgg::Avg,
                    "min" => MetricAgg::Min,
                    "max" => MetricAgg::Max,
                    "count" => MetricAgg::Count,
                    other => {
                        return Err(format!(
                            "invalid agg '{other}': expected sum, avg, min, max or count"
                        ));
                    }
                },
            )?,
            "by" => by.push(value),
            "q" => set_once(
                "q",
                &mut q,
                value
                    .parse::<f64>()
                    .ok()
                    .filter(|quantile| (0.0..=1.0).contains(quantile))
                    .ok_or_else(|| {
                        format!(
                            "invalid q '{value}': expected a quantile between 0 and 1, like q=0.99"
                        )
                    })?,
            )?,
            _ => unreachable!("every allowed parameter is matched above"),
        }
    }

    Ok(RawParams {
        start_ns,
        end_ns,
        limit,
        direction,
        bucket_ns,
        delay_seconds,
        attrs,
        line_filters,
        parse_stages,
        metric,
        at_ns,
        step_ns,
        func,
        range_ns,
        agg,
        by,
        lookback_ns,
        q,
    })
}

fn positive_duration_param(name: &str, value: &str) -> Result<i64, String> {
    let parsed = logql::parse_duration_ns(value)
        .map_err(|error| format!("invalid {name} '{value}': {error}"))?;
    if parsed <= 0 {
        return Err(format!(
            "invalid {name} '{value}': the duration must be positive, like {name}=30s"
        ));
    }
    Ok(parsed)
}

pub(crate) struct TraceFilterParams {
    pub start_ns: Option<i64>,
    pub end_ns: Option<i64>,
    pub filters: Vec<TraceFilter>,
    pub limit: Option<usize>,
}

pub(crate) fn parse_trace_filter_params(
    raw: &str,
    now_ns: i64,
    allowed: &'static [&'static str],
) -> Result<TraceFilterParams, String> {
    let raw = parse_raw_params(raw, now_ns, allowed)?;
    let filters = build_trace_filters(raw.attrs)?;
    Ok(TraceFilterParams {
        start_ns: raw.start_ns,
        end_ns: raw.end_ns,
        filters,
        limit: raw.limit,
    })
}

/// A trace-side `attr` filter, evaluated per span over `tag_value` — the
/// flattened lookup whose intrinsics are `name`, `duration`, and `status`. An
/// absent key behaves as the empty string, mirroring `LabelMatcher::matches`,
/// so `attr=k!=v` matches spans without `k` exactly as it does on logs.
pub(crate) enum TraceFilter {
    Eq { key: String, value: String },
    Neq { key: String, value: String },
    Re { key: String, regex: regex::Regex },
    NRe { key: String, regex: regex::Regex },
    Duration { op: logql::FieldOp, threshold_ns: u64 },
}

impl TraceFilter {
    pub(crate) fn matches(&self, span: &crate::trace::TraceSpan) -> bool {
        match self {
            Self::Eq { key, value } => span.tag_value(key).unwrap_or_default() == *value,
            Self::Neq { key, value } => span.tag_value(key).unwrap_or_default() != *value,
            Self::Re { key, regex } => regex.is_match(&span.tag_value(key).unwrap_or_default()),
            Self::NRe { key, regex } => !regex.is_match(&span.tag_value(key).unwrap_or_default()),
            Self::Duration { op, threshold_ns } => {
                let duration = span.duration_ns();
                match op {
                    logql::FieldOp::Eq => duration == *threshold_ns,
                    logql::FieldOp::Neq => duration != *threshold_ns,
                    logql::FieldOp::Lt => duration < *threshold_ns,
                    logql::FieldOp::Lte => duration <= *threshold_ns,
                    logql::FieldOp::Gt => duration > *threshold_ns,
                    logql::FieldOp::Gte => duration >= *threshold_ns,
                    logql::FieldOp::Regex | logql::FieldOp::NotRegex => {
                        unreachable!("build_trace_filters never builds a regex duration")
                    }
                }
            }
        }
    }
}

/// Comparisons are `duration`-only, deliberately: every other value is stored
/// stringified, and a lexicographic `>=` over strings would answer wrongly
/// without saying so. Equality on `duration` parses the value as a duration
/// too — `attr=duration=150ms` compares nanoseconds, not the string "150ms"
/// against "150000000".
fn build_trace_filters(attrs: Vec<(String, AttrOp, String)>) -> Result<Vec<TraceFilter>, String> {
    attrs
        .into_iter()
        .map(|(key, op, value)| match op {
            AttrOp::Compare(op) => {
                if key != "duration" {
                    return Err(format!(
                        "attr filter '{key}{symbol}{value}' compares a key that is not duration: \
>=, <=, >, < apply to the duration intrinsic only, like attr=duration>=250ms — \
see docs/QUERY_API.md",
                        symbol = compare_symbol(op)
                    ));
                }
                Ok(TraceFilter::Duration {
                    op,
                    threshold_ns: trace_duration_threshold(compare_symbol(op), &value)?,
                })
            }
            AttrOp::Match(logql::MatcherOp::Eq) if key == "duration" => Ok(TraceFilter::Duration {
                op: logql::FieldOp::Eq,
                threshold_ns: trace_duration_threshold("=", &value)?,
            }),
            AttrOp::Match(logql::MatcherOp::Neq) if key == "duration" => {
                Ok(TraceFilter::Duration {
                    op: logql::FieldOp::Neq,
                    threshold_ns: trace_duration_threshold("!=", &value)?,
                })
            }
            AttrOp::Match(logql::MatcherOp::Eq) => Ok(TraceFilter::Eq { key, value }),
            AttrOp::Match(logql::MatcherOp::Neq) => Ok(TraceFilter::Neq { key, value }),
            AttrOp::Match(logql::MatcherOp::Re) => Ok(TraceFilter::Re {
                regex: anchored_regex(&value)?,
                key,
            }),
            AttrOp::Match(logql::MatcherOp::NRe) => Ok(TraceFilter::NRe {
                regex: anchored_regex(&value)?,
                key,
            }),
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MetricFunc {
    Rate,
    Increase,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MetricAgg {
    Sum,
    Avg,
    Min,
    Max,
    Count,
}

/// A metric-side `attr` filter, evaluated over a series' label values. An
/// absent key behaves as the empty string — `LabelMatcher::matches`' rule, so
/// `attr=k!=v` matches series without `k` exactly as it does on logs.
pub(crate) enum MetricFilter {
    Eq { key: String, value: String },
    Neq { key: String, value: String },
    Re { key: String, regex: regex::Regex },
    NRe { key: String, regex: regex::Regex },
}

impl MetricFilter {
    /// The label this filter is about.
    pub(crate) fn key(&self) -> &str {
        match self {
            Self::Eq { key, .. }
            | Self::Neq { key, .. }
            | Self::Re { key, .. }
            | Self::NRe { key, .. } => key,
        }
    }

    pub(crate) fn matches<'a>(&self, lookup: &dyn Fn(&str) -> Option<&'a str>) -> bool {
        match self {
            Self::Eq { key, value } => lookup(key).unwrap_or_default() == value.as_str(),
            Self::Neq { key, value } => lookup(key).unwrap_or_default() != value.as_str(),
            Self::Re { key, regex } => regex.is_match(lookup(key).unwrap_or_default()),
            Self::NRe { key, regex } => !regex.is_match(lookup(key).unwrap_or_default()),
        }
    }
}

pub(crate) struct MetricParams {
    pub metric: Option<String>,
    pub filters: Vec<MetricFilter>,
    pub start_ns: Option<i64>,
    pub end_ns: Option<i64>,
    pub at_ns: Option<i64>,
    pub step_ns: Option<i64>,
    pub func: Option<MetricFunc>,
    pub range_ns: Option<i64>,
    pub agg: Option<MetricAgg>,
    pub by: Vec<String>,
    pub lookback_ns: Option<i64>,
    pub limit: Option<usize>,
    pub q: Option<f64>,
}

/// The metric finisher. The cross-field rules live here rather than per
/// handler so every metric endpoint refuses with the same sentence: a `func`
/// without its window, a window without its `func`, and grouping without an
/// aggregation are each one teaching refusal.
pub(crate) fn parse_metric_params(
    raw: &str,
    now_ns: i64,
    allowed: &'static [&'static str],
) -> Result<MetricParams, String> {
    let raw = parse_raw_params(raw, now_ns, allowed)?;
    let filters = build_metric_filters(raw.attrs)?;
    if raw.func.is_some() && raw.range_ns.is_none() {
        return Err(
            "func was given without range: a rate needs its window, like func=rate&range=60s — \
see docs/QUERY_API.md"
                .to_string(),
        );
    }
    if raw.range_ns.is_some() && raw.func.is_none() && !allowed.contains(&"q") {
        return Err(
            "range was given without func: the window belongs to rate or increase, like \
func=rate&range=60s — see docs/QUERY_API.md"
                .to_string(),
        );
    }
    if !raw.by.is_empty() && raw.agg.is_none() {
        return Err(
            "by was given without agg: grouping is a property of the aggregation, like \
agg=sum&by=service — see docs/QUERY_API.md"
                .to_string(),
        );
    }
    Ok(MetricParams {
        metric: raw.metric,
        filters,
        start_ns: raw.start_ns,
        end_ns: raw.end_ns,
        at_ns: raw.at_ns,
        step_ns: raw.step_ns,
        func: raw.func,
        range_ns: raw.range_ns,
        agg: raw.agg,
        by: raw.by,
        lookback_ns: raw.lookback_ns,
        limit: raw.limit,
        q: raw.q,
    })
}

/// Metric labels are stored stringified, so a lexicographic comparison would
/// answer wrongly without saying so — the same reasoning that keeps
/// comparisons `duration`-only on the trace surface keeps them out entirely
/// here.
fn build_metric_filters(
    attrs: Vec<(String, AttrOp, String)>,
) -> Result<Vec<MetricFilter>, String> {
    attrs
        .into_iter()
        .map(|(key, op, value)| match op {
            AttrOp::Compare(op) => Err(format!(
                "attr filter '{key}{symbol}{value}' uses a comparison: metric label filters \
support =, !=, =~, !~ — duration comparisons belong to the trace endpoints, see \
docs/QUERY_API.md",
                symbol = compare_symbol(op)
            )),
            AttrOp::Match(logql::MatcherOp::Eq) => Ok(MetricFilter::Eq { key, value }),
            AttrOp::Match(logql::MatcherOp::Neq) => Ok(MetricFilter::Neq { key, value }),
            AttrOp::Match(logql::MatcherOp::Re) => Ok(MetricFilter::Re {
                regex: anchored_regex(&value)?,
                key,
            }),
            AttrOp::Match(logql::MatcherOp::NRe) => Ok(MetricFilter::NRe {
                regex: anchored_regex(&value)?,
                key,
            }),
        })
        .collect()
}

fn trace_duration_threshold(symbol: &str, value: &str) -> Result<u64, String> {
    let ns = logql::parse_duration_ns(value).map_err(|_| {
        format!(
            "invalid duration '{value}' in attr filter 'duration{symbol}{value}': \
write a unit, like attr=duration>=250ms"
        )
    })?;
    u64::try_from(ns).map_err(|_| {
        format!(
            "invalid duration '{value}' in attr filter 'duration{symbol}{value}': \
the duration must not be negative"
        )
    })
}

fn set_once<T>(name: &str, slot: &mut Option<T>, value: T) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("parameter '{name}' was given more than once"));
    }
    *slot = Some(value);
    Ok(())
}

/// `-1h` is one hour before now; `-123` stays a negative epoch. The unit
/// suffix is what separates them, so the two never collide.
pub(crate) fn parse_time_or_relative_ns(input: &str, now_ns: i64) -> Result<i64, String> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix('-')
        && rest.bytes().any(|byte| byte.is_ascii_alphabetic())
        && !rest.contains(['e', 'E'])
    {
        let duration = logql::parse_duration_ns(rest)
            .map_err(|error| format!("invalid relative time '{trimmed}': {error}"))?;
        return Ok(now_ns.saturating_sub(duration));
    }
    parse_time_ns(trimmed)
}

/// An `attr` filter's operator: one of the four matchers every endpoint
/// accepts, or a comparison, which only the trace endpoints do — the log
/// surface refuses comparisons in `build_log_query`, so the refusal is
/// inherited by every handler that builds a `LogQuery`.
#[derive(Debug, Clone)]
pub(crate) enum AttrOp {
    Match(logql::MatcherOp),
    Compare(logql::FieldOp),
}

/// Only the four comparison operators reach this: `split_attr` never
/// constructs `AttrOp::Compare` from anything else.
pub(crate) fn compare_symbol(op: logql::FieldOp) -> &'static str {
    match op {
        logql::FieldOp::Gte => ">=",
        logql::FieldOp::Lte => "<=",
        logql::FieldOp::Gt => ">",
        logql::FieldOp::Lt => "<",
        _ => unreachable!("split_attr only constructs comparison operators"),
    }
}

/// The key ends at the first operator: `!=`, `!~`, `=~`, `=`, `>=`, `<=`, `>`,
/// or `<`, longest match at that position — the two-byte forms are checked
/// before their one-byte prefixes, so `duration>=1.5s` splits at `>=`, never at
/// `>` with `=1.5s` as the value. Operators are ASCII, so splitting there is
/// always a char boundary however non-ASCII the key or value.
fn split_attr(input: &str) -> Result<(String, AttrOp, String), String> {
    let bytes = input.as_bytes();
    for at in 0..bytes.len() {
        let (op, op_len) = match bytes[at] {
            b'!' if bytes.get(at + 1) == Some(&b'=') => (AttrOp::Match(logql::MatcherOp::Neq), 2),
            b'!' if bytes.get(at + 1) == Some(&b'~') => (AttrOp::Match(logql::MatcherOp::NRe), 2),
            b'=' if bytes.get(at + 1) == Some(&b'~') => (AttrOp::Match(logql::MatcherOp::Re), 2),
            b'=' => (AttrOp::Match(logql::MatcherOp::Eq), 1),
            b'>' if bytes.get(at + 1) == Some(&b'=') => (AttrOp::Compare(logql::FieldOp::Gte), 2),
            b'>' => (AttrOp::Compare(logql::FieldOp::Gt), 1),
            b'<' if bytes.get(at + 1) == Some(&b'=') => (AttrOp::Compare(logql::FieldOp::Lte), 2),
            b'<' => (AttrOp::Compare(logql::FieldOp::Lt), 1),
            _ => continue,
        };
        if at == 0 {
            return Err(format!(
                "attr filter '{input}' has an empty key: write attr=key=value"
            ));
        }
        return Ok((
            input[..at].to_string(),
            op,
            input[at + op_len..].to_string(),
        ));
    }
    Err(format!(
        "attr filter '{input}' has no operator: write attr=key=value (also !=, =~, !~, and >=, <=, >, < on the trace endpoints)"
    ))
}

fn line_regex(param: &str, value: &str) -> Result<regex::Regex, String> {
    regex::Regex::new(value)
        .map_err(|error| format!("invalid {param} regex '{value}': {error}"))
}

/// Without `parse=`, `attr` filters are selector matchers — a statement about
/// the row's pushed attributes, and the position exact-field pruning reads.
/// With `parse=`, they become field filters after the parse stages, so they
/// see pushed attributes and extracted fields alike (pushed attributes shadow
/// same-named extractions — `merge_extracted`'s rule, stated in QUERY_API.md).
fn build_log_query(
    attrs: Vec<(String, AttrOp, String)>,
    line_filters: Vec<logql::LineFilter>,
    parse_stages: Vec<logql::PipelineStage>,
) -> Result<logql::LogQuery, String> {
    let attrs = attrs
        .into_iter()
        .map(|(name, op, value)| match op {
            AttrOp::Match(op) => Ok((name, op, value)),
            AttrOp::Compare(op) => Err(format!(
                "attr filter '{name}{symbol}{value}' uses a comparison: log filters support =, !=, =~, !~ — duration comparisons belong to the trace endpoints, see docs/QUERY_API.md",
                symbol = compare_symbol(op)
            )),
        })
        .collect::<Result<Vec<_>, String>>()?;
    if parse_stages.is_empty() {
        let matchers = attrs
            .into_iter()
            .map(|(name, op, value)| logql::LabelMatcher::new(name, op, value))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(logql::LogQuery {
            matchers,
            line_filters,
            stages: Vec::new(),
        });
    }
    let mut stages = parse_stages;
    for (name, op, value) in attrs {
        let (op, value) = match op {
            logql::MatcherOp::Eq => (logql::FieldOp::Eq, logql::FieldValue::String(value)),
            logql::MatcherOp::Neq => (logql::FieldOp::Neq, logql::FieldValue::String(value)),
            logql::MatcherOp::Re => (
                logql::FieldOp::Regex,
                logql::FieldValue::Regex(anchored_regex(&value)?),
            ),
            logql::MatcherOp::NRe => (
                logql::FieldOp::NotRegex,
                logql::FieldValue::Regex(anchored_regex(&value)?),
            ),
        };
        stages.push(logql::PipelineStage::Field(logql::FieldFilter {
            name,
            op,
            value,
        }));
    }
    Ok(logql::LogQuery {
        matchers: Vec::new(),
        line_filters,
        stages,
    })
}

/// Anchored like a selector matcher's regex, so `attr=k=~v` means the same
/// thing with and without `parse=`.
fn anchored_regex(value: &str) -> Result<regex::Regex, String> {
    regex::Regex::new(&format!("^(?:{value})$"))
        .map_err(|error| format!("invalid regular expression '{value}': {error}"))
}

/// The canonical serialized form of a filter-only query — what a delete
/// request persists, and exactly what `parse_filter_params` reads back: one
/// parser total, and the stored form is the documented form.
pub(crate) fn canonical_filter_query(query: &logql::LogQuery) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for matcher in &query.matchers {
        let op = match matcher.op {
            logql::MatcherOp::Eq => "=",
            logql::MatcherOp::Neq => "!=",
            logql::MatcherOp::Re => "=~",
            logql::MatcherOp::NRe => "!~",
        };
        serializer.append_pair("attr", &format!("{}{op}{}", matcher.name, matcher.value));
    }
    for filter in &query.line_filters {
        let (name, value) = match filter {
            logql::LineFilter::Contains(value) => ("contains", value.as_str()),
            logql::LineFilter::NotContains(value) => ("not_contains", value.as_str()),
            logql::LineFilter::Regex(regex) => ("regex", regex.as_str()),
            logql::LineFilter::NotRegex(regex) => ("not_regex", regex.as_str()),
        };
        serializer.append_pair(name, value);
    }
    serializer.finish()
}

/// `effective_start = max(requested_start, now - retention(tenant))`.
fn clamp_to_retention(start_ns: i64, retention_floor_ns: Option<i64>) -> i64 {
    match retention_floor_ns {
        Some(floor_ns) => start_ns.max(floor_ns),
        None => start_ns,
    }
}

fn duration_to_i64_ns(duration: std::time::Duration) -> i64 {
    duration
        .as_nanos()
        .min(i64::MAX as u128) as i64
}
