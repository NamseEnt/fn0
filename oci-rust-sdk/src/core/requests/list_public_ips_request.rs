use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPublicIpsRequest {
    /// Whether the public IP is regional or specific to a particular availability domain. <p> {@code REGION}: The public IP exists within a region and is assigned to a regional entity (such as a {@link NatGateway}), or can be assigned to a private IP in any availability domain in the region. Reserved public IPs have {@code scope} = {@code REGION}, as do ephemeral public IPs assigned to a regional entity. <p> {@code AVAILABILITY_DOMAIN}: The public IP exists within the availability domain of the entity it's assigned to, which is specified by the {@code availabilityDomain} property of the public IP object. Ephemeral public IPs that are assigned to private IPs have {@code scope} = {@code AVAILABILITY_DOMAIN}.
    pub scope: ListPublicIpsRequestScope,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// For list pagination. The maximum number of results per page, or items to return in a paginated \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). <p> Example: {@code 50}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// For list pagination. The value of the {@code opc-next-page} response header from the previous \"List\" call. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// The name of the availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// A filter to return only public IPs that match given lifetime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<ListPublicIpsRequestLifetime>,

    /// A filter to return only resources that belong to the given public IP pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip_pool_id: Option<String>,
}

/// Required fields for ListPublicIpsRequest
pub struct ListPublicIpsRequestRequired {
    /// Whether the public IP is regional or specific to a particular availability domain. <p> {@code REGION}: The public IP exists within a region and is assigned to a regional entity (such as a {@link NatGateway}), or can be assigned to a private IP in any availability domain in the region. Reserved public IPs have {@code scope} = {@code REGION}, as do ephemeral public IPs assigned to a regional entity. <p> {@code AVAILABILITY_DOMAIN}: The public IP exists within the availability domain of the entity it's assigned to, which is specified by the {@code availabilityDomain} property of the public IP object. Ephemeral public IPs that are assigned to private IPs have {@code scope} = {@code AVAILABILITY_DOMAIN}.
    pub scope: ListPublicIpsRequestScope,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl ListPublicIpsRequest {
    /// Create a new ListPublicIpsRequest with required fields
    pub fn new(required: ListPublicIpsRequestRequired) -> Self {
        Self {
            scope: required.scope,

            compartment_id: required.compartment_id,

            limit: None,

            page: None,

            availability_domain: None,

            lifetime: None,

            public_ip_pool_id: None,
        }
    }

    /// Set scope
    pub fn set_scope(mut self, value: ListPublicIpsRequestScope) -> Self {
        self.scope = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set limit
    pub fn set_limit(mut self, value: Option<i64>) -> Self {
        self.limit = value;
        self
    }

    /// Set page
    pub fn set_page(mut self, value: Option<String>) -> Self {
        self.page = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set lifetime
    pub fn set_lifetime(mut self, value: Option<ListPublicIpsRequestLifetime>) -> Self {
        self.lifetime = value;
        self
    }

    /// Set public_ip_pool_id
    pub fn set_public_ip_pool_id(mut self, value: Option<String>) -> Self {
        self.public_ip_pool_id = value;
        self
    }

    /// Set limit (unwraps Option)
    pub fn with_limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    /// Set page (unwraps Option)
    pub fn with_page(mut self, value: impl Into<String>) -> Self {
        self.page = Some(value.into());
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set lifetime (unwraps Option)
    pub fn with_lifetime(mut self, value: ListPublicIpsRequestLifetime) -> Self {
        self.lifetime = Some(value);
        self
    }

    /// Set public_ip_pool_id (unwraps Option)
    pub fn with_public_ip_pool_id(mut self, value: impl Into<String>) -> Self {
        self.public_ip_pool_id = Some(value.into());
        self
    }
}
