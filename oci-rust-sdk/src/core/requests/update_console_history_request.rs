use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConsoleHistoryRequest {
    /// The OCID of the console history.
    pub instance_console_history_id: String,

    /// Update instance fields
    pub update_console_history_details: UpdateConsoleHistoryDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateConsoleHistoryRequest
pub struct UpdateConsoleHistoryRequestRequired {
    /// The OCID of the console history.
    pub instance_console_history_id: String,

    /// Update instance fields
    pub update_console_history_details: UpdateConsoleHistoryDetails,
}

impl UpdateConsoleHistoryRequest {
    /// Create a new UpdateConsoleHistoryRequest with required fields
    pub fn new(required: UpdateConsoleHistoryRequestRequired) -> Self {
        Self {
            instance_console_history_id: required.instance_console_history_id,

            update_console_history_details: required.update_console_history_details,

            if_match: None,
}
    }

    /// Set instance_console_history_id
    pub fn set_instance_console_history_id(mut self, value: String) -> Self {
        self.instance_console_history_id = value;
        self
    }

    /// Set update_console_history_details
    pub fn set_update_console_history_details(mut self, value: UpdateConsoleHistoryDetails) -> Self {
        self.update_console_history_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


