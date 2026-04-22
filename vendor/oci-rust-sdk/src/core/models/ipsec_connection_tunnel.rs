use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about a single IPSec tunnel in an IPSec connection. This object does not include the tunnel's shared secret (pre-shared key), which is found in the {@link IPSecConnectionTunnelSharedSecret} object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPSecConnectionTunnel {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the tunnel.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tunnel.
    pub id: String,

    /// The tunnel's lifecycle state.
    pub lifecycle_state: IPSecConnectionTunnelLifecycleState,

    /// The IP address of the Oracle VPN headend for the connection. <p> Example: {@code 203.0.113.21}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpn_ip: Option<String>,

    /// The IP address of the CPE device's VPN headend. <p> Example: {@code 203.0.113.22}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_ip: Option<String>,

    /// The status of the tunnel based on IPSec protocol characteristics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<IPSecConnectionTunnelStatus>,

    /// Internet Key Exchange protocol version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ike_version: Option<IPSecConnectionTunnelIkeVersion>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgp_session_info: Option<BgpSessionInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_domain_config: Option<EncryptionDomainConfig>,

    /// The type of routing used for this tunnel (BGP dynamic routing, static routing, or policy-based routing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<IPSecConnectionTunnelRouting>,

    /// The date and time the IPSec tunnel was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// When the status of the IPSec tunnel last changed, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_status_updated: Option<DateTime<Utc>>,

    /// Indicates whether Oracle can only respond to a request to start an IPSec tunnel from the CPE device, or both respond to and initiate requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_can_initiate: Option<IPSecConnectionTunnelOracleCanInitiate>,

    /// By default (the {@code AUTO} setting), IKE sends packets with a source and destination port set to 500, and when it detects that the port used to forward packets has changed (most likely because a NAT device is between the CPE device and the Oracle VPN headend) it will try to negotiate the use of NAT-T. <p> The {@code ENABLED} option sets the IKE protocol to use port 4500 instead of 500 and forces encapsulating traffic with the ESP protocol inside UDP packets. <p> The {@code DISABLED} option directs IKE to completely refuse to negotiate NAT-T even if it senses there may be a NAT device in use. <p> .
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_translation_enabled: Option<IPSecConnectionTunnelNatTranslationEnabled>,

    /// Dead peer detection (DPD) mode set on the Oracle side of the connection. This mode sets whether Oracle can only respond to a request from the CPE device to start DPD, or both respond to and initiate requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpd_mode: Option<IPSecConnectionTunnelDpdMode>,

    /// DPD timeout in seconds. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpd_timeout_in_sec: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_one_details: Option<TunnelPhaseOneDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_two_details: Option<TunnelPhaseTwoDetails>,

    /// The list of virtual circuit [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)s over which your network can reach this tunnel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_virtual_circuits: Option<Vec<String>>,
}

/// Required fields for IPSecConnectionTunnel
pub struct IPSecConnectionTunnelRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the tunnel.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tunnel.
    pub id: String,

    /// The tunnel's lifecycle state.
    pub lifecycle_state: IPSecConnectionTunnelLifecycleState,
}

impl IPSecConnectionTunnel {
    /// Create a new IPSecConnectionTunnel with required fields
    pub fn new(required: IPSecConnectionTunnelRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            vpn_ip: None,

            cpe_ip: None,

            status: None,

            ike_version: None,

            display_name: None,

            bgp_session_info: None,

            encryption_domain_config: None,

            routing: None,

            time_created: None,

            time_status_updated: None,

            oracle_can_initiate: None,

            nat_translation_enabled: None,

            dpd_mode: None,

            dpd_timeout_in_sec: None,

            phase_one_details: None,

            phase_two_details: None,

            associated_virtual_circuits: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set vpn_ip
    pub fn set_vpn_ip(mut self, value: Option<String>) -> Self {
        self.vpn_ip = value;
        self
    }

    /// Set cpe_ip
    pub fn set_cpe_ip(mut self, value: Option<String>) -> Self {
        self.cpe_ip = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: Option<IPSecConnectionTunnelStatus>) -> Self {
        self.status = value;
        self
    }

    /// Set ike_version
    pub fn set_ike_version(mut self, value: Option<IPSecConnectionTunnelIkeVersion>) -> Self {
        self.ike_version = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: IPSecConnectionTunnelLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set bgp_session_info
    pub fn set_bgp_session_info(mut self, value: Option<BgpSessionInfo>) -> Self {
        self.bgp_session_info = value;
        self
    }

    /// Set encryption_domain_config
    pub fn set_encryption_domain_config(mut self, value: Option<EncryptionDomainConfig>) -> Self {
        self.encryption_domain_config = value;
        self
    }

    /// Set routing
    pub fn set_routing(mut self, value: Option<IPSecConnectionTunnelRouting>) -> Self {
        self.routing = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_status_updated
    pub fn set_time_status_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_status_updated = value;
        self
    }

    /// Set oracle_can_initiate
    pub fn set_oracle_can_initiate(
        mut self,
        value: Option<IPSecConnectionTunnelOracleCanInitiate>,
    ) -> Self {
        self.oracle_can_initiate = value;
        self
    }

    /// Set nat_translation_enabled
    pub fn set_nat_translation_enabled(
        mut self,
        value: Option<IPSecConnectionTunnelNatTranslationEnabled>,
    ) -> Self {
        self.nat_translation_enabled = value;
        self
    }

    /// Set dpd_mode
    pub fn set_dpd_mode(mut self, value: Option<IPSecConnectionTunnelDpdMode>) -> Self {
        self.dpd_mode = value;
        self
    }

    /// Set dpd_timeout_in_sec
    pub fn set_dpd_timeout_in_sec(mut self, value: Option<i64>) -> Self {
        self.dpd_timeout_in_sec = value;
        self
    }

    /// Set phase_one_details
    pub fn set_phase_one_details(mut self, value: Option<TunnelPhaseOneDetails>) -> Self {
        self.phase_one_details = value;
        self
    }

    /// Set phase_two_details
    pub fn set_phase_two_details(mut self, value: Option<TunnelPhaseTwoDetails>) -> Self {
        self.phase_two_details = value;
        self
    }

    /// Set associated_virtual_circuits
    pub fn set_associated_virtual_circuits(mut self, value: Option<Vec<String>>) -> Self {
        self.associated_virtual_circuits = value;
        self
    }

    /// Set vpn_ip (unwraps Option)
    pub fn with_vpn_ip(mut self, value: impl Into<String>) -> Self {
        self.vpn_ip = Some(value.into());
        self
    }

    /// Set cpe_ip (unwraps Option)
    pub fn with_cpe_ip(mut self, value: impl Into<String>) -> Self {
        self.cpe_ip = Some(value.into());
        self
    }

    /// Set status (unwraps Option)
    pub fn with_status(mut self, value: IPSecConnectionTunnelStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set ike_version (unwraps Option)
    pub fn with_ike_version(mut self, value: IPSecConnectionTunnelIkeVersion) -> Self {
        self.ike_version = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set bgp_session_info (unwraps Option)
    pub fn with_bgp_session_info(mut self, value: BgpSessionInfo) -> Self {
        self.bgp_session_info = Some(value);
        self
    }

    /// Set encryption_domain_config (unwraps Option)
    pub fn with_encryption_domain_config(mut self, value: EncryptionDomainConfig) -> Self {
        self.encryption_domain_config = Some(value);
        self
    }

    /// Set routing (unwraps Option)
    pub fn with_routing(mut self, value: IPSecConnectionTunnelRouting) -> Self {
        self.routing = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set time_status_updated (unwraps Option)
    pub fn with_time_status_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_status_updated = Some(value);
        self
    }

    /// Set oracle_can_initiate (unwraps Option)
    pub fn with_oracle_can_initiate(
        mut self,
        value: IPSecConnectionTunnelOracleCanInitiate,
    ) -> Self {
        self.oracle_can_initiate = Some(value);
        self
    }

    /// Set nat_translation_enabled (unwraps Option)
    pub fn with_nat_translation_enabled(
        mut self,
        value: IPSecConnectionTunnelNatTranslationEnabled,
    ) -> Self {
        self.nat_translation_enabled = Some(value);
        self
    }

    /// Set dpd_mode (unwraps Option)
    pub fn with_dpd_mode(mut self, value: IPSecConnectionTunnelDpdMode) -> Self {
        self.dpd_mode = Some(value);
        self
    }

    /// Set dpd_timeout_in_sec (unwraps Option)
    pub fn with_dpd_timeout_in_sec(mut self, value: i64) -> Self {
        self.dpd_timeout_in_sec = Some(value);
        self
    }

    /// Set phase_one_details (unwraps Option)
    pub fn with_phase_one_details(mut self, value: TunnelPhaseOneDetails) -> Self {
        self.phase_one_details = Some(value);
        self
    }

    /// Set phase_two_details (unwraps Option)
    pub fn with_phase_two_details(mut self, value: TunnelPhaseTwoDetails) -> Self {
        self.phase_two_details = Some(value);
        self
    }

    /// Set associated_virtual_circuits (unwraps Option)
    pub fn with_associated_virtual_circuits(mut self, value: Vec<String>) -> Self {
        self.associated_virtual_circuits = Some(value);
        self
    }
}
