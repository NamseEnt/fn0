use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The CIDR utilization details of a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInventoryCidrUtilizationSummary {
    /// The CIDR range of a subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr: Option<String>,

    /// The CIDR utilisation of a subnet. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utilization: Option<i64>,

    /// Address type of the CIDR within a subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_type: Option<String>,
}

impl IpInventoryCidrUtilizationSummary {
    /// Create a new IpInventoryCidrUtilizationSummary
    pub fn new() -> Self {
        Self {
            cidr: None,

            utilization: None,

            address_type: None,
        }
    }

    /// Set cidr
    pub fn set_cidr(mut self, value: Option<String>) -> Self {
        self.cidr = value;
        self
    }

    /// Set utilization
    pub fn set_utilization(mut self, value: Option<i64>) -> Self {
        self.utilization = value;
        self
    }

    /// Set address_type
    pub fn set_address_type(mut self, value: Option<String>) -> Self {
        self.address_type = value;
        self
    }

    /// Set cidr (unwraps Option)
    pub fn with_cidr(mut self, value: impl Into<String>) -> Self {
        self.cidr = Some(value.into());
        self
    }

    /// Set utilization (unwraps Option)
    pub fn with_utilization(mut self, value: i64) -> Self {
        self.utilization = Some(value);
        self
    }

    /// Set address_type (unwraps Option)
    pub fn with_address_type(mut self, value: impl Into<String>) -> Self {
        self.address_type = Some(value.into());
        self
    }
}

impl Default for IpInventoryCidrUtilizationSummary {
    fn default() -> Self {
        Self::new()
    }
}
