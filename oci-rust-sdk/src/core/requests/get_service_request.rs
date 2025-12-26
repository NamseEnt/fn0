use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetServiceRequest {
    /// The service's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub service_id: String,
}


/// Required fields for GetServiceRequest
pub struct GetServiceRequestRequired {
    /// The service's [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub service_id: String,
}

impl GetServiceRequest {
    /// Create a new GetServiceRequest with required fields
    pub fn new(required: GetServiceRequestRequired) -> Self {
        Self {
            service_id: required.service_id,
}
    }

    /// Set service_id
    pub fn set_service_id(mut self, value: String) -> Self {
        self.service_id = value;
        self
    }
}


