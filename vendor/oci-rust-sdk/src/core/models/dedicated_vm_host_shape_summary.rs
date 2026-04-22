use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape used to launch the dedicated virtual machine (VM) host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedVmHostShapeSummary {
    /// The name of the dedicated VM host shape. You can enumerate all available shapes by calling {@link #listDedicatedVmHostShapes(ListDedicatedVmHostShapesRequest) listDedicatedVmHostShapes}.
    pub dedicated_vm_host_shape: String,

    /// The shape's availability domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// A list of capacity configs that are supported by this dedicated VM host shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_configs: Option<Vec<CapacityConfig>>,
}

/// Required fields for DedicatedVmHostShapeSummary
pub struct DedicatedVmHostShapeSummaryRequired {
    /// The name of the dedicated VM host shape. You can enumerate all available shapes by calling {@link #listDedicatedVmHostShapes(ListDedicatedVmHostShapesRequest) listDedicatedVmHostShapes}.
    pub dedicated_vm_host_shape: String,
}

impl DedicatedVmHostShapeSummary {
    /// Create a new DedicatedVmHostShapeSummary with required fields
    pub fn new(required: DedicatedVmHostShapeSummaryRequired) -> Self {
        Self {
            dedicated_vm_host_shape: required.dedicated_vm_host_shape,

            availability_domain: None,

            capacity_configs: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set dedicated_vm_host_shape
    pub fn set_dedicated_vm_host_shape(mut self, value: String) -> Self {
        self.dedicated_vm_host_shape = value;
        self
    }

    /// Set capacity_configs
    pub fn set_capacity_configs(mut self, value: Option<Vec<CapacityConfig>>) -> Self {
        self.capacity_configs = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set capacity_configs (unwraps Option)
    pub fn with_capacity_configs(mut self, value: Vec<CapacityConfig>) -> Self {
        self.capacity_configs = Some(value);
        self
    }
}
