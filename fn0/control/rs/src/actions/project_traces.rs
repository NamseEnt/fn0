use crate::common::auth;
use crate::common::telemetry::{TelemetryClient, TraceSearch, TraceSummary};
use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// The largest number of trace summaries one query may pull back, whatever the
/// caller asks for. loggytracy caps this too, but control holds its own ceiling
/// so the bound does not depend on the backend's configuration.
const MAX_LIMIT: u32 = 200;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub start: String,
    pub end: Option<String>,
    pub status: Option<String>,
    /// loggytracy duration syntax with a unit, e.g. `250ms` or `1.5s`.
    pub min_duration: Option<String>,
    pub name_contains: Option<String>,
    pub name_regex: Option<String>,
    pub limit: u32,
    /// Pagination cursor: the `start` (nanoseconds, as a string) of the oldest
    /// trace already shown. The next page is traces older than it. A trace
    /// spanning the cursor can reappear — the caller deduplicates by trace id.
    pub before_start: Option<String>,
}

#[derive(Serialize)]
pub struct TraceSummaryOut {
    pub trace_id: String,
    pub root_service: String,
    pub root_name: String,
    pub start: String,
    pub end: String,
    pub duration: String,
    pub span_count: u64,
}

#[derive(Serialize)]
pub enum Output {
    Ok { traces: Vec<TraceSummaryOut> },
    NotLoggedIn,
    NotFound,
    Error { message: String },
    InternalError,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::current_user(req.jar).await else {
        return Output::NotLoggedIn;
    };
    let owns_project = user
        .projects
        .iter()
        .any(|entry| entry.project_id == req.body.project_id);
    if !owns_project {
        return Output::NotFound;
    }

    let input = &req.body;
    let status = normalize_status(input.status.as_deref());
    let min_duration = non_empty(input.min_duration.as_deref());
    let name_regex_owned = name_regex(input.name_contains.as_deref(), input.name_regex.as_deref());
    let limit = input.limit.clamp(1, MAX_LIMIT);
    let end = input.before_start.as_deref().or(input.end.as_deref());

    let client = TelemetryClient::from_env();
    match client
        .search_traces(TraceSearch {
            project_id: &input.project_id,
            start: &input.start,
            end,
            status,
            min_duration,
            name_regex: name_regex_owned.as_deref(),
            limit,
        })
        .await
    {
        Ok(traces) => Output::Ok {
            traces: traces.into_iter().map(summary_out).collect(),
        },
        Err(e) => classify(e),
    }
}

fn normalize_status(status: Option<&str>) -> Option<&str> {
    match status {
        Some("unset") => Some("unset"),
        Some("ok") => Some("ok"),
        Some("error") => Some("error"),
        _ => None,
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.is_empty())
}

/// loggytracy's `=~` regexes are anchored — the whole span name must match —
/// so the contains form wraps the escaped text in `.*`.
fn name_regex(contains: Option<&str>, regex: Option<&str>) -> Option<String> {
    if let Some(regex) = non_empty(regex) {
        return Some(regex.to_string());
    }
    let contains = non_empty(contains)?;
    Some(format!(".*{}.*", regex_escape(contains)))
}

fn regex_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        if matches!(
            character,
            '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$' | '\\'
        ) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn summary_out(summary: TraceSummary) -> TraceSummaryOut {
    TraceSummaryOut {
        trace_id: summary.trace_id,
        root_service: summary.root_service,
        root_name: summary.root_name,
        start: summary.start,
        end: summary.end,
        duration: summary.duration,
        span_count: summary.span_count,
    }
}

/// A 4xx from loggytracy names a bad filter or an over-broad window the user
/// can fix, so it goes back as a visible message; anything else is ours to log
/// and hide behind a generic error.
fn classify(error: anyhow::Error) -> Output {
    let text = error.to_string();
    if text.contains("status=4") {
        Output::Error { message: text }
    } else {
        tracing::error!("project_traces: {error}");
        Output::InternalError
    }
}
