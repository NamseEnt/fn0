use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A BasicImagePullSecret is a ImagePullSecret which accepts username and password as credentials information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicImagePullSecret {
    pub secret_type: String,
}

/// Required fields for BasicImagePullSecret
pub struct BasicImagePullSecretRequired {
    pub secret_type: String,
}

impl BasicImagePullSecret {
    /// Create a new BasicImagePullSecret with required fields
    pub fn new(required: BasicImagePullSecretRequired) -> Self {
        Self {
            secret_type: required.secret_type,
        }
    }

    /// Set secret_type
    pub fn set_secret_type(mut self, value: String) -> Self {
        self.secret_type = value;
        self
    }
}
