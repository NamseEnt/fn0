use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the VCN overlap details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInventoryVcnOverlapSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN .
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlapping_vcn_id: Option<String>,

    /// Name of the overlapping VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlapping_vcn_name: Option<String>,

    /// The overlapping CIDR prefix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlapping_cidr: Option<String>,

    /// CIDR prefix of the VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr: Option<String>,
}

impl IpInventoryVcnOverlapSummary {
    /// Create a new IpInventoryVcnOverlapSummary
    pub fn new() -> Self {
        Self {
            overlapping_vcn_id: None,

            overlapping_vcn_name: None,

            overlapping_cidr: None,

            cidr: None,
        }
    }

    /// Set overlapping_vcn_id
    pub fn set_overlapping_vcn_id(mut self, value: Option<String>) -> Self {
        self.overlapping_vcn_id = value;
        self
    }

    /// Set overlapping_vcn_name
    pub fn set_overlapping_vcn_name(mut self, value: Option<String>) -> Self {
        self.overlapping_vcn_name = value;
        self
    }

    /// Set overlapping_cidr
    pub fn set_overlapping_cidr(mut self, value: Option<String>) -> Self {
        self.overlapping_cidr = value;
        self
    }

    /// Set cidr
    pub fn set_cidr(mut self, value: Option<String>) -> Self {
        self.cidr = value;
        self
    }

    /// Set overlapping_vcn_id (unwraps Option)
    pub fn with_overlapping_vcn_id(mut self, value: impl Into<String>) -> Self {
        self.overlapping_vcn_id = Some(value.into());
        self
    }

    /// Set overlapping_vcn_name (unwraps Option)
    pub fn with_overlapping_vcn_name(mut self, value: impl Into<String>) -> Self {
        self.overlapping_vcn_name = Some(value.into());
        self
    }

    /// Set overlapping_cidr (unwraps Option)
    pub fn with_overlapping_cidr(mut self, value: impl Into<String>) -> Self {
        self.overlapping_cidr = Some(value.into());
        self
    }

    /// Set cidr (unwraps Option)
    pub fn with_cidr(mut self, value: impl Into<String>) -> Self {
        self.cidr = Some(value.into());
        self
    }
}

impl Default for IpInventoryVcnOverlapSummary {
    fn default() -> Self {
        Self::new()
    }
}
