use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// This resource contains the rules governing what traffic a VTAP mirrors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VtapCaptureFilterRuleDetails {
    /// The traffic direction the VTAP is configured to mirror.
    pub traffic_direction: VtapCaptureFilterRuleDetailsTrafficDirection,

    /// Include or exclude packets meeting this definition from mirrored traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_action: Option<VtapCaptureFilterRuleDetailsRuleAction>,

    /// Traffic from this CIDR block to the VTAP source will be mirrored to the VTAP target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_cidr: Option<String>,

    /// Traffic sent to this CIDR block through the VTAP source will be mirrored to the VTAP target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_cidr: Option<String>,

    /// The transport protocol used in the filter. If do not choose a protocol, all protocols will be used in the filter. Supported options are: * 1 = ICMP * 6 = TCP * 17 = UDP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icmp_options: Option<IcmpOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_options: Option<TcpOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_options: Option<UdpOptions>,
}

/// Required fields for VtapCaptureFilterRuleDetails
pub struct VtapCaptureFilterRuleDetailsRequired {
    /// The traffic direction the VTAP is configured to mirror.
    pub traffic_direction: VtapCaptureFilterRuleDetailsTrafficDirection,
}

impl VtapCaptureFilterRuleDetails {
    /// Create a new VtapCaptureFilterRuleDetails with required fields
    pub fn new(required: VtapCaptureFilterRuleDetailsRequired) -> Self {
        Self {
            traffic_direction: required.traffic_direction,

            rule_action: None,

            source_cidr: None,

            destination_cidr: None,

            protocol: None,

            icmp_options: None,

            tcp_options: None,

            udp_options: None,
        }
    }

    /// Set traffic_direction
    pub fn set_traffic_direction(
        mut self,
        value: VtapCaptureFilterRuleDetailsTrafficDirection,
    ) -> Self {
        self.traffic_direction = value;
        self
    }

    /// Set rule_action
    pub fn set_rule_action(
        mut self,
        value: Option<VtapCaptureFilterRuleDetailsRuleAction>,
    ) -> Self {
        self.rule_action = value;
        self
    }

    /// Set source_cidr
    pub fn set_source_cidr(mut self, value: Option<String>) -> Self {
        self.source_cidr = value;
        self
    }

    /// Set destination_cidr
    pub fn set_destination_cidr(mut self, value: Option<String>) -> Self {
        self.destination_cidr = value;
        self
    }

    /// Set protocol
    pub fn set_protocol(mut self, value: Option<String>) -> Self {
        self.protocol = value;
        self
    }

    /// Set icmp_options
    pub fn set_icmp_options(mut self, value: Option<IcmpOptions>) -> Self {
        self.icmp_options = value;
        self
    }

    /// Set tcp_options
    pub fn set_tcp_options(mut self, value: Option<TcpOptions>) -> Self {
        self.tcp_options = value;
        self
    }

    /// Set udp_options
    pub fn set_udp_options(mut self, value: Option<UdpOptions>) -> Self {
        self.udp_options = value;
        self
    }

    /// Set rule_action (unwraps Option)
    pub fn with_rule_action(mut self, value: VtapCaptureFilterRuleDetailsRuleAction) -> Self {
        self.rule_action = Some(value);
        self
    }

    /// Set source_cidr (unwraps Option)
    pub fn with_source_cidr(mut self, value: impl Into<String>) -> Self {
        self.source_cidr = Some(value.into());
        self
    }

    /// Set destination_cidr (unwraps Option)
    pub fn with_destination_cidr(mut self, value: impl Into<String>) -> Self {
        self.destination_cidr = Some(value.into());
        self
    }

    /// Set protocol (unwraps Option)
    pub fn with_protocol(mut self, value: impl Into<String>) -> Self {
        self.protocol = Some(value.into());
        self
    }

    /// Set icmp_options (unwraps Option)
    pub fn with_icmp_options(mut self, value: IcmpOptions) -> Self {
        self.icmp_options = Some(value);
        self
    }

    /// Set tcp_options (unwraps Option)
    pub fn with_tcp_options(mut self, value: TcpOptions) -> Self {
        self.tcp_options = Some(value);
        self
    }

    /// Set udp_options (unwraps Option)
    pub fn with_udp_options(mut self, value: UdpOptions) -> Self {
        self.udp_options = Some(value);
        self
    }
}
