use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServiceGatewayDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the service gateway.
    pub compartment_id: String,

    /// List of the OCIDs of the {@link Service} objects to enable for the service gateway. This list can be empty if you don't want to enable any {@code Service} objects when you create the gateway. You can enable a {@code Service} object later by using either {@link #attachServiceId(AttachServiceIdRequest) attachServiceId} or {@link #updateServiceGateway(UpdateServiceGatewayRequest) updateServiceGateway}. <p> For each enabled {@code Service}, make sure there's a route rule with the {@code Service} object's {@code cidrBlock} as the rule's destination and the service gateway as the rule's target. See {@link RouteTable}.
    pub services: Vec<ServiceIdRequestDetails>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the service gateway will use. <p> If you don't specify a route table here, the service gateway is created without an associated route table. The Networking service does NOT automatically associate the attached VCN's default route table with the service gateway. <p> For information about why you would associate a route table with a service gateway, see [Transit Routing: Private Access to Oracle Services](https://docs.oracle.com/iaas/Content/Network/Tasks/transitroutingoracleservices.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,
}

/// Required fields for CreateServiceGatewayDetails
pub struct CreateServiceGatewayDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the service gateway.
    pub compartment_id: String,

    /// List of the OCIDs of the {@link Service} objects to enable for the service gateway. This list can be empty if you don't want to enable any {@code Service} objects when you create the gateway. You can enable a {@code Service} object later by using either {@link #attachServiceId(AttachServiceIdRequest) attachServiceId} or {@link #updateServiceGateway(UpdateServiceGatewayRequest) updateServiceGateway}. <p> For each enabled {@code Service}, make sure there's a route rule with the {@code Service} object's {@code cidrBlock} as the rule's destination and the service gateway as the rule's target. See {@link RouteTable}.
    pub services: Vec<ServiceIdRequestDetails>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN.
    pub vcn_id: String,
}

impl CreateServiceGatewayDetails {
    /// Create a new CreateServiceGatewayDetails with required fields
    pub fn new(required: CreateServiceGatewayDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            services: required.services,

            vcn_id: required.vcn_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            route_table_id: None,
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

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set services
    pub fn set_services(mut self, value: Vec<ServiceIdRequestDetails>) -> Self {
        self.services = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
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
}
