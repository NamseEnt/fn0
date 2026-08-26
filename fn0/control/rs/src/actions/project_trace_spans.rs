use crate::common::auth;
use crate::common::telemetry::{TelemetryClient, TraceSpanEvent, TraceSpanRow};
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub trace_id: String,
}

#[derive(Serialize)]
pub struct AttributePair {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct SpanEventOut {
    pub timestamp: String,
    pub name: String,
    pub attributes: Vec<AttributePair>,
}

#[derive(Serialize)]
pub struct SpanOut {
    pub span_id: String,
    pub parent_span_id: String,
    pub name: String,
    pub kind: String,
    pub service: String,
    pub status: String,
    pub start: String,
    pub end: String,
    pub duration: String,
    pub attributes: Vec<AttributePair>,
    pub events: Vec<SpanEventOut>,
}

#[derive(Serialize)]
pub enum Output {
    Ok { spans: Vec<SpanOut> },
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

    let trace_id = req.body.trace_id.trim();
    if trace_id.len() != 32 || !trace_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Output::NotFound;
    }

    let client = TelemetryClient::from_env();
    match client
        .project_trace_spans(&req.body.project_id, trace_id)
        .await
    {
        Ok(spans) if spans.is_empty() => Output::NotFound,
        Ok(spans) => Output::Ok {
            spans: spans.into_iter().map(span_out).collect(),
        },
        Err(e) => classify(e),
    }
}

fn span_out(span: TraceSpanRow) -> SpanOut {
    SpanOut {
        span_id: span.span_id,
        parent_span_id: span.parent_span_id,
        name: span.name,
        kind: span.kind,
        service: span.service,
        status: span.status,
        start: span.start,
        end: span.end,
        duration: span.duration,
        attributes: attribute_pairs(span.attributes),
        events: span.events.into_iter().map(event_out).collect(),
    }
}

fn event_out(event: TraceSpanEvent) -> SpanEventOut {
    SpanEventOut {
        timestamp: event.timestamp,
        name: event.name,
        attributes: attribute_pairs(event.attributes),
    }
}

fn attribute_pairs(attributes: std::collections::BTreeMap<String, String>) -> Vec<AttributePair> {
    attributes
        .into_iter()
        .map(|(key, value)| AttributePair { key, value })
        .collect()
}

/// An unknown id is loggytracy's 404 and a retention-emptied one answers the
/// same; both are the caller's NotFound. A remaining 4xx (a 413 trace too big
/// for one response) names a real condition the user should see; anything else
/// is ours to log and hide behind a generic error.
fn classify(error: anyhow::Error) -> Output {
    let text = error.to_string();
    if text.contains("status=404") {
        Output::NotFound
    } else if text.contains("status=4") {
        Output::Error { message: text }
    } else {
        tracing::error!("project_trace_spans: {error}");
        Output::InternalError
    }
}
