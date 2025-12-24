use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Information to create a new container within a container instance. <p> The container created by this call contains both the tags specified in this object and any tags specified in the parent container instance. <p> The container is created in the same compartment, availability domain, and fault domain as its container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerDetails {
    /// A URL identifying the image that the container runs in, such as docker.io/library/busybox:latest. If you do not provide a tag, the tag will default to latest. <p> If no registry is provided, will default the registry to public docker hub {@code docker.io/library}. <p> The registry used for container image must be reachable over the Container Instance's VNIC.
    pub image_url: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information. If you don't provide a name, a name is generated automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// An optional command that overrides the ENTRYPOINT process. If you do not provide a value, the existing ENTRYPOINT process defined in the image is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    /// A list of string arguments for a container's ENTRYPOINT process. <p> Many containers use an ENTRYPOINT process pointing to a shell (/bin/bash). For those containers, this argument list specifies the main command in the container process. <p> The total size of all arguments combined must be 64 KB or smaller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,

    /// The working directory within the container's filesystem for the container process. If not specified, the default working directory from the image is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// A map of additional environment variables to set in the environment of the container's ENTRYPOINT process. These variables are in addition to any variables already defined in the container's image. <p> The total size of all environment variables combined, name and values, must be 64 KB or smaller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<HashMap<String, String>>,

    /// List of the volume mounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_mounts: Option<Vec<CreateVolumeMountDetails>>,

    /// Determines if the container will have access to the container instance resource principal. <p> This method utilizes resource principal version 2.2. For information on how to use the exposed resource principal elements, see https://docs.oracle.com/en-us/iaas/Content/API/Concepts/sdk_authentication_methods.htm#sdk_authentication_methods_resource_principal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_resource_principal_disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_config: Option<CreateContainerResourceConfigDetails>,

    /// list of container health checks to check container status and take appropriate action if container status is failed. There are two types of health checks that we currently support HTTP and TCP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_checks: Option<Vec<CreateContainerHealthCheckDetails>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_context: Option<CreateLinuxSecurityContextDetails>,

    /// Simple key-value pair that is applied without any predefined name, type or scope. Exists for cross-compatibility only. Example: {@code {\"bar-key\": \"value\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateContainerDetails
pub struct CreateContainerDetailsRequired {
    /// A URL identifying the image that the container runs in, such as docker.io/library/busybox:latest. If you do not provide a tag, the tag will default to latest. <p> If no registry is provided, will default the registry to public docker hub {@code docker.io/library}. <p> The registry used for container image must be reachable over the Container Instance's VNIC.
    pub image_url: String,
}

impl CreateContainerDetails {
    /// Create a new CreateContainerDetails with required fields
    pub fn new(required: CreateContainerDetailsRequired) -> Self {
        Self {
            image_url: required.image_url,

            display_name: None,

            command: None,

            arguments: None,

            working_directory: None,

            environment_variables: None,

            volume_mounts: None,

            is_resource_principal_disabled: None,

            resource_config: None,

            health_checks: None,

            security_context: None,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set image_url
    pub fn set_image_url(mut self, value: String) -> Self {
        self.image_url = value;
        self
    }

    /// Set command
    pub fn set_command(mut self, value: Option<Vec<String>>) -> Self {
        self.command = value;
        self
    }

    /// Set arguments
    pub fn set_arguments(mut self, value: Option<Vec<String>>) -> Self {
        self.arguments = value;
        self
    }

    /// Set working_directory
    pub fn set_working_directory(mut self, value: Option<String>) -> Self {
        self.working_directory = value;
        self
    }

    /// Set environment_variables
    pub fn set_environment_variables(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.environment_variables = value;
        self
    }

    /// Set volume_mounts
    pub fn set_volume_mounts(mut self, value: Option<Vec<CreateVolumeMountDetails>>) -> Self {
        self.volume_mounts = value;
        self
    }

    /// Set is_resource_principal_disabled
    pub fn set_is_resource_principal_disabled(mut self, value: Option<bool>) -> Self {
        self.is_resource_principal_disabled = value;
        self
    }

    /// Set resource_config
    pub fn set_resource_config(
        mut self,
        value: Option<CreateContainerResourceConfigDetails>,
    ) -> Self {
        self.resource_config = value;
        self
    }

    /// Set health_checks
    pub fn set_health_checks(
        mut self,
        value: Option<Vec<CreateContainerHealthCheckDetails>>,
    ) -> Self {
        self.health_checks = value;
        self
    }

    /// Set security_context
    pub fn set_security_context(
        mut self,
        value: Option<CreateLinuxSecurityContextDetails>,
    ) -> Self {
        self.security_context = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set command (unwraps Option)
    pub fn with_command(mut self, value: Vec<String>) -> Self {
        self.command = Some(value);
        self
    }

    /// Set arguments (unwraps Option)
    pub fn with_arguments(mut self, value: Vec<String>) -> Self {
        self.arguments = Some(value);
        self
    }

    /// Set working_directory (unwraps Option)
    pub fn with_working_directory(mut self, value: impl Into<String>) -> Self {
        self.working_directory = Some(value.into());
        self
    }

    /// Set environment_variables (unwraps Option)
    pub fn with_environment_variables(mut self, value: HashMap<String, String>) -> Self {
        self.environment_variables = Some(value);
        self
    }

    /// Set volume_mounts (unwraps Option)
    pub fn with_volume_mounts(mut self, value: Vec<CreateVolumeMountDetails>) -> Self {
        self.volume_mounts = Some(value);
        self
    }

    /// Set is_resource_principal_disabled (unwraps Option)
    pub fn with_is_resource_principal_disabled(mut self, value: bool) -> Self {
        self.is_resource_principal_disabled = Some(value);
        self
    }

    /// Set resource_config (unwraps Option)
    pub fn with_resource_config(mut self, value: CreateContainerResourceConfigDetails) -> Self {
        self.resource_config = Some(value);
        self
    }

    /// Set health_checks (unwraps Option)
    pub fn with_health_checks(mut self, value: Vec<CreateContainerHealthCheckDetails>) -> Self {
        self.health_checks = Some(value);
        self
    }

    /// Set security_context (unwraps Option)
    pub fn with_security_context(mut self, value: CreateLinuxSecurityContextDetails) -> Self {
        self.security_context = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }
}
