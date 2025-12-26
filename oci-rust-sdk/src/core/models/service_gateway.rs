use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Represents a router that lets your VCN privately access specific Oracle services such as Object Storage without exposing the VCN to the public internet. Traffic leaving the VCN and destined for a supported Oracle service (use the {@link #listServices(ListServicesRequest) listServices} operation to find available service CIDR labels) is routed through the service gateway and does not traverse the internet. The instances in the VCN do not need to have public IP addresses nor be in a public subnet. The VCN does not need an internet gateway for this traffic. For more information, see [Access to Oracle Services: Service Gateway](https://docs.oracle.com/iaas/Content/Network/Tasks/servicegateway.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceGateway {
    /// Whether the service gateway blocks all traffic through it. The default is {@code false}. When this is {@code true}, traffic is not routed to any services, regardless of route rules. <p> Example: {@code true}
    pub block_traffic: bool,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the service gateway.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service gateway.
    pub id: String,

    /// The service gateway's current state.
    pub lifecycle_state: ServiceGatewayLifecycleState,

    /// List of the {@link Service} objects enabled for this service gateway. The list can be empty. You can enable a particular {@code Service} by using {@link #attachServiceId(AttachServiceIdRequest) attachServiceId} or {@link #updateServiceGateway(UpdateServiceGatewayRequest) updateServiceGateway}.
    pub services: Vec<ServiceIdResponseDetails>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the service gateway belongs to.
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

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the service gateway is using. For information about why you would associate a route table with a service gateway, see [Transit Routing: Private Access to Oracle Services](https://docs.oracle.com/iaas/Content/Network/Tasks/transitroutingoracleservices.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// The date and time the service gateway was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

/// Required fields for ServiceGateway
pub struct ServiceGatewayRequired {
    /// Whether the service gateway blocks all traffic through it. The default is {@code false}. When this is {@code true}, traffic is not routed to any services, regardless of route rules. <p> Example: {@code true}
    pub block_traffic: bool,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the service gateway.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service gateway.
    pub id: String,

    /// The service gateway's current state.
    pub lifecycle_state: ServiceGatewayLifecycleState,

    /// List of the {@link Service} objects enabled for this service gateway. The list can be empty. You can enable a particular {@code Service} by using {@link #attachServiceId(AttachServiceIdRequest) attachServiceId} or {@link #updateServiceGateway(UpdateServiceGatewayRequest) updateServiceGateway}.
    pub services: Vec<ServiceIdResponseDetails>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the service gateway belongs to.
    pub vcn_id: String,
}

impl ServiceGateway {
    /// Create a new ServiceGateway with required fields
    pub fn new(required: ServiceGatewayRequired) -> Self {
        Self {
            block_traffic: required.block_traffic,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            services: required.services,

            vcn_id: required.vcn_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            route_table_id: None,

            time_created: None,
        }
    }

    /// Set block_traffic
    pub fn set_block_traffic(mut self, value: bool) -> Self {
        self.block_traffic = value;
        self
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ServiceGatewayLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set services
    pub fn set_services(mut self, value: Vec<ServiceIdResponseDetails>) -> Self {
        self.services = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
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

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}
