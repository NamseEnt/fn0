use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the origin asn.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByoipRangeOriginAsn {
    /// The Autonomous System Number (ASN) you are importing to the Oracle cloud. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub asn: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byoasn_id: Option<String>,

    /// The as path prepend length. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_path_prepend_length: Option<i64>,
}

/// Required fields for ByoipRangeOriginAsn
pub struct ByoipRangeOriginAsnRequired {
    /// The Autonomous System Number (ASN) you are importing to the Oracle cloud. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub asn: i64,
}

impl ByoipRangeOriginAsn {
    /// Create a new ByoipRangeOriginAsn with required fields
    pub fn new(required: ByoipRangeOriginAsnRequired) -> Self {
        Self {
            asn: required.asn,

            byoasn_id: None,

            as_path_prepend_length: None,
        }
    }

    /// Set byoasn_id
    pub fn set_byoasn_id(mut self, value: Option<String>) -> Self {
        self.byoasn_id = value;
        self
    }

    /// Set asn
    pub fn set_asn(mut self, value: i64) -> Self {
        self.asn = value;
        self
    }

    /// Set as_path_prepend_length
    pub fn set_as_path_prepend_length(mut self, value: Option<i64>) -> Self {
        self.as_path_prepend_length = value;
        self
    }

    /// Set byoasn_id (unwraps Option)
    pub fn with_byoasn_id(mut self, value: impl Into<String>) -> Self {
        self.byoasn_id = Some(value.into());
        self
    }

    /// Set as_path_prepend_length (unwraps Option)
    pub fn with_as_path_prepend_length(mut self, value: i64) -> Self {
        self.as_path_prepend_length = Some(value);
        self
    }
}
