use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A container object for state change attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChange {
    /// Provides the previous state of fields that may have changed during an operation. To determine how the current operation changed a resource, compare the information in this attribute to {@code current}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<HashMap<String, serde_json::Value>>,

    /// Provides the current state of fields that may have changed during an operation. To determine how the current operation changed a resource, compare the information in this attribute to {@code previous}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<HashMap<String, serde_json::Value>>,
}

impl StateChange {
    /// Create a new StateChange
    pub fn new() -> Self {
        Self {
            previous: None,

            current: None,
        }
    }

    /// Set previous
    pub fn set_previous(mut self, value: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.previous = value;
        self
    }

    /// Set current
    pub fn set_current(mut self, value: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.current = value;
        self
    }

    /// Set previous (unwraps Option)
    pub fn with_previous(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.previous = Some(value);
        self
    }

    /// Set current (unwraps Option)
    pub fn with_current(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.current = Some(value);
        self
    }
}

impl Default for StateChange {
    fn default() -> Self {
        Self::new()
    }
}
