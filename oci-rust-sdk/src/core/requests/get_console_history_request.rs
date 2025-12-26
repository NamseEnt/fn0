use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConsoleHistoryRequest {
    /// The OCID of the console history.
    pub instance_console_history_id: String,
}


/// Required fields for GetConsoleHistoryRequest
pub struct GetConsoleHistoryRequestRequired {
    /// The OCID of the console history.
    pub instance_console_history_id: String,
}

impl GetConsoleHistoryRequest {
    /// Create a new GetConsoleHistoryRequest with required fields
    pub fn new(required: GetConsoleHistoryRequestRequired) -> Self {
        Self {
            instance_console_history_id: required.instance_console_history_id,
}
    }

    /// Set instance_console_history_id
    pub fn set_instance_console_history_id(mut self, value: String) -> Self {
        self.instance_console_history_id = value;
        self
    }
}


