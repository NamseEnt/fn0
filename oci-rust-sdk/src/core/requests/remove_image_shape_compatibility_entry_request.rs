use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveImageShapeCompatibilityEntryRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,

    /// Shape name.
    pub shape_name: String,
}


/// Required fields for RemoveImageShapeCompatibilityEntryRequest
pub struct RemoveImageShapeCompatibilityEntryRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,

    /// Shape name.
    pub shape_name: String,
}

impl RemoveImageShapeCompatibilityEntryRequest {
    /// Create a new RemoveImageShapeCompatibilityEntryRequest with required fields
    pub fn new(required: RemoveImageShapeCompatibilityEntryRequestRequired) -> Self {
        Self {
            image_id: required.image_id,

            shape_name: required.shape_name,
}
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: String) -> Self {
        self.image_id = value;
        self
    }

    /// Set shape_name
    pub fn set_shape_name(mut self, value: String) -> Self {
        self.shape_name = value;
        self
    }
}


