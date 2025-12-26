use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddImageShapeCompatibilityEntryResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ImageShapeCompatibilityEntry instance.
    pub image_shape_compatibility_entry: ImageShapeCompatibilityEntry,
}

/// Required fields for AddImageShapeCompatibilityEntryResponse
pub struct AddImageShapeCompatibilityEntryResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ImageShapeCompatibilityEntry instance.
    pub image_shape_compatibility_entry: ImageShapeCompatibilityEntry,
}

impl AddImageShapeCompatibilityEntryResponse {
    /// Create a new AddImageShapeCompatibilityEntryResponse with required fields
    pub fn new(required: AddImageShapeCompatibilityEntryResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            image_shape_compatibility_entry: required.image_shape_compatibility_entry,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set image_shape_compatibility_entry
    pub fn set_image_shape_compatibility_entry(
        mut self,
        value: ImageShapeCompatibilityEntry,
    ) -> Self {
        self.image_shape_compatibility_entry = value;
        self
    }
}
