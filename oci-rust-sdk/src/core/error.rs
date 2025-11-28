/// Result type alias for OCI operations
pub type Result<T> = std::result::Result<T, OciError>;

/// Main error type for OCI SDK operations
#[derive(Debug, thiserror::Error)]
pub enum OciError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Service returned an error
    #[error("Service error {status}: {message}")]
    ServiceError {
        status: u16,
        code: String,
        message: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid region
    #[error("Invalid region: {0}")]
    InvalidRegion(String),

    /// RSA signing error
    #[error("RSA signing error: {0}")]
    SigningError(String),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

impl OciError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            // 429 Too Many Requests - retryable
            OciError::ServiceError { status: 429, .. } => true,
            // 5xx Server Errors - retryable
            OciError::ServiceError { status, .. } if *status >= 500 => true,
            // Network errors - retryable
            OciError::HttpError(_) => true,
            // Everything else is not retryable
            _ => false,
        }
    }

    /// Create a service error from response
    pub fn from_response(status: u16, code: String, message: String) -> Self {
        OciError::ServiceError {
            status,
            code,
            message,
        }
    }
}

/// Error response from OCI services
#[derive(Debug, serde::Deserialize)]
pub struct ServiceErrorResponse {
    pub code: String,
    pub message: String,
}
