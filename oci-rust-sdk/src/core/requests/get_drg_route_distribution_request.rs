use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDrgRouteDistributionRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,
}


/// Required fields for GetDrgRouteDistributionRequest
pub struct GetDrgRouteDistributionRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,
}

impl GetDrgRouteDistributionRequest {
    /// Create a new GetDrgRouteDistributionRequest with required fields
    pub fn new(required: GetDrgRouteDistributionRequestRequired) -> Self {
        Self {
            drg_route_distribution_id: required.drg_route_distribution_id,
}
    }

    /// Set drg_route_distribution_id
    pub fn set_drg_route_distribution_id(mut self, value: String) -> Self {
        self.drg_route_distribution_id = value;
        self
    }
}


