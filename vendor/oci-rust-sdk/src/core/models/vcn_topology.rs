use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the representation of a virtual network topology for a VCN. See [Network Visualizer Documentation](https://docs.oracle.com/iaas/Content/Network/Concepts/network_visualizer.htm) for more information, including conventions and pictures of symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VcnTopology {
    #[serde(rename = "type")]
    pub r#type: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN for which the topology is generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_id: Option<String>,
}

/// Required fields for VcnTopology
pub struct VcnTopologyRequired {
    pub r#type: String,
}

impl VcnTopology {
    /// Create a new VcnTopology with required fields
    pub fn new(required: VcnTopologyRequired) -> Self {
        Self {
            r#type: required.r#type,

            vcn_id: None,
        }
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: Option<String>) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set vcn_id (unwraps Option)
    pub fn with_vcn_id(mut self, value: impl Into<String>) -> Self {
        self.vcn_id = Some(value.into());
        self
    }
}
