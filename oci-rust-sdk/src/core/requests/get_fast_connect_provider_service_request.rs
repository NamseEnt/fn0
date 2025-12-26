use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFastConnectProviderServiceRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the provider service.
    pub provider_service_id: String,
}

/// Required fields for GetFastConnectProviderServiceRequest
pub struct GetFastConnectProviderServiceRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the provider service.
    pub provider_service_id: String,
}

impl GetFastConnectProviderServiceRequest {
    /// Create a new GetFastConnectProviderServiceRequest with required fields
    pub fn new(required: GetFastConnectProviderServiceRequestRequired) -> Self {
        Self {
            provider_service_id: required.provider_service_id,
        }
    }

    /// Set provider_service_id
    pub fn set_provider_service_id(mut self, value: String) -> Self {
        self.provider_service_id = value;
        self
    }
}
