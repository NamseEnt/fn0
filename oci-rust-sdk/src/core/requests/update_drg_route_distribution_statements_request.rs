use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteDistributionStatementsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,

    /// Request to update one or more route distribution statements in the route distribution.
    pub update_drg_route_distribution_statements_details:
        UpdateDrgRouteDistributionStatementsDetails,
}

/// Required fields for UpdateDrgRouteDistributionStatementsRequest
pub struct UpdateDrgRouteDistributionStatementsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,

    /// Request to update one or more route distribution statements in the route distribution.
    pub update_drg_route_distribution_statements_details:
        UpdateDrgRouteDistributionStatementsDetails,
}

impl UpdateDrgRouteDistributionStatementsRequest {
    /// Create a new UpdateDrgRouteDistributionStatementsRequest with required fields
    pub fn new(required: UpdateDrgRouteDistributionStatementsRequestRequired) -> Self {
        Self {
            drg_route_distribution_id: required.drg_route_distribution_id,

            update_drg_route_distribution_statements_details: required
                .update_drg_route_distribution_statements_details,
        }
    }

    /// Set drg_route_distribution_id
    pub fn set_drg_route_distribution_id(mut self, value: String) -> Self {
        self.drg_route_distribution_id = value;
        self
    }

    /// Set update_drg_route_distribution_statements_details
    pub fn set_update_drg_route_distribution_statements_details(
        mut self,
        value: UpdateDrgRouteDistributionStatementsDetails,
    ) -> Self {
        self.update_drg_route_distribution_statements_details = value;
        self
    }
}
