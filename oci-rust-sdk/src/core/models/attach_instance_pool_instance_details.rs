use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An instance that is to be attached to an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachInstancePoolInstanceDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

/// Required fields for AttachInstancePoolInstanceDetails
pub struct AttachInstancePoolInstanceDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

impl AttachInstancePoolInstanceDetails {
    /// Create a new AttachInstancePoolInstanceDetails with required fields
    pub fn new(required: AttachInstancePoolInstanceDetailsRequired) -> Self {
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
