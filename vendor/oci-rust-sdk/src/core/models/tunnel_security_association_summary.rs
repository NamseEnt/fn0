use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A summary of the IPSec tunnel security association details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelSecurityAssociationSummary {
    /// The IP address and mask of the partner subnet used in policy based VPNs or static routes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_subnet: Option<String>,

    /// The IP address and mask of the local subnet used in policy based VPNs or static routes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_subnet: Option<String>,

    /// The IPSec tunnel's phase one status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_sa_status: Option<TunnelSecurityAssociationSummaryTunnelSaStatus>,

    /// Current state if the IPSec tunnel status is not {@code UP}, including phase one and phase two details and a possible reason the tunnel is not {@code UP}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_sa_error_info: Option<String>,

    /// Time in the current state, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
}

impl TunnelSecurityAssociationSummary {
    /// Create a new TunnelSecurityAssociationSummary
    pub fn new() -> Self {
        Self {
            cpe_subnet: None,

            oracle_subnet: None,

            tunnel_sa_status: None,

            tunnel_sa_error_info: None,

            time: None,
        }
    }

    /// Set cpe_subnet
    pub fn set_cpe_subnet(mut self, value: Option<String>) -> Self {
        self.cpe_subnet = value;
        self
    }

    /// Set oracle_subnet
    pub fn set_oracle_subnet(mut self, value: Option<String>) -> Self {
        self.oracle_subnet = value;
        self
    }

    /// Set tunnel_sa_status
    pub fn set_tunnel_sa_status(
        mut self,
        value: Option<TunnelSecurityAssociationSummaryTunnelSaStatus>,
    ) -> Self {
        self.tunnel_sa_status = value;
        self
    }

    /// Set tunnel_sa_error_info
    pub fn set_tunnel_sa_error_info(mut self, value: Option<String>) -> Self {
        self.tunnel_sa_error_info = value;
        self
    }

    /// Set time
    pub fn set_time(mut self, value: Option<String>) -> Self {
        self.time = value;
        self
    }

    /// Set cpe_subnet (unwraps Option)
    pub fn with_cpe_subnet(mut self, value: impl Into<String>) -> Self {
        self.cpe_subnet = Some(value.into());
        self
    }

    /// Set oracle_subnet (unwraps Option)
    pub fn with_oracle_subnet(mut self, value: impl Into<String>) -> Self {
        self.oracle_subnet = Some(value.into());
        self
    }

    /// Set tunnel_sa_status (unwraps Option)
    pub fn with_tunnel_sa_status(
        mut self,
        value: TunnelSecurityAssociationSummaryTunnelSaStatus,
    ) -> Self {
        self.tunnel_sa_status = Some(value);
        self
    }

    /// Set tunnel_sa_error_info (unwraps Option)
    pub fn with_tunnel_sa_error_info(mut self, value: impl Into<String>) -> Self {
        self.tunnel_sa_error_info = Some(value.into());
        self
    }

    /// Set time (unwraps Option)
    pub fn with_time(mut self, value: impl Into<String>) -> Self {
        self.time = Some(value.into());
        self
    }
}

impl Default for TunnelSecurityAssociationSummary {
    fn default() -> Self {
        Self::new()
    }
}
