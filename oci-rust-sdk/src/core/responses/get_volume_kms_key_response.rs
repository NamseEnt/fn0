use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeKmsKeyResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeKmsKey instance.
    pub volume_kms_key: VolumeKmsKey,
}


/// Required fields for GetVolumeKmsKeyResponse
pub struct GetVolumeKmsKeyResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeKmsKey instance.
    pub volume_kms_key: VolumeKmsKey,
}

impl GetVolumeKmsKeyResponse {
    /// Create a new GetVolumeKmsKeyResponse with required fields
    pub fn new(required: GetVolumeKmsKeyResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            volume_kms_key: required.volume_kms_key,
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

    /// Set volume_kms_key
    pub fn set_volume_kms_key(mut self, value: VolumeKmsKey) -> Self {
        self.volume_kms_key = value;
        self
    }
}


