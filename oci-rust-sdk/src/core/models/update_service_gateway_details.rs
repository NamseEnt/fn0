use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServiceGatewayDetails {
    /// Whether the service gateway blocks all traffic through it. The default is {@code false}. When this is {@code true}, traffic is not routed to any services, regardless of route rules. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_traffic: Option<bool>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the service gateway will use. For information about why you would associate a route table with a service gateway, see [Transit Routing: Private Access to Oracle Services](https://docs.oracle.com/iaas/Content/Network/Tasks/transitroutingoracleservices.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// List of all the {@code Service} objects you want enabled on this service gateway. Sending an empty list means you want to disable all services. Omitting this parameter entirely keeps the existing list of services intact. <p> You can also enable or disable a particular {@code Service} by using {@link #attachServiceId(AttachServiceIdRequest) attachServiceId} or {@link #detachServiceId(DetachServiceIdRequest) detachServiceId}. <p> For each enabled {@code Service}, make sure there's a route rule with the {@code Service} object's {@code cidrBlock} as the rule's destination and the service gateway as the rule's target. See {@link RouteTable}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<ServiceIdRequestDetails>>,
}

impl UpdateServiceGatewayDetails {
    /// Create a new UpdateServiceGatewayDetails
    pub fn new() -> Self {
        Self {
            block_traffic: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            route_table_id: None,

            services: None,
        }
    }

    /// Set block_traffic
    pub fn set_block_traffic(mut self, value: Option<bool>) -> Self {
        self.block_traffic = value;
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

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set services
    pub fn set_services(mut self, value: Option<Vec<ServiceIdRequestDetails>>) -> Self {
        self.services = value;
        self
    }

    /// Set block_traffic (unwraps Option)
    pub fn with_block_traffic(mut self, value: bool) -> Self {
        self.block_traffic = Some(value);
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

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set services (unwraps Option)
    pub fn with_services(mut self, value: Vec<ServiceIdRequestDetails>) -> Self {
        self.services = Some(value);
        self
    }
}

impl Default for UpdateServiceGatewayDetails {
    fn default() -> Self {
        Self::new()
    }
}
