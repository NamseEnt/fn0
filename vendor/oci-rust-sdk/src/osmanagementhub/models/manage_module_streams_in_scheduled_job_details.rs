use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of changes to make to the state of the modules, streams, and profiles on the managed target.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageModuleStreamsInScheduledJobDetails {
    /// The set of module streams to enable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<Vec<ModuleStreamDetails>>,

    /// The set of module streams to disable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<Vec<ModuleStreamDetails>>,

    /// The set of module stream profiles to install.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<Vec<ModuleStreamProfileDetails>>,

    /// The set of module stream profiles to remove.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<Vec<ModuleStreamProfileDetails>>,
}

impl ManageModuleStreamsInScheduledJobDetails {
    /// Create a new ManageModuleStreamsInScheduledJobDetails
    pub fn new() -> Self {
        Self {
            enable: None,

            disable: None,

            install: None,

            remove: None,
        }
    }

    /// Set enable
    pub fn set_enable(mut self, value: Option<Vec<ModuleStreamDetails>>) -> Self {
        self.enable = value;
        self
    }

    /// Set disable
    pub fn set_disable(mut self, value: Option<Vec<ModuleStreamDetails>>) -> Self {
        self.disable = value;
        self
    }

    /// Set install
    pub fn set_install(mut self, value: Option<Vec<ModuleStreamProfileDetails>>) -> Self {
        self.install = value;
        self
    }

    /// Set remove
    pub fn set_remove(mut self, value: Option<Vec<ModuleStreamProfileDetails>>) -> Self {
        self.remove = value;
        self
    }

    /// Set enable (unwraps Option)
    pub fn with_enable(mut self, value: Vec<ModuleStreamDetails>) -> Self {
        self.enable = Some(value);
        self
    }

    /// Set disable (unwraps Option)
    pub fn with_disable(mut self, value: Vec<ModuleStreamDetails>) -> Self {
        self.disable = Some(value);
        self
    }

    /// Set install (unwraps Option)
    pub fn with_install(mut self, value: Vec<ModuleStreamProfileDetails>) -> Self {
        self.install = Some(value);
        self
    }

    /// Set remove (unwraps Option)
    pub fn with_remove(mut self, value: Vec<ModuleStreamProfileDetails>) -> Self {
        self.remove = Some(value);
        self
    }
}

impl Default for ManageModuleStreamsInScheduledJobDetails {
    fn default() -> Self {
        Self::new()
    }
}
