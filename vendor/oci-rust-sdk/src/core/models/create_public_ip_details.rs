use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePublicIpDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the public IP. For ephemeral public IPs, you must set this to the private IP's compartment [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub compartment_id: String,

    /// Defines when the public IP is deleted and released back to the Oracle Cloud Infrastructure public IP pool. For more information, see [Public IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingpublicIPs.htm).
    pub lifetime: CreatePublicIpDetailsLifetime,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the private IP to assign the public IP to. <p> Required for an ephemeral public IP because it must always be assigned to a private IP (specifically a *primary* private IP). <p> Optional for a reserved public IP. If you don't provide it, the public IP is created but not assigned to a private IP. You can later assign the public IP with {@link #updatePublicIp(UpdatePublicIpRequest) updatePublicIp}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the public IP pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip_pool_id: Option<String>,
}

/// Required fields for CreatePublicIpDetails
pub struct CreatePublicIpDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the public IP. For ephemeral public IPs, you must set this to the private IP's compartment [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub compartment_id: String,

    /// Defines when the public IP is deleted and released back to the Oracle Cloud Infrastructure public IP pool. For more information, see [Public IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingpublicIPs.htm).
    pub lifetime: CreatePublicIpDetailsLifetime,
}

impl CreatePublicIpDetails {
    /// Create a new CreatePublicIpDetails with required fields
    pub fn new(required: CreatePublicIpDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            lifetime: required.lifetime,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            private_ip_id: None,

            public_ip_pool_id: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
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

    /// Set lifetime
    pub fn set_lifetime(mut self, value: CreatePublicIpDetailsLifetime) -> Self {
        self.lifetime = value;
        self
    }

    /// Set private_ip_id
    pub fn set_private_ip_id(mut self, value: Option<String>) -> Self {
        self.private_ip_id = value;
        self
    }

    /// Set public_ip_pool_id
    pub fn set_public_ip_pool_id(mut self, value: Option<String>) -> Self {
        self.public_ip_pool_id = value;
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

    /// Set private_ip_id (unwraps Option)
    pub fn with_private_ip_id(mut self, value: impl Into<String>) -> Self {
        self.private_ip_id = Some(value.into());
        self
    }

    /// Set public_ip_pool_id (unwraps Option)
    pub fn with_public_ip_pool_id(mut self, value: impl Into<String>) -> Self {
        self.public_ip_pool_id = Some(value.into());
        self
    }
}
