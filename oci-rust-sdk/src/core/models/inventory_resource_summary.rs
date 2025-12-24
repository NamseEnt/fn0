use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Lists resources and its properties under a given subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryResourceSummary {
    /// The name of the resource created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,

    /// Resource types of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<InventoryResourceSummaryResourceType>,

    /// Mac Address of IP Resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,

    /// Lists the 'IpAddressCollection' object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address_collection: Option<Vec<InventoryIpAddressSummary>>,

    /// The region name of the corresponding resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,
}

impl InventoryResourceSummary {
    /// Create a new InventoryResourceSummary
    pub fn new() -> Self {
        Self {
            resource_name: None,

            resource_type: None,

            mac_address: None,

            ip_address_collection: None,

            region: None,

            compartment_id: None,
        }
    }

    /// Set resource_name
    pub fn set_resource_name(mut self, value: Option<String>) -> Self {
        self.resource_name = value;
        self
    }

    /// Set resource_type
    pub fn set_resource_type(
        mut self,
        value: Option<InventoryResourceSummaryResourceType>,
    ) -> Self {
        self.resource_type = value;
        self
    }

    /// Set mac_address
    pub fn set_mac_address(mut self, value: Option<String>) -> Self {
        self.mac_address = value;
        self
    }

    /// Set ip_address_collection
    pub fn set_ip_address_collection(
        mut self,
        value: Option<Vec<InventoryIpAddressSummary>>,
    ) -> Self {
        self.ip_address_collection = value;
        self
    }

    /// Set region
    pub fn set_region(mut self, value: Option<String>) -> Self {
        self.region = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set resource_name (unwraps Option)
    pub fn with_resource_name(mut self, value: impl Into<String>) -> Self {
        self.resource_name = Some(value.into());
        self
    }

    /// Set resource_type (unwraps Option)
    pub fn with_resource_type(mut self, value: InventoryResourceSummaryResourceType) -> Self {
        self.resource_type = Some(value);
        self
    }

    /// Set mac_address (unwraps Option)
    pub fn with_mac_address(mut self, value: impl Into<String>) -> Self {
        self.mac_address = Some(value.into());
        self
    }

    /// Set ip_address_collection (unwraps Option)
    pub fn with_ip_address_collection(mut self, value: Vec<InventoryIpAddressSummary>) -> Self {
        self.ip_address_collection = Some(value);
        self
    }

    /// Set region (unwraps Option)
    pub fn with_region(mut self, value: impl Into<String>) -> Self {
        self.region = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }
}

impl Default for InventoryResourceSummary {
    fn default() -> Self {
        Self::new()
    }
}
