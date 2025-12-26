use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNetworkingTopologyRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,

    /// Valid values are {@code ANY} and {@code ACCESSIBLE}. The default is {@code ANY}. Setting this to {@code ACCESSIBLE} returns only compartments for which a user has INSPECT permissions, either directly or indirectly (permissions can be on a resource in a subcompartment). A restricted set of fields is returned for compartments in which a user has indirect INSPECT permissions. <p> When set to {@code ANY} permissions are not checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_level: Option<GetNetworkingTopologyRequestAccessLevel>,

    /// When set to true, the hierarchy of compartments is traversed and the specified compartment and its subcompartments are inspected depending on the the setting of {@code accessLevel}. Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_compartment_subtree: Option<bool>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,

    /// For querying if there is a cached value on the server. The If-None-Match HTTP request header makes the request conditional. For GET and HEAD methods, the server will send back the requested resource, with a 200 status, only if it doesn't have an ETag matching the given ones. For other methods, the request will be processed only if the eventually existing resource's ETag doesn't match any of the values listed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_none_match: Option<String>,

    /// The Cache-Control HTTP header holds directives (instructions) for caching in both requests and responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<String>,
}

/// Required fields for GetNetworkingTopologyRequest
pub struct GetNetworkingTopologyRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    pub compartment_id: String,
}

impl GetNetworkingTopologyRequest {
    /// Create a new GetNetworkingTopologyRequest with required fields
    pub fn new(required: GetNetworkingTopologyRequestRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            access_level: None,

            query_compartment_subtree: None,

            opc_request_id: None,

            if_none_match: None,

            cache_control: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set access_level
    pub fn set_access_level(
        mut self,
        value: Option<GetNetworkingTopologyRequestAccessLevel>,
    ) -> Self {
        self.access_level = value;
        self
    }

    /// Set query_compartment_subtree
    pub fn set_query_compartment_subtree(mut self, value: Option<bool>) -> Self {
        self.query_compartment_subtree = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set if_none_match
    pub fn set_if_none_match(mut self, value: Option<String>) -> Self {
        self.if_none_match = value;
        self
    }

    /// Set cache_control
    pub fn set_cache_control(mut self, value: Option<String>) -> Self {
        self.cache_control = value;
        self
    }

    /// Set access_level (unwraps Option)
    pub fn with_access_level(mut self, value: GetNetworkingTopologyRequestAccessLevel) -> Self {
        self.access_level = Some(value);
        self
    }

    /// Set query_compartment_subtree (unwraps Option)
    pub fn with_query_compartment_subtree(mut self, value: bool) -> Self {
        self.query_compartment_subtree = Some(value);
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set if_none_match (unwraps Option)
    pub fn with_if_none_match(mut self, value: impl Into<String>) -> Self {
        self.if_none_match = Some(value.into());
        self
    }

    /// Set cache_control (unwraps Option)
    pub fn with_cache_control(mut self, value: impl Into<String>) -> Self {
        self.cache_control = Some(value.into());
        self
    }
}
