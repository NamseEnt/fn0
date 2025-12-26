use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceIdRequestDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@link Service}.
    pub service_id: String,
}

/// Required fields for ServiceIdRequestDetails
pub struct ServiceIdRequestDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@link Service}.
    pub service_id: String,
}

impl ServiceIdRequestDetails {
    /// Create a new ServiceIdRequestDetails with required fields
    pub fn new(required: ServiceIdRequestDetailsRequired) -> Self {
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
