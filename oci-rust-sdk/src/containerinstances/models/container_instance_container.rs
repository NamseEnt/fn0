use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A container on a container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInstanceContainer {
    /// The OCID of the container.
    pub container_id: String,

    /// Display name for the Container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Required fields for ContainerInstanceContainer
pub struct ContainerInstanceContainerRequired {
    /// The OCID of the container.
    pub container_id: String,
}

impl ContainerInstanceContainer {
    /// Create a new ContainerInstanceContainer with required fields
    pub fn new(required: ContainerInstanceContainerRequired) -> Self {
        Self {
            container_id: required.container_id,

            display_name: None,
        }
    }

    /// Set container_id
    pub fn set_container_id(mut self, value: String) -> Self {
        self.container_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
