use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// IPsec tunnel detail information specific to phase two.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelPhaseTwoDetails {
    /// Indicates whether custom phase two configuration is enabled. If this option is not enabled, default settings are proposed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_custom_phase_two_config: Option<bool>,

    /// The total configured lifetime of the IKE security association. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<i64>,

    /// The remaining lifetime before the key is refreshed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_lifetime: Option<i64>,

    /// Phase two authentication algorithm proposed during tunnel negotiation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_authentication_algorithm: Option<String>,

    /// The negotiated phase two authentication algorithm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negotiated_authentication_algorithm: Option<String>,

    /// The proposed custom phase two encryption algorithm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_encryption_algorithm: Option<String>,

    /// The negotiated encryption algorithm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negotiated_encryption_algorithm: Option<String>,

    /// The proposed Diffie-Hellman group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dh_group: Option<String>,

    /// The negotiated Diffie-Hellman group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negotiated_dh_group: Option<String>,

    /// Indicates that ESP phase two is established.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_esp_established: Option<bool>,

    /// Indicates that PFS (perfect forward secrecy) is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pfs_enabled: Option<bool>,

    /// The date and time the remaining lifetime was last retrieved, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_lifetime_last_retrieved: Option<DateTime<Utc>>,
}

impl TunnelPhaseTwoDetails {
    /// Create a new TunnelPhaseTwoDetails
    pub fn new() -> Self {
        Self {
            is_custom_phase_two_config: None,

            lifetime: None,

            remaining_lifetime: None,

            custom_authentication_algorithm: None,

            negotiated_authentication_algorithm: None,

            custom_encryption_algorithm: None,

            negotiated_encryption_algorithm: None,

            dh_group: None,

            negotiated_dh_group: None,

            is_esp_established: None,

            is_pfs_enabled: None,

            remaining_lifetime_last_retrieved: None,
        }
    }

    /// Set is_custom_phase_two_config
    pub fn set_is_custom_phase_two_config(mut self, value: Option<bool>) -> Self {
        self.is_custom_phase_two_config = value;
        self
    }

    /// Set lifetime
    pub fn set_lifetime(mut self, value: Option<i64>) -> Self {
        self.lifetime = value;
        self
    }

    /// Set remaining_lifetime
    pub fn set_remaining_lifetime(mut self, value: Option<i64>) -> Self {
        self.remaining_lifetime = value;
        self
    }

    /// Set custom_authentication_algorithm
    pub fn set_custom_authentication_algorithm(mut self, value: Option<String>) -> Self {
        self.custom_authentication_algorithm = value;
        self
    }

    /// Set negotiated_authentication_algorithm
    pub fn set_negotiated_authentication_algorithm(mut self, value: Option<String>) -> Self {
        self.negotiated_authentication_algorithm = value;
        self
    }

    /// Set custom_encryption_algorithm
    pub fn set_custom_encryption_algorithm(mut self, value: Option<String>) -> Self {
        self.custom_encryption_algorithm = value;
        self
    }

    /// Set negotiated_encryption_algorithm
    pub fn set_negotiated_encryption_algorithm(mut self, value: Option<String>) -> Self {
        self.negotiated_encryption_algorithm = value;
        self
    }

    /// Set dh_group
    pub fn set_dh_group(mut self, value: Option<String>) -> Self {
        self.dh_group = value;
        self
    }

    /// Set negotiated_dh_group
    pub fn set_negotiated_dh_group(mut self, value: Option<String>) -> Self {
        self.negotiated_dh_group = value;
        self
    }

    /// Set is_esp_established
    pub fn set_is_esp_established(mut self, value: Option<bool>) -> Self {
        self.is_esp_established = value;
        self
    }

    /// Set is_pfs_enabled
    pub fn set_is_pfs_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pfs_enabled = value;
        self
    }

    /// Set remaining_lifetime_last_retrieved
    pub fn set_remaining_lifetime_last_retrieved(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.remaining_lifetime_last_retrieved = value;
        self
    }

    /// Set is_custom_phase_two_config (unwraps Option)
    pub fn with_is_custom_phase_two_config(mut self, value: bool) -> Self {
        self.is_custom_phase_two_config = Some(value);
        self
    }

    /// Set lifetime (unwraps Option)
    pub fn with_lifetime(mut self, value: i64) -> Self {
        self.lifetime = Some(value);
        self
    }

    /// Set remaining_lifetime (unwraps Option)
    pub fn with_remaining_lifetime(mut self, value: i64) -> Self {
        self.remaining_lifetime = Some(value);
        self
    }

    /// Set custom_authentication_algorithm (unwraps Option)
    pub fn with_custom_authentication_algorithm(mut self, value: impl Into<String>) -> Self {
        self.custom_authentication_algorithm = Some(value.into());
        self
    }

    /// Set negotiated_authentication_algorithm (unwraps Option)
    pub fn with_negotiated_authentication_algorithm(mut self, value: impl Into<String>) -> Self {
        self.negotiated_authentication_algorithm = Some(value.into());
        self
    }

    /// Set custom_encryption_algorithm (unwraps Option)
    pub fn with_custom_encryption_algorithm(mut self, value: impl Into<String>) -> Self {
        self.custom_encryption_algorithm = Some(value.into());
        self
    }

    /// Set negotiated_encryption_algorithm (unwraps Option)
    pub fn with_negotiated_encryption_algorithm(mut self, value: impl Into<String>) -> Self {
        self.negotiated_encryption_algorithm = Some(value.into());
        self
    }

    /// Set dh_group (unwraps Option)
    pub fn with_dh_group(mut self, value: impl Into<String>) -> Self {
        self.dh_group = Some(value.into());
        self
    }

    /// Set negotiated_dh_group (unwraps Option)
    pub fn with_negotiated_dh_group(mut self, value: impl Into<String>) -> Self {
        self.negotiated_dh_group = Some(value.into());
        self
    }

    /// Set is_esp_established (unwraps Option)
    pub fn with_is_esp_established(mut self, value: bool) -> Self {
        self.is_esp_established = Some(value);
        self
    }

    /// Set is_pfs_enabled (unwraps Option)
    pub fn with_is_pfs_enabled(mut self, value: bool) -> Self {
        self.is_pfs_enabled = Some(value);
        self
    }

    /// Set remaining_lifetime_last_retrieved (unwraps Option)
    pub fn with_remaining_lifetime_last_retrieved(mut self, value: DateTime<Utc>) -> Self {
        self.remaining_lifetime_last_retrieved = Some(value);
        self
    }
}

impl Default for TunnelPhaseTwoDetails {
    fn default() -> Self {
        Self::new()
    }
}
