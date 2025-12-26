use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Linux Container capabilities to configure capabilities of container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerCapabilities {
    /// A list of additional configurable container capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_capabilities: Option<Vec<ContainerCapabilitiesAddCapabilities>>,

    /// A list of container capabilities that can be dropped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drop_capabilities: Option<Vec<ContainerCapabilitiesDropCapabilities>>,
}

impl ContainerCapabilities {
    /// Create a new ContainerCapabilities
    pub fn new() -> Self {
        Self {
            add_capabilities: None,

            drop_capabilities: None,
        }
    }

    /// Set add_capabilities
    pub fn set_add_capabilities(
        mut self,
        value: Option<Vec<ContainerCapabilitiesAddCapabilities>>,
    ) -> Self {
        self.add_capabilities = value;
        self
    }

    /// Set drop_capabilities
    pub fn set_drop_capabilities(
        mut self,
        value: Option<Vec<ContainerCapabilitiesDropCapabilities>>,
    ) -> Self {
        self.drop_capabilities = value;
        self
    }

    /// Set add_capabilities (unwraps Option)
    pub fn with_add_capabilities(
        mut self,
        value: Vec<ContainerCapabilitiesAddCapabilities>,
    ) -> Self {
        self.add_capabilities = Some(value);
        self
    }

    /// Set drop_capabilities (unwraps Option)
    pub fn with_drop_capabilities(
        mut self,
        value: Vec<ContainerCapabilitiesDropCapabilities>,
    ) -> Self {
        self.drop_capabilities = Some(value);
        self
    }
}

impl Default for ContainerCapabilities {
    fn default() -> Self {
        Self::new()
    }
}
