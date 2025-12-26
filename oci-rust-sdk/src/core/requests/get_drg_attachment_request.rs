use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDrgAttachmentRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG attachment.
    pub drg_attachment_id: String,
}


/// Required fields for GetDrgAttachmentRequest
pub struct GetDrgAttachmentRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG attachment.
    pub drg_attachment_id: String,
}

impl GetDrgAttachmentRequest {
    /// Create a new GetDrgAttachmentRequest with required fields
    pub fn new(required: GetDrgAttachmentRequestRequired) -> Self {
        Self {
            drg_attachment_id: required.drg_attachment_id,
}
    }

    /// Set drg_attachment_id
    pub fn set_drg_attachment_id(mut self, value: String) -> Self {
        self.drg_attachment_id = value;
        self
    }
}


