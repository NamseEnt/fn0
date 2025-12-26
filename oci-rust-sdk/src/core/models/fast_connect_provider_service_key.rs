use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A provider service key and its details. A provider service key is an identifier for a provider's virtual circuit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FastConnectProviderServiceKey {
    /// The service key that the provider gives you when you set up a virtual circuit connection from the provider to Oracle Cloud Infrastructure. Use this value as the {@code providerServiceKeyName} query parameter for {@link #getFastConnectProviderServiceKey(GetFastConnectProviderServiceKeyRequest) getFastConnectProviderServiceKey}.
    pub name: String,

    /// The provisioned data rate of the connection. To get a list of the available bandwidth levels (that is, shapes), see {@link #listFastConnectProviderVirtualCircuitBandwidthShapes(ListFastConnectProviderVirtualCircuitBandwidthShapesRequest) listFastConnectProviderVirtualCircuitBandwidthShapes}. <p> Example: {@code 10 Gbps}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_shape_name: Option<String>,

    /// The provider's peering location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peering_location: Option<String>,
}

/// Required fields for FastConnectProviderServiceKey
pub struct FastConnectProviderServiceKeyRequired {
    /// The service key that the provider gives you when you set up a virtual circuit connection from the provider to Oracle Cloud Infrastructure. Use this value as the {@code providerServiceKeyName} query parameter for {@link #getFastConnectProviderServiceKey(GetFastConnectProviderServiceKeyRequest) getFastConnectProviderServiceKey}.
    pub name: String,
}

impl FastConnectProviderServiceKey {
    /// Create a new FastConnectProviderServiceKey with required fields
    pub fn new(required: FastConnectProviderServiceKeyRequired) -> Self {
        Self {
            name: required.name,

            bandwidth_shape_name: None,

            peering_location: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set bandwidth_shape_name
    pub fn set_bandwidth_shape_name(mut self, value: Option<String>) -> Self {
        self.bandwidth_shape_name = value;
        self
    }

    /// Set peering_location
    pub fn set_peering_location(mut self, value: Option<String>) -> Self {
        self.peering_location = value;
        self
    }

    /// Set bandwidth_shape_name (unwraps Option)
    pub fn with_bandwidth_shape_name(mut self, value: impl Into<String>) -> Self {
        self.bandwidth_shape_name = Some(value.into());
        self
    }

    /// Set peering_location (unwraps Option)
    pub fn with_peering_location(mut self, value: impl Into<String>) -> Self {
        self.peering_location = Some(value.into());
        self
    }
}
