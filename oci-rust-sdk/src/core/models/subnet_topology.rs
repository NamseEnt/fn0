use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the visualization of a subnet in a VCN. See [Network Visualizer Documentation](https://docs.oracle.com/iaas/Content/Network/Concepts/network_visualizer.htm) for more information, including conventions and pictures of symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetTopology {
    #[serde(rename = "type")]
    pub r#type: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet for which the visualization is generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,
}

/// Required fields for SubnetTopology
pub struct SubnetTopologyRequired {
    pub r#type: String,
}

impl SubnetTopology {
    /// Create a new SubnetTopology with required fields
    pub fn new(required: SubnetTopologyRequired) -> Self {
        Self {
            r#type: required.r#type,

            subnet_id: None,
        }
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: Option<String>) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set subnet_id (unwraps Option)
    pub fn with_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.subnet_id = Some(value.into());
        self
    }
}
