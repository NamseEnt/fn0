use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateImageResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.Image instance.
    pub image: Image,
}


/// Required fields for UpdateImageResponse
pub struct UpdateImageResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.Image instance.
    pub image: Image,
}

impl UpdateImageResponse {
    /// Create a new UpdateImageResponse with required fields
    pub fn new(required: UpdateImageResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            image: required.image,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set image
    pub fn set_image(mut self, value: Image) -> Self {
        self.image = value;
        self
    }
}


