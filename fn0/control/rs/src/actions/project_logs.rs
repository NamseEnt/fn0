use crate::common::auth;
use crate::common::telemetry::{
    AttributeEquals, Direction, HistogramBucket, LogHistogram, LogRow, LogSearch, TelemetryClient,
};
use forte_sdk::*;
use serde::{Deserialize, Serialize};

/// The largest number of rows one query may pull back, whatever the caller
/// asks for. loggytracy caps this too, but control holds its own ceiling so the
/// bound does not depend on the backend's configuration.
const MAX_LIMIT: u32 = 500;

#[derive(Deserialize)]
pub struct AttributeEqualsInput {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub start: String,
    pub end: Option<String>,
    pub stream: Option<String>,
    pub attributes: Vec<AttributeEqualsInput>,
    pub contains: Vec<String>,
    pub regex: Option<String>,
    pub limit: u32,
    /// Pagination cursor: the timestamp (nanoseconds, as a string) of the
    /// oldest row already shown. The next page is rows older than it.
    pub before: Option<String>,
    /// Live-tail cursor: the timestamp (nanoseconds, as a string) of the newest
    /// row already shown. Set, the query runs forward from it and returns rows
    /// ascending in time; the cursor row itself can reappear, so the caller
    /// deduplicates at the boundary. Wins over `before`/`end`.
    pub after: Option<String>,
    pub include_histogram: bool,
}

#[derive(Serialize)]
pub struct AttributePair {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct LogRowOut {
    pub timestamp: String,
    pub line: String,
    pub attributes: Vec<AttributePair>,
}

#[derive(Serialize)]
pub struct HistogramBucketOut {
    pub bucket_start: String,
    pub bucket_end: String,
    pub count: u64,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        rows: Vec<LogRowOut>,
        histogram: Option<Vec<HistogramBucketOut>>,
    },
    NotLoggedIn,
    NotFound,
    Error {
        message: String,
    },
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
    let stream = normalize_stream(input.stream.as_deref());
    let attribute_equals = match attribute_equals(&input.attributes) {
        Ok(filters) => filters,
        Err(message) => return Output::Error { message },
    };
    let contains: Vec<String> = input
        .contains
        .iter()
        .filter(|term| !term.is_empty())
        .cloned()
        .collect();
    let regex = non_empty(input.regex.as_deref());
    let limit = input.limit.clamp(1, MAX_LIMIT);
    let (start, end, direction) = match non_empty(input.after.as_deref()) {
        Some(after) => (after, None, Direction::Forward),
        None => (
            input.start.as_str(),
            input.before.as_deref().or(input.end.as_deref()),
            Direction::Backward,
        ),
    };

    let client = TelemetryClient::from_env();

    let rows = match client
        .search_logs(LogSearch {
            project_id: &input.project_id,
            start,
            end,
            stream,
            attribute_equals: &attribute_equals,
            contains: &contains,
            regex,
            limit,
            direction,
        })
        .await
    {
        Ok(rows) => rows,
        Err(e) => return classify(e),
    };

    let histogram = if input.include_histogram {
        match client
            .histogram(LogHistogram {
                project_id: &input.project_id,
                start: &input.start,
                end: input.end.as_deref(),
                stream,
                attribute_equals: &attribute_equals,
                contains: &contains,
                regex,
            })
            .await
        {
            Ok(buckets) => Some(buckets.into_iter().map(histogram_out).collect()),
            Err(e) => return classify(e),
        }
    } else {
        None
    };

    Output::Ok {
        rows: rows.into_iter().map(row_out).collect(),
        histogram,
    }
}

fn normalize_stream(stream: Option<&str>) -> Option<&str> {
    match stream {
        Some("stdout") => Some("stdout"),
        Some("stderr") => Some("stderr"),
        _ => None,
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.is_empty())
}

fn attribute_equals(inputs: &[AttributeEqualsInput]) -> Result<Vec<AttributeEquals>, String> {
    inputs
        .iter()
        .map(|input| AttributeEquals::new(input.key.clone(), input.value.clone()))
        .collect()
}

fn row_out(row: LogRow) -> LogRowOut {
    LogRowOut {
        timestamp: row.timestamp,
        line: row.line,
        attributes: row
            .attributes
            .into_iter()
            .map(|(key, value)| AttributePair { key, value })
            .collect(),
    }
}

fn histogram_out(bucket: HistogramBucket) -> HistogramBucketOut {
    HistogramBucketOut {
        bucket_start: bucket.bucket_start,
        bucket_end: bucket.bucket_end,
        count: bucket.count,
    }
}

/// A 4xx from loggytracy names a bad filter the user can fix, so it goes back as
/// a visible message; anything else is ours to log and hide behind a generic
/// error.
fn classify(error: anyhow::Error) -> Output {
    let text = error.to_string();
    if text.contains("status=4") {
        Output::Error { message: text }
    } else {
        tracing::error!("project_logs: {error}");
        Output::InternalError
    }
}
