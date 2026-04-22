use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the virtual circuit attached to the DRG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualCircuitDrgAttachmentNetworkDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// Boolean flag that determines wether all traffic over the virtual circuits is encrypted. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport_only_mode: Option<bool>,
}

/// Required fields for VirtualCircuitDrgAttachmentNetworkDetails
pub struct VirtualCircuitDrgAttachmentNetworkDetailsRequired {
    pub r#type: String,
}

impl VirtualCircuitDrgAttachmentNetworkDetails {
    /// Create a new VirtualCircuitDrgAttachmentNetworkDetails with required fields
    pub fn new(required: VirtualCircuitDrgAttachmentNetworkDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            transport_only_mode: None,
        }
    }

    /// Set transport_only_mode
    pub fn set_transport_only_mode(mut self, value: Option<bool>) -> Self {
        self.transport_only_mode = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set transport_only_mode (unwraps Option)
    pub fn with_transport_only_mode(mut self, value: bool) -> Self {
        self.transport_only_mode = Some(value);
        self
    }
}
