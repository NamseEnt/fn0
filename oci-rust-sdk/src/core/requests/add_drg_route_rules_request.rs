use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDrgRouteRulesRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Request for one or more route rules to be inserted into the DRG route table.
    pub add_drg_route_rules_details: AddDrgRouteRulesDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,
}

/// Required fields for AddDrgRouteRulesRequest
pub struct AddDrgRouteRulesRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub drg_route_table_id: String,

    /// Request for one or more route rules to be inserted into the DRG route table.
    pub add_drg_route_rules_details: AddDrgRouteRulesDetails,
}

impl AddDrgRouteRulesRequest {
    /// Create a new AddDrgRouteRulesRequest with required fields
    pub fn new(required: AddDrgRouteRulesRequestRequired) -> Self {
        Self {
            drg_route_table_id: required.drg_route_table_id,

            add_drg_route_rules_details: required.add_drg_route_rules_details,

            opc_retry_token: None,
        }
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: String) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set add_drg_route_rules_details
    pub fn set_add_drg_route_rules_details(mut self, value: AddDrgRouteRulesDetails) -> Self {
        self.add_drg_route_rules_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }
}
