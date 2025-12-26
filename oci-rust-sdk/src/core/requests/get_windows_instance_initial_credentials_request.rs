use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWindowsInstanceInitialCredentialsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}


/// Required fields for GetWindowsInstanceInitialCredentialsRequest
pub struct GetWindowsInstanceInitialCredentialsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance.
    pub instance_id: String,
}

impl GetWindowsInstanceInitialCredentialsRequest {
    /// Create a new GetWindowsInstanceInitialCredentialsRequest with required fields
    pub fn new(required: GetWindowsInstanceInitialCredentialsRequestRequired) -> Self {
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


