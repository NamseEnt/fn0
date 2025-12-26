use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFastConnectProviderServiceKeyRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the provider service.
    pub provider_service_id: String,

    /// The provider service key that the provider gives you when you set up a virtual circuit connection from the provider to Oracle Cloud Infrastructure. You can set up that connection and get your provider service key at the provider's website or portal. For the portal location, see the {@code description} attribute of the {@link FastConnectProviderService}.
    pub provider_service_key_name: String,
}

/// Required fields for GetFastConnectProviderServiceKeyRequest
pub struct GetFastConnectProviderServiceKeyRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the provider service.
    pub provider_service_id: String,

    /// The provider service key that the provider gives you when you set up a virtual circuit connection from the provider to Oracle Cloud Infrastructure. You can set up that connection and get your provider service key at the provider's website or portal. For the portal location, see the {@code description} attribute of the {@link FastConnectProviderService}.
    pub provider_service_key_name: String,
}

impl GetFastConnectProviderServiceKeyRequest {
    /// Create a new GetFastConnectProviderServiceKeyRequest with required fields
    pub fn new(required: GetFastConnectProviderServiceKeyRequestRequired) -> Self {
        Self {
            provider_service_id: required.provider_service_id,

            provider_service_key_name: required.provider_service_key_name,
        }
    }

    /// Set provider_service_id
    pub fn set_provider_service_id(mut self, value: String) -> Self {
        self.provider_service_id = value;
        self
    }

    /// Set provider_service_key_name
    pub fn set_provider_service_key_name(mut self, value: String) -> Self {
        self.provider_service_key_name = value;
        self
    }
}
