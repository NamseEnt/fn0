use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of changes to make to the state of the modules, streams, and profiles on a managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageModuleStreamsOnManagedInstanceGroupDetails {
    /// Indicates if this operation is a dry run or if the operation should be committed.  If set to true, the result of the operation will be evaluated but not committed.  If set to false, the operation is committed to the managed instance(s).  The default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_dry_run: Option<bool>,

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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

impl ManageModuleStreamsOnManagedInstanceGroupDetails {
    /// Create a new ManageModuleStreamsOnManagedInstanceGroupDetails
    pub fn new() -> Self {
        Self {
            is_dry_run: None,

            enable: None,

            disable: None,

            install: None,

            remove: None,

            work_request_details: None,
        }
    }

    /// Set is_dry_run
    pub fn set_is_dry_run(mut self, value: Option<bool>) -> Self {
        self.is_dry_run = value;
        self
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

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set is_dry_run (unwraps Option)
    pub fn with_is_dry_run(mut self, value: bool) -> Self {
        self.is_dry_run = Some(value);
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

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}

impl Default for ManageModuleStreamsOnManagedInstanceGroupDetails {
    fn default() -> Self {
        Self::new()
    }
}
