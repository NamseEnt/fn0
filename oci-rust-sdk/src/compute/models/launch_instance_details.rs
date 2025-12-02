use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{CreateVnicDetails, InstanceSourceDetails};

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

impl LaunchInstanceDetails {
    /// Create a new instance launch details builder
    pub fn builder() -> LaunchInstanceDetailsBuilder {
        LaunchInstanceDetailsBuilder::default()
    }
}

/// Builder for LaunchInstanceDetails
#[derive(Default)]
pub struct LaunchInstanceDetailsBuilder {
    compartment_id: Option<String>,
    availability_domain: Option<String>,
    create_vnic_details: Option<CreateVnicDetails>,
    dedicated_vm_host_id: Option<String>,
    defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    display_name: Option<String>,
    metadata: Option<HashMap<String, String>>,
    extended_metadata: Option<HashMap<String, serde_json::Value>>,
    freeform_tags: Option<HashMap<String, String>>,
    shape: Option<String>,
    source_details: Option<InstanceSourceDetails>,
    fault_domain: Option<String>,
    capacity_reservation_id: Option<String>,
    is_pv_encryption_in_transit_enabled: Option<bool>,
}

impl LaunchInstanceDetailsBuilder {
    /// Set the compartment ID (required)
    pub fn compartment_id(mut self, id: impl Into<String>) -> Self {
        self.compartment_id = Some(id.into());
        self
    }

    /// Set the availability domain (required)
    pub fn availability_domain(mut self, ad: impl Into<String>) -> Self {
        self.availability_domain = Some(ad.into());
        self
    }

    /// Set the shape (required)
    pub fn shape(mut self, shape: impl Into<String>) -> Self {
        self.shape = Some(shape.into());
        self
    }

    /// Set the source details (required)
    pub fn source_details(mut self, details: InstanceSourceDetails) -> Self {
        self.source_details = Some(details);
        self
    }

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
    pub fn defined_tags(mut self, tags: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
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

    /// Build the LaunchInstanceDetails
    pub fn build(self) -> LaunchInstanceDetails {
        LaunchInstanceDetails {
            compartment_id: self.compartment_id.expect("compartment_id is required"),
            availability_domain: self.availability_domain.expect("availability_domain is required"),
            shape: self.shape.expect("shape is required"),
            source_details: self.source_details.expect("source_details is required"),
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
