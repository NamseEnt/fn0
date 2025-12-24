use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The details for creating a new compute capacity reservation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeCapacityReservationDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the capacity reservation.
    pub compartment_id: String,

    /// The availability domain of this compute capacity reservation. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Whether this capacity reservation is the default. For more information, see [Capacity Reservations](https://docs.oracle.com/iaas/Content/Compute/Tasks/reserve-capacity.htm#default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default_reservation: Option<bool>,

    /// The capacity configurations for the capacity reservation. <p> To use the reservation for the desired shape, specify the shape, count, and optionally the fault domain where you want this configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_reservation_configs: Option<Vec<InstanceReservationConfigDetails>>,
}

/// Required fields for CreateComputeCapacityReservationDetails
pub struct CreateComputeCapacityReservationDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the capacity reservation.
    pub compartment_id: String,

    /// The availability domain of this compute capacity reservation. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,
}

impl CreateComputeCapacityReservationDetails {
    /// Create a new CreateComputeCapacityReservationDetails with required fields
    pub fn new(required: CreateComputeCapacityReservationDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            availability_domain: required.availability_domain,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            is_default_reservation: None,

            instance_reservation_configs: None,
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

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set is_default_reservation
    pub fn set_is_default_reservation(mut self, value: Option<bool>) -> Self {
        self.is_default_reservation = value;
        self
    }

    /// Set instance_reservation_configs
    pub fn set_instance_reservation_configs(
        mut self,
        value: Option<Vec<InstanceReservationConfigDetails>>,
    ) -> Self {
        self.instance_reservation_configs = value;
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

    /// Set is_default_reservation (unwraps Option)
    pub fn with_is_default_reservation(mut self, value: bool) -> Self {
        self.is_default_reservation = Some(value);
        self
    }

    /// Set instance_reservation_configs (unwraps Option)
    pub fn with_instance_reservation_configs(
        mut self,
        value: Vec<InstanceReservationConfigDetails>,
    ) -> Self {
        self.instance_reservation_configs = Some(value);
        self
    }
}
