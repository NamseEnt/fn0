use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetImageShapeCompatibilityEntryResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ImageShapeCompatibilityEntry instance.
    pub image_shape_compatibility_entry: ImageShapeCompatibilityEntry,
}

/// Required fields for GetImageShapeCompatibilityEntryResponse
pub struct GetImageShapeCompatibilityEntryResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ImageShapeCompatibilityEntry instance.
    pub image_shape_compatibility_entry: ImageShapeCompatibilityEntry,
}

impl GetImageShapeCompatibilityEntryResponse {
    /// Create a new GetImageShapeCompatibilityEntryResponse with required fields
    pub fn new(required: GetImageShapeCompatibilityEntryResponseRequired) -> Self {
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
