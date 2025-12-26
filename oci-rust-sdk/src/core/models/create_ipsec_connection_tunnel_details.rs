use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIPSecConnectionTunnelDetails {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The type of routing to use for this tunnel (BGP dynamic routing, static routing, or policy-based routing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<CreateIPSecConnectionTunnelDetailsRouting>,

    /// Internet Key Exchange protocol version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ike_version: Option<CreateIPSecConnectionTunnelDetailsIkeVersion>,

    /// The shared secret (pre-shared key) to use for the IPSec tunnel. Only numbers, letters, and spaces are allowed. If you don't provide a value, Oracle generates a value for you. You can specify your own shared secret later if you like with {@link #updateIPSecConnectionTunnelSharedSecret(UpdateIPSecConnectionTunnelSharedSecretRequest) updateIPSecConnectionTunnelSharedSecret}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_secret: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_session_config: Option<CreateIPSecTunnelBgpSessionDetails>,

    /// Indicates whether the Oracle end of the IPSec connection is able to initiate starting up the IPSec tunnel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_initiation: Option<CreateIPSecConnectionTunnelDetailsOracleInitiation>,

    /// By default (the {@code AUTO} setting), IKE sends packets with a source and destination port set to 500, and when it detects that the port used to forward packets has changed (most likely because a NAT device is between the CPE device and the Oracle VPN headend) it will try to negotiate the use of NAT-T. <p> The {@code ENABLED} option sets the IKE protocol to use port 4500 instead of 500 and forces encapsulating traffic with the ESP protocol inside UDP packets. <p> The {@code DISABLED} option directs IKE to completely refuse to negotiate NAT-T even if it senses there may be a NAT device in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_translation_enabled: Option<CreateIPSecConnectionTunnelDetailsNatTranslationEnabled>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_one_config: Option<PhaseOneConfigDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_two_config: Option<PhaseTwoConfigDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpd_config: Option<DpdConfig>,

    /// The headend IP that you can choose on the Oracle side to terminate your private IPSec tunnel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_tunnel_ip: Option<String>,

    /// The list of virtual circuit [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)s over which your network can reach this tunnel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_virtual_circuits: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table assigned to this attachment. <p> The DRG route table manages traffic inside the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drg_route_table_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_domain_config: Option<CreateIPSecTunnelEncryptionDomainDetails>,
}

impl CreateIPSecConnectionTunnelDetails {
    /// Create a new CreateIPSecConnectionTunnelDetails
    pub fn new() -> Self {
        Self {
            display_name: None,

            routing: None,

            ike_version: None,

            shared_secret: None,

            bgp_session_config: None,

            oracle_initiation: None,

            nat_translation_enabled: None,

            phase_one_config: None,

            phase_two_config: None,

            dpd_config: None,

            oracle_tunnel_ip: None,

            associated_virtual_circuits: None,

            drg_route_table_id: None,

            encryption_domain_config: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set routing
    pub fn set_routing(mut self, value: Option<CreateIPSecConnectionTunnelDetailsRouting>) -> Self {
        self.routing = value;
        self
    }

    /// Set ike_version
    pub fn set_ike_version(
        mut self,
        value: Option<CreateIPSecConnectionTunnelDetailsIkeVersion>,
    ) -> Self {
        self.ike_version = value;
        self
    }

    /// Set shared_secret
    pub fn set_shared_secret(mut self, value: Option<String>) -> Self {
        self.shared_secret = value;
        self
    }

    /// Set bgp_session_config
    pub fn set_bgp_session_config(
        mut self,
        value: Option<CreateIPSecTunnelBgpSessionDetails>,
    ) -> Self {
        self.bgp_session_config = value;
        self
    }

    /// Set oracle_initiation
    pub fn set_oracle_initiation(
        mut self,
        value: Option<CreateIPSecConnectionTunnelDetailsOracleInitiation>,
    ) -> Self {
        self.oracle_initiation = value;
        self
    }

    /// Set nat_translation_enabled
    pub fn set_nat_translation_enabled(
        mut self,
        value: Option<CreateIPSecConnectionTunnelDetailsNatTranslationEnabled>,
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

    /// Set oracle_tunnel_ip
    pub fn set_oracle_tunnel_ip(mut self, value: Option<String>) -> Self {
        self.oracle_tunnel_ip = value;
        self
    }

    /// Set associated_virtual_circuits
    pub fn set_associated_virtual_circuits(mut self, value: Option<Vec<String>>) -> Self {
        self.associated_virtual_circuits = value;
        self
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: Option<String>) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set encryption_domain_config
    pub fn set_encryption_domain_config(
        mut self,
        value: Option<CreateIPSecTunnelEncryptionDomainDetails>,
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
    pub fn with_routing(mut self, value: CreateIPSecConnectionTunnelDetailsRouting) -> Self {
        self.routing = Some(value);
        self
    }

    /// Set ike_version (unwraps Option)
    pub fn with_ike_version(mut self, value: CreateIPSecConnectionTunnelDetailsIkeVersion) -> Self {
        self.ike_version = Some(value);
        self
    }

    /// Set shared_secret (unwraps Option)
    pub fn with_shared_secret(mut self, value: impl Into<String>) -> Self {
        self.shared_secret = Some(value.into());
        self
    }

    /// Set bgp_session_config (unwraps Option)
    pub fn with_bgp_session_config(mut self, value: CreateIPSecTunnelBgpSessionDetails) -> Self {
        self.bgp_session_config = Some(value);
        self
    }

    /// Set oracle_initiation (unwraps Option)
    pub fn with_oracle_initiation(
        mut self,
        value: CreateIPSecConnectionTunnelDetailsOracleInitiation,
    ) -> Self {
        self.oracle_initiation = Some(value);
        self
    }

    /// Set nat_translation_enabled (unwraps Option)
    pub fn with_nat_translation_enabled(
        mut self,
        value: CreateIPSecConnectionTunnelDetailsNatTranslationEnabled,
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

    /// Set oracle_tunnel_ip (unwraps Option)
    pub fn with_oracle_tunnel_ip(mut self, value: impl Into<String>) -> Self {
        self.oracle_tunnel_ip = Some(value.into());
        self
    }

    /// Set associated_virtual_circuits (unwraps Option)
    pub fn with_associated_virtual_circuits(mut self, value: Vec<String>) -> Self {
        self.associated_virtual_circuits = Some(value);
        self
    }

    /// Set drg_route_table_id (unwraps Option)
    pub fn with_drg_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.drg_route_table_id = Some(value.into());
        self
    }

    /// Set encryption_domain_config (unwraps Option)
    pub fn with_encryption_domain_config(
        mut self,
        value: CreateIPSecTunnelEncryptionDomainDetails,
    ) -> Self {
        self.encryption_domain_config = Some(value);
        self
    }
}

impl Default for CreateIPSecConnectionTunnelDetails {
    fn default() -> Self {
        Self::new()
    }
}
