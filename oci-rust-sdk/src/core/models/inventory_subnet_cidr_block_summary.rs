use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Lists CIDRs and utilization within a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySubnetCidrBlockSummary {
    /// The CIDR Prefix within a VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_cidr_block: Option<String>,

    /// The CIDR utilization of a VCN. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utilization: Option<i64>,
}

impl InventorySubnetCidrBlockSummary {
    /// Create a new InventorySubnetCidrBlockSummary
    pub fn new() -> Self {
        Self {
            ip_cidr_block: None,

            utilization: None,
        }
    }

    /// Set ip_cidr_block
    pub fn set_ip_cidr_block(mut self, value: Option<String>) -> Self {
        self.ip_cidr_block = value;
        self
    }

    /// Set utilization
    pub fn set_utilization(mut self, value: Option<i64>) -> Self {
        self.utilization = value;
        self
    }

    /// Set ip_cidr_block (unwraps Option)
    pub fn with_ip_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.ip_cidr_block = Some(value.into());
        self
    }

    /// Set utilization (unwraps Option)
    pub fn with_utilization(mut self, value: i64) -> Self {
        self.utilization = Some(value);
        self
    }
}

impl Default for InventorySubnetCidrBlockSummary {
    fn default() -> Self {
        Self::new()
    }
}
