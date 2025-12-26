use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of rules governing what traffic the VCN flow log collects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowLogCaptureFilterRuleDetails {
    /// Indicates whether a VCN flow log capture filter rule is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,

    /// A lower number indicates a higher priority, range 0-9. Each rule must have a distinct priority. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,

    /// Sampling interval as {@code 1} of {@code X}, where {@code X} is an integer not greater than {@code 100000}. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<i64>,

    /// Traffic from this CIDR will be captured in the VCN flow log.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_cidr: Option<String>,

    /// Traffic to this CIDR will be captured in the VCN flow log.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_cidr: Option<String>,

    /// The transport protocol the filter uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icmp_options: Option<IcmpOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_options: Option<TcpOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_options: Option<UdpOptions>,

    /// Type or types of VCN flow logs to store. {@code ALL} includes records for both accepted traffic and rejected traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_log_type: Option<FlowLogCaptureFilterRuleDetailsFlowLogType>,

    /// Include or exclude a {@code ruleAction} object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_action: Option<FlowLogCaptureFilterRuleDetailsRuleAction>,
}

impl FlowLogCaptureFilterRuleDetails {
    /// Create a new FlowLogCaptureFilterRuleDetails
    pub fn new() -> Self {
        Self {
            is_enabled: None,

            priority: None,

            sampling_rate: None,

            source_cidr: None,

            destination_cidr: None,

            protocol: None,

            icmp_options: None,

            tcp_options: None,

            udp_options: None,

            flow_log_type: None,

            rule_action: None,
        }
    }

    /// Set is_enabled
    pub fn set_is_enabled(mut self, value: Option<bool>) -> Self {
        self.is_enabled = value;
        self
    }

    /// Set priority
    pub fn set_priority(mut self, value: Option<i64>) -> Self {
        self.priority = value;
        self
    }

    /// Set sampling_rate
    pub fn set_sampling_rate(mut self, value: Option<i64>) -> Self {
        self.sampling_rate = value;
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

    /// Set flow_log_type
    pub fn set_flow_log_type(
        mut self,
        value: Option<FlowLogCaptureFilterRuleDetailsFlowLogType>,
    ) -> Self {
        self.flow_log_type = value;
        self
    }

    /// Set rule_action
    pub fn set_rule_action(
        mut self,
        value: Option<FlowLogCaptureFilterRuleDetailsRuleAction>,
    ) -> Self {
        self.rule_action = value;
        self
    }

    /// Set is_enabled (unwraps Option)
    pub fn with_is_enabled(mut self, value: bool) -> Self {
        self.is_enabled = Some(value);
        self
    }

    /// Set priority (unwraps Option)
    pub fn with_priority(mut self, value: i64) -> Self {
        self.priority = Some(value);
        self
    }

    /// Set sampling_rate (unwraps Option)
    pub fn with_sampling_rate(mut self, value: i64) -> Self {
        self.sampling_rate = Some(value);
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

    /// Set flow_log_type (unwraps Option)
    pub fn with_flow_log_type(mut self, value: FlowLogCaptureFilterRuleDetailsFlowLogType) -> Self {
        self.flow_log_type = Some(value);
        self
    }

    /// Set rule_action (unwraps Option)
    pub fn with_rule_action(mut self, value: FlowLogCaptureFilterRuleDetailsRuleAction) -> Self {
        self.rule_action = Some(value);
        self
    }
}

impl Default for FlowLogCaptureFilterRuleDetails {
    fn default() -> Self {
        Self::new()
    }
}
