use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Compute Instance Configuration instance details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeInstanceDetails {
    pub instance_type: String,

    /// Block volume parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_volumes: Option<Vec<InstanceConfigurationBlockVolumeDetails>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_details: Option<InstanceConfigurationLaunchInstanceDetails>,

    /// Secondary VNIC parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_vnics: Option<Vec<InstanceConfigurationAttachVnicDetails>>,
}

/// Required fields for ComputeInstanceDetails
pub struct ComputeInstanceDetailsRequired {
    pub instance_type: String,
}

impl ComputeInstanceDetails {
    /// Create a new ComputeInstanceDetails with required fields
    pub fn new(required: ComputeInstanceDetailsRequired) -> Self {
        Self {
            instance_type: required.instance_type,

            block_volumes: None,

            launch_details: None,

            secondary_vnics: None,
        }
    }

    /// Set block_volumes
    pub fn set_block_volumes(
        mut self,
        value: Option<Vec<InstanceConfigurationBlockVolumeDetails>>,
    ) -> Self {
        self.block_volumes = value;
        self
    }

    /// Set launch_details
    pub fn set_launch_details(
        mut self,
        value: Option<InstanceConfigurationLaunchInstanceDetails>,
    ) -> Self {
        self.launch_details = value;
        self
    }

    /// Set secondary_vnics
    pub fn set_secondary_vnics(
        mut self,
        value: Option<Vec<InstanceConfigurationAttachVnicDetails>>,
    ) -> Self {
        self.secondary_vnics = value;
        self
    }

    /// Set instance_type
    pub fn set_instance_type(mut self, value: String) -> Self {
        self.instance_type = value;
        self
    }

    /// Set block_volumes (unwraps Option)
    pub fn with_block_volumes(
        mut self,
        value: Vec<InstanceConfigurationBlockVolumeDetails>,
    ) -> Self {
        self.block_volumes = Some(value);
        self
    }

    /// Set launch_details (unwraps Option)
    pub fn with_launch_details(
        mut self,
        value: InstanceConfigurationLaunchInstanceDetails,
    ) -> Self {
        self.launch_details = Some(value);
        self
    }

    /// Set secondary_vnics (unwraps Option)
    pub fn with_secondary_vnics(
        mut self,
        value: Vec<InstanceConfigurationAttachVnicDetails>,
    ) -> Self {
        self.secondary_vnics = Some(value);
        self
    }
}
