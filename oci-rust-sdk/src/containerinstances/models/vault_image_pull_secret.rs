use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A VaultImagePullSecret is a ImagePullSecret which accepts secretId as credentials information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultImagePullSecret {
    /// The OCID of the secret for registry credentials.
    pub secret_id: String,

    pub secret_type: String,
}

/// Required fields for VaultImagePullSecret
pub struct VaultImagePullSecretRequired {
    /// The OCID of the secret for registry credentials.
    pub secret_id: String,

    pub secret_type: String,
}

impl VaultImagePullSecret {
    /// Create a new VaultImagePullSecret with required fields
    pub fn new(required: VaultImagePullSecretRequired) -> Self {
        Self {
            secret_id: required.secret_id,

            secret_type: required.secret_type,
        }
    }

    /// Set secret_id
    pub fn set_secret_id(mut self, value: String) -> Self {
        self.secret_id = value;
        self
    }

    /// Set secret_type
    pub fn set_secret_type(mut self, value: String) -> Self {
        self.secret_type = value;
        self
    }
}
