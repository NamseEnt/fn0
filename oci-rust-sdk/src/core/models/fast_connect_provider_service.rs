use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A service offering from a supported provider. For more information, see [FastConnect Overview](https://docs.oracle.com/iaas/Content/Network/Concepts/fastconnect.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FastConnectProviderService {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service offered by the provider.
    pub id: String,

    /// Who is responsible for managing the private peering BGP information.
    pub private_peering_bgp_management: FastConnectProviderServicePrivatePeeringBgpManagement,

    /// The name of the provider.
    pub provider_name: String,

    /// The name of the service offered by the provider.
    pub provider_service_name: String,

    /// Who is responsible for managing the public peering BGP information.
    pub public_peering_bgp_management: FastConnectProviderServicePublicPeeringBgpManagement,

    /// Who is responsible for managing the ASN information for the network at the other end of the connection from Oracle.
    pub customer_asn_management: FastConnectProviderServiceCustomerAsnManagement,

    /// Who is responsible for managing the provider service key.
    pub provider_service_key_management: FastConnectProviderServiceProviderServiceKeyManagement,

    /// Who is responsible for managing the virtual circuit bandwidth.
    pub bandwith_shape_management: FastConnectProviderServiceBandwithShapeManagement,

    /// Total number of cross-connect or cross-connect groups required for the virtual circuit. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub required_total_cross_connects: i64,

    /// Provider service type.
    #[serde(rename = "type")]
    pub r#type: FastConnectProviderServiceType,

    /// The location of the provider's website or portal. This portal is where you can get information about the provider service, create a virtual circuit connection from the provider to Oracle Cloud Infrastructure, and retrieve your provider service key for that virtual circuit connection. <p> Example: {@code https://example.com}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// An array of virtual circuit types supported by this service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_virtual_circuit_types:
        Option<Vec<FastConnectProviderServiceSupportedVirtualCircuitTypes>>,
}

/// Required fields for FastConnectProviderService
pub struct FastConnectProviderServiceRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the service offered by the provider.
    pub id: String,

    /// Who is responsible for managing the private peering BGP information.
    pub private_peering_bgp_management: FastConnectProviderServicePrivatePeeringBgpManagement,

    /// The name of the provider.
    pub provider_name: String,

    /// The name of the service offered by the provider.
    pub provider_service_name: String,

    /// Who is responsible for managing the public peering BGP information.
    pub public_peering_bgp_management: FastConnectProviderServicePublicPeeringBgpManagement,

    /// Who is responsible for managing the ASN information for the network at the other end of the connection from Oracle.
    pub customer_asn_management: FastConnectProviderServiceCustomerAsnManagement,

    /// Who is responsible for managing the provider service key.
    pub provider_service_key_management: FastConnectProviderServiceProviderServiceKeyManagement,

    /// Who is responsible for managing the virtual circuit bandwidth.
    pub bandwith_shape_management: FastConnectProviderServiceBandwithShapeManagement,

    /// Total number of cross-connect or cross-connect groups required for the virtual circuit. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub required_total_cross_connects: i64,

    /// Provider service type.
    pub r#type: FastConnectProviderServiceType,
}

impl FastConnectProviderService {
    /// Create a new FastConnectProviderService with required fields
    pub fn new(required: FastConnectProviderServiceRequired) -> Self {
        Self {
            id: required.id,

            private_peering_bgp_management: required.private_peering_bgp_management,

            provider_name: required.provider_name,

            provider_service_name: required.provider_service_name,

            public_peering_bgp_management: required.public_peering_bgp_management,

            customer_asn_management: required.customer_asn_management,

            provider_service_key_management: required.provider_service_key_management,

            bandwith_shape_management: required.bandwith_shape_management,

            required_total_cross_connects: required.required_total_cross_connects,

            r#type: required.r#type,

            description: None,

            supported_virtual_circuit_types: None,
        }
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set private_peering_bgp_management
    pub fn set_private_peering_bgp_management(
        mut self,
        value: FastConnectProviderServicePrivatePeeringBgpManagement,
    ) -> Self {
        self.private_peering_bgp_management = value;
        self
    }

    /// Set provider_name
    pub fn set_provider_name(mut self, value: String) -> Self {
        self.provider_name = value;
        self
    }

    /// Set provider_service_name
    pub fn set_provider_service_name(mut self, value: String) -> Self {
        self.provider_service_name = value;
        self
    }

    /// Set public_peering_bgp_management
    pub fn set_public_peering_bgp_management(
        mut self,
        value: FastConnectProviderServicePublicPeeringBgpManagement,
    ) -> Self {
        self.public_peering_bgp_management = value;
        self
    }

    /// Set supported_virtual_circuit_types
    pub fn set_supported_virtual_circuit_types(
        mut self,
        value: Option<Vec<FastConnectProviderServiceSupportedVirtualCircuitTypes>>,
    ) -> Self {
        self.supported_virtual_circuit_types = value;
        self
    }

    /// Set customer_asn_management
    pub fn set_customer_asn_management(
        mut self,
        value: FastConnectProviderServiceCustomerAsnManagement,
    ) -> Self {
        self.customer_asn_management = value;
        self
    }

    /// Set provider_service_key_management
    pub fn set_provider_service_key_management(
        mut self,
        value: FastConnectProviderServiceProviderServiceKeyManagement,
    ) -> Self {
        self.provider_service_key_management = value;
        self
    }

    /// Set bandwith_shape_management
    pub fn set_bandwith_shape_management(
        mut self,
        value: FastConnectProviderServiceBandwithShapeManagement,
    ) -> Self {
        self.bandwith_shape_management = value;
        self
    }

    /// Set required_total_cross_connects
    pub fn set_required_total_cross_connects(mut self, value: i64) -> Self {
        self.required_total_cross_connects = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: FastConnectProviderServiceType) -> Self {
        self.r#type = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set supported_virtual_circuit_types (unwraps Option)
    pub fn with_supported_virtual_circuit_types(
        mut self,
        value: Vec<FastConnectProviderServiceSupportedVirtualCircuitTypes>,
    ) -> Self {
        self.supported_virtual_circuit_types = Some(value);
        self
    }
}
