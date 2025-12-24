use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration details for IKE phase one (ISAKMP) configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseOneConfigDetails {
    /// Indicates whether custom configuration is enabled for phase one options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_custom_phase_one_config: Option<bool>,

    /// The custom authentication algorithm proposed during phase one tunnel negotiation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_algorithm: Option<PhaseOneConfigDetailsAuthenticationAlgorithm>,

    /// The custom encryption algorithm proposed during phase one tunnel negotiation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_algorithm: Option<PhaseOneConfigDetailsEncryptionAlgorithm>,

    /// The custom Diffie-Hellman group proposed during phase one tunnel negotiation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffie_helman_group: Option<PhaseOneConfigDetailsDiffieHelmanGroup>,

    /// Internet key association (IKE) session key lifetime in seconds for IPSec phase one. The default is 28800 which is equivalent to 8 hours. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime_in_seconds: Option<i64>,
}

impl PhaseOneConfigDetails {
    /// Create a new PhaseOneConfigDetails
    pub fn new() -> Self {
        Self {
            is_custom_phase_one_config: None,

            authentication_algorithm: None,

            encryption_algorithm: None,

            diffie_helman_group: None,

            lifetime_in_seconds: None,
        }
    }

    /// Set is_custom_phase_one_config
    pub fn set_is_custom_phase_one_config(mut self, value: Option<bool>) -> Self {
        self.is_custom_phase_one_config = value;
        self
    }

    /// Set authentication_algorithm
    pub fn set_authentication_algorithm(
        mut self,
        value: Option<PhaseOneConfigDetailsAuthenticationAlgorithm>,
    ) -> Self {
        self.authentication_algorithm = value;
        self
    }

    /// Set encryption_algorithm
    pub fn set_encryption_algorithm(
        mut self,
        value: Option<PhaseOneConfigDetailsEncryptionAlgorithm>,
    ) -> Self {
        self.encryption_algorithm = value;
        self
    }

    /// Set diffie_helman_group
    pub fn set_diffie_helman_group(
        mut self,
        value: Option<PhaseOneConfigDetailsDiffieHelmanGroup>,
    ) -> Self {
        self.diffie_helman_group = value;
        self
    }

    /// Set lifetime_in_seconds
    pub fn set_lifetime_in_seconds(mut self, value: Option<i64>) -> Self {
        self.lifetime_in_seconds = value;
        self
    }

    /// Set is_custom_phase_one_config (unwraps Option)
    pub fn with_is_custom_phase_one_config(mut self, value: bool) -> Self {
        self.is_custom_phase_one_config = Some(value);
        self
    }

    /// Set authentication_algorithm (unwraps Option)
    pub fn with_authentication_algorithm(
        mut self,
        value: PhaseOneConfigDetailsAuthenticationAlgorithm,
    ) -> Self {
        self.authentication_algorithm = Some(value);
        self
    }

    /// Set encryption_algorithm (unwraps Option)
    pub fn with_encryption_algorithm(
        mut self,
        value: PhaseOneConfigDetailsEncryptionAlgorithm,
    ) -> Self {
        self.encryption_algorithm = Some(value);
        self
    }

    /// Set diffie_helman_group (unwraps Option)
    pub fn with_diffie_helman_group(
        mut self,
        value: PhaseOneConfigDetailsDiffieHelmanGroup,
    ) -> Self {
        self.diffie_helman_group = Some(value);
        self
    }

    /// Set lifetime_in_seconds (unwraps Option)
    pub fn with_lifetime_in_seconds(mut self, value: i64) -> Self {
        self.lifetime_in_seconds = Some(value);
        self
    }
}

impl Default for PhaseOneConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
