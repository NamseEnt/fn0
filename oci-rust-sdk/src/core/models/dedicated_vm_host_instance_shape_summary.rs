use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape used to launch instances associated with the dedicated VM host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedVmHostInstanceShapeSummary {
    /// The name of the virtual machine instance shapes that can be launched on a dedicated VM host.
    pub instance_shape_name: String,

    /// The shape's availability domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_capabilities: Option<SupportedCapabilities>,
}

/// Required fields for DedicatedVmHostInstanceShapeSummary
pub struct DedicatedVmHostInstanceShapeSummaryRequired {
    /// The name of the virtual machine instance shapes that can be launched on a dedicated VM host.
    pub instance_shape_name: String,
}

impl DedicatedVmHostInstanceShapeSummary {
    /// Create a new DedicatedVmHostInstanceShapeSummary with required fields
    pub fn new(required: DedicatedVmHostInstanceShapeSummaryRequired) -> Self {
        Self {
            instance_shape_name: required.instance_shape_name,

            availability_domain: None,

            supported_capabilities: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set instance_shape_name
    pub fn set_instance_shape_name(mut self, value: String) -> Self {
        self.instance_shape_name = value;
        self
    }

    /// Set supported_capabilities
    pub fn set_supported_capabilities(mut self, value: Option<SupportedCapabilities>) -> Self {
        self.supported_capabilities = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set supported_capabilities (unwraps Option)
    pub fn with_supported_capabilities(mut self, value: SupportedCapabilities) -> Self {
        self.supported_capabilities = Some(value);
        self
    }
}
