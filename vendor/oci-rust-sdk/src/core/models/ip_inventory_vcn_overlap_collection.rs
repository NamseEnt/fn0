use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details of the overlapping VCNs and compartments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInventoryVcnOverlapCollection {
    /// The timestamp of the latest update from the database in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_timestamp: Option<DateTime<Utc>>,

    /// Lists {@code IpInventoryVcnOverlapSummary} object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_inventory_vcn_overlap_summary: Option<Vec<IpInventoryVcnOverlapSummary>>,

    /// Indicates the status of the data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// The overlap count for the given VCN and compartments. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlap_count: Option<i64>,
}

impl IpInventoryVcnOverlapCollection {
    /// Create a new IpInventoryVcnOverlapCollection
    pub fn new() -> Self {
        Self {
            last_updated_timestamp: None,

            ip_inventory_vcn_overlap_summary: None,

            message: None,

            overlap_count: None,
        }
    }

    /// Set last_updated_timestamp
    pub fn set_last_updated_timestamp(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.last_updated_timestamp = value;
        self
    }

    /// Set ip_inventory_vcn_overlap_summary
    pub fn set_ip_inventory_vcn_overlap_summary(
        mut self,
        value: Option<Vec<IpInventoryVcnOverlapSummary>>,
    ) -> Self {
        self.ip_inventory_vcn_overlap_summary = value;
        self
    }

    /// Set message
    pub fn set_message(mut self, value: Option<String>) -> Self {
        self.message = value;
        self
    }

    /// Set overlap_count
    pub fn set_overlap_count(mut self, value: Option<i64>) -> Self {
        self.overlap_count = value;
        self
    }

    /// Set last_updated_timestamp (unwraps Option)
    pub fn with_last_updated_timestamp(mut self, value: DateTime<Utc>) -> Self {
        self.last_updated_timestamp = Some(value);
        self
    }

    /// Set ip_inventory_vcn_overlap_summary (unwraps Option)
    pub fn with_ip_inventory_vcn_overlap_summary(
        mut self,
        value: Vec<IpInventoryVcnOverlapSummary>,
    ) -> Self {
        self.ip_inventory_vcn_overlap_summary = Some(value);
        self
    }

    /// Set message (unwraps Option)
    pub fn with_message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    /// Set overlap_count (unwraps Option)
    pub fn with_overlap_count(mut self, value: i64) -> Self {
        self.overlap_count = Some(value);
        self
    }
}

impl Default for IpInventoryVcnOverlapCollection {
    fn default() -> Self {
        Self::new()
    }
}
