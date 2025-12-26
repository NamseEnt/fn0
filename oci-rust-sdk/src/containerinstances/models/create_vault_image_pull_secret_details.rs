use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A CreateVaultImagePullSecretDetails is a ImagePullSecret which accepts secretId as credentials information. **Sample Format for username and password in Vault Secret** {@code { \"username\": \"this-is-not-the-secret\", \"password\": \"example-password\" } }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVaultImagePullSecretDetails {
    /// The OCID of the secret for registry credentials.
    pub secret_id: String,

    pub secret_type: String,
}

/// Required fields for CreateVaultImagePullSecretDetails
pub struct CreateVaultImagePullSecretDetailsRequired {
    /// The OCID of the secret for registry credentials.
    pub secret_id: String,

    pub secret_type: String,
}

impl CreateVaultImagePullSecretDetails {
    /// Create a new CreateVaultImagePullSecretDetails with required fields
    pub fn new(required: CreateVaultImagePullSecretDetailsRequired) -> Self {
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
