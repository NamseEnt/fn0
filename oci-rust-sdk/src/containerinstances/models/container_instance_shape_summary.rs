use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details about a shape for a container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInstanceShapeSummary {
    /// The name identifying the shape.
    pub name: String,

    /// A short description of the container instance's processor (CPU).
    pub processor_description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpu_options: Option<ShapeOcpuOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_options: Option<ShapeMemoryOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking_bandwidth_options: Option<ShapeNetworkingBandwidthOptions>,
}

/// Required fields for ContainerInstanceShapeSummary
pub struct ContainerInstanceShapeSummaryRequired {
    /// The name identifying the shape.
    pub name: String,

    /// A short description of the container instance's processor (CPU).
    pub processor_description: String,
}

impl ContainerInstanceShapeSummary {
    /// Create a new ContainerInstanceShapeSummary with required fields
    pub fn new(required: ContainerInstanceShapeSummaryRequired) -> Self {
        Self {
            name: required.name,

            processor_description: required.processor_description,

            ocpu_options: None,

            memory_options: None,

            networking_bandwidth_options: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set processor_description
    pub fn set_processor_description(mut self, value: String) -> Self {
        self.processor_description = value;
        self
    }

    /// Set ocpu_options
    pub fn set_ocpu_options(mut self, value: Option<ShapeOcpuOptions>) -> Self {
        self.ocpu_options = value;
        self
    }

    /// Set memory_options
    pub fn set_memory_options(mut self, value: Option<ShapeMemoryOptions>) -> Self {
        self.memory_options = value;
        self
    }

    /// Set networking_bandwidth_options
    pub fn set_networking_bandwidth_options(
        mut self,
        value: Option<ShapeNetworkingBandwidthOptions>,
    ) -> Self {
        self.networking_bandwidth_options = value;
        self
    }

    /// Set ocpu_options (unwraps Option)
    pub fn with_ocpu_options(mut self, value: ShapeOcpuOptions) -> Self {
        self.ocpu_options = Some(value);
        self
    }

    /// Set memory_options (unwraps Option)
    pub fn with_memory_options(mut self, value: ShapeMemoryOptions) -> Self {
        self.memory_options = Some(value);
        self
    }

    /// Set networking_bandwidth_options (unwraps Option)
    pub fn with_networking_bandwidth_options(
        mut self,
        value: ShapeNetworkingBandwidthOptions,
    ) -> Self {
        self.networking_bandwidth_options = Some(value);
        self
    }
}
