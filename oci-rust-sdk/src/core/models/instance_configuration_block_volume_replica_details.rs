use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the block volume replica
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationBlockVolumeReplicaDetails {
    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The display name of the block volume replica. You may optionally specify a *display name* for the block volume replica, otherwise a default is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Required fields for InstanceConfigurationBlockVolumeReplicaDetails
pub struct InstanceConfigurationBlockVolumeReplicaDetailsRequired {
    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,
}

impl InstanceConfigurationBlockVolumeReplicaDetails {
    /// Create a new InstanceConfigurationBlockVolumeReplicaDetails with required fields
    pub fn new(required: InstanceConfigurationBlockVolumeReplicaDetailsRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            display_name: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
