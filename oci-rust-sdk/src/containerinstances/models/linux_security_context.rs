use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Security context for Linux container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxSecurityContext {
    pub security_context_type: String,

    /// The user ID (UID) to run the entrypoint process of the container. Defaults to user specified UID in container image metadata if not provided. This must be provided if runAsGroup is provided. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_as_user: Option<i64>,

    /// The group ID (GID) to run the entrypoint process of the container. Uses runtime default if not provided. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_as_group: Option<i64>,

    /// Indicates if the container must run as a non-root user. If true, the service validates the container image at runtime to ensure that it is not going to run with UID 0 (root) and fails the container instance creation if the validation fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_non_root_user_check_enabled: Option<bool>,

    /// Determines if the container will have a read-only root file system. Default value is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_root_file_system_readonly: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ContainerCapabilities>,
}

/// Required fields for LinuxSecurityContext
pub struct LinuxSecurityContextRequired {
    pub security_context_type: String,
}

impl LinuxSecurityContext {
    /// Create a new LinuxSecurityContext with required fields
    pub fn new(required: LinuxSecurityContextRequired) -> Self {
        Self {
            security_context_type: required.security_context_type,

            run_as_user: None,

            run_as_group: None,

            is_non_root_user_check_enabled: None,

            is_root_file_system_readonly: None,

            capabilities: None,
        }
    }

    /// Set run_as_user
    pub fn set_run_as_user(mut self, value: Option<i64>) -> Self {
        self.run_as_user = value;
        self
    }

    /// Set run_as_group
    pub fn set_run_as_group(mut self, value: Option<i64>) -> Self {
        self.run_as_group = value;
        self
    }

    /// Set is_non_root_user_check_enabled
    pub fn set_is_non_root_user_check_enabled(mut self, value: Option<bool>) -> Self {
        self.is_non_root_user_check_enabled = value;
        self
    }

    /// Set is_root_file_system_readonly
    pub fn set_is_root_file_system_readonly(mut self, value: Option<bool>) -> Self {
        self.is_root_file_system_readonly = value;
        self
    }

    /// Set capabilities
    pub fn set_capabilities(mut self, value: Option<ContainerCapabilities>) -> Self {
        self.capabilities = value;
        self
    }

    /// Set security_context_type
    pub fn set_security_context_type(mut self, value: String) -> Self {
        self.security_context_type = value;
        self
    }

    /// Set run_as_user (unwraps Option)
    pub fn with_run_as_user(mut self, value: i64) -> Self {
        self.run_as_user = Some(value);
        self
    }

    /// Set run_as_group (unwraps Option)
    pub fn with_run_as_group(mut self, value: i64) -> Self {
        self.run_as_group = Some(value);
        self
    }

    /// Set is_non_root_user_check_enabled (unwraps Option)
    pub fn with_is_non_root_user_check_enabled(mut self, value: bool) -> Self {
        self.is_non_root_user_check_enabled = Some(value);
        self
    }

    /// Set is_root_file_system_readonly (unwraps Option)
    pub fn with_is_root_file_system_readonly(mut self, value: bool) -> Self {
        self.is_root_file_system_readonly = Some(value);
        self
    }

    /// Set capabilities (unwraps Option)
    pub fn with_capabilities(mut self, value: ContainerCapabilities) -> Self {
        self.capabilities = Some(value);
        self
    }
}
