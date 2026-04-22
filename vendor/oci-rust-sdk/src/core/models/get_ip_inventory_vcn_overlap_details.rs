use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Lists the compartment to find VCN overlap.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIpInventoryVcnOverlapDetails {
    /// Lists the selected regions.
    pub region_list: Vec<String>,

    /// The list of [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartments.
    pub compartment_list: Vec<String>,
}

/// Required fields for GetIpInventoryVcnOverlapDetails
pub struct GetIpInventoryVcnOverlapDetailsRequired {
    /// Lists the selected regions.
    pub region_list: Vec<String>,

    /// The list of [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartments.
    pub compartment_list: Vec<String>,
}

impl GetIpInventoryVcnOverlapDetails {
    /// Create a new GetIpInventoryVcnOverlapDetails with required fields
    pub fn new(required: GetIpInventoryVcnOverlapDetailsRequired) -> Self {
        Self {
            region_list: required.region_list,

            compartment_list: required.compartment_list,
        }
    }

    /// Set region_list
    pub fn set_region_list(mut self, value: Vec<String>) -> Self {
        self.region_list = value;
        self
    }

    /// Set compartment_list
    pub fn set_compartment_list(mut self, value: Vec<String>) -> Self {
        self.compartment_list = value;
        self
    }
}
