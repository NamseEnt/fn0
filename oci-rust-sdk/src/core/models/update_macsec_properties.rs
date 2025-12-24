use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Properties used to update MACsec settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMacsecProperties {
    /// Indicates whether or not MACsec is enabled.
    pub state: MacsecState,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_key: Option<UpdateMacsecKey>,

    /// Type of encryption cipher suite to use for the MACsec connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_cipher: Option<MacsecEncryptionCipher>,

    /// Indicates whether unencrypted traffic is allowed if MACsec Key Agreement protocol (MKA) fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_unprotected_traffic_allowed: Option<bool>,
}

/// Required fields for UpdateMacsecProperties
pub struct UpdateMacsecPropertiesRequired {
    /// Indicates whether or not MACsec is enabled.
    pub state: MacsecState,
}

impl UpdateMacsecProperties {
    /// Create a new UpdateMacsecProperties with required fields
    pub fn new(required: UpdateMacsecPropertiesRequired) -> Self {
        Self {
            state: required.state,

            primary_key: None,

            encryption_cipher: None,

            is_unprotected_traffic_allowed: None,
        }
    }

    /// Set state
    pub fn set_state(mut self, value: MacsecState) -> Self {
        self.state = value;
        self
    }

    /// Set primary_key
    pub fn set_primary_key(mut self, value: Option<UpdateMacsecKey>) -> Self {
        self.primary_key = value;
        self
    }

    /// Set encryption_cipher
    pub fn set_encryption_cipher(mut self, value: Option<MacsecEncryptionCipher>) -> Self {
        self.encryption_cipher = value;
        self
    }

    /// Set is_unprotected_traffic_allowed
    pub fn set_is_unprotected_traffic_allowed(mut self, value: Option<bool>) -> Self {
        self.is_unprotected_traffic_allowed = value;
        self
    }

    /// Set primary_key (unwraps Option)
    pub fn with_primary_key(mut self, value: UpdateMacsecKey) -> Self {
        self.primary_key = Some(value);
        self
    }

    /// Set encryption_cipher (unwraps Option)
    pub fn with_encryption_cipher(mut self, value: MacsecEncryptionCipher) -> Self {
        self.encryption_cipher = Some(value);
        self
    }

    /// Set is_unprotected_traffic_allowed (unwraps Option)
    pub fn with_is_unprotected_traffic_allowed(mut self, value: bool) -> Self {
        self.is_unprotected_traffic_allowed = Some(value);
        self
    }
}
