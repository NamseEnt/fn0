use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides additional information about the kernel event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelEventAdditionalDetails {
    pub vmcore: VmcoreDetails,
}

/// Required fields for KernelEventAdditionalDetails
pub struct KernelEventAdditionalDetailsRequired {
    pub vmcore: VmcoreDetails,
}

impl KernelEventAdditionalDetails {
    /// Create a new KernelEventAdditionalDetails with required fields
    pub fn new(required: KernelEventAdditionalDetailsRequired) -> Self {
        Self {
            vmcore: required.vmcore,
        }
    }

    /// Set vmcore
    pub fn set_vmcore(mut self, value: VmcoreDetails) -> Self {
        self.vmcore = value;
        self
    }
}
