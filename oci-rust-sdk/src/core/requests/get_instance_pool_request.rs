use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstancePoolRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,
}

/// Required fields for GetInstancePoolRequest
pub struct GetInstancePoolRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,
}

impl GetInstancePoolRequest {
    /// Create a new GetInstancePoolRequest with required fields
    pub fn new(required: GetInstancePoolRequestRequired) -> Self {
        Self {
            instance_pool_id: required.instance_pool_id,
        }
    }

    /// Set instance_pool_id
    pub fn set_instance_pool_id(mut self, value: String) -> Self {
        self.instance_pool_id = value;
        self
    }
}
