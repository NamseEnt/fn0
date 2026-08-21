use crate::Body;
use base64::Engine;
use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

const MESSAGE_KIND_HEADER: &str = "x-fn0-websocket-message-kind";
const DELIVERY_STATE_HEADER: &str = "x-fn0-websocket-delivery-state";
const CONNECT_URL_HEADER: &str = "x-fn0-websocket-connect-url";
const CONNECT_PATH_HEADER: &str = "x-fn0-websocket-receive-path";
const SINGLETON_PROJECT_HEADER: &str = "x-fn0-websocket-singleton-project";
const SINGLETON_ID_HEADER: &str = "x-fn0-websocket-singleton-id";
const SINGLETON_ROUTE_HEADER: &str = "x-fn0-websocket-singleton-route";

#[derive(serde::Deserialize)]
struct SingletonConnectInput {
    url: String,
    headers: Vec<(String, String)>,
    protocols: Vec<String>,
    claim_token: String,
    initial_lease_deadline: i64,
}

#[derive(serde::Deserialize)]
struct SingletonActivationInput {
    claim_token: String,
    connection_id: String,
}

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
pub type WebSocketConnectFuture =
    Pin<Box<dyn Future<Output = Result<String, WebSocketCommandError>> + Send + 'static>>;

pub trait WebSocketCommandDispatcher: Send + Sync {
    fn connect(
        &self,
        caller_project_id: String,
        url: String,
        receive_path: String,
        remaining: std::time::Duration,
    ) -> WebSocketConnectFuture {
        let _ = (caller_project_id, url, receive_path, remaining);
        Box::pin(async {
            Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ))
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn connect_singleton(
        &self,
        project_id: String,
        singleton_id: String,
        url: String,
        route_path: String,
        headers: Vec<(String, String)>,
        protocols: Vec<String>,
        claim_token: String,
        initial_lease_deadline: i64,
        remaining: std::time::Duration,
    ) -> WebSocketConnectFuture {
        let _ = (
            project_id,
            singleton_id,
            url,
            route_path,
            headers,
            protocols,
            claim_token,
            initial_lease_deadline,
            remaining,
        );
        Box::pin(async {
            Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ))
        })
    }

    fn activate_singleton(
        &self,
        project_id: String,
        singleton_id: String,
        claim_token: String,
        connection_id: String,
        remaining: std::time::Duration,
    ) -> WebSocketCommandFuture {
        let _ = (
            project_id,
            singleton_id,
            claim_token,
            connection_id,
            remaining,
        );
        Box::pin(async {
            Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ))
        })
    }

    fn abort_singleton(
        &self,
        project_id: String,
        singleton_id: String,
        claim_token: String,
        connection_id: String,
        remaining: std::time::Duration,
    ) -> WebSocketCommandFuture {
        let _ = (
            project_id,
            singleton_id,
            claim_token,
            connection_id,
            remaining,
        );
        Box::pin(async {
            Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ))
        })
    }

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
    control_project_id: String,
    dispatcher: Arc<OnceLock<Arc<dyn WebSocketCommandDispatcher>>>,
}

impl WebSocketHijack {
    pub fn new(placeholder_host: String) -> Self {
        Self {
            placeholder_host,
            control_project_id: "fn0-control".to_string(),
            dispatcher: Arc::new(OnceLock::new()),
        }
    }

    pub fn from_env() -> Self {
        let placeholder_host = std::env::var("FN0_WEBSOCKET_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-websocket.fn0.dev".to_string());
        let control_project_id =
            std::env::var("FN0_CONTROL_PROJECT_ID").unwrap_or_else(|_| "fn0-control".to_string());
        Self {
            placeholder_host,
            control_project_id,
            dispatcher: Arc::new(OnceLock::new()),
        }
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
        if request.uri().path() == "/connect" {
            let Some(url) = request
                .headers()
                .get(CONNECT_URL_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
            else {
                return response(400, WebSocketDeliveryState::NotSent);
            };
            let Some(receive_path) = request
                .headers()
                .get(CONNECT_PATH_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
            else {
                return response(400, WebSocketDeliveryState::NotSent);
            };
            if !valid_receive_path(&receive_path) {
                return response(400, WebSocketDeliveryState::NotSent);
            }
            let Some(dispatcher) = self.dispatcher.get() else {
                return response(503, WebSocketDeliveryState::NotSent);
            };
            return match dispatcher
                .connect(caller_project_id.to_string(), url, receive_path, remaining)
                .await
            {
                Ok(connection_id) => response_with_body(
                    201,
                    WebSocketDeliveryState::NotSent,
                    Bytes::from(connection_id),
                ),
                Err(error) => response(status_for(error.kind), error.delivery),
            };
        }
        if request.uri().path() == "/connect-singleton" {
            if caller_project_id != self.control_project_id {
                return response(403, WebSocketDeliveryState::NotSent);
            }
            let Some(project_id) = request
                .headers()
                .get(SINGLETON_PROJECT_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
            else {
                return response(400, WebSocketDeliveryState::NotSent);
            };
            let Some(singleton_id) = request
                .headers()
                .get(SINGLETON_ID_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
            else {
                return response(400, WebSocketDeliveryState::NotSent);
            };
            let Some(route_path) = request
                .headers()
                .get(SINGLETON_ROUTE_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
            else {
                return response(400, WebSocketDeliveryState::NotSent);
            };
            if project_id.is_empty()
                || singleton_id.is_empty()
                || !valid_singleton_route(&route_path)
            {
                return response(400, WebSocketDeliveryState::NotSent);
            }
            let body = match request.into_body().collect().await {
                Ok(body) => body.to_bytes(),
                Err(_) => return response(400, WebSocketDeliveryState::NotSent),
            };
            let input: SingletonConnectInput = match serde_json::from_slice(&body) {
                Ok(input) => input,
                Err(_) => return response(400, WebSocketDeliveryState::NotSent),
            };
            if input
                .headers
                .iter()
                .any(|(header_name, _)| singleton_system_header(header_name))
                || input.protocols.iter().any(|protocol| {
                    protocol.is_empty()
                        || protocol.contains(',')
                        || protocol.chars().any(char::is_whitespace)
                })
            {
                return response(400, WebSocketDeliveryState::NotSent);
            }
            let Some(dispatcher) = self.dispatcher.get() else {
                return response(503, WebSocketDeliveryState::NotSent);
            };
            return match dispatcher
                .connect_singleton(
                    project_id,
                    singleton_id,
                    input.url,
                    route_path,
                    input.headers,
                    input.protocols,
                    input.claim_token,
                    input.initial_lease_deadline,
                    remaining,
                )
                .await
            {
                Ok(connection_id) => response_with_body(
                    201,
                    WebSocketDeliveryState::NotSent,
                    Bytes::from(connection_id),
                ),
                Err(error) => response(status_for(error.kind), error.delivery),
            };
        }
        if matches!(
            request.uri().path(),
            "/activate-singleton" | "/abort-singleton"
        ) {
            let activate = request.uri().path() == "/activate-singleton";
            if caller_project_id != self.control_project_id {
                return response(403, WebSocketDeliveryState::NotSent);
            }
            let Some(project_id) = request
                .headers()
                .get(SINGLETON_PROJECT_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
            else {
                return response(400, WebSocketDeliveryState::NotSent);
            };
            let Some(singleton_id) = request
                .headers()
                .get(SINGLETON_ID_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
            else {
                return response(400, WebSocketDeliveryState::NotSent);
            };
            let body = match request.into_body().collect().await {
                Ok(body) => body.to_bytes(),
                Err(_) => return response(400, WebSocketDeliveryState::NotSent),
            };
            let input: SingletonActivationInput = match serde_json::from_slice(&body) {
                Ok(input) => input,
                Err(_) => return response(400, WebSocketDeliveryState::NotSent),
            };
            if project_id.is_empty()
                || singleton_id.is_empty()
                || input.claim_token.is_empty()
                || !valid_connection_id(&input.connection_id)
            {
                return response(400, WebSocketDeliveryState::NotSent);
            }
            let Some(dispatcher) = self.dispatcher.get() else {
                return response(503, WebSocketDeliveryState::NotSent);
            };
            let result = if activate {
                dispatcher
                    .activate_singleton(
                        project_id,
                        singleton_id,
                        input.claim_token,
                        input.connection_id,
                        remaining,
                    )
                    .await
            } else {
                dispatcher
                    .abort_singleton(
                        project_id,
                        singleton_id,
                        input.claim_token,
                        input.connection_id,
                        remaining,
                    )
                    .await
            };
            return match result {
                Ok(()) => response(204, WebSocketDeliveryState::NotSent),
                Err(error) => response(status_for(error.kind), error.delivery),
            };
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

fn valid_singleton_route(route_path: &str) -> bool {
    route_path.starts_with("/ws_singleton/")
        && !route_path.contains('?')
        && !route_path.split('/').any(|segment| segment == "..")
}

fn singleton_system_header(header_name: &str) -> bool {
    let header_name = header_name.to_ascii_lowercase();
    matches!(
        header_name.as_str(),
        "host"
            | "upgrade"
            | "connection"
            | "content-length"
            | "transfer-encoding"
            | "sec-websocket-key"
            | "sec-websocket-version"
            | "sec-websocket-extensions"
            | "sec-websocket-accept"
            | "sec-websocket-protocol"
    ) || header_name.starts_with("x-fn0-")
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

fn valid_receive_path(receive_path: &str) -> bool {
    (receive_path == "/ws_out" || receive_path.starts_with("/ws_out/"))
        && !receive_path.split('/').any(|segment| segment == "..")
        && !receive_path.contains('?')
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
    response_with_body(status, delivery, Bytes::new())
}

fn response_with_body(
    status: u16,
    delivery: WebSocketDeliveryState,
    body_bytes: Bytes,
) -> Result<hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
    let delivery_value = match delivery {
        WebSocketDeliveryState::NotSent => "not-sent",
        WebSocketDeliveryState::Unknown => "unknown",
    };
    let body = Full::new(body_bytes)
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

    struct LifecycleDispatcher {
        commands: Arc<Mutex<Vec<String>>>,
    }

    impl WebSocketCommandDispatcher for RecordingDispatcher {
        fn connect(
            &self,
            _caller_project_id: String,
            _url: String,
            _receive_path: String,
            _remaining: std::time::Duration,
        ) -> WebSocketConnectFuture {
            Box::pin(async { Ok("v1.test".to_string()) })
        }

        fn connect_singleton(
            &self,
            project_id: String,
            singleton_id: String,
            url: String,
            route_path: String,
            headers: Vec<(String, String)>,
            protocols: Vec<String>,
            claim_token: String,
            initial_lease_deadline: i64,
            _remaining: std::time::Duration,
        ) -> WebSocketConnectFuture {
            Box::pin(async move {
                assert_eq!(project_id, "target-project");
                assert_eq!(singleton_id, "market-feed");
                assert_eq!(url, "wss://example.com/socket");
                assert_eq!(route_path, "/ws_singleton/market-feed");
                assert_eq!(
                    headers,
                    vec![("authorization".to_string(), "secret".to_string())]
                );
                assert_eq!(protocols, vec!["graphql-ws".to_string()]);
                assert_eq!(claim_token, "claim-token");
                assert_eq!(initial_lease_deadline, 1234);
                Ok("v1.singleton".to_string())
            })
        }

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

    impl WebSocketCommandDispatcher for LifecycleDispatcher {
        fn activate_singleton(
            &self,
            project_id: String,
            singleton_id: String,
            claim_token: String,
            connection_id: String,
            _remaining: std::time::Duration,
        ) -> WebSocketCommandFuture {
            assert_eq!(project_id, "target-project");
            assert_eq!(singleton_id, "market-feed");
            assert_eq!(claim_token, "claim-token");
            assert!(valid_connection_id(&connection_id));
            let commands = self.commands.clone();
            Box::pin(async move {
                commands
                    .lock()
                    .expect("lifecycle commands lock")
                    .push("activate".to_string());
                Ok(())
            })
        }

        fn abort_singleton(
            &self,
            project_id: String,
            singleton_id: String,
            claim_token: String,
            connection_id: String,
            _remaining: std::time::Duration,
        ) -> WebSocketCommandFuture {
            assert_eq!(project_id, "target-project");
            assert_eq!(singleton_id, "market-feed");
            assert_eq!(claim_token, "claim-token");
            assert!(valid_connection_id(&connection_id));
            let commands = self.commands.clone();
            Box::pin(async move {
                commands
                    .lock()
                    .expect("lifecycle commands lock")
                    .push("abort".to_string());
                Ok(())
            })
        }

        fn send(
            &self,
            _caller_project_id: String,
            _connection_id: String,
            _message_kind: WebSocketMessageKind,
            _body: Body,
            _remaining: std::time::Duration,
        ) -> WebSocketCommandFuture {
            Box::pin(async { Ok(()) })
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

    #[tokio::test]
    async fn connect_returns_connection_id() {
        let hijack = WebSocketHijack::new("fn0-websocket.test".to_string());
        hijack.set_dispatcher(Arc::new(RecordingDispatcher {
            body: Arc::new(Mutex::new(Vec::new())),
        }));
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri("http://fn0-websocket.test/connect")
            .header(CONNECT_URL_HEADER, "wss://example.com/socket")
            .header(CONNECT_PATH_HEADER, "/ws_out/slack")
            .body(
                Full::new(Bytes::new())
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
            )
            .expect("request");
        let response = hijack
            .handle_command("project", request, std::time::Duration::from_secs(15))
            .await
            .expect("response");
        assert_eq!(response.status(), hyper::StatusCode::CREATED);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes();
        assert_eq!(body, Bytes::from_static(b"v1.test"));
    }

    #[tokio::test]
    async fn singleton_connect_is_control_only() {
        let hijack = WebSocketHijack::new("fn0-websocket.test".to_string());
        hijack.set_dispatcher(Arc::new(RecordingDispatcher {
            body: Arc::new(Mutex::new(Vec::new())),
        }));
        let request = || {
            hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri("http://fn0-websocket.test/connect-singleton")
                .header(SINGLETON_PROJECT_HEADER, "target-project")
                .header(SINGLETON_ID_HEADER, "market-feed")
                .header(SINGLETON_ROUTE_HEADER, "/ws_singleton/market-feed")
                .body(
                    Full::new(Bytes::from_static(
                        br#"{"url":"wss://example.com/socket","headers":[["authorization","secret"]],"protocols":["graphql-ws"],"claim_token":"claim-token","initial_lease_deadline":1234}"#,
                    ))
                        .map_err(|never: std::convert::Infallible| match never {})
                        .boxed_unsync(),
                )
                .expect("request")
        };

        let forbidden = hijack
            .handle_command(
                "other-project",
                request(),
                std::time::Duration::from_secs(15),
            )
            .await
            .expect("response");
        assert_eq!(forbidden.status(), hyper::StatusCode::FORBIDDEN);

        let connected = hijack
            .handle_command("fn0-control", request(), std::time::Duration::from_secs(15))
            .await
            .expect("response");
        assert_eq!(connected.status(), hyper::StatusCode::CREATED);
        let body = connected
            .into_body()
            .collect()
            .await
            .expect("response body")
            .to_bytes();
        assert_eq!(body, Bytes::from_static(b"v1.singleton"));
    }

    #[tokio::test]
    async fn singleton_lifecycle_commands_are_control_only() {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let hijack = WebSocketHijack::new("fn0-websocket.test".to_string());
        hijack.set_dispatcher(Arc::new(LifecycleDispatcher {
            commands: commands.clone(),
        }));
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([4_u8; 32]);
        let request = |command: &str| {
            hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri(format!("http://fn0-websocket.test/{command}-singleton"))
                .header(SINGLETON_PROJECT_HEADER, "target-project")
                .header(SINGLETON_ID_HEADER, "market-feed")
                .body(
                    Full::new(Bytes::from(format!(
                        r#"{{"claim_token":"claim-token","connection_id":"v1.{encoded}"}}"#
                    )))
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
                )
                .expect("request")
        };

        let forbidden = hijack
            .handle_command(
                "other-project",
                request("activate"),
                std::time::Duration::from_secs(15),
            )
            .await
            .expect("response");
        assert_eq!(forbidden.status(), hyper::StatusCode::FORBIDDEN);

        for command in ["activate", "abort"] {
            let response = hijack
                .handle_command(
                    "fn0-control",
                    request(command),
                    std::time::Duration::from_secs(15),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), hyper::StatusCode::NO_CONTENT);
        }
        assert_eq!(
            commands.lock().expect("lifecycle commands lock").as_slice(),
            ["activate", "abort"]
        );
    }
}
