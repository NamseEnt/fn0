use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstancePoolInstanceRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}


/// Required fields for GetInstancePoolInstanceRequest
pub struct GetInstancePoolInstanceRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub instance_pool_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

impl GetInstancePoolInstanceRequest {
    /// Create a new GetInstancePoolInstanceRequest with required fields
    pub fn new(required: GetInstancePoolInstanceRequestRequired) -> Self {
        Self {
            instance_pool_id: required.instance_pool_id,

            instance_id: required.instance_id,
}
    }

    /// Set instance_pool_id
    pub fn set_instance_pool_id(mut self, value: String) -> Self {
        self.instance_pool_id = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }
}


