use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBootVolumeKmsKeyResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BootVolumeKmsKey instance.
    pub boot_volume_kms_key: BootVolumeKmsKey,
}

/// Required fields for UpdateBootVolumeKmsKeyResponse
pub struct UpdateBootVolumeKmsKeyResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BootVolumeKmsKey instance.
    pub boot_volume_kms_key: BootVolumeKmsKey,
}

impl UpdateBootVolumeKmsKeyResponse {
    /// Create a new UpdateBootVolumeKmsKeyResponse with required fields
    pub fn new(required: UpdateBootVolumeKmsKeyResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            boot_volume_kms_key: required.boot_volume_kms_key,
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

    /// Set boot_volume_kms_key
    pub fn set_boot_volume_kms_key(mut self, value: BootVolumeKmsKey) -> Self {
        self.boot_volume_kms_key = value;
        self
    }
}
