use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The tunnel's shared secret (pre-shared key).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPSecConnectionTunnelSharedSecret {
    /// The tunnel's shared secret (pre-shared key).
    pub shared_secret: String,
}

/// Required fields for IPSecConnectionTunnelSharedSecret
pub struct IPSecConnectionTunnelSharedSecretRequired {
    /// The tunnel's shared secret (pre-shared key).
    pub shared_secret: String,
}

impl IPSecConnectionTunnelSharedSecret {
    /// Create a new IPSecConnectionTunnelSharedSecret with required fields
    pub fn new(required: IPSecConnectionTunnelSharedSecretRequired) -> Self {
        Self {
            shared_secret: required.shared_secret,
        }
    }

    /// Set shared_secret
    pub fn set_shared_secret(mut self, value: String) -> Self {
        self.shared_secret = value;
        self
    }
}
