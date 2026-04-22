use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Optional and valid only for ICMP and ICMPv6. Use to specify a particular ICMP type and code as defined in: - [ICMP Parameters](http://www.iana.org/assignments/icmp-parameters/icmp-parameters.xhtml) - [ICMPv6 Parameters](https://www.iana.org/assignments/icmpv6-parameters/icmpv6-parameters.xhtml) <p> If you specify ICMP or ICMPv6 as the protocol but omit this object, then all ICMP types and codes are allowed. If you do provide this object, the type is required and the code is optional. To enable MTU negotiation for ingress internet traffic via IPv4, make sure to allow type 3 (\"Destination Unreachable\") code 4 (\"Fragmentation Needed and Don't Fragment was Set\"). If you need to specify multiple codes for a single type, create a separate security list rule for each.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IcmpOptions {
    /// The ICMP type. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(rename = "type")]
    pub r#type: i64,

    /// The ICMP code (optional). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<i64>,
}

/// Required fields for IcmpOptions
pub struct IcmpOptionsRequired {
    /// The ICMP type. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub r#type: i64,
}

impl IcmpOptions {
    /// Create a new IcmpOptions with required fields
    pub fn new(required: IcmpOptionsRequired) -> Self {
        Self {
            r#type: required.r#type,

            code: None,
        }
    }

    /// Set code
    pub fn set_code(mut self, value: Option<i64>) -> Self {
        self.code = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: i64) -> Self {
        self.r#type = value;
        self
    }

    /// Set code (unwraps Option)
    pub fn with_code(mut self, value: i64) -> Self {
        self.code = Some(value);
        self
    }
}
