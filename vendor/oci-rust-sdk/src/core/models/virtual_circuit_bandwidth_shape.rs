use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An individual bandwidth level for virtual circuits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualCircuitBandwidthShape {
    /// The name of the bandwidth shape. <p> Example: {@code 10 Gbps}
    pub name: String,

    /// The bandwidth in Mbps. <p> Example: {@code 10000} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_in_mbps: Option<i64>,
}

/// Required fields for VirtualCircuitBandwidthShape
pub struct VirtualCircuitBandwidthShapeRequired {
    /// The name of the bandwidth shape. <p> Example: {@code 10 Gbps}
    pub name: String,
}

impl VirtualCircuitBandwidthShape {
    /// Create a new VirtualCircuitBandwidthShape with required fields
    pub fn new(required: VirtualCircuitBandwidthShapeRequired) -> Self {
        Self {
            name: required.name,

            bandwidth_in_mbps: None,
        }
    }

    /// Set bandwidth_in_mbps
    pub fn set_bandwidth_in_mbps(mut self, value: Option<i64>) -> Self {
        self.bandwidth_in_mbps = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set bandwidth_in_mbps (unwraps Option)
    pub fn with_bandwidth_in_mbps(mut self, value: i64) -> Self {
        self.bandwidth_in_mbps = Some(value);
        self
    }
}
