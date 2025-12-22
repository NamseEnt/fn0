use serde::{Deserialize, Serialize};

/// The configuration of plugins associated with this instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceAgentPluginConfigDetails {
    /// The plugin name. To get a list of available plugins, use the
    /// ListInstanceagentAvailablePlugins operation in the Oracle Cloud Agent API.
    pub name: String,

    /// Whether the plugin should be enabled or disabled.
    /// To enable the monitoring and management plugins, the isMonitoringDisabled and
    /// isManagementDisabled attributes must also be set to false.
    pub desired_state: DesiredState,
}

/// The desired state of the plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DesiredState {
    Enabled,
    Disabled,
}

/// Required fields for InstanceAgentPluginConfigDetails
pub struct InstanceAgentPluginConfigDetailsRequired {
    pub name: String,
    pub desired_state: DesiredState,
}

impl InstanceAgentPluginConfigDetails {
    /// Create new instance with required fields
    pub fn new(required: InstanceAgentPluginConfigDetailsRequired) -> Self {
        Self {
            name: required.name,
            desired_state: required.desired_state,
        }
    }

    /// Set the plugin name
    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Set the desired state
    pub fn set_desired_state(mut self, desired_state: DesiredState) -> Self {
        self.desired_state = desired_state;
        self
    }

    /// Factory method to create enabled plugin config
    pub fn enabled(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            desired_state: DesiredState::Enabled,
        }
    }

    /// Factory method to create disabled plugin config
    pub fn disabled(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            desired_state: DesiredState::Disabled,
        }
    }
}
