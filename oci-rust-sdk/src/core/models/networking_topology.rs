use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the representation of a virtual network topology for a region. See [Network Visualizer Documentation](https://docs.oracle.com/iaas/Content/Network/Concepts/network_visualizer.htm) for more information, including conventions and pictures of symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkingTopology {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for NetworkingTopology
pub struct NetworkingTopologyRequired {
    pub r#type: String,
}

impl NetworkingTopology {
    /// Create a new NetworkingTopology with required fields
    pub fn new(required: NetworkingTopologyRequired) -> Self {
        Self {
            r#type: required.r#type,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
