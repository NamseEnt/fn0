use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDrgRouteDistributionStatementsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,

    /// Request with one or more route distribution statements to remove from the route distribution.
    pub remove_drg_route_distribution_statements_details:
        RemoveDrgRouteDistributionStatementsDetails,
}

/// Required fields for RemoveDrgRouteDistributionStatementsRequest
pub struct RemoveDrgRouteDistributionStatementsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,

    /// Request with one or more route distribution statements to remove from the route distribution.
    pub remove_drg_route_distribution_statements_details:
        RemoveDrgRouteDistributionStatementsDetails,
}

impl RemoveDrgRouteDistributionStatementsRequest {
    /// Create a new RemoveDrgRouteDistributionStatementsRequest with required fields
    pub fn new(required: RemoveDrgRouteDistributionStatementsRequestRequired) -> Self {
        Self {
            drg_route_distribution_id: required.drg_route_distribution_id,

            remove_drg_route_distribution_statements_details: required
                .remove_drg_route_distribution_statements_details,
        }
    }

    /// Set drg_route_distribution_id
    pub fn set_drg_route_distribution_id(mut self, value: String) -> Self {
        self.drg_route_distribution_id = value;
        self
    }

    /// Set remove_drg_route_distribution_statements_details
    pub fn set_remove_drg_route_distribution_statements_details(
        mut self,
        value: RemoveDrgRouteDistributionStatementsDetails,
    ) -> Self {
        self.remove_drg_route_distribution_statements_details = value;
        self
    }
}
