use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDrgRouteDistributionResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgRouteDistribution instance.
    pub drg_route_distribution: DrgRouteDistribution,
}


/// Required fields for CreateDrgRouteDistributionResponse
pub struct CreateDrgRouteDistributionResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgRouteDistribution instance.
    pub drg_route_distribution: DrgRouteDistribution,
}

impl CreateDrgRouteDistributionResponse {
    /// Create a new CreateDrgRouteDistributionResponse with required fields
    pub fn new(required: CreateDrgRouteDistributionResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            drg_route_distribution: required.drg_route_distribution,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set drg_route_distribution
    pub fn set_drg_route_distribution(mut self, value: DrgRouteDistribution) -> Self {
        self.drg_route_distribution = value;
        self
    }
}


