use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDrgRouteDistributionStatementsRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,

    /// Request with one or more route distribution statements to be inserted into the route distribution.
    pub add_drg_route_distribution_statements_details: AddDrgRouteDistributionStatementsDetails,
}

/// Required fields for AddDrgRouteDistributionStatementsRequest
pub struct AddDrgRouteDistributionStatementsRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route distribution.
    pub drg_route_distribution_id: String,

    /// Request with one or more route distribution statements to be inserted into the route distribution.
    pub add_drg_route_distribution_statements_details: AddDrgRouteDistributionStatementsDetails,
}

impl AddDrgRouteDistributionStatementsRequest {
    /// Create a new AddDrgRouteDistributionStatementsRequest with required fields
    pub fn new(required: AddDrgRouteDistributionStatementsRequestRequired) -> Self {
        Self {
            drg_route_distribution_id: required.drg_route_distribution_id,

            add_drg_route_distribution_statements_details: required
                .add_drg_route_distribution_statements_details,
        }
    }

    /// Set drg_route_distribution_id
    pub fn set_drg_route_distribution_id(mut self, value: String) -> Self {
        self.drg_route_distribution_id = value;
        self
    }

    /// Set add_drg_route_distribution_statements_details
    pub fn set_add_drg_route_distribution_statements_details(
        mut self,
        value: AddDrgRouteDistributionStatementsDetails,
    ) -> Self {
        self.add_drg_route_distribution_statements_details = value;
        self
    }
}
