use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The Vault service master encryption key associated with this volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolumeKmsKey {
    /// The OCID of the Vault service key assigned to this volume. If the volume is not using Vault service, then the {@code kmsKeyId} will be a null string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,
}

impl BootVolumeKmsKey {
    /// Create a new BootVolumeKmsKey
    pub fn new() -> Self {
        Self { kms_key_id: None }
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }
}

impl Default for BootVolumeKmsKey {
    fn default() -> Self {
        Self::new()
    }
}
