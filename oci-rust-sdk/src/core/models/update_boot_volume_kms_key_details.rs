use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBootVolumeKmsKeyDetails {
    /// The OCID of the new Vault service key to assign to protect the specified volume. This key has to be a valid Vault service key, and policies must exist to allow the user and the Block Volume service to access this key. If you specify the same OCID as the previous key's OCID, the Block Volume service will use it to regenerate a volume encryption key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,
}

impl UpdateBootVolumeKmsKeyDetails {
    /// Create a new UpdateBootVolumeKmsKeyDetails
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

impl Default for UpdateBootVolumeKmsKeyDetails {
    fn default() -> Self {
        Self::new()
    }
}
