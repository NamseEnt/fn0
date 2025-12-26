use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the summary of a VCN's IP Inventory data under specified compartments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryVcnSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN .
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_id: Option<String>,

    /// Name of the VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_name: Option<String>,

    /// Resource types of the VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<InventoryVcnSummaryResourceType>,

    /// Lists {@code InventoryVcnCidrBlockSummary} objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_vcn_cidr_block_collection: Option<Vec<InventoryVcnCidrBlockSummary>>,

    /// DNS domain name of the VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_domain_name: Option<String>,

    /// Region name of the VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Lists {@code Subnetcollection} objects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_subnetcollection: Option<Vec<InventorySubnetSummary>>,
}

impl InventoryVcnSummary {
    /// Create a new InventoryVcnSummary
    pub fn new() -> Self {
        Self {
            vcn_id: None,

            vcn_name: None,

            resource_type: None,

            inventory_vcn_cidr_block_collection: None,

            dns_domain_name: None,

            region: None,

            compartment_id: None,

            inventory_subnetcollection: None,
        }
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: Option<String>) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set vcn_name
    pub fn set_vcn_name(mut self, value: Option<String>) -> Self {
        self.vcn_name = value;
        self
    }

    /// Set resource_type
    pub fn set_resource_type(mut self, value: Option<InventoryVcnSummaryResourceType>) -> Self {
        self.resource_type = value;
        self
    }

    /// Set inventory_vcn_cidr_block_collection
    pub fn set_inventory_vcn_cidr_block_collection(
        mut self,
        value: Option<Vec<InventoryVcnCidrBlockSummary>>,
    ) -> Self {
        self.inventory_vcn_cidr_block_collection = value;
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

    /// Set inventory_subnetcollection
    pub fn set_inventory_subnetcollection(
        mut self,
        value: Option<Vec<InventorySubnetSummary>>,
    ) -> Self {
        self.inventory_subnetcollection = value;
        self
    }

    /// Set vcn_id (unwraps Option)
    pub fn with_vcn_id(mut self, value: impl Into<String>) -> Self {
        self.vcn_id = Some(value.into());
        self
    }

    /// Set vcn_name (unwraps Option)
    pub fn with_vcn_name(mut self, value: impl Into<String>) -> Self {
        self.vcn_name = Some(value.into());
        self
    }

    /// Set resource_type (unwraps Option)
    pub fn with_resource_type(mut self, value: InventoryVcnSummaryResourceType) -> Self {
        self.resource_type = Some(value);
        self
    }

    /// Set inventory_vcn_cidr_block_collection (unwraps Option)
    pub fn with_inventory_vcn_cidr_block_collection(
        mut self,
        value: Vec<InventoryVcnCidrBlockSummary>,
    ) -> Self {
        self.inventory_vcn_cidr_block_collection = Some(value);
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

    /// Set inventory_subnetcollection (unwraps Option)
    pub fn with_inventory_subnetcollection(mut self, value: Vec<InventorySubnetSummary>) -> Self {
        self.inventory_subnetcollection = Some(value);
        self
    }
}

impl Default for InventoryVcnSummary {
    fn default() -> Self {
        Self::new()
    }
}
