use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Container Health Check HTTP type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerHttpHealthCheckDetails {
    /// Container health check HTTP path.
    pub path: String,

    /// Container health check HTTP port. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    pub health_check_type: String,

    /// Container health check HTTP headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HealthCheckHttpHeader>>,
}

/// Required fields for CreateContainerHttpHealthCheckDetails
pub struct CreateContainerHttpHealthCheckDetailsRequired {
    /// Container health check HTTP path.
    pub path: String,

    /// Container health check HTTP port. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    pub health_check_type: String,
}

impl CreateContainerHttpHealthCheckDetails {
    /// Create a new CreateContainerHttpHealthCheckDetails with required fields
    pub fn new(required: CreateContainerHttpHealthCheckDetailsRequired) -> Self {
        Self {
            path: required.path,

            port: required.port,

            health_check_type: required.health_check_type,

            headers: None,
        }
    }

    /// Set path
    pub fn set_path(mut self, value: String) -> Self {
        self.path = value;
        self
    }

    /// Set port
    pub fn set_port(mut self, value: i64) -> Self {
        self.port = value;
        self
    }

    /// Set headers
    pub fn set_headers(mut self, value: Option<Vec<HealthCheckHttpHeader>>) -> Self {
        self.headers = value;
        self
    }

    /// Set health_check_type
    pub fn set_health_check_type(mut self, value: String) -> Self {
        self.health_check_type = value;
        self
    }

    /// Set headers (unwraps Option)
    pub fn with_headers(mut self, value: Vec<HealthCheckHttpHeader>) -> Self {
        self.headers = Some(value);
        self
    }
}
