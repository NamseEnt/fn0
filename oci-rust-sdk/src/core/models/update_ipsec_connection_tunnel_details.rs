use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIPSecConnectionTunnelDetails {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The type of routing to use for this tunnel (BGP dynamic routing, static routing, or policy-based routing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<UpdateIPSecConnectionTunnelDetailsRouting>,

    /// Internet Key Exchange protocol version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ike_version: Option<UpdateIPSecConnectionTunnelDetailsIkeVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_session_config: Option<UpdateIPSecTunnelBgpSessionDetails>,

    /// Indicates whether the Oracle end of the IPSec connection is able to initiate starting up the IPSec tunnel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_initiation: Option<UpdateIPSecConnectionTunnelDetailsOracleInitiation>,

    /// By default (the {@code AUTO} setting), IKE sends packets with a source and destination port set to 500, and when it detects that the port used to forward packets has changed (most likely because a NAT device is between the CPE device and the Oracle VPN headend) it will try to negotiate the use of NAT-T. <p> The {@code ENABLED} option sets the IKE protocol to use port 4500 instead of 500 and forces encapsulating traffic with the ESP protocol inside UDP packets. <p> The {@code DISABLED} option directs IKE to completely refuse to negotiate NAT-T even if it senses there may be a NAT device in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_translation_enabled: Option<UpdateIPSecConnectionTunnelDetailsNatTranslationEnabled>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_one_config: Option<PhaseOneConfigDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_two_config: Option<PhaseTwoConfigDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpd_config: Option<DpdConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_domain_config: Option<UpdateIPSecTunnelEncryptionDomainDetails>,
}

impl UpdateIPSecConnectionTunnelDetails {
    /// Create a new UpdateIPSecConnectionTunnelDetails
    pub fn new() -> Self {
        Self {
            display_name: None,

            routing: None,

            ike_version: None,

            bgp_session_config: None,

            oracle_initiation: None,

            nat_translation_enabled: None,

            phase_one_config: None,

            phase_two_config: None,

            dpd_config: None,

            encryption_domain_config: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set routing
    pub fn set_routing(mut self, value: Option<UpdateIPSecConnectionTunnelDetailsRouting>) -> Self {
        self.routing = value;
        self
    }

    /// Set ike_version
    pub fn set_ike_version(
        mut self,
        value: Option<UpdateIPSecConnectionTunnelDetailsIkeVersion>,
    ) -> Self {
        self.ike_version = value;
        self
    }

    /// Set bgp_session_config
    pub fn set_bgp_session_config(
        mut self,
        value: Option<UpdateIPSecTunnelBgpSessionDetails>,
    ) -> Self {
        self.bgp_session_config = value;
        self
    }

    /// Set oracle_initiation
    pub fn set_oracle_initiation(
        mut self,
        value: Option<UpdateIPSecConnectionTunnelDetailsOracleInitiation>,
    ) -> Self {
        self.oracle_initiation = value;
        self
    }

    /// Set nat_translation_enabled
    pub fn set_nat_translation_enabled(
        mut self,
        value: Option<UpdateIPSecConnectionTunnelDetailsNatTranslationEnabled>,
    ) -> Self {
        self.nat_translation_enabled = value;
        self
    }

    /// Set phase_one_config
    pub fn set_phase_one_config(mut self, value: Option<PhaseOneConfigDetails>) -> Self {
        self.phase_one_config = value;
        self
    }

    /// Set phase_two_config
    pub fn set_phase_two_config(mut self, value: Option<PhaseTwoConfigDetails>) -> Self {
        self.phase_two_config = value;
        self
    }

    /// Set dpd_config
    pub fn set_dpd_config(mut self, value: Option<DpdConfig>) -> Self {
        self.dpd_config = value;
        self
    }

    /// Set encryption_domain_config
    pub fn set_encryption_domain_config(
        mut self,
        value: Option<UpdateIPSecTunnelEncryptionDomainDetails>,
    ) -> Self {
        self.encryption_domain_config = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set routing (unwraps Option)
    pub fn with_routing(mut self, value: UpdateIPSecConnectionTunnelDetailsRouting) -> Self {
        self.routing = Some(value);
        self
    }

    /// Set ike_version (unwraps Option)
    pub fn with_ike_version(mut self, value: UpdateIPSecConnectionTunnelDetailsIkeVersion) -> Self {
        self.ike_version = Some(value);
        self
    }

    /// Set bgp_session_config (unwraps Option)
    pub fn with_bgp_session_config(mut self, value: UpdateIPSecTunnelBgpSessionDetails) -> Self {
        self.bgp_session_config = Some(value);
        self
    }

    /// Set oracle_initiation (unwraps Option)
    pub fn with_oracle_initiation(
        mut self,
        value: UpdateIPSecConnectionTunnelDetailsOracleInitiation,
    ) -> Self {
        self.oracle_initiation = Some(value);
        self
    }

    /// Set nat_translation_enabled (unwraps Option)
    pub fn with_nat_translation_enabled(
        mut self,
        value: UpdateIPSecConnectionTunnelDetailsNatTranslationEnabled,
    ) -> Self {
        self.nat_translation_enabled = Some(value);
        self
    }

    /// Set phase_one_config (unwraps Option)
    pub fn with_phase_one_config(mut self, value: PhaseOneConfigDetails) -> Self {
        self.phase_one_config = Some(value);
        self
    }

    /// Set phase_two_config (unwraps Option)
    pub fn with_phase_two_config(mut self, value: PhaseTwoConfigDetails) -> Self {
        self.phase_two_config = Some(value);
        self
    }

    /// Set dpd_config (unwraps Option)
    pub fn with_dpd_config(mut self, value: DpdConfig) -> Self {
        self.dpd_config = Some(value);
        self
    }

    /// Set encryption_domain_config (unwraps Option)
    pub fn with_encryption_domain_config(
        mut self,
        value: UpdateIPSecTunnelEncryptionDomainDetails,
    ) -> Self {
        self.encryption_domain_config = Some(value);
        self
    }
}

impl Default for UpdateIPSecConnectionTunnelDetails {
    fn default() -> Self {
        Self::new()
    }
}
