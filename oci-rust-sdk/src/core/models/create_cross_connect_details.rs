use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCrossConnectDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the cross-connect.
    pub compartment_id: String,

    /// The name of the FastConnect location where this cross-connect will be installed. To get a list of the available locations, see {@link #listCrossConnectLocations(ListCrossConnectLocationsRequest) listCrossConnectLocations}. <p> Example: {@code CyrusOne, Chandler, AZ}
    pub location_name: String,

    /// The port speed for this cross-connect. To get a list of the available port speeds, see {@link #listCrossconnectPortSpeedShapes(ListCrossconnectPortSpeedShapesRequest) listCrossconnectPortSpeedShapes}. <p> Example: {@code 10 Gbps}
    pub port_speed_shape_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect group to put this cross-connect in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_connect_group_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// If you already have an existing cross-connect or cross-connect group at this FastConnect location, and you want this new cross-connect to be on a different router (for the purposes of redundancy), provide the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of that existing cross-connect or cross-connect group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub far_cross_connect_or_cross_connect_group_id: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// If you already have an existing cross-connect or cross-connect group at this FastConnect location, and you want this new cross-connect to be on the same router, provide the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of that existing cross-connect or cross-connect group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub near_cross_connect_or_cross_connect_group_id: Option<String>,

    /// A reference name or identifier for the physical fiber connection that this cross-connect uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_reference_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub macsec_properties: Option<CreateMacsecProperties>,
}

/// Required fields for CreateCrossConnectDetails
pub struct CreateCrossConnectDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the cross-connect.
    pub compartment_id: String,

    /// The name of the FastConnect location where this cross-connect will be installed. To get a list of the available locations, see {@link #listCrossConnectLocations(ListCrossConnectLocationsRequest) listCrossConnectLocations}. <p> Example: {@code CyrusOne, Chandler, AZ}
    pub location_name: String,

    /// The port speed for this cross-connect. To get a list of the available port speeds, see {@link #listCrossconnectPortSpeedShapes(ListCrossconnectPortSpeedShapesRequest) listCrossconnectPortSpeedShapes}. <p> Example: {@code 10 Gbps}
    pub port_speed_shape_name: String,
}

impl CreateCrossConnectDetails {
    /// Create a new CreateCrossConnectDetails with required fields
    pub fn new(required: CreateCrossConnectDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            location_name: required.location_name,

            port_speed_shape_name: required.port_speed_shape_name,

            cross_connect_group_id: None,

            defined_tags: None,

            display_name: None,

            far_cross_connect_or_cross_connect_group_id: None,

            freeform_tags: None,

            near_cross_connect_or_cross_connect_group_id: None,

            customer_reference_name: None,

            macsec_properties: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
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

    /// Set far_cross_connect_or_cross_connect_group_id
    pub fn set_far_cross_connect_or_cross_connect_group_id(
        mut self,
        value: Option<String>,
    ) -> Self {
        self.far_cross_connect_or_cross_connect_group_id = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set location_name
    pub fn set_location_name(mut self, value: String) -> Self {
        self.location_name = value;
        self
    }

    /// Set near_cross_connect_or_cross_connect_group_id
    pub fn set_near_cross_connect_or_cross_connect_group_id(
        mut self,
        value: Option<String>,
    ) -> Self {
        self.near_cross_connect_or_cross_connect_group_id = value;
        self
    }

    /// Set port_speed_shape_name
    pub fn set_port_speed_shape_name(mut self, value: String) -> Self {
        self.port_speed_shape_name = value;
        self
    }

    /// Set customer_reference_name
    pub fn set_customer_reference_name(mut self, value: Option<String>) -> Self {
        self.customer_reference_name = value;
        self
    }

    /// Set macsec_properties
    pub fn set_macsec_properties(mut self, value: Option<CreateMacsecProperties>) -> Self {
        self.macsec_properties = value;
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

    /// Set far_cross_connect_or_cross_connect_group_id (unwraps Option)
    pub fn with_far_cross_connect_or_cross_connect_group_id(
        mut self,
        value: impl Into<String>,
    ) -> Self {
        self.far_cross_connect_or_cross_connect_group_id = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set near_cross_connect_or_cross_connect_group_id (unwraps Option)
    pub fn with_near_cross_connect_or_cross_connect_group_id(
        mut self,
        value: impl Into<String>,
    ) -> Self {
        self.near_cross_connect_or_cross_connect_group_id = Some(value.into());
        self
    }

    /// Set customer_reference_name (unwraps Option)
    pub fn with_customer_reference_name(mut self, value: impl Into<String>) -> Self {
        self.customer_reference_name = Some(value.into());
        self
    }

    /// Set macsec_properties (unwraps Option)
    pub fn with_macsec_properties(mut self, value: CreateMacsecProperties) -> Self {
        self.macsec_properties = Some(value);
        self
    }
}
