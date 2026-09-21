use crate::body_limit::{BodyLimitError, collect_body_limited, declared_content_length_exceeds_limit};
use crate::{DibiBridge, DibiBridgeError};
use bytes::Bytes;
use dibi_protocol::MAX_FRAME_SIZE;
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use hyper::http::{Method, Request, Response, StatusCode, Uri, header};
use std::sync::Arc;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

pub struct DibiHijack {
    placeholder_host: String,
    bridge: Arc<DibiBridge>,
}

impl DibiHijack {
    pub fn new(placeholder_host: String, bridge: Arc<DibiBridge>) -> Self {
        Self {
            placeholder_host,
            bridge,
        }
    }

    pub fn from_env() -> Result<Option<Arc<Self>>, DibiBridgeError> {
        let Some(bridge) = DibiBridge::from_env()? else {
            return Ok(None);
        };
        let placeholder_host = bridge.placeholder_host().to_owned();
        Ok(Some(Arc::new(Self::new(placeholder_host, bridge))))
    }

    pub fn placeholder_host(&self) -> &str {
        &self.placeholder_host
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub fn matches(&self, uri: &Uri) -> bool {
        uri.host()
            .is_some_and(|host| host.eq_ignore_ascii_case(&self.placeholder_host))
    }

    pub async fn handle_http(
        &self,
        project_id: &str,
        request: Request<UnsyncBoxBody<Bytes, ErrorCode>>,
    ) -> Result<Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
        if request.method() != Method::POST {
            return response(StatusCode::METHOD_NOT_ALLOWED);
        }
        if request.uri().path() != "/" {
            return response(StatusCode::NOT_FOUND);
        }
        if request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            != Some("application/octet-stream")
        {
            return response(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
        if declared_content_length_exceeds_limit(request.headers(), MAX_FRAME_SIZE) {
            return response(StatusCode::PAYLOAD_TOO_LARGE);
        }
        let (_, body) = request.into_parts();
        let limited_body = match collect_body_limited(body, MAX_FRAME_SIZE).await {
            Ok(body) => body,
            Err(BodyLimitError::TooLarge) => return response(StatusCode::PAYLOAD_TOO_LARGE),
            Err(BodyLimitError::Body(error)) => return Err(error),
        };
        match self.bridge.request(project_id, &limited_body.bytes).await {
            Ok(frame) => binary_response(StatusCode::OK, frame),
            Err(error) => response(bridge_status(&error)),
        }
    }
}

fn bridge_status(error: &DibiBridgeError) -> StatusCode {
    match error {
        DibiBridgeError::InvalidRequest => StatusCode::BAD_REQUEST,
        DibiBridgeError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        DibiBridgeError::AuthenticationFailed
        | DibiBridgeError::Configuration(_)
        | DibiBridgeError::Unavailable(_)
        | DibiBridgeError::ResponseTooLarge
        | DibiBridgeError::Protocol(_) => StatusCode::BAD_GATEWAY,
    }
}

fn response(status: StatusCode) -> Result<Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
    Response::builder()
        .status(status)
        .body(
            Full::new(Bytes::new())
                .map_err(|never: std::convert::Infallible| match never {})
                .boxed_unsync(),
        )
        .map_err(|error| ErrorCode::InternalError(Some(error.to_string())))
}

fn binary_response(
    status: StatusCode,
    body: Vec<u8>,
) -> Result<Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(
            Full::new(Bytes::from(body))
                .map_err(|never: std::convert::Infallible| match never {})
                .boxed_unsync(),
        )
        .map_err(|error| ErrorCode::InternalError(Some(error.to_string())))
}
