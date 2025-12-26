use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// This resource provides redundancy level details for the virtual circuit. For more about redundancy, see [FastConnect Redundancy Best Practices](https://docs.oracle.com/iaas/Content/Network/Concepts/fastconnectresiliency.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualCircuitRedundancyMetadata {
    /// The configured redundancy level of the virtual circuit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_redundancy_level:
        Option<VirtualCircuitRedundancyMetadataConfiguredRedundancyLevel>,

    /// Indicates if the configured level is met for IPv4 BGP redundancy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4bgp_session_redundancy_status:
        Option<VirtualCircuitRedundancyMetadataIpv4bgpSessionRedundancyStatus>,

    /// Indicates if the configured level is met for IPv6 BGP redundancy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6bgp_session_redundancy_status:
        Option<VirtualCircuitRedundancyMetadataIpv6bgpSessionRedundancyStatus>,
}

impl VirtualCircuitRedundancyMetadata {
    /// Create a new VirtualCircuitRedundancyMetadata
    pub fn new() -> Self {
        Self {
            configured_redundancy_level: None,

            ipv4bgp_session_redundancy_status: None,

            ipv6bgp_session_redundancy_status: None,
        }
    }

    /// Set configured_redundancy_level
    pub fn set_configured_redundancy_level(
        mut self,
        value: Option<VirtualCircuitRedundancyMetadataConfiguredRedundancyLevel>,
    ) -> Self {
        self.configured_redundancy_level = value;
        self
    }

    /// Set ipv4bgp_session_redundancy_status
    pub fn set_ipv4bgp_session_redundancy_status(
        mut self,
        value: Option<VirtualCircuitRedundancyMetadataIpv4bgpSessionRedundancyStatus>,
    ) -> Self {
        self.ipv4bgp_session_redundancy_status = value;
        self
    }

    /// Set ipv6bgp_session_redundancy_status
    pub fn set_ipv6bgp_session_redundancy_status(
        mut self,
        value: Option<VirtualCircuitRedundancyMetadataIpv6bgpSessionRedundancyStatus>,
    ) -> Self {
        self.ipv6bgp_session_redundancy_status = value;
        self
    }

    /// Set configured_redundancy_level (unwraps Option)
    pub fn with_configured_redundancy_level(
        mut self,
        value: VirtualCircuitRedundancyMetadataConfiguredRedundancyLevel,
    ) -> Self {
        self.configured_redundancy_level = Some(value);
        self
    }

    /// Set ipv4bgp_session_redundancy_status (unwraps Option)
    pub fn with_ipv4bgp_session_redundancy_status(
        mut self,
        value: VirtualCircuitRedundancyMetadataIpv4bgpSessionRedundancyStatus,
    ) -> Self {
        self.ipv4bgp_session_redundancy_status = Some(value);
        self
    }

    /// Set ipv6bgp_session_redundancy_status (unwraps Option)
    pub fn with_ipv6bgp_session_redundancy_status(
        mut self,
        value: VirtualCircuitRedundancyMetadataIpv6bgpSessionRedundancyStatus,
    ) -> Self {
        self.ipv6bgp_session_redundancy_status = Some(value);
        self
    }
}

impl Default for VirtualCircuitRedundancyMetadata {
    fn default() -> Self {
        Self::new()
    }
}
