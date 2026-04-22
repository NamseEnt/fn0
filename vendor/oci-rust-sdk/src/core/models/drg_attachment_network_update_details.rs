use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgAttachmentNetworkUpdateDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for DrgAttachmentNetworkUpdateDetails
pub struct DrgAttachmentNetworkUpdateDetailsRequired {
    pub r#type: String,
}

impl DrgAttachmentNetworkUpdateDetails {
    /// Create a new DrgAttachmentNetworkUpdateDetails with required fields
    pub fn new(required: DrgAttachmentNetworkUpdateDetailsRequired) -> Self {
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
