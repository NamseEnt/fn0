use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Lists subnet and its associated resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySubnetSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// Name of the subnet within a VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_name: Option<String>,

    /// Resource types of the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<InventorySubnetSummaryResourceType>,

    /// Lists CIDRs and utilization within the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_subnet_cidr_collection: Option<Vec<InventorySubnetCidrBlockSummary>>,

    /// DNS domain name of the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_domain_name: Option<String>,

    /// Region name of the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Lists the {@code ResourceCollection} object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_resource_summary: Option<Vec<InventoryResourceSummary>>,
}

impl InventorySubnetSummary {
    /// Create a new InventorySubnetSummary
    pub fn new() -> Self {
        Self {
            subnet_id: None,

            subnet_name: None,

            resource_type: None,

            inventory_subnet_cidr_collection: None,

            dns_domain_name: None,

            region: None,

            compartment_id: None,

            inventory_resource_summary: None,
        }
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: Option<String>) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set subnet_name
    pub fn set_subnet_name(mut self, value: Option<String>) -> Self {
        self.subnet_name = value;
        self
    }

    /// Set resource_type
    pub fn set_resource_type(mut self, value: Option<InventorySubnetSummaryResourceType>) -> Self {
        self.resource_type = value;
        self
    }

    /// Set inventory_subnet_cidr_collection
    pub fn set_inventory_subnet_cidr_collection(
        mut self,
        value: Option<Vec<InventorySubnetCidrBlockSummary>>,
    ) -> Self {
        self.inventory_subnet_cidr_collection = value;
        self
    }

    /// Set dns_domain_name
    pub fn set_dns_domain_name(mut self, value: Option<String>) -> Self {
        self.dns_domain_name = value;
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

    /// Set inventory_resource_summary
    pub fn set_inventory_resource_summary(
        mut self,
        value: Option<Vec<InventoryResourceSummary>>,
    ) -> Self {
        self.inventory_resource_summary = value;
        self
    }

    /// Set subnet_id (unwraps Option)
    pub fn with_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.subnet_id = Some(value.into());
        self
    }

    /// Set subnet_name (unwraps Option)
    pub fn with_subnet_name(mut self, value: impl Into<String>) -> Self {
        self.subnet_name = Some(value.into());
        self
    }

    /// Set resource_type (unwraps Option)
    pub fn with_resource_type(mut self, value: InventorySubnetSummaryResourceType) -> Self {
        self.resource_type = Some(value);
        self
    }

    /// Set inventory_subnet_cidr_collection (unwraps Option)
    pub fn with_inventory_subnet_cidr_collection(
        mut self,
        value: Vec<InventorySubnetCidrBlockSummary>,
    ) -> Self {
        self.inventory_subnet_cidr_collection = Some(value);
        self
    }

    /// Set dns_domain_name (unwraps Option)
    pub fn with_dns_domain_name(mut self, value: impl Into<String>) -> Self {
        self.dns_domain_name = Some(value.into());
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

    /// Set inventory_resource_summary (unwraps Option)
    pub fn with_inventory_resource_summary(mut self, value: Vec<InventoryResourceSummary>) -> Self {
        self.inventory_resource_summary = Some(value);
        self
    }
}

impl Default for InventorySubnetSummary {
    fn default() -> Self {
        Self::new()
    }
}
