use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the available capacity for a shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityReportShapeAvailability {
    /// The fault domain for the capacity report. <p> If you do not specify the fault domain, the capacity report includes information about all fault domains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// The shape that the capacity report was requested for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_shape: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_shape_config: Option<CapacityReportInstanceShapeConfig>,

    /// The total number of new instances that can be created with the specified shape configuration. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_count: Option<i64>,

    /// A flag denoting whether capacity is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_status: Option<CapacityReportShapeAvailabilityAvailabilityStatus>,
}

impl CapacityReportShapeAvailability {
    /// Create a new CapacityReportShapeAvailability
    pub fn new() -> Self {
        Self {
            fault_domain: None,

            instance_shape: None,

            instance_shape_config: None,

            available_count: None,

            availability_status: None,
        }
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set instance_shape
    pub fn set_instance_shape(mut self, value: Option<String>) -> Self {
        self.instance_shape = value;
        self
    }

    /// Set instance_shape_config
    pub fn set_instance_shape_config(
        mut self,
        value: Option<CapacityReportInstanceShapeConfig>,
    ) -> Self {
        self.instance_shape_config = value;
        self
    }

    /// Set available_count
    pub fn set_available_count(mut self, value: Option<i64>) -> Self {
        self.available_count = value;
        self
    }

    /// Set availability_status
    pub fn set_availability_status(
        mut self,
        value: Option<CapacityReportShapeAvailabilityAvailabilityStatus>,
    ) -> Self {
        self.availability_status = value;
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set instance_shape (unwraps Option)
    pub fn with_instance_shape(mut self, value: impl Into<String>) -> Self {
        self.instance_shape = Some(value.into());
        self
    }

    /// Set instance_shape_config (unwraps Option)
    pub fn with_instance_shape_config(mut self, value: CapacityReportInstanceShapeConfig) -> Self {
        self.instance_shape_config = Some(value);
        self
    }

    /// Set available_count (unwraps Option)
    pub fn with_available_count(mut self, value: i64) -> Self {
        self.available_count = Some(value);
        self
    }

    /// Set availability_status (unwraps Option)
    pub fn with_availability_status(
        mut self,
        value: CapacityReportShapeAvailabilityAvailabilityStatus,
    ) -> Self {
        self.availability_status = Some(value);
        self
    }
}

impl Default for CapacityReportShapeAvailability {
    fn default() -> Self {
        Self::new()
    }
}
