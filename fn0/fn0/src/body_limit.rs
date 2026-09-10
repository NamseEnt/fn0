use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::HeaderMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

pub(crate) const QUEUE_BODY_LIMIT: usize = 128 * 1024;
pub(crate) const VAULT_BODY_LIMIT: usize = 16 * 1024;
pub(crate) const STATIC_PAGE_CACHE_BODY_LIMIT: usize = 256 * 1024;
pub(crate) const SINGLETON_CONNECT_BODY_LIMIT: usize = 256 * 1024;
pub(crate) const SINGLETON_ACTIVATION_BODY_LIMIT: usize = 16 * 1024;
pub(crate) const OTLP_BODY_LIMIT: usize = 8 * 1024 * 1024;
const AGGREGATE_BODY_BUDGET_CHUNK_SIZE: usize = 64 * 1024;
const AGGREGATE_BODY_BUDGET_SIZE: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub(crate) enum BodyLimitError {
    TooLarge,
    Body(ErrorCode),
}

pub(crate) struct LimitedBody {
    pub(crate) bytes: Bytes,
    pub(crate) permit: OwnedSemaphorePermit,
}

pub(crate) fn declared_content_length_exceeds_limit(headers: &HeaderMap, limit: usize) -> bool {
    headers
        .get(hyper::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > limit as u64)
}

pub(crate) async fn collect_body_limited(
    mut body: UnsyncBoxBody<Bytes, ErrorCode>,
    limit: usize,
) -> Result<LimitedBody, BodyLimitError> {
    let permit_count = limit.div_ceil(AGGREGATE_BODY_BUDGET_CHUNK_SIZE) as u32;
    let permit = aggregate_body_budget()
        .acquire_many_owned(permit_count)
        .await
        .map_err(|error| {
            BodyLimitError::Body(ErrorCode::InternalError(Some(format!(
                "request body budget unavailable: {error}"
            ))))
        })?;
    let mut collected = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(BodyLimitError::Body)?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        if data.len() > limit.saturating_sub(collected.len()) {
            return Err(BodyLimitError::TooLarge);
        }
        collected.try_reserve_exact(data.len()).map_err(|error| {
            BodyLimitError::Body(ErrorCode::InternalError(Some(format!(
                "request body buffer allocation failed: {error}"
            ))))
        })?;
        collected.extend_from_slice(data);
    }
    Ok(LimitedBody {
        bytes: Bytes::from(collected),
        permit,
    })
}

fn aggregate_body_budget() -> Arc<Semaphore> {
    static BUDGET: OnceLock<Arc<Semaphore>> = OnceLock::new();
    BUDGET
        .get_or_init(|| {
            Arc::new(Semaphore::new(
                AGGREGATE_BODY_BUDGET_SIZE / AGGREGATE_BODY_BUDGET_CHUNK_SIZE,
            ))
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use http_body::Frame;
    use http_body_util::{BodyExt, StreamBody};

    fn body(chunks: Vec<Bytes>) -> UnsyncBoxBody<Bytes, ErrorCode> {
        StreamBody::new(stream::iter(
            chunks
                .into_iter()
                .map(|chunk| Ok::<_, std::convert::Infallible>(Frame::data(chunk))),
        ))
        .map_err(|never| match never {})
        .boxed_unsync()
    }

    #[tokio::test]
    async fn accepts_body_at_limit_across_chunks() {
        let result = collect_body_limited(
            body(vec![Bytes::from_static(b"abc"), Bytes::from_static(b"def")]),
            6,
        )
        .await
        .expect("body within limit");
        assert_eq!(result.bytes, Bytes::from_static(b"abcdef"));
    }

    #[tokio::test]
    async fn refuses_body_after_crossing_limit() {
        let result = collect_body_limited(
            body(vec![Bytes::from_static(b"abc"), Bytes::from_static(b"def")]),
            5,
        )
        .await;
        assert!(matches!(result, Err(BodyLimitError::TooLarge)));
    }

    #[test]
    fn refuses_declared_length_above_limit() {
        let headers = HeaderMap::from_iter([(hyper::header::CONTENT_LENGTH, "7".parse().unwrap())]);
        assert!(declared_content_length_exceeds_limit(&headers, 6));
        assert!(!declared_content_length_exceeds_limit(&headers, 7));
    }
}
