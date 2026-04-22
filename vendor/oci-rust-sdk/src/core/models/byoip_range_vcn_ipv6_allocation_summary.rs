use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A summary of IPv6 prefix subranges currently allocated to a VCN.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoipRangeVcnIpv6AllocationSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource to which the CIDR block belongs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byoip_range_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the {@code ByoipRange}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The BYOIPv6 prefix range or subrange allocated to a VCN. This could be all or part of a BYOIPv6 prefix. Each VCN allocation must be /64 or larger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_block: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Vcn} resource to which the ByoipRange belongs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_id: Option<String>,
}

impl ByoipRangeVcnIpv6AllocationSummary {
    /// Create a new ByoipRangeVcnIpv6AllocationSummary
    pub fn new() -> Self {
        Self {
            byoip_range_id: None,

            compartment_id: None,

            ipv6_cidr_block: None,

            vcn_id: None,
        }
    }

    /// Set byoip_range_id
    pub fn set_byoip_range_id(mut self, value: Option<String>) -> Self {
        self.byoip_range_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set ipv6_cidr_block
    pub fn set_ipv6_cidr_block(mut self, value: Option<String>) -> Self {
        self.ipv6_cidr_block = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: Option<String>) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set byoip_range_id (unwraps Option)
    pub fn with_byoip_range_id(mut self, value: impl Into<String>) -> Self {
        self.byoip_range_id = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set ipv6_cidr_block (unwraps Option)
    pub fn with_ipv6_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.ipv6_cidr_block = Some(value.into());
        self
    }

    /// Set vcn_id (unwraps Option)
    pub fn with_vcn_id(mut self, value: impl Into<String>) -> Self {
        self.vcn_id = Some(value.into());
        self
    }
}

impl Default for ByoipRangeVcnIpv6AllocationSummary {
    fn default() -> Self {
        Self::new()
    }
}
