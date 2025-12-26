use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A *public IP* is a conceptual term that refers to a public IP address and related properties. The {@code publicIp} object is the API representation of a public IP. <p> There are two types of public IPs: 1. Ephemeral 2. Reserved <p> For more information and comparison of the two types, see [Public IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingpublicIPs.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIp {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the entity the public IP is assigned to, or in the process of being assigned to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_entity_id: Option<String>,

    /// The type of entity the public IP is assigned to, or in the process of being assigned to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_entity_type: Option<PublicIpAssignedEntityType>,

    /// The public IP's availability domain. This property is set only for ephemeral public IPs that are assigned to a private IP (that is, when the {@code scope} of the public IP is set to AVAILABILITY_DOMAIN). The value is the availability domain of the assigned private IP. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the public IP. For an ephemeral public IP, this is the compartment of its assigned entity (which can be a private IP or a regional entity such as a NAT gateway). For a reserved public IP that is currently assigned, its compartment can be different from the assigned private IP's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The public IP's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The public IP address of the {@code publicIp} object. <p> Example: {@code 203.0.113.2}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// The public IP's current state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<PublicIpLifecycleState>,

    /// Defines when the public IP is deleted and released back to Oracle's public IP pool. <p> {@code EPHEMERAL}: The lifetime is tied to the lifetime of its assigned entity. An ephemeral public IP must always be assigned to an entity. If the assigned entity is a private IP, the ephemeral public IP is automatically deleted when the private IP is deleted, when the VNIC is terminated, or when the instance is terminated. If the assigned entity is a {@link NatGateway}, the ephemeral public IP is automatically deleted when the NAT gateway is terminated. <p> {@code RESERVED}: You control the public IP's lifetime. You can delete a reserved public IP whenever you like. It does not need to be assigned to a private IP at all times. <p> For more information and comparison of the two types, see [Public IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingpublicIPs.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<PublicIpLifetime>,

    /// Deprecated. Use {@code assignedEntityId} instead. <p> The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the private IP that the public IP is currently assigned to, or in the process of being assigned to. <p> *Note:** This is {@code null} if the public IP is not assigned to a private IP, or is in the process of being assigned to one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip_id: Option<String>,

    /// Whether the public IP is regional or specific to a particular availability domain. <p> {@code REGION}: The public IP exists within a region and is assigned to a regional entity (such as a {@link NatGateway}), or can be assigned to a private IP in any availability domain in the region. Reserved public IPs and ephemeral public IPs assigned to a regional entity have {@code scope} = {@code REGION}. <p> {@code AVAILABILITY_DOMAIN}: The public IP exists within the availability domain of the entity it's assigned to, which is specified by the {@code availabilityDomain} property of the public IP object. Ephemeral public IPs that are assigned to private IPs have {@code scope} = {@code AVAILABILITY_DOMAIN}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<PublicIpScope>,

    /// The date and time the public IP was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the pool object created in the current tenancy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip_pool_id: Option<String>,
}

impl PublicIp {
    /// Create a new PublicIp
    pub fn new() -> Self {
        Self {
            assigned_entity_id: None,

            assigned_entity_type: None,

            availability_domain: None,

            compartment_id: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            id: None,

            ip_address: None,

            lifecycle_state: None,

            lifetime: None,

            private_ip_id: None,

            scope: None,

            time_created: None,

            public_ip_pool_id: None,
        }
    }

    /// Set assigned_entity_id
    pub fn set_assigned_entity_id(mut self, value: Option<String>) -> Self {
        self.assigned_entity_id = value;
        self
    }

    /// Set assigned_entity_type
    pub fn set_assigned_entity_type(mut self, value: Option<PublicIpAssignedEntityType>) -> Self {
        self.assigned_entity_type = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
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

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: Option<String>) -> Self {
        self.ip_address = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<PublicIpLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set lifetime
    pub fn set_lifetime(mut self, value: Option<PublicIpLifetime>) -> Self {
        self.lifetime = value;
        self
    }

    /// Set private_ip_id
    pub fn set_private_ip_id(mut self, value: Option<String>) -> Self {
        self.private_ip_id = value;
        self
    }

    /// Set scope
    pub fn set_scope(mut self, value: Option<PublicIpScope>) -> Self {
        self.scope = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set public_ip_pool_id
    pub fn set_public_ip_pool_id(mut self, value: Option<String>) -> Self {
        self.public_ip_pool_id = value;
        self
    }

    /// Set assigned_entity_id (unwraps Option)
    pub fn with_assigned_entity_id(mut self, value: impl Into<String>) -> Self {
        self.assigned_entity_id = Some(value.into());
        self
    }

    /// Set assigned_entity_type (unwraps Option)
    pub fn with_assigned_entity_type(mut self, value: PublicIpAssignedEntityType) -> Self {
        self.assigned_entity_type = Some(value);
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
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

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set ip_address (unwraps Option)
    pub fn with_ip_address(mut self, value: impl Into<String>) -> Self {
        self.ip_address = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: PublicIpLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set lifetime (unwraps Option)
    pub fn with_lifetime(mut self, value: PublicIpLifetime) -> Self {
        self.lifetime = Some(value);
        self
    }

    /// Set private_ip_id (unwraps Option)
    pub fn with_private_ip_id(mut self, value: impl Into<String>) -> Self {
        self.private_ip_id = Some(value.into());
        self
    }

    /// Set scope (unwraps Option)
    pub fn with_scope(mut self, value: PublicIpScope) -> Self {
        self.scope = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set public_ip_pool_id (unwraps Option)
    pub fn with_public_ip_pool_id(mut self, value: impl Into<String>) -> Self {
        self.public_ip_pool_id = Some(value.into());
        self
    }
}

impl Default for PublicIp {
    fn default() -> Self {
        Self::new()
    }
}
