use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Secondary multipath device, it uses the charUsername and chapSecret from primary volume attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipathDevice {
    /// The volume's iSCSI IP address. <p> Example: {@code 169.254.2.2}
    pub ipv4: String,

    /// The target volume's iSCSI Qualified Name in the format defined by [RFC 3720](https://tools.ietf.org/html/rfc3720#page-32). <p> Example: {@code iqn.2015-12.com.oracleiaas:40b7ee03-883f-46c6-a951-63d2841d2195}
    pub iqn: String,

    /// The volume's iSCSI port, usually port 860 or 3260. <p> Example: {@code 3260} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i64>,
}

/// Required fields for MultipathDevice
pub struct MultipathDeviceRequired {
    /// The volume's iSCSI IP address. <p> Example: {@code 169.254.2.2}
    pub ipv4: String,

    /// The target volume's iSCSI Qualified Name in the format defined by [RFC 3720](https://tools.ietf.org/html/rfc3720#page-32). <p> Example: {@code iqn.2015-12.com.oracleiaas:40b7ee03-883f-46c6-a951-63d2841d2195}
    pub iqn: String,
}

impl MultipathDevice {
    /// Create a new MultipathDevice with required fields
    pub fn new(required: MultipathDeviceRequired) -> Self {
        Self {
            ipv4: required.ipv4,

            iqn: required.iqn,

            port: None,
        }
    }

    /// Set ipv4
    pub fn set_ipv4(mut self, value: String) -> Self {
        self.ipv4 = value;
        self
    }

    /// Set iqn
    pub fn set_iqn(mut self, value: String) -> Self {
        self.iqn = value;
        self
    }

    /// Set port
    pub fn set_port(mut self, value: Option<i64>) -> Self {
        self.port = value;
        self
    }

    /// Set port (unwraps Option)
    pub fn with_port(mut self, value: i64) -> Self {
        self.port = Some(value);
        self
    }
}
