use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Information to create a container instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerInstanceDetails {
    /// The compartment OCID.
    pub compartment_id: String,

    /// The availability domain where the container instance runs.
    pub availability_domain: String,

    /// The shape of the container instance. The shape determines the resources available to the container instance.
    pub shape: String,

    pub shape_config: CreateContainerInstanceShapeConfigDetails,

    /// The containers to create on this container instance.
    pub containers: Vec<CreateContainerDetails>,

    /// The networks available to containers on this container instance.
    pub vnics: Vec<CreateContainerVnicDetails>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information. If you don't provide a name, a name is generated automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The fault domain where the container instance runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// A volume is a directory with data that is accessible across multiple containers in a container instance. <p> You can attach up to 32 volumes to single container instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<CreateContainerVolumeDetails>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_config: Option<CreateContainerDnsConfigDetails>,

    /// The amount of time that processes in a container have to gracefully end when the container must be stopped. For example, when you delete a container instance. After the timeout is reached, the processes are sent a signal to be deleted. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graceful_shutdown_timeout_in_seconds: Option<i64>,

    /// The image pulls secrets so you can access private registry to pull container images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_pull_secrets: Option<Vec<CreateImagePullSecretDetails>>,

    /// Container restart policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_restart_policy: Option<String>,

    /// Simple key-value pair that is applied without any predefined name, type or scope. Exists for cross-compatibility only. Example: {@code {\"bar-key\": \"value\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateContainerInstanceDetails
pub struct CreateContainerInstanceDetailsRequired {
    /// The compartment OCID.
    pub compartment_id: String,

    /// The availability domain where the container instance runs.
    pub availability_domain: String,

    /// The shape of the container instance. The shape determines the resources available to the container instance.
    pub shape: String,

    pub shape_config: CreateContainerInstanceShapeConfigDetails,

    /// The containers to create on this container instance.
    pub containers: Vec<CreateContainerDetails>,

    /// The networks available to containers on this container instance.
    pub vnics: Vec<CreateContainerVnicDetails>,
}

impl CreateContainerInstanceDetails {
    /// Create a new CreateContainerInstanceDetails with required fields
    pub fn new(required: CreateContainerInstanceDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            availability_domain: required.availability_domain,

            shape: required.shape,

            shape_config: required.shape_config,

            containers: required.containers,

            vnics: required.vnics,

            display_name: None,

            fault_domain: None,

            volumes: None,

            dns_config: None,

            graceful_shutdown_timeout_in_seconds: None,

            image_pull_secrets: None,

            container_restart_policy: None,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set shape_config
    pub fn set_shape_config(mut self, value: CreateContainerInstanceShapeConfigDetails) -> Self {
        self.shape_config = value;
        self
    }

    /// Set volumes
    pub fn set_volumes(mut self, value: Option<Vec<CreateContainerVolumeDetails>>) -> Self {
        self.volumes = value;
        self
    }

    /// Set containers
    pub fn set_containers(mut self, value: Vec<CreateContainerDetails>) -> Self {
        self.containers = value;
        self
    }

    /// Set vnics
    pub fn set_vnics(mut self, value: Vec<CreateContainerVnicDetails>) -> Self {
        self.vnics = value;
        self
    }

    /// Set dns_config
    pub fn set_dns_config(mut self, value: Option<CreateContainerDnsConfigDetails>) -> Self {
        self.dns_config = value;
        self
    }

    /// Set graceful_shutdown_timeout_in_seconds
    pub fn set_graceful_shutdown_timeout_in_seconds(mut self, value: Option<i64>) -> Self {
        self.graceful_shutdown_timeout_in_seconds = value;
        self
    }

    /// Set image_pull_secrets
    pub fn set_image_pull_secrets(
        mut self,
        value: Option<Vec<CreateImagePullSecretDetails>>,
    ) -> Self {
        self.image_pull_secrets = value;
        self
    }

    /// Set container_restart_policy
    pub fn set_container_restart_policy(mut self, value: Option<String>) -> Self {
        self.container_restart_policy = value;
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

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set volumes (unwraps Option)
    pub fn with_volumes(mut self, value: Vec<CreateContainerVolumeDetails>) -> Self {
        self.volumes = Some(value);
        self
    }

    /// Set dns_config (unwraps Option)
    pub fn with_dns_config(mut self, value: CreateContainerDnsConfigDetails) -> Self {
        self.dns_config = Some(value);
        self
    }

    /// Set graceful_shutdown_timeout_in_seconds (unwraps Option)
    pub fn with_graceful_shutdown_timeout_in_seconds(mut self, value: i64) -> Self {
        self.graceful_shutdown_timeout_in_seconds = Some(value);
        self
    }

    /// Set image_pull_secrets (unwraps Option)
    pub fn with_image_pull_secrets(mut self, value: Vec<CreateImagePullSecretDetails>) -> Self {
        self.image_pull_secrets = Some(value);
        self
    }

    /// Set container_restart_policy (unwraps Option)
    pub fn with_container_restart_policy(mut self, value: impl Into<String>) -> Self {
        self.container_restart_policy = Some(value.into());
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
