use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An instance to be marked for termination in an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminationProceedInstancePoolInstanceDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

/// Required fields for TerminationProceedInstancePoolInstanceDetails
pub struct TerminationProceedInstancePoolInstanceDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

impl TerminationProceedInstancePoolInstanceDetails {
    /// Create a new TerminationProceedInstancePoolInstanceDetails with required fields
    pub fn new(required: TerminationProceedInstancePoolInstanceDetailsRequired) -> Self {
        Self {
            instance_id: required.instance_id,
        }
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }
}
