use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A single container on a container instance. <p> If you delete a container, the record remains visible for a short period of time before being permanently removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the compartment that contains the container.
    pub compartment_id: String,

    /// The availability domain where the container instance that hosts the container runs.
    pub availability_domain: String,

    /// The current state of the container.
    pub lifecycle_state: ContainerLifecycleState,

    /// The time the container was created, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container instance that the container is running on.
    pub container_instance_id: String,

    /// The container image information. Currently only supports public Docker registry. <p> You can provide either the image name (containerImage), image name with version (containerImagev1), or complete Docker image URL {@code docker.io/library/containerImage:latest}. <p> If you do not provide a registry, the registry defaults to public Docker hub {@code docker.io/library}. The registry used for the container image must be reachable over the VNIC of the container instance.
    pub image_url: String,

    /// Simple key-value pair that is applied without any predefined name, type or scope. Exists for cross-compatibility only. Example: {@code {\"bar-key\": \"value\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Usage of system tag keys. These predefined keys are scoped to namespaces. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The fault domain of the container instance that hosts the container runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// A message that describes the current state of the container in more detail. Can be used to provide actionable information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<String>,

    /// The exit code of the container process when it stopped running. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i64>,

    /// The time when the container last deleted (terminated), in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_terminated: Option<DateTime<Utc>>,

    /// The time the container was updated, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// This command overrides ENTRYPOINT process of the container. If you do not specify this command, the existing ENTRYPOINT process defined in the image is the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    /// A list of string arguments for the ENTRYPOINT process of the container. <p> Many containers use an ENTRYPOINT process pointing to a shell {@code /bin/bash}. For those containers, you can use the argument list to specify the main command in the container process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,

    /// The working directory within the container's filesystem for the container process. If not specified, the default working directory from the image is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// A map of additional environment variables to set in the environment of the ENTRYPOINT process of the container. These variables are in addition to any variables already defined in the container's image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<HashMap<String, String>>,

    /// List of the volume mounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_mounts: Option<Vec<VolumeMount>>,

    /// List of container health checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_checks: Option<Vec<ContainerHealthCheck>>,

    /// Determines if the container will have access to the container instance resource principal. <p> This method utilizes resource principal version 2.2. For more information on how to use the exposed resource principal elements, see https://docs.oracle.com/en-us/iaas/Content/API/Concepts/sdk_authentication_methods.htm#sdk_authentication_methods_resource_principal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_resource_principal_disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_config: Option<ContainerResourceConfig>,

    /// The number of container restart attempts. Depending on the restart policy, a restart might be attempted after a health check failure or a container exit. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_restart_attempt_count: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_context: Option<LinuxSecurityContext>,
}

/// Required fields for Container
pub struct ContainerRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the compartment that contains the container.
    pub compartment_id: String,

    /// The availability domain where the container instance that hosts the container runs.
    pub availability_domain: String,

    /// The current state of the container.
    pub lifecycle_state: ContainerLifecycleState,

    /// The time the container was created, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container instance that the container is running on.
    pub container_instance_id: String,

    /// The container image information. Currently only supports public Docker registry. <p> You can provide either the image name (containerImage), image name with version (containerImagev1), or complete Docker image URL {@code docker.io/library/containerImage:latest}. <p> If you do not provide a registry, the registry defaults to public Docker hub {@code docker.io/library}. The registry used for the container image must be reachable over the VNIC of the container instance.
    pub image_url: String,
}

impl Container {
    /// Create a new Container with required fields
    pub fn new(required: ContainerRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,

            compartment_id: required.compartment_id,

            availability_domain: required.availability_domain,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            container_instance_id: required.container_instance_id,

            image_url: required.image_url,

            freeform_tags: None,

            defined_tags: None,

            system_tags: None,

            fault_domain: None,

            lifecycle_details: None,

            exit_code: None,

            time_terminated: None,

            time_updated: None,

            command: None,

            arguments: None,

            working_directory: None,

            environment_variables: None,

            volume_mounts: None,

            health_checks: None,

            is_resource_principal_disabled: None,

            resource_config: None,

            container_restart_attempt_count: None,

            security_context: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ContainerLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(mut self, value: Option<String>) -> Self {
        self.lifecycle_details = value;
        self
    }

    /// Set exit_code
    pub fn set_exit_code(mut self, value: Option<i64>) -> Self {
        self.exit_code = value;
        self
    }

    /// Set time_terminated
    pub fn set_time_terminated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_terminated = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
        self
    }

    /// Set container_instance_id
    pub fn set_container_instance_id(mut self, value: String) -> Self {
        self.container_instance_id = value;
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
    pub fn set_volume_mounts(mut self, value: Option<Vec<VolumeMount>>) -> Self {
        self.volume_mounts = value;
        self
    }

    /// Set health_checks
    pub fn set_health_checks(mut self, value: Option<Vec<ContainerHealthCheck>>) -> Self {
        self.health_checks = value;
        self
    }

    /// Set is_resource_principal_disabled
    pub fn set_is_resource_principal_disabled(mut self, value: Option<bool>) -> Self {
        self.is_resource_principal_disabled = value;
        self
    }

    /// Set resource_config
    pub fn set_resource_config(mut self, value: Option<ContainerResourceConfig>) -> Self {
        self.resource_config = value;
        self
    }

    /// Set container_restart_attempt_count
    pub fn set_container_restart_attempt_count(mut self, value: Option<i64>) -> Self {
        self.container_restart_attempt_count = value;
        self
    }

    /// Set security_context
    pub fn set_security_context(mut self, value: Option<LinuxSecurityContext>) -> Self {
        self.security_context = value;
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

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set lifecycle_details (unwraps Option)
    pub fn with_lifecycle_details(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_details = Some(value.into());
        self
    }

    /// Set exit_code (unwraps Option)
    pub fn with_exit_code(mut self, value: i64) -> Self {
        self.exit_code = Some(value);
        self
    }

    /// Set time_terminated (unwraps Option)
    pub fn with_time_terminated(mut self, value: DateTime<Utc>) -> Self {
        self.time_terminated = Some(value);
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
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
    pub fn with_volume_mounts(mut self, value: Vec<VolumeMount>) -> Self {
        self.volume_mounts = Some(value);
        self
    }

    /// Set health_checks (unwraps Option)
    pub fn with_health_checks(mut self, value: Vec<ContainerHealthCheck>) -> Self {
        self.health_checks = Some(value);
        self
    }

    /// Set is_resource_principal_disabled (unwraps Option)
    pub fn with_is_resource_principal_disabled(mut self, value: bool) -> Self {
        self.is_resource_principal_disabled = Some(value);
        self
    }

    /// Set resource_config (unwraps Option)
    pub fn with_resource_config(mut self, value: ContainerResourceConfig) -> Self {
        self.resource_config = Some(value);
        self
    }

    /// Set container_restart_attempt_count (unwraps Option)
    pub fn with_container_restart_attempt_count(mut self, value: i64) -> Self {
        self.container_restart_attempt_count = Some(value);
        self
    }

    /// Set security_context (unwraps Option)
    pub fn with_security_context(mut self, value: LinuxSecurityContext) -> Self {
        self.security_context = Some(value);
        self
    }
}
