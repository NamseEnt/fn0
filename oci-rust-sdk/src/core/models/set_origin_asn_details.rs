use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Update Origin ASN of a BYOIP Range
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetOriginAsnDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} Resource to be associated.
    pub byoasn_id: String,

    /// The as path prepend length. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_path_prepend_length: Option<i64>,
}

/// Required fields for SetOriginAsnDetails
pub struct SetOriginAsnDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code Byoasn} Resource to be associated.
    pub byoasn_id: String,
}

impl SetOriginAsnDetails {
    /// Create a new SetOriginAsnDetails with required fields
    pub fn new(required: SetOriginAsnDetailsRequired) -> Self {
        Self {
            byoasn_id: required.byoasn_id,

            as_path_prepend_length: None,
        }
    }

    /// Set byoasn_id
    pub fn set_byoasn_id(mut self, value: String) -> Self {
        self.byoasn_id = value;
        self
    }

    /// Set as_path_prepend_length
    pub fn set_as_path_prepend_length(mut self, value: Option<i64>) -> Self {
        self.as_path_prepend_length = value;
        self
    }

    /// Set as_path_prepend_length (unwraps Option)
    pub fn with_as_path_prepend_length(mut self, value: i64) -> Self {
        self.as_path_prepend_length = Some(value);
        self
    }
}
