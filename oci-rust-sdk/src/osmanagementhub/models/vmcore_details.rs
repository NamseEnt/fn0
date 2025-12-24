use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Kernel event vmcore details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmcoreDetails {
    /// Kernel vmcore backtrace.
    pub backtrace: String,

    /// Kernel vmcore component.
    pub component: String,
}

/// Required fields for VmcoreDetails
pub struct VmcoreDetailsRequired {
    /// Kernel vmcore backtrace.
    pub backtrace: String,

    /// Kernel vmcore component.
    pub component: String,
}

impl VmcoreDetails {
    /// Create a new VmcoreDetails with required fields
    pub fn new(required: VmcoreDetailsRequired) -> Self {
        Self {
            backtrace: required.backtrace,

            component: required.component,
        }
    }

    /// Set backtrace
    pub fn set_backtrace(mut self, value: String) -> Self {
        self.backtrace = value;
        self
    }

    /// Set component
    pub fn set_component(mut self, value: String) -> Self {
        self.component = value;
        self
    }
}
