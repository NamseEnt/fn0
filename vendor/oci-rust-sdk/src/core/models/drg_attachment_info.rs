use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The {@code DrgAttachmentInfo} resource contains the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgAttachmentInfo {
    /// The Oracle-assigned ID of the DRG attachment
    pub id: String,
}

/// Required fields for DrgAttachmentInfo
pub struct DrgAttachmentInfoRequired {
    /// The Oracle-assigned ID of the DRG attachment
    pub id: String,
}

impl DrgAttachmentInfo {
    /// Create a new DrgAttachmentInfo with required fields
    pub fn new(required: DrgAttachmentInfoRequired) -> Self {
        Self { id: required.id }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }
}
