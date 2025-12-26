use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Container Health Check with TCP type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerTcpHealthCheck {
    /// Container health check port. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    pub health_check_type: String,
}

/// Required fields for ContainerTcpHealthCheck
pub struct ContainerTcpHealthCheckRequired {
    /// Container health check port. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port: i64,

    pub health_check_type: String,
}

impl ContainerTcpHealthCheck {
    /// Create a new ContainerTcpHealthCheck with required fields
    pub fn new(required: ContainerTcpHealthCheckRequired) -> Self {
        Self {
            port: required.port,

            health_check_type: required.health_check_type,
        }
    }

    /// Set port
    pub fn set_port(mut self, value: i64) -> Self {
        self.port = value;
        self
    }

    /// Set health_check_type
    pub fn set_health_check_type(mut self, value: String) -> Self {
        self.health_check_type = value;
        self
    }
}
