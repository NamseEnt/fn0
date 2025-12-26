use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Deprecated. For tunnel information, instead see: <p> {@link IPSecConnectionTunnel} * {@link IPSecConnectionTunnelSharedSecret}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    /// The IP address of Oracle's VPN headend. <p> Example: {@code 203.0.113.50 }
    pub ip_address: String,

    /// The shared secret of the IPSec tunnel.
    pub shared_secret: String,

    /// The date and time the IPSec connection was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

/// Required fields for TunnelConfig
pub struct TunnelConfigRequired {
    /// The IP address of Oracle's VPN headend. <p> Example: {@code 203.0.113.50 }
    pub ip_address: String,

    /// The shared secret of the IPSec tunnel.
    pub shared_secret: String,
}

impl TunnelConfig {
    /// Create a new TunnelConfig with required fields
    pub fn new(required: TunnelConfigRequired) -> Self {
        Self {
            ip_address: required.ip_address,

            shared_secret: required.shared_secret,

            time_created: None,
        }
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: String) -> Self {
        self.ip_address = value;
        self
    }

    /// Set shared_secret
    pub fn set_shared_secret(mut self, value: String) -> Self {
        self.shared_secret = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}
