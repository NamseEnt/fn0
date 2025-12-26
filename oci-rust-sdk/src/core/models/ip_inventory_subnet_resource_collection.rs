use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The results returned by a {@code ListIpInventorySubnet} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInventorySubnetResourceCollection {
    /// Specifies the count for the number of results for the response. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,

    /// The Timestamp of the latest update from the database in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_timestamp: Option<DateTime<Utc>>,

    /// Lists {@code SubnetResourceSummary} objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_inventory_subnet_resource_summary: Option<Vec<IpInventorySubnetResourceSummary>>,

    /// Indicates the status of the data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// The compartment of the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,
}

impl IpInventorySubnetResourceCollection {
    /// Create a new IpInventorySubnetResourceCollection
    pub fn new() -> Self {
        Self {
            count: None,

            last_updated_timestamp: None,

            ip_inventory_subnet_resource_summary: None,

            message: None,

            compartment_id: None,
        }
    }

    /// Set count
    pub fn set_count(mut self, value: Option<i64>) -> Self {
        self.count = value;
        self
    }

    /// Set last_updated_timestamp
    pub fn set_last_updated_timestamp(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.last_updated_timestamp = value;
        self
    }

    /// Set ip_inventory_subnet_resource_summary
    pub fn set_ip_inventory_subnet_resource_summary(
        mut self,
        value: Option<Vec<IpInventorySubnetResourceSummary>>,
    ) -> Self {
        self.ip_inventory_subnet_resource_summary = value;
        self
    }

    /// Set message
    pub fn set_message(mut self, value: Option<String>) -> Self {
        self.message = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set count (unwraps Option)
    pub fn with_count(mut self, value: i64) -> Self {
        self.count = Some(value);
        self
    }

    /// Set last_updated_timestamp (unwraps Option)
    pub fn with_last_updated_timestamp(mut self, value: DateTime<Utc>) -> Self {
        self.last_updated_timestamp = Some(value);
        self
    }

    /// Set ip_inventory_subnet_resource_summary (unwraps Option)
    pub fn with_ip_inventory_subnet_resource_summary(
        mut self,
        value: Vec<IpInventorySubnetResourceSummary>,
    ) -> Self {
        self.ip_inventory_subnet_resource_summary = Some(value);
        self
    }

    /// Set message (unwraps Option)
    pub fn with_message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }
}

impl Default for IpInventorySubnetResourceCollection {
    fn default() -> Self {
        Self::new()
    }
}
