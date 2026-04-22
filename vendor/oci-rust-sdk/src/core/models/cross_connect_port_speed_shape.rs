use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An individual port speed level for cross-connects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossConnectPortSpeedShape {
    /// The name of the port speed shape. <p> Example: {@code 10 Gbps}
    pub name: String,

    /// The port speed in Gbps. <p> Example: {@code 10} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port_speed_in_gbps: i64,
}

/// Required fields for CrossConnectPortSpeedShape
pub struct CrossConnectPortSpeedShapeRequired {
    /// The name of the port speed shape. <p> Example: {@code 10 Gbps}
    pub name: String,

    /// The port speed in Gbps. <p> Example: {@code 10} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub port_speed_in_gbps: i64,
}

impl CrossConnectPortSpeedShape {
    /// Create a new CrossConnectPortSpeedShape with required fields
    pub fn new(required: CrossConnectPortSpeedShapeRequired) -> Self {
        Self {
            name: required.name,

            port_speed_in_gbps: required.port_speed_in_gbps,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set port_speed_in_gbps
    pub fn set_port_speed_in_gbps(mut self, value: i64) -> Self {
        self.port_speed_in_gbps = value;
        self
    }
}
