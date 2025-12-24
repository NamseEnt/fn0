use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that represents one or multiple Oracle services that you can enable for a {@link ServiceGateway}. In the User Guide topic [Access to Oracle Services: Service Gateway](https://docs.oracle.com/iaas/Content/Network/Tasks/servicegateway.htm), the term *service CIDR label* is used to refer to the string that represents the regional public IP address ranges of the Oracle service or services covered by a given {@code Service} object. That unique string is the value of the {@code Service} object's {@code cidrBlock} attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    /// A string that represents the regional public IP address ranges for the Oracle service or services covered by this {@code Service} object. Also known as the {@code Service} object's *service CIDR label*. <p> When you set up a route rule to route traffic to the service gateway, use this value as the rule's destination. See {@link RouteTable}. Also, when you set up a security list rule to cover traffic with the service gateway, use the {@code cidrBlock} value as the rule's destination (for an egress rule) or the source (for an ingress rule). See {@link SecurityList}. <p> Example: {@code oci-phx-objectstorage}
    pub cidr_block: String,

    /// Description of the Oracle service or services covered by this {@code Service} object. <p> Example: {@code OCI PHX Object Storage}
    pub description: String,

    /// The {@code Service} object's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub id: String,

    /// Name of the {@code Service} object. This name can change and is not guaranteed to be unique. <p> Example: {@code OCI PHX Object Storage}
    pub name: String,
}

/// Required fields for Service
pub struct ServiceRequired {
    /// A string that represents the regional public IP address ranges for the Oracle service or services covered by this {@code Service} object. Also known as the {@code Service} object's *service CIDR label*. <p> When you set up a route rule to route traffic to the service gateway, use this value as the rule's destination. See {@link RouteTable}. Also, when you set up a security list rule to cover traffic with the service gateway, use the {@code cidrBlock} value as the rule's destination (for an egress rule) or the source (for an ingress rule). See {@link SecurityList}. <p> Example: {@code oci-phx-objectstorage}
    pub cidr_block: String,

    /// Description of the Oracle service or services covered by this {@code Service} object. <p> Example: {@code OCI PHX Object Storage}
    pub description: String,

    /// The {@code Service} object's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub id: String,

    /// Name of the {@code Service} object. This name can change and is not guaranteed to be unique. <p> Example: {@code OCI PHX Object Storage}
    pub name: String,
}

impl Service {
    /// Create a new Service with required fields
    pub fn new(required: ServiceRequired) -> Self {
        Self {
            cidr_block: required.cidr_block,

            description: required.description,

            id: required.id,

            name: required.name,
        }
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: String) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: String) -> Self {
        self.description = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }
}
