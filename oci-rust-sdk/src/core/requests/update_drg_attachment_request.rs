use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgAttachmentRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG attachment.
    pub drg_attachment_id: String,

    /// Details object for updating a {@code DrgAttachment}.
    pub update_drg_attachment_details: UpdateDrgAttachmentDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateDrgAttachmentRequest
pub struct UpdateDrgAttachmentRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG attachment.
    pub drg_attachment_id: String,

    /// Details object for updating a {@code DrgAttachment}.
    pub update_drg_attachment_details: UpdateDrgAttachmentDetails,
}

impl UpdateDrgAttachmentRequest {
    /// Create a new UpdateDrgAttachmentRequest with required fields
    pub fn new(required: UpdateDrgAttachmentRequestRequired) -> Self {
        Self {
            drg_attachment_id: required.drg_attachment_id,

            update_drg_attachment_details: required.update_drg_attachment_details,

            if_match: None,
        }
    }

    /// Set drg_attachment_id
    pub fn set_drg_attachment_id(mut self, value: String) -> Self {
        self.drg_attachment_id = value;
        self
    }

    /// Set update_drg_attachment_details
    pub fn set_update_drg_attachment_details(mut self, value: UpdateDrgAttachmentDetails) -> Self {
        self.update_drg_attachment_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}
