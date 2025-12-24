use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIPSecConnectionDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Your identifier for your CPE device. Can be either an IP address or a hostname (specifically, the fully qualified domain name (FQDN)). The type of identifier you provide here must correspond to the value for {@code cpeLocalIdentifierType}. <p> For information about why you'd provide this value, see [If Your CPE Is Behind a NAT Device](https://docs.oracle.com/iaas/Content/Network/Tasks/overviewIPsec.htm#nat). <p> Example IP address: {@code 10.0.3.3} <p> Example hostname: {@code cpe.example.com}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_local_identifier: Option<String>,

    /// The type of identifier for your CPE device. The value you provide here must correspond to the value for {@code cpeLocalIdentifier}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpe_local_identifier_type: Option<UpdateIPSecConnectionDetailsCpeLocalIdentifierType>,

    /// Static routes to the CPE. If you provide this attribute, it replaces the entire current set of static routes. A static route's CIDR must not be a multicast address or class E address. The CIDR can be either IPv4 or IPv6. IPv6 addressing is supported for all commercial and government regions. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 10.0.1.0/24} <p> Example: {@code 2001:db8::/32}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_routes: Option<Vec<String>>,
}

impl UpdateIPSecConnectionDetails {
    /// Create a new UpdateIPSecConnectionDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            cpe_local_identifier: None,

            cpe_local_identifier_type: None,

            static_routes: None,
        }
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set cpe_local_identifier
    pub fn set_cpe_local_identifier(mut self, value: Option<String>) -> Self {
        self.cpe_local_identifier = value;
        self
    }

    /// Set cpe_local_identifier_type
    pub fn set_cpe_local_identifier_type(
        mut self,
        value: Option<UpdateIPSecConnectionDetailsCpeLocalIdentifierType>,
    ) -> Self {
        self.cpe_local_identifier_type = value;
        self
    }

    /// Set static_routes
    pub fn set_static_routes(mut self, value: Option<Vec<String>>) -> Self {
        self.static_routes = value;
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set cpe_local_identifier (unwraps Option)
    pub fn with_cpe_local_identifier(mut self, value: impl Into<String>) -> Self {
        self.cpe_local_identifier = Some(value.into());
        self
    }

    /// Set cpe_local_identifier_type (unwraps Option)
    pub fn with_cpe_local_identifier_type(
        mut self,
        value: UpdateIPSecConnectionDetailsCpeLocalIdentifierType,
    ) -> Self {
        self.cpe_local_identifier_type = Some(value);
        self
    }

    /// Set static_routes (unwraps Option)
    pub fn with_static_routes(mut self, value: Vec<String>) -> Self {
        self.static_routes = Some(value);
        self
    }
}

impl Default for UpdateIPSecConnectionDetails {
    fn default() -> Self {
        Self::new()
    }
}
