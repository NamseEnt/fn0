use crate::Body;
use base64::Engine;
use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Empty};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

const MESSAGE_KIND_HEADER: &str = "x-fn0-websocket-message-kind";
const DELIVERY_STATE_HEADER: &str = "x-fn0-websocket-delivery-state";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSocketMessageKind {
    Text,
    Binary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSocketDeliveryState {
    NotSent,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WebSocketCommandErrorKind {
    ConnectionNotFound,
    Backpressure,
    DeadlineExceeded,
    Transport,
    InvalidText,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebSocketCommandError {
    pub kind: WebSocketCommandErrorKind,
    pub delivery: WebSocketDeliveryState,
}

impl WebSocketCommandError {
    pub fn not_sent(kind: WebSocketCommandErrorKind) -> Self {
        Self {
            kind,
            delivery: WebSocketDeliveryState::NotSent,
        }
    }

    pub fn unknown(kind: WebSocketCommandErrorKind) -> Self {
        Self {
            kind,
            delivery: WebSocketDeliveryState::Unknown,
        }
    }
}

pub type WebSocketCommandFuture =
    Pin<Box<dyn Future<Output = Result<(), WebSocketCommandError>> + Send + 'static>>;

pub trait WebSocketCommandDispatcher: Send + Sync {
    fn send(
        &self,
        caller_project_id: String,
        connection_id: String,
        message_kind: WebSocketMessageKind,
        body: Body,
        remaining: std::time::Duration,
    ) -> WebSocketCommandFuture;

    fn disconnect(
        &self,
        caller_project_id: String,
        connection_id: String,
        remaining: std::time::Duration,
    ) -> WebSocketCommandFuture;
}

#[derive(Clone)]
pub struct WebSocketHijack {
    placeholder_host: String,
    dispatcher: Arc<OnceLock<Arc<dyn WebSocketCommandDispatcher>>>,
}

impl WebSocketHijack {
    pub fn new(placeholder_host: String) -> Self {
        Self {
            placeholder_host,
            dispatcher: Arc::new(OnceLock::new()),
        }
    }

    pub fn from_env() -> Self {
        let placeholder_host = std::env::var("FN0_WEBSOCKET_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-websocket.fn0.dev".to_string());
        Self::new(placeholder_host)
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub fn set_dispatcher(&self, dispatcher: Arc<dyn WebSocketCommandDispatcher>) {
        if self.dispatcher.set(dispatcher).is_err() {
            panic!("WebSocketHijack dispatcher already set");
        }
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host()
            .is_some_and(|host| host.eq_ignore_ascii_case(&self.placeholder_host))
    }

    pub(crate) async fn handle_command(
        &self,
        caller_project_id: &str,
        request: hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>,
        remaining: std::time::Duration,
    ) -> Result<hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
        if request.method() != hyper::Method::POST {
            return response(405, WebSocketDeliveryState::NotSent);
        }
        let Some((command, connection_id)) = command_and_connection(request.uri().path()) else {
            return response(404, WebSocketDeliveryState::NotSent);
        };
        let command = command.to_string();
        let connection_id = connection_id.to_string();
        if !valid_connection_id(&connection_id) {
            return response(404, WebSocketDeliveryState::NotSent);
        }
        let Some(dispatcher) = self.dispatcher.get() else {
            return response(503, WebSocketDeliveryState::NotSent);
        };

        let result = match command.as_str() {
            "send" => {
                let message_kind = match request
                    .headers()
                    .get(MESSAGE_KIND_HEADER)
                    .and_then(|value| value.to_str().ok())
                {
                    Some("text") => WebSocketMessageKind::Text,
                    Some("binary") => WebSocketMessageKind::Binary,
                    _ => return response(400, WebSocketDeliveryState::NotSent),
                };
                let body = request
                    .into_body()
                    .map_err(|error| anyhow::anyhow!("websocket body: {error:?}"))
                    .boxed_unsync();
                dispatcher
                    .send(
                        caller_project_id.to_string(),
                        connection_id.clone(),
                        message_kind,
                        body,
                        remaining,
                    )
                    .await
            }
            "disconnect" => {
                dispatcher
                    .disconnect(
                        caller_project_id.to_string(),
                        connection_id.clone(),
                        remaining,
                    )
                    .await
            }
            _ => return response(404, WebSocketDeliveryState::NotSent),
        };

        match result {
            Ok(()) => response(204, WebSocketDeliveryState::NotSent),
            Err(error)
                if command == "disconnect"
                    && error.kind == WebSocketCommandErrorKind::ConnectionNotFound =>
            {
                response(204, WebSocketDeliveryState::NotSent)
            }
            Err(error) => response(status_for(error.kind), error.delivery),
        }
    }
}

fn command_and_connection(path: &str) -> Option<(&str, &str)> {
    let mut segments = path.trim_start_matches('/').split('/');
    let command = segments.next()?;
    let connection_id = segments.next()?;
    if connection_id.is_empty() || segments.next().is_some() {
        return None;
    }
    Some((command, connection_id))
}

fn valid_connection_id(connection_id: &str) -> bool {
    let Some(encoded) = connection_id.strip_prefix("v1.") else {
        return false;
    };
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .is_ok_and(|decoded| decoded.len() == 32)
}

fn status_for(kind: WebSocketCommandErrorKind) -> u16 {
    match kind {
        WebSocketCommandErrorKind::ConnectionNotFound => 404,
        WebSocketCommandErrorKind::Backpressure => 429,
        WebSocketCommandErrorKind::DeadlineExceeded => 504,
        WebSocketCommandErrorKind::Transport => 503,
        WebSocketCommandErrorKind::InvalidText => 422,
        WebSocketCommandErrorKind::Internal => 500,
    }
}

fn response(
    status: u16,
    delivery: WebSocketDeliveryState,
) -> Result<hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
    let delivery_value = match delivery {
        WebSocketDeliveryState::NotSent => "not-sent",
        WebSocketDeliveryState::Unknown => "unknown",
    };
    let body = Empty::<Bytes>::new()
        .map_err(|never: std::convert::Infallible| match never {})
        .boxed_unsync();
    hyper::Response::builder()
        .status(status)
        .header(DELIVERY_STATE_HEADER, delivery_value)
        .body(body)
        .map_err(|error| ErrorCode::InternalError(Some(error.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::Full;
    use std::sync::Mutex;

    struct RecordingDispatcher {
        body: Arc<Mutex<Vec<u8>>>,
    }

    impl WebSocketCommandDispatcher for RecordingDispatcher {
        fn send(
            &self,
            caller_project_id: String,
            connection_id: String,
            message_kind: WebSocketMessageKind,
            body: Body,
            remaining: std::time::Duration,
        ) -> WebSocketCommandFuture {
            let recorded_body = self.body.clone();
            Box::pin(async move {
                assert_eq!(caller_project_id, "project");
                assert!(valid_connection_id(&connection_id));
                assert_eq!(message_kind, WebSocketMessageKind::Text);
                assert!(remaining <= std::time::Duration::from_secs(15));
                let bytes = body
                    .collect()
                    .await
                    .map_err(|_| {
                        WebSocketCommandError::unknown(WebSocketCommandErrorKind::Internal)
                    })?
                    .to_bytes();
                *recorded_body.lock().expect("recorded body lock") = bytes.to_vec();
                Ok(())
            })
        }

        fn disconnect(
            &self,
            _caller_project_id: String,
            _connection_id: String,
            _remaining: std::time::Duration,
        ) -> WebSocketCommandFuture {
            Box::pin(async { Ok(()) })
        }
    }

    #[test]
    fn connection_id_requires_version_and_random_bytes() {
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([7_u8; 32]);
        assert!(valid_connection_id(&format!("v1.{encoded}")));
        assert!(!valid_connection_id("v1.short"));
        assert!(!valid_connection_id(&encoded));
    }

    #[test]
    fn command_path_has_exactly_two_segments() {
        assert_eq!(
            command_and_connection("/send/v1.value"),
            Some(("send", "v1.value"))
        );
        assert_eq!(command_and_connection("/send/v1.value/extra"), None);
    }

    #[tokio::test]
    async fn send_stream_reaches_dispatcher() {
        let recorded_body = Arc::new(Mutex::new(Vec::new()));
        let hijack = WebSocketHijack::new("fn0-websocket.test".to_string());
        hijack.set_dispatcher(Arc::new(RecordingDispatcher {
            body: recorded_body.clone(),
        }));
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([9_u8; 32]);
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(format!("http://fn0-websocket.test/send/v1.{encoded}"))
            .header(MESSAGE_KIND_HEADER, "text")
            .body(
                Full::new(Bytes::from_static(b"hello"))
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
            )
            .expect("request");
        let response = hijack
            .handle_command("project", request, std::time::Duration::from_secs(15))
            .await
            .expect("response");
        assert_eq!(response.status(), hyper::StatusCode::NO_CONTENT);
        assert_eq!(
            recorded_body.lock().expect("recorded body lock").as_slice(),
            b"hello"
        );
    }
}
