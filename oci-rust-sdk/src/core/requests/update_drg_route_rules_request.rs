use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteRulesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Request to update one or more route rules in the DRG route table.
    pub update_drg_route_rules_details: UpdateDrgRouteRulesDetails,
}


/// Required fields for UpdateDrgRouteRulesRequest
pub struct UpdateDrgRouteRulesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Request to update one or more route rules in the DRG route table.
    pub update_drg_route_rules_details: UpdateDrgRouteRulesDetails,
}

impl UpdateDrgRouteRulesRequest {
    /// Create a new UpdateDrgRouteRulesRequest with required fields
    pub fn new(required: UpdateDrgRouteRulesRequestRequired) -> Self {
        Self {
            drg_route_table_id: required.drg_route_table_id,

            update_drg_route_rules_details: required.update_drg_route_rules_details,
}
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: String) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set update_drg_route_rules_details
    pub fn set_update_drg_route_rules_details(mut self, value: UpdateDrgRouteRulesDetails) -> Self {
        self.update_drg_route_rules_details = value;
        self
    }
}


