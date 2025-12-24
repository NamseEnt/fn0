use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The size and amount of resources available to the container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerInstanceShapeConfigDetails {
    /// The total number of OCPUs available to the container instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub ocpus: i64,

    /// The total amount of memory available to the container instance (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_in_g_bs: Option<i64>,
}

/// Required fields for CreateContainerInstanceShapeConfigDetails
pub struct CreateContainerInstanceShapeConfigDetailsRequired {
    /// The total number of OCPUs available to the container instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub ocpus: i64,
}

impl CreateContainerInstanceShapeConfigDetails {
    /// Create a new CreateContainerInstanceShapeConfigDetails with required fields
    pub fn new(required: CreateContainerInstanceShapeConfigDetailsRequired) -> Self {
        Self {
            ocpus: required.ocpus,

            memory_in_g_bs: None,
        }
    }

    /// Set ocpus
    pub fn set_ocpus(mut self, value: i64) -> Self {
        self.ocpus = value;
        self
    }

    /// Set memory_in_g_bs
    pub fn set_memory_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.memory_in_g_bs = value;
        self
    }

    /// Set memory_in_g_bs (unwraps Option)
    pub fn with_memory_in_g_bs(mut self, value: i64) -> Self {
        self.memory_in_g_bs = Some(value);
        self
    }
}
