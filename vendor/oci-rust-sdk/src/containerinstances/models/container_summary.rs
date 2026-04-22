use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Summary information about a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The compartment [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub compartment_id: String,

    /// The availability domain where the container instance that hosts this container runs.
    pub availability_domain: String,

    /// The current state of the container.
    pub lifecycle_state: String,

    /// The time the the container was created in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container instance on which the container is running.
    pub container_instance_id: String,

    /// A URL identifying the image that the container runs in, such as docker.io/library/busybox:latest. If you do not provide a tag, the tag will default to latest. <p> If no registry is provided, will default the registry to public docker hub {@code docker.io/library}. The registry used for container image must be reachable over the Container Instance's VNIC.
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

    /// The fault domain where the container instance that hosts the container runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// A message that describes the current state of the container in more detail. Can be used to provide actionable information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<String>,

    /// The time the container was updated in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_config: Option<ContainerResourceConfig>,

    /// Determines whether the container will have access to the container instance resource principal. <p> This method utilizes resource principal version 2.2. For information on how to use the exposed resource principal elements, see https://docs.oracle.com/en-us/iaas/Content/API/Concepts/sdk_authentication_methods.htm#sdk_authentication_methods_resource_principal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_resource_principal_disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_context: Option<LinuxSecurityContext>,
}

/// Required fields for ContainerSummary
pub struct ContainerSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container.
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The compartment [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub compartment_id: String,

    /// The availability domain where the container instance that hosts this container runs.
    pub availability_domain: String,

    /// The current state of the container.
    pub lifecycle_state: String,

    /// The time the the container was created in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the container instance on which the container is running.
    pub container_instance_id: String,

    /// A URL identifying the image that the container runs in, such as docker.io/library/busybox:latest. If you do not provide a tag, the tag will default to latest. <p> If no registry is provided, will default the registry to public docker hub {@code docker.io/library}. The registry used for container image must be reachable over the Container Instance's VNIC.
    pub image_url: String,
}

impl ContainerSummary {
    /// Create a new ContainerSummary with required fields
    pub fn new(required: ContainerSummaryRequired) -> Self {
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

            time_updated: None,

            resource_config: None,

            is_resource_principal_disabled: None,

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
    pub fn set_lifecycle_state(mut self, value: String) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(mut self, value: Option<String>) -> Self {
        self.lifecycle_details = value;
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

    /// Set resource_config
    pub fn set_resource_config(mut self, value: Option<ContainerResourceConfig>) -> Self {
        self.resource_config = value;
        self
    }

    /// Set image_url
    pub fn set_image_url(mut self, value: String) -> Self {
        self.image_url = value;
        self
    }

    /// Set is_resource_principal_disabled
    pub fn set_is_resource_principal_disabled(mut self, value: Option<bool>) -> Self {
        self.is_resource_principal_disabled = value;
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

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
        self
    }

    /// Set resource_config (unwraps Option)
    pub fn with_resource_config(mut self, value: ContainerResourceConfig) -> Self {
        self.resource_config = Some(value);
        self
    }

    /// Set is_resource_principal_disabled (unwraps Option)
    pub fn with_is_resource_principal_disabled(mut self, value: bool) -> Self {
        self.is_resource_principal_disabled = Some(value);
        self
    }

    /// Set security_context (unwraps Option)
    pub fn with_security_context(mut self, value: LinuxSecurityContext) -> Self {
        self.security_context = Some(value);
        self
    }
}
