use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration details for IPSec phase two configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseTwoConfigDetails {
    /// Indicates whether custom configuration is enabled for phase two options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_custom_phase_two_config: Option<bool>,

    /// The authentication algorithm proposed during phase two tunnel negotiation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_algorithm: Option<PhaseTwoConfigDetailsAuthenticationAlgorithm>,

    /// The encryption algorithm proposed during phase two tunnel negotiation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_algorithm: Option<PhaseTwoConfigDetailsEncryptionAlgorithm>,

    /// Lifetime in seconds for the IPSec session key set in phase two. The default is 3600 which is equivalent to 1 hour. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime_in_seconds: Option<i64>,

    /// Indicates whether perfect forward secrecy (PFS) is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pfs_enabled: Option<bool>,

    /// The Diffie-Hellman group used for PFS, if PFS is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfs_dh_group: Option<PhaseTwoConfigDetailsPfsDhGroup>,
}

impl PhaseTwoConfigDetails {
    /// Create a new PhaseTwoConfigDetails
    pub fn new() -> Self {
        Self {
            is_custom_phase_two_config: None,

            authentication_algorithm: None,

            encryption_algorithm: None,

            lifetime_in_seconds: None,

            is_pfs_enabled: None,

            pfs_dh_group: None,
        }
    }

    /// Set is_custom_phase_two_config
    pub fn set_is_custom_phase_two_config(mut self, value: Option<bool>) -> Self {
        self.is_custom_phase_two_config = value;
        self
    }

    /// Set authentication_algorithm
    pub fn set_authentication_algorithm(
        mut self,
        value: Option<PhaseTwoConfigDetailsAuthenticationAlgorithm>,
    ) -> Self {
        self.authentication_algorithm = value;
        self
    }

    /// Set encryption_algorithm
    pub fn set_encryption_algorithm(
        mut self,
        value: Option<PhaseTwoConfigDetailsEncryptionAlgorithm>,
    ) -> Self {
        self.encryption_algorithm = value;
        self
    }

    /// Set lifetime_in_seconds
    pub fn set_lifetime_in_seconds(mut self, value: Option<i64>) -> Self {
        self.lifetime_in_seconds = value;
        self
    }

    /// Set is_pfs_enabled
    pub fn set_is_pfs_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pfs_enabled = value;
        self
    }

    /// Set pfs_dh_group
    pub fn set_pfs_dh_group(mut self, value: Option<PhaseTwoConfigDetailsPfsDhGroup>) -> Self {
        self.pfs_dh_group = value;
        self
    }

    /// Set is_custom_phase_two_config (unwraps Option)
    pub fn with_is_custom_phase_two_config(mut self, value: bool) -> Self {
        self.is_custom_phase_two_config = Some(value);
        self
    }

    /// Set authentication_algorithm (unwraps Option)
    pub fn with_authentication_algorithm(
        mut self,
        value: PhaseTwoConfigDetailsAuthenticationAlgorithm,
    ) -> Self {
        self.authentication_algorithm = Some(value);
        self
    }

    /// Set encryption_algorithm (unwraps Option)
    pub fn with_encryption_algorithm(
        mut self,
        value: PhaseTwoConfigDetailsEncryptionAlgorithm,
    ) -> Self {
        self.encryption_algorithm = Some(value);
        self
    }

    /// Set lifetime_in_seconds (unwraps Option)
    pub fn with_lifetime_in_seconds(mut self, value: i64) -> Self {
        self.lifetime_in_seconds = Some(value);
        self
    }

    /// Set is_pfs_enabled (unwraps Option)
    pub fn with_is_pfs_enabled(mut self, value: bool) -> Self {
        self.is_pfs_enabled = Some(value);
        self
    }

    /// Set pfs_dh_group (unwraps Option)
    pub fn with_pfs_dh_group(mut self, value: PhaseTwoConfigDetailsPfsDhGroup) -> Self {
        self.pfs_dh_group = Some(value);
        self
    }
}

impl Default for PhaseTwoConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
