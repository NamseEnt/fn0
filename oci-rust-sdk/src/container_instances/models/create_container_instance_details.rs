use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerInstanceDetails {
    pub compartment_id: String,
    pub availability_domain: String,
    pub shape: String,
    pub shape_config: CreateContainerInstanceShapeConfigDetails,
    pub containers: Vec<CreateContainerDetails>,
    pub vnics: Vec<CreateContainerVnicDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub graceful_shutdown_timeout_in_seconds: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_restart_policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateContainerInstanceDetails
pub struct CreateContainerInstanceDetailsRequired {
    pub compartment_id: String,
    pub availability_domain: String,
    pub shape: String,
    pub shape_config: CreateContainerInstanceShapeConfigDetails,
    pub containers: Vec<CreateContainerDetails>,
    pub vnics: Vec<CreateContainerVnicDetails>,
}

impl CreateContainerInstanceDetails {
    /// Create new instance with required fields
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
            graceful_shutdown_timeout_in_seconds: None,
            container_restart_policy: None,
            freeform_tags: None,
            defined_tags: None,
        }
    }

    /// Set the compartment ID
    pub fn set_compartment_id(mut self, compartment_id: String) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    /// Set the availability domain
    pub fn set_availability_domain(mut self, availability_domain: String) -> Self {
        self.availability_domain = availability_domain;
        self
    }

    /// Set the shape
    pub fn set_shape(mut self, shape: String) -> Self {
        self.shape = shape;
        self
    }

    /// Set the shape config
    pub fn set_shape_config(mut self, shape_config: CreateContainerInstanceShapeConfigDetails) -> Self {
        self.shape_config = shape_config;
        self
    }

    /// Set the containers
    pub fn set_containers(mut self, containers: Vec<CreateContainerDetails>) -> Self {
        self.containers = containers;
        self
    }

    /// Set the VNICs
    pub fn set_vnics(mut self, vnics: Vec<CreateContainerVnicDetails>) -> Self {
        self.vnics = vnics;
        self
    }

    /// Set the display name
    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Set the fault domain
    pub fn set_fault_domain(mut self, fault_domain: Option<String>) -> Self {
        self.fault_domain = fault_domain;
        self
    }

    /// Set the graceful shutdown timeout in seconds
    pub fn set_graceful_shutdown_timeout_in_seconds(mut self, timeout: Option<i64>) -> Self {
        self.graceful_shutdown_timeout_in_seconds = timeout;
        self
    }

    /// Set the container restart policy
    pub fn set_container_restart_policy(mut self, policy: Option<String>) -> Self {
        self.container_restart_policy = policy;
        self
    }

    /// Set freeform tags
    pub fn set_freeform_tags(mut self, tags: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = tags;
        self
    }

    /// Set defined tags
    pub fn set_defined_tags(mut self, tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.defined_tags = tags;
        self
    }

    /// Set the display name (builder pattern)
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Set the fault domain (builder pattern)
    pub fn with_fault_domain(mut self, fault_domain: impl Into<String>) -> Self {
        self.fault_domain = Some(fault_domain.into());
        self
    }

    /// Set the graceful shutdown timeout in seconds (builder pattern)
    pub fn with_graceful_shutdown_timeout_in_seconds(mut self, timeout: i64) -> Self {
        self.graceful_shutdown_timeout_in_seconds = Some(timeout);
        self
    }

    /// Set the container restart policy (builder pattern)
    pub fn with_container_restart_policy(mut self, policy: impl Into<String>) -> Self {
        self.container_restart_policy = Some(policy.into());
        self
    }

    /// Set freeform tags (builder pattern)
    pub fn with_freeform_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(tags);
        self
    }

    /// Set defined tags (builder pattern)
    pub fn with_defined_tags(mut self, tags: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
        self.defined_tags = Some(tags);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerInstanceShapeConfigDetails {
    pub ocpus: f32,
    pub memory_in_gbs: f32,
}

/// Required fields for CreateContainerInstanceShapeConfigDetails
pub struct CreateContainerInstanceShapeConfigDetailsRequired {
    pub ocpus: f32,
    pub memory_in_gbs: f32,
}

impl CreateContainerInstanceShapeConfigDetails {
    /// Create new instance with required fields
    pub fn new(required: CreateContainerInstanceShapeConfigDetailsRequired) -> Self {
        Self {
            ocpus: required.ocpus,
            memory_in_gbs: required.memory_in_gbs,
        }
    }

    /// Set OCPUs
    pub fn set_ocpus(mut self, ocpus: f32) -> Self {
        self.ocpus = ocpus;
        self
    }

    /// Set memory in GBs
    pub fn set_memory_in_gbs(mut self, memory_in_gbs: f32) -> Self {
        self.memory_in_gbs = memory_in_gbs;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerDetails {
    pub image_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_config: Option<CreateContainerResourceConfigDetails>,
}

/// Required fields for CreateContainerDetails
pub struct CreateContainerDetailsRequired {
    pub image_url: String,
}

impl CreateContainerDetails {
    /// Create new instance with required fields
    pub fn new(required: CreateContainerDetailsRequired) -> Self {
        Self {
            image_url: required.image_url,
            display_name: None,
            command: None,
            arguments: None,
            environment_variables: None,
            resource_config: None,
        }
    }

    /// Set the image URL
    pub fn set_image_url(mut self, image_url: String) -> Self {
        self.image_url = image_url;
        self
    }

    /// Set the display name
    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Set the command
    pub fn set_command(mut self, command: Option<Vec<String>>) -> Self {
        self.command = command;
        self
    }

    /// Set the arguments
    pub fn set_arguments(mut self, arguments: Option<Vec<String>>) -> Self {
        self.arguments = arguments;
        self
    }

    /// Set the environment variables
    pub fn set_environment_variables(mut self, environment_variables: Option<HashMap<String, String>>) -> Self {
        self.environment_variables = environment_variables;
        self
    }

    /// Set the resource config
    pub fn set_resource_config(mut self, resource_config: Option<CreateContainerResourceConfigDetails>) -> Self {
        self.resource_config = resource_config;
        self
    }

    /// Set the display name (builder pattern)
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Set the command (builder pattern)
    pub fn with_command(mut self, command: Vec<String>) -> Self {
        self.command = Some(command);
        self
    }

    /// Set the arguments (builder pattern)
    pub fn with_arguments(mut self, arguments: Vec<String>) -> Self {
        self.arguments = Some(arguments);
        self
    }

    /// Set the environment variables (builder pattern)
    pub fn with_environment_variables(mut self, environment_variables: HashMap<String, String>) -> Self {
        self.environment_variables = Some(environment_variables);
        self
    }

    /// Set the resource config (builder pattern)
    pub fn with_resource_config(mut self, resource_config: CreateContainerResourceConfigDetails) -> Self {
        self.resource_config = Some(resource_config);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerResourceConfigDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpus_limit: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_in_gbs: Option<f32>,
}

// This is PATTERN_A (no required fields)
impl CreateContainerResourceConfigDetails {
    /// Create a new instance with default values
    pub fn new() -> Self {
        Self {
            vcpus_limit: None,
            memory_limit_in_gbs: None,
        }
    }

    /// Set the VCPUs limit
    pub fn set_vcpus_limit(mut self, vcpus_limit: Option<f32>) -> Self {
        self.vcpus_limit = vcpus_limit;
        self
    }

    /// Set the memory limit in GBs
    pub fn set_memory_limit_in_gbs(mut self, memory_limit_in_gbs: Option<f32>) -> Self {
        self.memory_limit_in_gbs = memory_limit_in_gbs;
        self
    }

    /// Set the VCPUs limit (builder pattern)
    pub fn with_vcpus_limit(mut self, vcpus_limit: f32) -> Self {
        self.vcpus_limit = Some(vcpus_limit);
        self
    }

    /// Set the memory limit in GBs (builder pattern)
    pub fn with_memory_limit_in_gbs(mut self, memory_limit_in_gbs: f32) -> Self {
        self.memory_limit_in_gbs = Some(memory_limit_in_gbs);
        self
    }
}

impl Default for CreateContainerResourceConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerVnicDetails {
    pub subnet_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public_ip_assigned: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_source_dest_check: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateContainerVnicDetails
pub struct CreateContainerVnicDetailsRequired {
    pub subnet_id: String,
}

impl CreateContainerVnicDetails {
    /// Create new instance with required fields
    pub fn new(required: CreateContainerVnicDetailsRequired) -> Self {
        Self {
            subnet_id: required.subnet_id,
            display_name: None,
            hostname_label: None,
            is_public_ip_assigned: None,
            skip_source_dest_check: None,
            nsg_ids: None,
            private_ip: None,
            freeform_tags: None,
            defined_tags: None,
        }
    }

    /// Set the subnet ID
    pub fn set_subnet_id(mut self, subnet_id: String) -> Self {
        self.subnet_id = subnet_id;
        self
    }

    /// Set the display name
    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Set the hostname label
    pub fn set_hostname_label(mut self, hostname_label: Option<String>) -> Self {
        self.hostname_label = hostname_label;
        self
    }

    /// Set whether public IP is assigned
    pub fn set_is_public_ip_assigned(mut self, is_public_ip_assigned: Option<bool>) -> Self {
        self.is_public_ip_assigned = is_public_ip_assigned;
        self
    }

    /// Set whether to skip source/destination check
    pub fn set_skip_source_dest_check(mut self, skip_source_dest_check: Option<bool>) -> Self {
        self.skip_source_dest_check = skip_source_dest_check;
        self
    }

    /// Set network security group IDs
    pub fn set_nsg_ids(mut self, nsg_ids: Option<Vec<String>>) -> Self {
        self.nsg_ids = nsg_ids;
        self
    }

    /// Set the private IP
    pub fn set_private_ip(mut self, private_ip: Option<String>) -> Self {
        self.private_ip = private_ip;
        self
    }

    /// Set freeform tags
    pub fn set_freeform_tags(mut self, tags: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = tags;
        self
    }

    /// Set defined tags
    pub fn set_defined_tags(mut self, tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.defined_tags = tags;
        self
    }

    /// Set the display name (builder pattern)
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Set the hostname label (builder pattern)
    pub fn with_hostname_label(mut self, hostname_label: impl Into<String>) -> Self {
        self.hostname_label = Some(hostname_label.into());
        self
    }

    /// Set whether public IP is assigned (builder pattern)
    pub fn with_is_public_ip_assigned(mut self, is_public_ip_assigned: bool) -> Self {
        self.is_public_ip_assigned = Some(is_public_ip_assigned);
        self
    }

    /// Set whether to skip source/destination check (builder pattern)
    pub fn with_skip_source_dest_check(mut self, skip_source_dest_check: bool) -> Self {
        self.skip_source_dest_check = Some(skip_source_dest_check);
        self
    }

    /// Set network security group IDs (builder pattern)
    pub fn with_nsg_ids(mut self, nsg_ids: Vec<String>) -> Self {
        self.nsg_ids = Some(nsg_ids);
        self
    }

    /// Set the private IP (builder pattern)
    pub fn with_private_ip(mut self, private_ip: impl Into<String>) -> Self {
        self.private_ip = Some(private_ip.into());
        self
    }

    /// Set freeform tags (builder pattern)
    pub fn with_freeform_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(tags);
        self
    }

    /// Set defined tags (builder pattern)
    pub fn with_defined_tags(mut self, tags: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
        self.defined_tags = Some(tags);
        self
    }
}
