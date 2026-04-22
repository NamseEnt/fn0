use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the loopback attachment on the DRG. A loopback attachment can be used to terminate a virtual circuit that is carrying an IPSec tunnel, routing traffic directly to the IPSec tunnel attachment where the tunnel can terminate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopBackDrgAttachmentNetworkDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the target IPSec tunnel attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
}

/// Required fields for LoopBackDrgAttachmentNetworkDetails
pub struct LoopBackDrgAttachmentNetworkDetailsRequired {
    pub r#type: String,
}

impl LoopBackDrgAttachmentNetworkDetails {
    /// Create a new LoopBackDrgAttachmentNetworkDetails with required fields
    pub fn new(required: LoopBackDrgAttachmentNetworkDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            ids: None,
        }
    }

    /// Set ids
    pub fn set_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.ids = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set ids (unwraps Option)
    pub fn with_ids(mut self, value: Vec<String>) -> Self {
        self.ids = Some(value);
        self
    }
}
