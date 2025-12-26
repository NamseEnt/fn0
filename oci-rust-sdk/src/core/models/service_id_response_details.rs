use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceIdResponseDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service.
    pub service_id: String,

    /// The name of the service.
    pub service_name: String,
}

/// Required fields for ServiceIdResponseDetails
pub struct ServiceIdResponseDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service.
    pub service_id: String,

    /// The name of the service.
    pub service_name: String,
}

impl ServiceIdResponseDetails {
    /// Create a new ServiceIdResponseDetails with required fields
    pub fn new(required: ServiceIdResponseDetailsRequired) -> Self {
        Self {
            service_id: required.service_id,

            service_name: required.service_name,
        }
    }

    /// Set service_id
    pub fn set_service_id(mut self, value: String) -> Self {
        self.service_id = value;
        self
    }

    /// Set service_name
    pub fn set_service_name(mut self, value: String) -> Self {
        self.service_name = value;
        self
    }
}
