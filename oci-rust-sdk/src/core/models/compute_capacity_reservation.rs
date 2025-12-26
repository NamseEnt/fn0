use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A template that defines the settings to use when creating compute capacity reservations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeCapacityReservation {
    /// The availability domain of the compute capacity reservation. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the compute capacity reservation.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity reservation.
    pub id: String,

    /// The current state of the compute capacity reservation.
    pub lifecycle_state: ComputeCapacityReservationLifecycleState,

    /// The date and time the compute capacity reservation was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

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
    pub instance_reservation_configs: Option<Vec<InstanceReservationConfig>>,

    /// The number of instances for which capacity will be held with this compute capacity reservation. This number is the sum of the values of the {@code reservedCount} fields for all of the instance capacity configurations under this reservation. The purpose of this field is to calculate the percentage usage of the reservation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved_instance_count: Option<i64>,

    /// The date and time the compute capacity reservation was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// The total number of instances currently consuming space in this compute capacity reservation. This number is the sum of the values of the {@code usedCount} fields for all of the instance capacity configurations under this reservation. The purpose of this field is to calculate the percentage usage of the reservation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_instance_count: Option<i64>,
}

/// Required fields for ComputeCapacityReservation
pub struct ComputeCapacityReservationRequired {
    /// The availability domain of the compute capacity reservation. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the compute capacity reservation.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity reservation.
    pub id: String,

    /// The current state of the compute capacity reservation.
    pub lifecycle_state: ComputeCapacityReservationLifecycleState,

    /// The date and time the compute capacity reservation was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeCapacityReservation {
    /// Create a new ComputeCapacityReservation with required fields
    pub fn new(required: ComputeCapacityReservationRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            is_default_reservation: None,

            instance_reservation_configs: None,

            reserved_instance_count: None,

            time_updated: None,

            used_instance_count: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
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

    /// Set is_default_reservation
    pub fn set_is_default_reservation(mut self, value: Option<bool>) -> Self {
        self.is_default_reservation = value;
        self
    }

    /// Set instance_reservation_configs
    pub fn set_instance_reservation_configs(
        mut self,
        value: Option<Vec<InstanceReservationConfig>>,
    ) -> Self {
        self.instance_reservation_configs = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ComputeCapacityReservationLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set reserved_instance_count
    pub fn set_reserved_instance_count(mut self, value: Option<i64>) -> Self {
        self.reserved_instance_count = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set used_instance_count
    pub fn set_used_instance_count(mut self, value: Option<i64>) -> Self {
        self.used_instance_count = value;
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
        value: Vec<InstanceReservationConfig>,
    ) -> Self {
        self.instance_reservation_configs = Some(value);
        self
    }

    /// Set reserved_instance_count (unwraps Option)
    pub fn with_reserved_instance_count(mut self, value: i64) -> Self {
        self.reserved_instance_count = Some(value);
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
        self
    }

    /// Set used_instance_count (unwraps Option)
    pub fn with_used_instance_count(mut self, value: i64) -> Self {
        self.used_instance_count = Some(value);
        self
    }
}
