use bytes::Bytes;
use deno_core::anyhow;
use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::BodyExt;
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::source_map;
use crate::{FetchHandler, Response, SourceMapInfo};

#[derive(Serialize)]
pub struct FetchInterceptResult {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct FetchHandlerHolder(pub Option<Arc<dyn FetchHandler>>);

#[op2]
#[string]
pub fn op_rewrite_stack_trace(state: &mut OpState, #[string] stack: String) -> String {
    let Some(info) = state.try_borrow::<SourceMapInfo>() else {
        return stack;
    };
    source_map::rewrite_stack_trace(&stack, &info.script_url, &info.source_map_json)
}

#[op2(async)]
#[serde]
pub async fn op_fetch_intercept(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[string] method: String,
    #[serde] headers: Vec<(String, String)>,
    #[serde] body: Option<Vec<u8>>,
) -> Result<Option<FetchInterceptResult>, JsErrorBox> {
    let handler = {
        let state = state.borrow();
        state
            .try_borrow::<FetchHandlerHolder>()
            .and_then(|h| h.0.clone())
    };

    let Some(handler) = handler else {
        return Ok(None);
    };

    let mut request_builder = http::Request::builder()
        .method(method.as_str())
        .uri(&url);

    for (key, value) in &headers {
        request_builder = request_builder.header(key.as_str(), value.as_str());
    }

    let body: UnsyncBoxBody<Bytes, anyhow::Error> = match body {
        Some(bytes) => UnsyncBoxBody::new(
            http_body_util::Full::new(Bytes::from(bytes)).map_err(|never| match never {}),
        ),
        None => UnsyncBoxBody::new(
            http_body_util::Empty::new().map_err(|never| match never {}),
        ),
    };

    let request = request_builder
        .body(body)
        .map_err(|e: http::Error| JsErrorBox::generic(e.to_string()))?;

    let response: Option<Response> = handler.handle(request).await;

    let Some(response) = response else {
        return Ok(None);
    };

    let status = response.status().as_u16();
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v): (&http::HeaderName, &http::HeaderValue)| {
            (k.to_string(), v.to_str().unwrap_or("").to_string())
        })
        .collect();

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|e: anyhow::Error| JsErrorBox::generic(e.to_string()))?
        .to_bytes()
        .to_vec();

    Ok(Some(FetchInterceptResult {
        status,
        headers,
        body: body_bytes,
    }))
}

deno_core::extension!(
    fetch_intercept_extension,
    ops = [op_fetch_intercept, op_rewrite_stack_trace],
);
