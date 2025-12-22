use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{CreateVnicDetails, InstanceSourceDetails, LaunchInstanceShapeConfigDetails};

/// Details for launching a new compute instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceDetails {
    /// The OCID of the compartment (required)
    pub compartment_id: String,

    /// The availability domain to place the instance in (required)
    pub availability_domain: String,

    /// Details for creating the primary VNIC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_vnic_details: Option<CreateVnicDetails>,

    /// The OCID of the dedicated VM host to place the instance on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_vm_host_id: Option<String>,

    /// Defined tags for this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name for the instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Custom metadata key/value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Additional metadata key/value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_metadata: Option<HashMap<String, serde_json::Value>>,

    /// Free-form tags for this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The shape of the instance (required)
    pub shape: String,

    /// The shape configuration for the instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_config: Option<LaunchInstanceShapeConfigDetails>,

    /// Details for creating an instance from an image or boot volume
    pub source_details: InstanceSourceDetails,

    /// The fault domain to place the instance in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// The OCID of the compute capacity reservation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    /// Whether to enable in-transit encryption for the boot volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,
}

/// Required fields for launching an instance
pub struct LaunchInstanceDetailsRequired {
    /// The OCID of the compartment
    pub compartment_id: String,
    /// The availability domain to place the instance in
    pub availability_domain: String,
    /// The shape of the instance
    pub shape: String,
    /// Details for creating an instance from an image or boot volume
    pub source_details: InstanceSourceDetails,
}

impl LaunchInstanceDetails {
    /// Create a new instance launch details with required fields
    pub fn new(required: LaunchInstanceDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,
            availability_domain: required.availability_domain,
            shape: required.shape,
            source_details: required.source_details,
            create_vnic_details: None,
            dedicated_vm_host_id: None,
            defined_tags: None,
            display_name: None,
            metadata: None,
            extended_metadata: None,
            freeform_tags: None,
            shape_config: None,
            fault_domain: None,
            capacity_reservation_id: None,
            is_pv_encryption_in_transit_enabled: None,
        }
    }

    // Setters for all fields (required + optional)
    pub fn set_compartment_id(mut self, compartment_id: String) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    pub fn set_availability_domain(mut self, availability_domain: String) -> Self {
        self.availability_domain = availability_domain;
        self
    }

    pub fn set_shape(mut self, shape: String) -> Self {
        self.shape = shape;
        self
    }

    pub fn set_source_details(mut self, source_details: InstanceSourceDetails) -> Self {
        self.source_details = source_details;
        self
    }

    pub fn set_create_vnic_details(mut self, create_vnic_details: Option<CreateVnicDetails>) -> Self {
        self.create_vnic_details = create_vnic_details;
        self
    }

    pub fn set_dedicated_vm_host_id(mut self, dedicated_vm_host_id: Option<String>) -> Self {
        self.dedicated_vm_host_id = dedicated_vm_host_id;
        self
    }

    pub fn set_defined_tags(mut self, defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.defined_tags = defined_tags;
        self
    }

    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    pub fn set_metadata(mut self, metadata: Option<HashMap<String, String>>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn set_extended_metadata(mut self, extended_metadata: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.extended_metadata = extended_metadata;
        self
    }

    pub fn set_freeform_tags(mut self, freeform_tags: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = freeform_tags;
        self
    }

    pub fn set_shape_config(mut self, shape_config: Option<LaunchInstanceShapeConfigDetails>) -> Self {
        self.shape_config = shape_config;
        self
    }

    pub fn set_fault_domain(mut self, fault_domain: Option<String>) -> Self {
        self.fault_domain = fault_domain;
        self
    }

    pub fn set_capacity_reservation_id(mut self, capacity_reservation_id: Option<String>) -> Self {
        self.capacity_reservation_id = capacity_reservation_id;
        self
    }

    pub fn set_is_pv_encryption_in_transit_enabled(mut self, is_pv_encryption_in_transit_enabled: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = is_pv_encryption_in_transit_enabled;
        self
    }

    // With methods for optional fields only
    pub fn with_create_vnic_details(mut self, create_vnic_details: CreateVnicDetails) -> Self {
        self.create_vnic_details = Some(create_vnic_details);
        self
    }

    pub fn with_dedicated_vm_host_id(mut self, dedicated_vm_host_id: impl Into<String>) -> Self {
        self.dedicated_vm_host_id = Some(dedicated_vm_host_id.into());
        self
    }

    pub fn with_defined_tags(mut self, defined_tags: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
        self.defined_tags = Some(defined_tags);
        self
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_extended_metadata(mut self, extended_metadata: HashMap<String, serde_json::Value>) -> Self {
        self.extended_metadata = Some(extended_metadata);
        self
    }

    pub fn with_freeform_tags(mut self, freeform_tags: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(freeform_tags);
        self
    }

    pub fn with_shape_config(mut self, shape_config: LaunchInstanceShapeConfigDetails) -> Self {
        self.shape_config = Some(shape_config);
        self
    }

    pub fn with_fault_domain(mut self, fault_domain: impl Into<String>) -> Self {
        self.fault_domain = Some(fault_domain.into());
        self
    }

    pub fn with_capacity_reservation_id(mut self, capacity_reservation_id: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(capacity_reservation_id.into());
        self
    }

    pub fn with_is_pv_encryption_in_transit_enabled(mut self, is_pv_encryption_in_transit_enabled: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(is_pv_encryption_in_transit_enabled);
        self
    }
}
