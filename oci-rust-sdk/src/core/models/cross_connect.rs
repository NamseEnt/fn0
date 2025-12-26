use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// For use with Oracle Cloud Infrastructure FastConnect. A cross-connect represents a physical connection between an existing network and Oracle. Customers who are colocated with Oracle in a FastConnect location create and use cross-connects. For more information, see [FastConnect Overview](https://docs.oracle.com/iaas/Content/Network/Concepts/fastconnect.htm). <p> Oracle recommends you create each cross-connect in a {@link CrossConnectGroup} so you can use link aggregation with the connection. <p> *Note:** If you're a provider who is setting up a physical connection to Oracle so customers can use FastConnect over the connection, be aware that your connection is modeled the same way as a colocated customer's (with {@code CrossConnect} and {@code CrossConnectGroup} objects, and so on). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossConnect {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the cross-connect group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect group this cross-connect belongs to (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_connect_group_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The cross-connect's Oracle ID (OCID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The cross-connect's current state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<CrossConnectLifecycleState>,

    /// The name of the FastConnect location where this cross-connect is installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_name: Option<String>,

    /// A string identifying the meet-me room port for this cross-connect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_name: Option<String>,

    /// The port speed for this cross-connect. <p> Example: {@code 10 Gbps}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_speed_shape_name: Option<String>,

    /// A reference name or identifier for the physical fiber connection that this cross-connect uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_reference_name: Option<String>,

    /// The date and time the cross-connect was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub macsec_properties: Option<MacsecProperties>,

    /// The FastConnect device that terminates the physical connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oci_physical_device_name: Option<String>,

    /// The FastConnect device that terminates the logical connection. This device might be different than the device that terminates the physical connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oci_logical_device_name: Option<String>,
}

impl CrossConnect {
    /// Create a new CrossConnect
    pub fn new() -> Self {
        Self {
            compartment_id: None,

            cross_connect_group_id: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            id: None,

            lifecycle_state: None,

            location_name: None,

            port_name: None,

            port_speed_shape_name: None,

            customer_reference_name: None,

            time_created: None,

            macsec_properties: None,

            oci_physical_device_name: None,

            oci_logical_device_name: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set cross_connect_group_id
    pub fn set_cross_connect_group_id(mut self, value: Option<String>) -> Self {
        self.cross_connect_group_id = value;
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

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Option<CrossConnectLifecycleState>) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set location_name
    pub fn set_location_name(mut self, value: Option<String>) -> Self {
        self.location_name = value;
        self
    }

    /// Set port_name
    pub fn set_port_name(mut self, value: Option<String>) -> Self {
        self.port_name = value;
        self
    }

    /// Set port_speed_shape_name
    pub fn set_port_speed_shape_name(mut self, value: Option<String>) -> Self {
        self.port_speed_shape_name = value;
        self
    }

    /// Set customer_reference_name
    pub fn set_customer_reference_name(mut self, value: Option<String>) -> Self {
        self.customer_reference_name = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set macsec_properties
    pub fn set_macsec_properties(mut self, value: Option<MacsecProperties>) -> Self {
        self.macsec_properties = value;
        self
    }

    /// Set oci_physical_device_name
    pub fn set_oci_physical_device_name(mut self, value: Option<String>) -> Self {
        self.oci_physical_device_name = value;
        self
    }

    /// Set oci_logical_device_name
    pub fn set_oci_logical_device_name(mut self, value: Option<String>) -> Self {
        self.oci_logical_device_name = value;
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set cross_connect_group_id (unwraps Option)
    pub fn with_cross_connect_group_id(mut self, value: impl Into<String>) -> Self {
        self.cross_connect_group_id = Some(value.into());
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

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: CrossConnectLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set location_name (unwraps Option)
    pub fn with_location_name(mut self, value: impl Into<String>) -> Self {
        self.location_name = Some(value.into());
        self
    }

    /// Set port_name (unwraps Option)
    pub fn with_port_name(mut self, value: impl Into<String>) -> Self {
        self.port_name = Some(value.into());
        self
    }

    /// Set port_speed_shape_name (unwraps Option)
    pub fn with_port_speed_shape_name(mut self, value: impl Into<String>) -> Self {
        self.port_speed_shape_name = Some(value.into());
        self
    }

    /// Set customer_reference_name (unwraps Option)
    pub fn with_customer_reference_name(mut self, value: impl Into<String>) -> Self {
        self.customer_reference_name = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set macsec_properties (unwraps Option)
    pub fn with_macsec_properties(mut self, value: MacsecProperties) -> Self {
        self.macsec_properties = Some(value);
        self
    }

    /// Set oci_physical_device_name (unwraps Option)
    pub fn with_oci_physical_device_name(mut self, value: impl Into<String>) -> Self {
        self.oci_physical_device_name = Some(value.into());
        self
    }

    /// Set oci_logical_device_name (unwraps Option)
    pub fn with_oci_logical_device_name(mut self, value: impl Into<String>) -> Self {
        self.oci_logical_device_name = Some(value.into());
        self
    }
}

impl Default for CrossConnect {
    fn default() -> Self {
        Self::new()
    }
}
