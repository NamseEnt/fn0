use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The results returned by a {@code ListIpInventory} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInventoryCollection {
    /// Species the count for the number of results for the response. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,

    /// The timestamp of the latest update from the database in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_timestamp: Option<DateTime<Utc>>,

    /// The number of compartments per compartments per tenant. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartments_per_tenant: Option<i64>,

    /// Lists {@code IpInventoryVcnSummary} objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_vcn_collection: Option<Vec<InventoryVcnSummary>>,

    /// Indicates the status of the data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl IpInventoryCollection {
    /// Create a new IpInventoryCollection
    pub fn new() -> Self {
        Self {
            count: None,

            last_updated_timestamp: None,

            compartments_per_tenant: None,

            inventory_vcn_collection: None,

            message: None,
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

    /// Set compartments_per_tenant
    pub fn set_compartments_per_tenant(mut self, value: Option<i64>) -> Self {
        self.compartments_per_tenant = value;
        self
    }

    /// Set inventory_vcn_collection
    pub fn set_inventory_vcn_collection(mut self, value: Option<Vec<InventoryVcnSummary>>) -> Self {
        self.inventory_vcn_collection = value;
        self
    }

    /// Set message
    pub fn set_message(mut self, value: Option<String>) -> Self {
        self.message = value;
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

    /// Set compartments_per_tenant (unwraps Option)
    pub fn with_compartments_per_tenant(mut self, value: i64) -> Self {
        self.compartments_per_tenant = Some(value);
        self
    }

    /// Set inventory_vcn_collection (unwraps Option)
    pub fn with_inventory_vcn_collection(mut self, value: Vec<InventoryVcnSummary>) -> Self {
        self.inventory_vcn_collection = Some(value);
        self
    }

    /// Set message (unwraps Option)
    pub fn with_message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }
}

impl Default for IpInventoryCollection {
    fn default() -> Self {
        Self::new()
    }
}
