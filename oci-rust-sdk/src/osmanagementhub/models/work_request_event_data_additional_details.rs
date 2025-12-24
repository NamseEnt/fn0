use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides additional information for the work request associated with an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestEventDataAdditionalDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource that triggered the event, such as scheduled job id.
    pub initiator_id: String,

    /// List of all work request [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) associated with the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_ids: Option<Vec<String>>,
}

/// Required fields for WorkRequestEventDataAdditionalDetails
pub struct WorkRequestEventDataAdditionalDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource that triggered the event, such as scheduled job id.
    pub initiator_id: String,
}

impl WorkRequestEventDataAdditionalDetails {
    /// Create a new WorkRequestEventDataAdditionalDetails with required fields
    pub fn new(required: WorkRequestEventDataAdditionalDetailsRequired) -> Self {
        Self {
            initiator_id: required.initiator_id,

            work_request_ids: None,
        }
    }

    /// Set initiator_id
    pub fn set_initiator_id(mut self, value: String) -> Self {
        self.initiator_id = value;
        self
    }

    /// Set work_request_ids
    pub fn set_work_request_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.work_request_ids = value;
        self
    }

    /// Set work_request_ids (unwraps Option)
    pub fn with_work_request_ids(mut self, value: Vec<String>) -> Self {
        self.work_request_ids = Some(value);
        self
    }
}
