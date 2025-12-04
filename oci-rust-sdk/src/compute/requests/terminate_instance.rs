/// Required fields for TerminateInstanceRequest
pub struct TerminateInstanceRequestRequiredFields {
    /// The OCID of the instance
    pub instance_id: String,
}

/// Request to terminate a compute instance
#[derive(Debug, Clone)]
pub struct TerminateInstanceRequest {
    /// The OCID of the instance (required)
    pub instance_id: String,

    /// For optimistic concurrency control
    pub if_match: Option<String>,

    /// Whether to preserve the boot volume
    pub preserve_boot_volume: Option<bool>,

    /// Whether to preserve data volumes created at launch
    pub preserve_data_volumes_created_at_launch: Option<bool>,
}

impl TerminateInstanceRequest {
    /// Create a new builder for TerminateInstanceRequest
    pub fn builder(required: TerminateInstanceRequestRequiredFields) -> TerminateInstanceRequestBuilder {
        TerminateInstanceRequestBuilder {
            request: TerminateInstanceRequest {
                instance_id: required.instance_id,
                if_match: None,
                preserve_boot_volume: None,
                preserve_data_volumes_created_at_launch: None,
            },
        }
    }

    /// Create a simple request with just the instance ID
    pub fn new(instance_id: impl Into<String>) -> Self {
        Self {
            instance_id: instance_id.into(),
            if_match: None,
            preserve_boot_volume: None,
            preserve_data_volumes_created_at_launch: None,
        }
    }

    /// Convert request to query parameters
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        // Optional parameters
        if let Some(preserve) = self.preserve_boot_volume {
            params.push(("preserveBootVolume".to_string(), preserve.to_string()));
        }
        if let Some(preserve) = self.preserve_data_volumes_created_at_launch {
            params.push((
                "preserveDataVolumesCreatedAtLaunch".to_string(),
                preserve.to_string(),
            ));
        }

        params
    }
}

/// Builder for TerminateInstanceRequest
#[derive(Debug)]
pub struct TerminateInstanceRequestBuilder {
    request: TerminateInstanceRequest,
}

impl TerminateInstanceRequestBuilder {
    /// Set the if-match header for optimistic concurrency control
    pub fn if_match(mut self, etag: impl Into<String>) -> Self {
        self.request.if_match = Some(etag.into());
        self
    }

    /// Set whether to preserve the boot volume
    pub fn preserve_boot_volume(mut self, preserve: bool) -> Self {
        self.request.preserve_boot_volume = Some(preserve);
        self
    }

    /// Set whether to preserve data volumes created at launch
    pub fn preserve_data_volumes_created_at_launch(mut self, preserve: bool) -> Self {
        self.request.preserve_data_volumes_created_at_launch = Some(preserve);
        self
    }

    /// Build the request
    pub fn build(self) -> TerminateInstanceRequest {
        self.request
    }
}

/// Response from terminating an instance
#[derive(Debug, Clone)]
pub struct TerminateInstanceResponse {
    /// Unique Oracle-assigned identifier for the request
    pub opc_request_id: Option<String>,
}
