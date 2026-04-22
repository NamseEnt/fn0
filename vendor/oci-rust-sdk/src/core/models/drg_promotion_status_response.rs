use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The promotion/unpromotion status of a DRG
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgPromotionStatusResponse {
    /// OCID of the DRG
    pub drg_id: String,

    /// The promotion status of the DRG
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drg_promotion_status: Option<DrgPromotionStatusResponseDrgPromotionStatus>,

    /// A map of the promotion status of each RPC connection on this DRG {conn_id -> promo_status}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_promotion_status: Option<HashMap<String, String>>,

    /// A map of the promotion status of each VC on this DRG {conn_id -> promo_status}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vc_promotion_status: Option<HashMap<String, String>>,

    /// A map of the promotion status of each IPSec connection on this DRG {conn_id -> promo_status}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipsec_promotion_status: Option<HashMap<String, String>>,
}

/// Required fields for DrgPromotionStatusResponse
pub struct DrgPromotionStatusResponseRequired {
    /// OCID of the DRG
    pub drg_id: String,
}

impl DrgPromotionStatusResponse {
    /// Create a new DrgPromotionStatusResponse with required fields
    pub fn new(required: DrgPromotionStatusResponseRequired) -> Self {
        Self {
            drg_id: required.drg_id,

            drg_promotion_status: None,

            rpc_promotion_status: None,

            vc_promotion_status: None,

            ipsec_promotion_status: None,
        }
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
        self
    }

    /// Set drg_promotion_status
    pub fn set_drg_promotion_status(
        mut self,
        value: Option<DrgPromotionStatusResponseDrgPromotionStatus>,
    ) -> Self {
        self.drg_promotion_status = value;
        self
    }

    /// Set rpc_promotion_status
    pub fn set_rpc_promotion_status(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.rpc_promotion_status = value;
        self
    }

    /// Set vc_promotion_status
    pub fn set_vc_promotion_status(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.vc_promotion_status = value;
        self
    }

    /// Set ipsec_promotion_status
    pub fn set_ipsec_promotion_status(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.ipsec_promotion_status = value;
        self
    }

    /// Set drg_promotion_status (unwraps Option)
    pub fn with_drg_promotion_status(
        mut self,
        value: DrgPromotionStatusResponseDrgPromotionStatus,
    ) -> Self {
        self.drg_promotion_status = Some(value);
        self
    }

    /// Set rpc_promotion_status (unwraps Option)
    pub fn with_rpc_promotion_status(mut self, value: HashMap<String, String>) -> Self {
        self.rpc_promotion_status = Some(value);
        self
    }

    /// Set vc_promotion_status (unwraps Option)
    pub fn with_vc_promotion_status(mut self, value: HashMap<String, String>) -> Self {
        self.vc_promotion_status = Some(value);
        self
    }

    /// Set ipsec_promotion_status (unwraps Option)
    pub fn with_ipsec_promotion_status(mut self, value: HashMap<String, String>) -> Self {
        self.ipsec_promotion_status = Some(value);
        self
    }
}
