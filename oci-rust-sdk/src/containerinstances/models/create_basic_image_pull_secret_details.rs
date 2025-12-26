use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A CreateBasicImagePullSecretDetails is a ImagePullSecret which accepts username and password as credentials information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBasicImagePullSecretDetails {
    /// The username which should be used with the registry for authentication. The value is expected in base64 format.
    pub username: String,

    /// The password which should be used with the registry for authentication. The value is expected in base64 format.
    pub password: String,

    pub secret_type: String,
}

/// Required fields for CreateBasicImagePullSecretDetails
pub struct CreateBasicImagePullSecretDetailsRequired {
    /// The username which should be used with the registry for authentication. The value is expected in base64 format.
    pub username: String,

    /// The password which should be used with the registry for authentication. The value is expected in base64 format.
    pub password: String,

    pub secret_type: String,
}

impl CreateBasicImagePullSecretDetails {
    /// Create a new CreateBasicImagePullSecretDetails with required fields
    pub fn new(required: CreateBasicImagePullSecretDetailsRequired) -> Self {
        Self {
            username: required.username,

            password: required.password,

            secret_type: required.secret_type,
        }
    }

    /// Set username
    pub fn set_username(mut self, value: String) -> Self {
        self.username = value;
        self
    }

    /// Set password
    pub fn set_password(mut self, value: String) -> Self {
        self.password = value;
        self
    }

    /// Set secret_type
    pub fn set_secret_type(mut self, value: String) -> Self {
        self.secret_type = value;
        self
    }
}
