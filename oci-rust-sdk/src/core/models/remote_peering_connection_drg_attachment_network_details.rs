use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the DRG attachment to another DRG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotePeeringConnectionDrgAttachmentNetworkDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for RemotePeeringConnectionDrgAttachmentNetworkDetails
pub struct RemotePeeringConnectionDrgAttachmentNetworkDetailsRequired {
    pub r#type: String,
}

impl RemotePeeringConnectionDrgAttachmentNetworkDetails {
    /// Create a new RemotePeeringConnectionDrgAttachmentNetworkDetails with required fields
    pub fn new(required: RemotePeeringConnectionDrgAttachmentNetworkDetailsRequired) -> Self {
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
