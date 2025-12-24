use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about 'ByoipRange' that has {@code byoasn} as origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoasnByoipRange {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource to which the CIDR block belongs.
    pub byoip_range_id: String,

    /// The BYOIP CIDR block range or subrange allocated to an IP pool. This could be all or part of a BYOIP CIDR block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,

    /// The IPv6 prefix being imported to the Oracle cloud. This prefix must be /48 or larger, and can  be subdivided into sub-ranges used across multiple VCNs. A BYOIPv6 prefix can be assigned across multiple VCNs, and each VCN must be /64 or larger. You may specify a ULA or private IPv6 prefix of /64 or larger to use in the VCN. IPv6-enabled subnets will remain a fixed /64 in size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_block: Option<String>,

    /// The as path prepend length. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_path_prepend_length: Option<i64>,
}

/// Required fields for ByoasnByoipRange
pub struct ByoasnByoipRangeRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource to which the CIDR block belongs.
    pub byoip_range_id: String,
}

impl ByoasnByoipRange {
    /// Create a new ByoasnByoipRange with required fields
    pub fn new(required: ByoasnByoipRangeRequired) -> Self {
        Self {
            byoip_range_id: required.byoip_range_id,

            cidr_block: None,

            ipv6_cidr_block: None,

            as_path_prepend_length: None,
        }
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: Option<String>) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set ipv6_cidr_block
    pub fn set_ipv6_cidr_block(mut self, value: Option<String>) -> Self {
        self.ipv6_cidr_block = value;
        self
    }

    /// Set byoip_range_id
    pub fn set_byoip_range_id(mut self, value: String) -> Self {
        self.byoip_range_id = value;
        self
    }

    /// Set as_path_prepend_length
    pub fn set_as_path_prepend_length(mut self, value: Option<i64>) -> Self {
        self.as_path_prepend_length = value;
        self
    }

    /// Set cidr_block (unwraps Option)
    pub fn with_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.cidr_block = Some(value.into());
        self
    }

    /// Set ipv6_cidr_block (unwraps Option)
    pub fn with_ipv6_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.ipv6_cidr_block = Some(value.into());
        self
    }

    /// Set as_path_prepend_length (unwraps Option)
    pub fn with_as_path_prepend_length(mut self, value: i64) -> Self {
        self.as_path_prepend_length = Some(value);
        self
    }
}
