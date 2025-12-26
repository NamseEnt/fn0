use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A capacity source of bare metal hosts that is dedicated to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedicatedCapacitySource {
    pub capacity_type: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment of this capacity source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,
}

/// Required fields for DedicatedCapacitySource
pub struct DedicatedCapacitySourceRequired {
    pub capacity_type: String,
}

impl DedicatedCapacitySource {
    /// Create a new DedicatedCapacitySource with required fields
    pub fn new(required: DedicatedCapacitySourceRequired) -> Self {
        Self {
            capacity_type: required.capacity_type,

            compartment_id: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set capacity_type
    pub fn set_capacity_type(mut self, value: String) -> Self {
        self.capacity_type = value;
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }
}
