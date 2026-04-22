use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Helper definition required to perform authZ using SPLAT expressions on a Compartment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompartmentInternal {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl CompartmentInternal {
    /// Create a new CompartmentInternal
    pub fn new() -> Self {
        Self { id: None }
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }
}

impl Default for CompartmentInternal {
    fn default() -> Self {
        Self::new()
    }
}
