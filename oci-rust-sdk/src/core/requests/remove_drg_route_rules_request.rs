use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDrgRouteRulesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Request to remove one or more route rules in the DRG route table.
    pub remove_drg_route_rules_details: RemoveDrgRouteRulesDetails,
}


/// Required fields for RemoveDrgRouteRulesRequest
pub struct RemoveDrgRouteRulesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Request to remove one or more route rules in the DRG route table.
    pub remove_drg_route_rules_details: RemoveDrgRouteRulesDetails,
}

impl RemoveDrgRouteRulesRequest {
    /// Create a new RemoveDrgRouteRulesRequest with required fields
    pub fn new(required: RemoveDrgRouteRulesRequestRequired) -> Self {
        Self {
            drg_route_table_id: required.drg_route_table_id,

            remove_drg_route_rules_details: required.remove_drg_route_rules_details,
}
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: String) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set remove_drg_route_rules_details
    pub fn set_remove_drg_route_rules_details(mut self, value: RemoveDrgRouteRulesDetails) -> Self {
        self.remove_drg_route_rules_details = value;
        self
    }
}


