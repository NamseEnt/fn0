use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCrossConnectMappingsResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnectMappingDetailsCollection instance.
    pub cross_connect_mapping_details_collection: CrossConnectMappingDetailsCollection,
}


/// Required fields for ListCrossConnectMappingsResponse
pub struct ListCrossConnectMappingsResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnectMappingDetailsCollection instance.
    pub cross_connect_mapping_details_collection: CrossConnectMappingDetailsCollection,
}

impl ListCrossConnectMappingsResponse {
    /// Create a new ListCrossConnectMappingsResponse with required fields
    pub fn new(required: ListCrossConnectMappingsResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            cross_connect_mapping_details_collection: required.cross_connect_mapping_details_collection,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set cross_connect_mapping_details_collection
    pub fn set_cross_connect_mapping_details_collection(mut self, value: CrossConnectMappingDetailsCollection) -> Self {
        self.cross_connect_mapping_details_collection = value;
        self
    }
}


