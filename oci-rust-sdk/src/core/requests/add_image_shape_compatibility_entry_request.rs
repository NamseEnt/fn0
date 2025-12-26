use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddImageShapeCompatibilityEntryRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,

    /// Shape name.
    pub shape_name: String,

    /// Image shape compatibility details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_image_shape_compatibility_entry_details: Option<AddImageShapeCompatibilityEntryDetails>,
}


/// Required fields for AddImageShapeCompatibilityEntryRequest
pub struct AddImageShapeCompatibilityEntryRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the image.
    pub image_id: String,

    /// Shape name.
    pub shape_name: String,
}

impl AddImageShapeCompatibilityEntryRequest {
    /// Create a new AddImageShapeCompatibilityEntryRequest with required fields
    pub fn new(required: AddImageShapeCompatibilityEntryRequestRequired) -> Self {
        Self {
            image_id: required.image_id,

            shape_name: required.shape_name,

            add_image_shape_compatibility_entry_details: None,
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

    /// Set add_image_shape_compatibility_entry_details
    pub fn set_add_image_shape_compatibility_entry_details(mut self, value: Option<AddImageShapeCompatibilityEntryDetails>) -> Self {
        self.add_image_shape_compatibility_entry_details = value;
        self
    }

    /// Set add_image_shape_compatibility_entry_details (unwraps Option)
    pub fn with_add_image_shape_compatibility_entry_details(mut self, value: AddImageShapeCompatibilityEntryDetails) -> Self {
        self.add_image_shape_compatibility_entry_details = Some(value);
        self
    }
}


