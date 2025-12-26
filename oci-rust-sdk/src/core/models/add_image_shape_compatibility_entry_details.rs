use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Image shape compatibility details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddImageShapeCompatibilityEntryDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_constraints: Option<ImageMemoryConstraints>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpu_constraints: Option<ImageOcpuConstraints>,
}

impl AddImageShapeCompatibilityEntryDetails {
    /// Create a new AddImageShapeCompatibilityEntryDetails
    pub fn new() -> Self {
        Self {
            memory_constraints: None,

            ocpu_constraints: None,
        }
    }

    /// Set memory_constraints
    pub fn set_memory_constraints(mut self, value: Option<ImageMemoryConstraints>) -> Self {
        self.memory_constraints = value;
        self
    }

    /// Set ocpu_constraints
    pub fn set_ocpu_constraints(mut self, value: Option<ImageOcpuConstraints>) -> Self {
        self.ocpu_constraints = value;
        self
    }

    /// Set memory_constraints (unwraps Option)
    pub fn with_memory_constraints(mut self, value: ImageMemoryConstraints) -> Self {
        self.memory_constraints = Some(value);
        self
    }

    /// Set ocpu_constraints (unwraps Option)
    pub fn with_ocpu_constraints(mut self, value: ImageOcpuConstraints) -> Self {
        self.ocpu_constraints = Some(value);
        self
    }
}

impl Default for AddImageShapeCompatibilityEntryDetails {
    fn default() -> Self {
        Self::new()
    }
}
