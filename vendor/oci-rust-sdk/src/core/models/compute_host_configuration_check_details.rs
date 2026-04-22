use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Compute Host Group Configuration Details Check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeHostConfigurationCheckDetails {
    /// The type of configuration
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ComputeHostConfigurationCheckDetailsType>,

    /// The current state of the host configuration. The Host is either | CONFORMANT - current state matches the desired configuration NON_CONFORMANT - current state does not match the desired configuration PRE_APPLYING, APPLYING, CHECKING- transitional states UNKNOWN - current state is unknown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_state: Option<ConfigurationState>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique firmware bundle associated with the Host Configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_bundle_id: Option<String>,

    /// Preferred recycle level for hosts associated with the reservation config. * {@code SKIP_RECYCLE} - Skips host wipe. * {@code FULL_RECYCLE} - Does not skip host wipe. This is the default behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recycle_level: Option<ComputeHostConfigurationCheckDetailsRecycleLevel>,
}

impl ComputeHostConfigurationCheckDetails {
    /// Create a new ComputeHostConfigurationCheckDetails
    pub fn new() -> Self {
        Self {
            r#type: None,

            configuration_state: None,

            firmware_bundle_id: None,

            recycle_level: None,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: Option<ComputeHostConfigurationCheckDetailsType>) -> Self {
        self.r#type = value;
        self
    }

    /// Set configuration_state
    pub fn set_configuration_state(mut self, value: Option<ConfigurationState>) -> Self {
        self.configuration_state = value;
        self
    }

    /// Set firmware_bundle_id
    pub fn set_firmware_bundle_id(mut self, value: Option<String>) -> Self {
        self.firmware_bundle_id = value;
        self
    }

    /// Set recycle_level
    pub fn set_recycle_level(
        mut self,
        value: Option<ComputeHostConfigurationCheckDetailsRecycleLevel>,
    ) -> Self {
        self.recycle_level = value;
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: ComputeHostConfigurationCheckDetailsType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Set configuration_state (unwraps Option)
    pub fn with_configuration_state(mut self, value: ConfigurationState) -> Self {
        self.configuration_state = Some(value);
        self
    }

    /// Set firmware_bundle_id (unwraps Option)
    pub fn with_firmware_bundle_id(mut self, value: impl Into<String>) -> Self {
        self.firmware_bundle_id = Some(value.into());
        self
    }

    /// Set recycle_level (unwraps Option)
    pub fn with_recycle_level(
        mut self,
        value: ComputeHostConfigurationCheckDetailsRecycleLevel,
    ) -> Self {
        self.recycle_level = Some(value);
        self
    }
}

impl Default for ComputeHostConfigurationCheckDetails {
    fn default() -> Self {
        Self::new()
    }
}
