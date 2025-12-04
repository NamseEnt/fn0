use serde::{Deserialize, Serialize};

/// Compute Instance Configuration instance details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeInstanceDetails {
    /// Block volume parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_volumes: Option<Vec<super::InstanceConfigurationBlockVolumeDetails>>,

    /// Launch details for the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_details: Option<super::InstanceConfigurationLaunchInstanceDetails>,

    /// Secondary VNIC parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_vnics: Option<Vec<super::InstanceConfigurationAttachVnicDetails>>,
}

impl ComputeInstanceDetails {
    pub fn new() -> Self {
        Self {
            block_volumes: None,
            launch_details: None,
            secondary_vnics: None,
        }
    }

    pub fn with_launch_details(
        mut self,
        launch_details: super::InstanceConfigurationLaunchInstanceDetails,
    ) -> Self {
        self.launch_details = Some(launch_details);
        self
    }

    pub fn with_block_volumes(
        mut self,
        block_volumes: Vec<super::InstanceConfigurationBlockVolumeDetails>,
    ) -> Self {
        self.block_volumes = Some(block_volumes);
        self
    }

    pub fn with_secondary_vnics(
        mut self,
        secondary_vnics: Vec<super::InstanceConfigurationAttachVnicDetails>,
    ) -> Self {
        self.secondary_vnics = Some(secondary_vnics);
        self
    }
}

impl Default for ComputeInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
