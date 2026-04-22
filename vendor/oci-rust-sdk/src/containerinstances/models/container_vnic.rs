use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An interface to a virtual network available to containers on a container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerVnic {
    /// The identifier of the virtual network interface card (VNIC) over which the containers accessing this network can communicate with the larger virtual cloud network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_id: Option<String>,
}

impl ContainerVnic {
    /// Create a new ContainerVnic
    pub fn new() -> Self {
        Self { vnic_id: None }
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: Option<String>) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set vnic_id (unwraps Option)
    pub fn with_vnic_id(mut self, value: impl Into<String>) -> Self {
        self.vnic_id = Some(value.into());
        self
    }
}

impl Default for ContainerVnic {
    fn default() -> Self {
        Self::new()
    }
}
