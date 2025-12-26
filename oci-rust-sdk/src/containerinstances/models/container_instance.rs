use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A container instance to host containers. <p> If you delete a container instance, the record remains visible for a short period of time before being permanently removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInstance {
    /// An OCID that cannot be changed.
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The availability domain to place the container instance.
    pub availability_domain: String,

    /// The current state of the container instance.
    pub lifecycle_state: ContainerInstanceLifecycleState,

    /// The containers on the container instance.
    pub containers: Vec<ContainerInstanceContainer>,

    /// The number of containers on the container instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub container_count: i64,

    /// The time the container instance was created, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The shape of the container instance. The shape determines the number of OCPUs, amount of memory, and other resources that are allocated to a container instance.
    pub shape: String,

    pub shape_config: ContainerInstanceShapeConfig,

    /// The virtual networks available to the containers in the container instance.
    pub vnics: Vec<ContainerVnic>,

    /// The container restart policy is applied for all containers in container instance.
    pub container_restart_policy: ContainerInstanceContainerRestartPolicy,

    /// Simple key-value pair that is applied without any predefined name, type or scope. Exists for cross-compatibility only. Example: {@code {\"bar-key\": \"value\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Usage of system tag keys. These predefined keys are scoped to namespaces. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The fault domain to place the container instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// A message that describes the current state of the container in more detail. Can be used to provide actionable information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<String>,

    /// A volume is a directory with data that is accessible across multiple containers in a container instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<ContainerVolume>>,

    /// The number of volumes that are attached to the container instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_count: Option<i64>,

    /// The time the container instance was updated, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_config: Option<ContainerDnsConfig>,

    /// The amount of time that processes in a container have to gracefully end when the container must be stopped. For example, when you delete a container instance. After the timeout is reached, the processes are sent a signal to be deleted. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graceful_shutdown_timeout_in_seconds: Option<i64>,

    /// The image pulls secrets so you can access private registry to pull container images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_pull_secrets: Option<Vec<ImagePullSecret>>,
}

/// Required fields for ContainerInstance
pub struct ContainerInstanceRequired {
    /// An OCID that cannot be changed.
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the compartment.
    pub compartment_id: String,

    /// The availability domain to place the container instance.
    pub availability_domain: String,

    /// The current state of the container instance.
    pub lifecycle_state: ContainerInstanceLifecycleState,

    /// The containers on the container instance.
    pub containers: Vec<ContainerInstanceContainer>,

    /// The number of containers on the container instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub container_count: i64,

    /// The time the container instance was created, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The shape of the container instance. The shape determines the number of OCPUs, amount of memory, and other resources that are allocated to a container instance.
    pub shape: String,

    pub shape_config: ContainerInstanceShapeConfig,

    /// The virtual networks available to the containers in the container instance.
    pub vnics: Vec<ContainerVnic>,

    /// The container restart policy is applied for all containers in container instance.
    pub container_restart_policy: ContainerInstanceContainerRestartPolicy,
}

impl ContainerInstance {
    /// Create a new ContainerInstance with required fields
    pub fn new(required: ContainerInstanceRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,

            compartment_id: required.compartment_id,

            availability_domain: required.availability_domain,

            lifecycle_state: required.lifecycle_state,

            containers: required.containers,

            container_count: required.container_count,

            time_created: required.time_created,

            shape: required.shape,

            shape_config: required.shape_config,

            vnics: required.vnics,

            container_restart_policy: required.container_restart_policy,

            freeform_tags: None,

            defined_tags: None,

            system_tags: None,

            fault_domain: None,

            lifecycle_details: None,

            volumes: None,

            volume_count: None,

            time_updated: None,

            dns_config: None,

            graceful_shutdown_timeout_in_seconds: None,

            image_pull_secrets: None,
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
    pub fn set_lifecycle_state(mut self, value: ContainerInstanceLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(mut self, value: Option<String>) -> Self {
        self.lifecycle_details = value;
        self
    }

    /// Set volumes
    pub fn set_volumes(mut self, value: Option<Vec<ContainerVolume>>) -> Self {
        self.volumes = value;
        self
    }

    /// Set volume_count
    pub fn set_volume_count(mut self, value: Option<i64>) -> Self {
        self.volume_count = value;
        self
    }

    /// Set containers
    pub fn set_containers(mut self, value: Vec<ContainerInstanceContainer>) -> Self {
        self.containers = value;
        self
    }

    /// Set container_count
    pub fn set_container_count(mut self, value: i64) -> Self {
        self.container_count = value;
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

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set shape_config
    pub fn set_shape_config(mut self, value: ContainerInstanceShapeConfig) -> Self {
        self.shape_config = value;
        self
    }

    /// Set vnics
    pub fn set_vnics(mut self, value: Vec<ContainerVnic>) -> Self {
        self.vnics = value;
        self
    }

    /// Set dns_config
    pub fn set_dns_config(mut self, value: Option<ContainerDnsConfig>) -> Self {
        self.dns_config = value;
        self
    }

    /// Set graceful_shutdown_timeout_in_seconds
    pub fn set_graceful_shutdown_timeout_in_seconds(mut self, value: Option<i64>) -> Self {
        self.graceful_shutdown_timeout_in_seconds = value;
        self
    }

    /// Set image_pull_secrets
    pub fn set_image_pull_secrets(mut self, value: Option<Vec<ImagePullSecret>>) -> Self {
        self.image_pull_secrets = value;
        self
    }

    /// Set container_restart_policy
    pub fn set_container_restart_policy(
        mut self,
        value: ContainerInstanceContainerRestartPolicy,
    ) -> Self {
        self.container_restart_policy = value;
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

    /// Set volumes (unwraps Option)
    pub fn with_volumes(mut self, value: Vec<ContainerVolume>) -> Self {
        self.volumes = Some(value);
        self
    }

    /// Set volume_count (unwraps Option)
    pub fn with_volume_count(mut self, value: i64) -> Self {
        self.volume_count = Some(value);
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
        self
    }

    /// Set dns_config (unwraps Option)
    pub fn with_dns_config(mut self, value: ContainerDnsConfig) -> Self {
        self.dns_config = Some(value);
        self
    }

    /// Set graceful_shutdown_timeout_in_seconds (unwraps Option)
    pub fn with_graceful_shutdown_timeout_in_seconds(mut self, value: i64) -> Self {
        self.graceful_shutdown_timeout_in_seconds = Some(value);
        self
    }

    /// Set image_pull_secrets (unwraps Option)
    pub fn with_image_pull_secrets(mut self, value: Vec<ImagePullSecret>) -> Self {
        self.image_pull_secrets = Some(value);
        self
    }
}
