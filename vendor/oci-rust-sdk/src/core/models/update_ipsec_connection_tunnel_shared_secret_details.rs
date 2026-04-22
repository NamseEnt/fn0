use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIPSecConnectionTunnelSharedSecretDetails {
    /// The shared secret (pre-shared key) to use for the tunnel. Only numbers, letters, and spaces are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_secret: Option<String>,
}

impl UpdateIPSecConnectionTunnelSharedSecretDetails {
    /// Create a new UpdateIPSecConnectionTunnelSharedSecretDetails
    pub fn new() -> Self {
        Self {
            shared_secret: None,
        }
    }

    /// Set shared_secret
    pub fn set_shared_secret(mut self, value: Option<String>) -> Self {
        self.shared_secret = value;
        self
    }

    /// Set shared_secret (unwraps Option)
    pub fn with_shared_secret(mut self, value: impl Into<String>) -> Self {
        self.shared_secret = Some(value.into());
        self
    }
}

impl Default for UpdateIPSecConnectionTunnelSharedSecretDetails {
    fn default() -> Self {
        Self::new()
    }
}
