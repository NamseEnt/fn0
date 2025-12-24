use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape configuration requested when launching instances in a compute capacity reservation. <p> If the parameter is provided, the reservation is created with the resources that you specify. If some properties are missing or the parameter is not provided, the reservation is created with the default configuration values for the {@code shape} that you specify. <p> Each shape only supports certain configurable values. If the values that you provide are not valid for the specified {@code shape}, an error is returned. <p> For more information about customizing the resources that are allocated to flexible shapes, see [Flexible Shapes](https://docs.oracle.com/iaas/Content/Compute/References/computeshapes.htm#flexible).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceReservationShapeConfigDetails {
    /// The total number of OCPUs available to the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpus: Option<i64>,

    /// The total amount of memory available to the instance, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_in_g_bs: Option<i64>,
}

impl InstanceReservationShapeConfigDetails {
    /// Create a new InstanceReservationShapeConfigDetails
    pub fn new() -> Self {
        Self {
            ocpus: None,

            memory_in_g_bs: None,
        }
    }

    /// Set ocpus
    pub fn set_ocpus(mut self, value: Option<i64>) -> Self {
        self.ocpus = value;
        self
    }

    /// Set memory_in_g_bs
    pub fn set_memory_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.memory_in_g_bs = value;
        self
    }

    /// Set ocpus (unwraps Option)
    pub fn with_ocpus(mut self, value: i64) -> Self {
        self.ocpus = Some(value);
        self
    }

    /// Set memory_in_g_bs (unwraps Option)
    pub fn with_memory_in_g_bs(mut self, value: i64) -> Self {
        self.memory_in_g_bs = Some(value);
        self
    }
}

impl Default for InstanceReservationShapeConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
