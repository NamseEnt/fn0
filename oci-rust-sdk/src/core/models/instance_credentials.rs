use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The credentials for a particular instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceCredentials {
    /// The password for the username.
    pub password: String,

    /// The username.
    pub username: String,
}

/// Required fields for InstanceCredentials
pub struct InstanceCredentialsRequired {
    /// The password for the username.
    pub password: String,

    /// The username.
    pub username: String,
}

impl InstanceCredentials {
    /// Create a new InstanceCredentials with required fields
    pub fn new(required: InstanceCredentialsRequired) -> Self {
        Self {
            password: required.password,

            username: required.username,
        }
    }

    /// Set password
    pub fn set_password(mut self, value: String) -> Self {
        self.password = value;
        self
    }

    /// Set username
    pub fn set_username(mut self, value: String) -> Self {
        self.username = value;
        self
    }
}
