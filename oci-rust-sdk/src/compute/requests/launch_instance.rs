use crate::compute::models::*;

/// Request to launch a new compute instance
#[derive(Debug, Clone)]
pub struct LaunchInstanceRequest {
    /// Details for launching the instance
    pub launch_instance_details: LaunchInstanceDetails,

    /// A token that uniquely identifies a request for idempotency
    pub opc_retry_token: Option<String>,
}

impl LaunchInstanceRequest {
    /// Create a new builder for LaunchInstanceRequest
    pub fn builder() -> LaunchInstanceRequestBuilder {
        LaunchInstanceRequestBuilder::default()
    }

    /// Create a request from launch instance details
    pub fn new(launch_instance_details: LaunchInstanceDetails) -> Self {
        Self {
            launch_instance_details,
            opc_retry_token: None,
        }
    }
}

/// Builder for LaunchInstanceRequest
#[derive(Default)]
pub struct LaunchInstanceRequestBuilder {
    launch_instance_details: Option<LaunchInstanceDetails>,
    opc_retry_token: Option<String>,
}

impl LaunchInstanceRequestBuilder {
    /// Set the launch instance details (required)
    pub fn launch_instance_details(mut self, details: LaunchInstanceDetails) -> Self {
        self.launch_instance_details = Some(details);
        self
    }

    /// Set the compartment ID (convenience method)
    pub fn compartment_id(self, id: impl Into<String>) -> LaunchInstanceDetailsBuilderWrapper {
        LaunchInstanceDetailsBuilderWrapper {
            details_builder: LaunchInstanceDetails::builder().compartment_id(id),
            opc_retry_token: self.opc_retry_token,
        }
    }

    /// Set the retry token for idempotency
    pub fn opc_retry_token(mut self, token: impl Into<String>) -> Self {
        self.opc_retry_token = Some(token.into());
        self
    }

    /// Build the request
    pub fn build(self) -> LaunchInstanceRequest {
        LaunchInstanceRequest {
            launch_instance_details: self
                .launch_instance_details
                .expect("launch_instance_details is required"),
            opc_retry_token: self.opc_retry_token,
        }
    }
}

/// Wrapper to allow chaining LaunchInstanceDetails builder methods
pub struct LaunchInstanceDetailsBuilderWrapper {
    details_builder: LaunchInstanceDetailsBuilder,
    opc_retry_token: Option<String>,
}

impl LaunchInstanceDetailsBuilderWrapper {
    /// Set the availability domain (required)
    pub fn availability_domain(mut self, ad: impl Into<String>) -> Self {
        self.details_builder = self.details_builder.availability_domain(ad);
        self
    }

    /// Set the shape (required)
    pub fn shape(mut self, shape: impl Into<String>) -> Self {
        self.details_builder = self.details_builder.shape(shape);
        self
    }

    /// Set the source details (required)
    pub fn source_details(mut self, details: InstanceSourceDetails) -> Self {
        self.details_builder = self.details_builder.source_details(details);
        self
    }

    /// Set the VNIC details
    pub fn create_vnic_details(mut self, details: CreateVnicDetails) -> Self {
        self.details_builder = self.details_builder.create_vnic_details(details);
        self
    }

    /// Set the display name
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.details_builder = self.details_builder.display_name(name);
        self
    }

    /// Set custom metadata
    pub fn metadata(mut self, metadata: std::collections::HashMap<String, String>) -> Self {
        self.details_builder = self.details_builder.metadata(metadata);
        self
    }

    /// Set free-form tags
    pub fn freeform_tags(mut self, tags: std::collections::HashMap<String, String>) -> Self {
        self.details_builder = self.details_builder.freeform_tags(tags);
        self
    }

    /// Set the retry token for idempotency
    pub fn opc_retry_token(mut self, token: impl Into<String>) -> Self {
        self.opc_retry_token = Some(token.into());
        self
    }

    /// Build the request
    pub fn build(self) -> LaunchInstanceRequest {
        LaunchInstanceRequest {
            launch_instance_details: self.details_builder.build(),
            opc_retry_token: self.opc_retry_token,
        }
    }
}

/// Response from launching an instance
#[derive(Debug, Clone)]
pub struct LaunchInstanceResponse {
    /// The launched instance
    pub instance: Instance,

    /// Unique Oracle-assigned identifier for the request
    pub opc_request_id: Option<String>,

    /// Entity tag for the resource
    pub etag: Option<String>,
}
