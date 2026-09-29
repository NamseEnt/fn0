use axum::http::StatusCode;

/// Internal runtime failures shared by Loki and Tempo execution paths.
/// Protocol handlers convert this once at the HTTP boundary.
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum RuntimeError {
    Client(String),
    Limit(String),
    Timeout(String),
    Remote(String),
    Internal(String),
}

impl RuntimeError {
    pub(crate) fn into_http(self) -> (StatusCode, String) {
        match self {
            Self::Client(message) => (StatusCode::BAD_REQUEST, message),
            Self::Limit(message) => (StatusCode::PAYLOAD_TOO_LARGE, message),
            Self::Timeout(message) => (StatusCode::GATEWAY_TIMEOUT, message),
            Self::Remote(message) | Self::Internal(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message)
            }
        }
    }
}
