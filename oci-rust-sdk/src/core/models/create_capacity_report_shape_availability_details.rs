use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the shapes in a capacity report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCapacityReportShapeAvailabilityDetails {
    /// The shape that you want to request a capacity report for. You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    pub instance_shape: String,

    /// The fault domain for the capacity report. <p> If you do not specify a fault domain, the capacity report includes information about all fault domains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_shape_config: Option<CapacityReportInstanceShapeConfig>,
}

/// Required fields for CreateCapacityReportShapeAvailabilityDetails
pub struct CreateCapacityReportShapeAvailabilityDetailsRequired {
    /// The shape that you want to request a capacity report for. You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    pub instance_shape: String,
}

impl CreateCapacityReportShapeAvailabilityDetails {
    /// Create a new CreateCapacityReportShapeAvailabilityDetails with required fields
    pub fn new(required: CreateCapacityReportShapeAvailabilityDetailsRequired) -> Self {
        Self {
            instance_shape: required.instance_shape,

            fault_domain: None,

            instance_shape_config: None,
        }
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set instance_shape
    pub fn set_instance_shape(mut self, value: String) -> Self {
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

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set instance_shape_config (unwraps Option)
    pub fn with_instance_shape_config(mut self, value: CapacityReportInstanceShapeConfig) -> Self {
        self.instance_shape_config = Some(value);
        self
    }
}
