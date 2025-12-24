use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to move the lifecycle environment to another compartment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLifecycleEnvironmentCompartmentDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to move the lifecycle environment to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,
}

impl ChangeLifecycleEnvironmentCompartmentDetails {
    /// Create a new ChangeLifecycleEnvironmentCompartmentDetails
    pub fn new() -> Self {
        Self {
            compartment_id: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }
}

impl Default for ChangeLifecycleEnvironmentCompartmentDetails {
    fn default() -> Self {
        Self::new()
    }
}
