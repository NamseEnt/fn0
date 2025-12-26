use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the capabilities that the Dedicated Virtual Machine Host (DVMH) Shape or Virtual Machine Instance Shape could support.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedCapabilities {
    /// Whether the DVMH shape could support confidential VMs or the VM instance shape could be confidential.
    pub is_memory_encryption_supported: bool,
}

/// Required fields for SupportedCapabilities
pub struct SupportedCapabilitiesRequired {
    /// Whether the DVMH shape could support confidential VMs or the VM instance shape could be confidential.
    pub is_memory_encryption_supported: bool,
}

impl SupportedCapabilities {
    /// Create a new SupportedCapabilities with required fields
    pub fn new(required: SupportedCapabilitiesRequired) -> Self {
        Self {
            is_memory_encryption_supported: required.is_memory_encryption_supported,
        }
    }

    /// Set is_memory_encryption_supported
    pub fn set_is_memory_encryption_supported(mut self, value: bool) -> Self {
        self.is_memory_encryption_supported = value;
        self
    }
}
