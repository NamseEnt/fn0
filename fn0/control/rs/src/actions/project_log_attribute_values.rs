use crate::common::auth;
use crate::common::telemetry::{AttributeEquals, LogAttributeValues, TelemetryClient};
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AttributeEqualsInput {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    /// The attribute key whose values to suggest.
    pub key: String,
    pub start: String,
    pub end: Option<String>,
    /// Filters already placed in the search; the suggestions are sampled from
    /// rows those filters match, so every offered value still returns rows.
    pub attributes: Vec<AttributeEqualsInput>,
}

#[derive(Serialize)]
pub enum Output {
    Ok { values: Vec<String> },
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
    if let Err(message) = AttributeEquals::new(input.key.clone(), String::new()) {
        return Output::Error { message };
    }
    let attribute_equals: Vec<AttributeEquals> = match input
        .attributes
        .iter()
        .map(|filter| AttributeEquals::new(filter.key.clone(), filter.value.clone()))
        .collect()
    {
        Ok(filters) => filters,
        Err(message) => return Output::Error { message },
    };

    let client = TelemetryClient::from_env();
    match client
        .log_attribute_values(LogAttributeValues {
            project_id: &input.project_id,
            key: &input.key,
            start: &input.start,
            end: input.end.as_deref(),
            attribute_equals: &attribute_equals,
        })
        .await
    {
        Ok(values) => Output::Ok { values },
        Err(e) => classify(e),
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
        tracing::error!("project_log_attribute_values: {error}");
        Output::InternalError
    }
}
