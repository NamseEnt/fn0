use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides detailed information about the lifecycle stage promotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoteSoftwareSourceToLifecycleStageDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

impl PromoteSoftwareSourceToLifecycleStageDetails {
    /// Create a new PromoteSoftwareSourceToLifecycleStageDetails
    pub fn new() -> Self {
        Self {
            work_request_details: None,
        }
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}

impl Default for PromoteSoftwareSourceToLifecycleStageDetails {
    fn default() -> Self {
        Self::new()
    }
}
