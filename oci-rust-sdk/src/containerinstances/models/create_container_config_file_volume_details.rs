use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration files to pass to the container using volume mounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerConfigFileVolumeDetails {
    pub volume_type: String,

    /// Contains key value pairs which can be mounted as individual files inside the container. The value needs to be base64 encoded. It is decoded to plain text before the mount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configs: Option<Vec<ContainerConfigFile>>,
}

/// Required fields for CreateContainerConfigFileVolumeDetails
pub struct CreateContainerConfigFileVolumeDetailsRequired {
    pub volume_type: String,
}

impl CreateContainerConfigFileVolumeDetails {
    /// Create a new CreateContainerConfigFileVolumeDetails with required fields
    pub fn new(required: CreateContainerConfigFileVolumeDetailsRequired) -> Self {
        Self {
            volume_type: required.volume_type,

            configs: None,
        }
    }

    /// Set configs
    pub fn set_configs(mut self, value: Option<Vec<ContainerConfigFile>>) -> Self {
        self.configs = value;
        self
    }

    /// Set volume_type
    pub fn set_volume_type(mut self, value: String) -> Self {
        self.volume_type = value;
        self
    }

    /// Set configs (unwraps Option)
    pub fn with_configs(mut self, value: Vec<ContainerConfigFile>) -> Self {
        self.configs = Some(value);
        self
    }
}
