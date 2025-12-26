use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteImageRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for DeleteImageRequest
pub struct DeleteImageRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,
}

impl DeleteImageRequest {
    /// Create a new DeleteImageRequest with required fields
    pub fn new(required: DeleteImageRequestRequired) -> Self {
        Self {
            image_id: required.image_id,

            if_match: None,
}
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: String) -> Self {
        self.image_id = value;
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


