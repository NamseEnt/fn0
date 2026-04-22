use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The image pull secrets for accessing private registry to pull images for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePullSecret {
    /// The registry endpoint of the container image.
    pub registry_endpoint: String,

    pub secret_type: String,
}

/// Required fields for ImagePullSecret
pub struct ImagePullSecretRequired {
    /// The registry endpoint of the container image.
    pub registry_endpoint: String,

    pub secret_type: String,
}

impl ImagePullSecret {
    /// Create a new ImagePullSecret with required fields
    pub fn new(required: ImagePullSecretRequired) -> Self {
        Self {
            registry_endpoint: required.registry_endpoint,

            secret_type: required.secret_type,
        }
    }

    /// Set registry_endpoint
    pub fn set_registry_endpoint(mut self, value: String) -> Self {
        self.registry_endpoint = value;
        self
    }

    /// Set secret_type
    pub fn set_secret_type(mut self, value: String) -> Self {
        self.secret_type = value;
        self
    }
}
