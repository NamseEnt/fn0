use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConsoleHistoryResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ConsoleHistory instance.
    pub console_history: ConsoleHistory,
}

/// Required fields for GetConsoleHistoryResponse
pub struct GetConsoleHistoryResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ConsoleHistory instance.
    pub console_history: ConsoleHistory,
}

impl GetConsoleHistoryResponse {
    /// Create a new GetConsoleHistoryResponse with required fields
    pub fn new(required: GetConsoleHistoryResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            console_history: required.console_history,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set console_history
    pub fn set_console_history(mut self, value: ConsoleHistory) -> Self {
        self.console_history = value;
        self
    }
}
