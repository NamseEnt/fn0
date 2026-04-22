use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConsoleHistoryContentRequest {
    /// The OCID of the console history.
    pub instance_console_history_id: String,

    /// Offset of the snapshot data to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,

    /// Length of the snapshot data to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<i64>,
}

/// Required fields for GetConsoleHistoryContentRequest
pub struct GetConsoleHistoryContentRequestRequired {
    /// The OCID of the console history.
    pub instance_console_history_id: String,
}

impl GetConsoleHistoryContentRequest {
    /// Create a new GetConsoleHistoryContentRequest with required fields
    pub fn new(required: GetConsoleHistoryContentRequestRequired) -> Self {
        Self {
            instance_console_history_id: required.instance_console_history_id,

            offset: None,

            length: None,
        }
    }

    /// Set instance_console_history_id
    pub fn set_instance_console_history_id(mut self, value: String) -> Self {
        self.instance_console_history_id = value;
        self
    }

    /// Set offset
    pub fn set_offset(mut self, value: Option<i64>) -> Self {
        self.offset = value;
        self
    }

    /// Set length
    pub fn set_length(mut self, value: Option<i64>) -> Self {
        self.length = value;
        self
    }

    /// Set offset (unwraps Option)
    pub fn with_offset(mut self, value: i64) -> Self {
        self.offset = Some(value);
        self
    }

    /// Set length (unwraps Option)
    pub fn with_length(mut self, value: i64) -> Self {
        self.length = Some(value);
        self
    }
}
