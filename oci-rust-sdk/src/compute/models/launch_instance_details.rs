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
pub struct LaunchInstanceRequiredFields {
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
    /// Create a new instance launch details builder with required fields
    pub fn builder(required: LaunchInstanceRequiredFields) -> LaunchInstanceDetailsBuilder {
        LaunchInstanceDetailsBuilder {
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
}

/// Builder for LaunchInstanceDetails
pub struct LaunchInstanceDetailsBuilder {
    compartment_id: String,
    availability_domain: String,
    create_vnic_details: Option<CreateVnicDetails>,
    dedicated_vm_host_id: Option<String>,
    defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    display_name: Option<String>,
    metadata: Option<HashMap<String, String>>,
    extended_metadata: Option<HashMap<String, serde_json::Value>>,
    freeform_tags: Option<HashMap<String, String>>,
    shape: String,
    shape_config: Option<LaunchInstanceShapeConfigDetails>,
    source_details: InstanceSourceDetails,
    fault_domain: Option<String>,
    capacity_reservation_id: Option<String>,
    is_pv_encryption_in_transit_enabled: Option<bool>,
}

impl LaunchInstanceDetailsBuilder {
    /// Set the VNIC details
    pub fn create_vnic_details(mut self, details: CreateVnicDetails) -> Self {
        self.create_vnic_details = Some(details);
        self
    }

    /// Set the display name
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set custom metadata
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set extended metadata
    pub fn extended_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.extended_metadata = Some(metadata);
        self
    }

    /// Set free-form tags
    pub fn freeform_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(tags);
        self
    }

    /// Set defined tags
    pub fn defined_tags(
        mut self,
        tags: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(tags);
        self
    }

    /// Set the dedicated VM host ID
    pub fn dedicated_vm_host_id(mut self, id: impl Into<String>) -> Self {
        self.dedicated_vm_host_id = Some(id.into());
        self
    }

    /// Set the fault domain
    pub fn fault_domain(mut self, domain: impl Into<String>) -> Self {
        self.fault_domain = Some(domain.into());
        self
    }

    /// Set the capacity reservation ID
    pub fn capacity_reservation_id(mut self, id: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(id.into());
        self
    }

    /// Set whether to enable PV encryption in transit
    pub fn is_pv_encryption_in_transit_enabled(mut self, enabled: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(enabled);
        self
    }

    /// Set the shape configuration
    pub fn shape_config(mut self, config: LaunchInstanceShapeConfigDetails) -> Self {
        self.shape_config = Some(config);
        self
    }

    /// Build the LaunchInstanceDetails
    pub fn build(self) -> LaunchInstanceDetails {
        LaunchInstanceDetails {
            compartment_id: self.compartment_id,
            availability_domain: self.availability_domain,
            shape: self.shape,
            shape_config: self.shape_config,
            source_details: self.source_details,
            create_vnic_details: self.create_vnic_details,
            dedicated_vm_host_id: self.dedicated_vm_host_id,
            defined_tags: self.defined_tags,
            display_name: self.display_name,
            metadata: self.metadata,
            extended_metadata: self.extended_metadata,
            freeform_tags: self.freeform_tags,
            fault_domain: self.fault_domain,
            capacity_reservation_id: self.capacity_reservation_id,
            is_pv_encryption_in_transit_enabled: self.is_pv_encryption_in_transit_enabled,
        }
    }
}
