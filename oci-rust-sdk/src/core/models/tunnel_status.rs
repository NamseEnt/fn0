use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Deprecated. For tunnel information, instead see {@link IPSecConnectionTunnel}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    /// The IP address of Oracle's VPN headend. <p> Example: {@code 203.0.113.50}
    pub ip_address: String,

    /// The tunnel's current state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<TunnelStatusLifecycleState>,

    /// The date and time the IPSec connection was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// When the state of the tunnel last changed, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_state_modified: Option<DateTime<Utc>>,
}

/// Required fields for TunnelStatus
pub struct TunnelStatusRequired {
    /// The IP address of Oracle's VPN headend. <p> Example: {@code 203.0.113.50}
    pub ip_address: String,
}

impl TunnelStatus {
    /// Create a new TunnelStatus with required fields
    pub fn new(required: TunnelStatusRequired) -> Self {
        Self {
            ip_address: required.ip_address,

            lifecycle_state: None,

            time_created: None,

            time_state_modified: None,
        }
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: String) -> Self {
        self.ip_address = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<TunnelStatusLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_state_modified
    pub fn set_time_state_modified(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_state_modified = value;
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: TunnelStatusLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set time_state_modified (unwraps Option)
    pub fn with_time_state_modified(mut self, value: DateTime<Utc>) -> Self {
        self.time_state_modified = Some(value);
        self
    }
}
