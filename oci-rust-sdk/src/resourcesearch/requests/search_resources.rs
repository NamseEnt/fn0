use crate::resourcesearch::models::SearchDetails;

/// Required fields for SearchResourcesRequest
pub struct SearchResourcesRequestRequired {
    /// The search criteria (structured query or free text)
    pub search_details: SearchDetails,
}

/// Request to search for resources.
#[derive(Debug, Clone)]
pub struct SearchResourcesRequest {
    /// The search criteria (structured query or free text).
    pub search_details: SearchDetails,

    /// The maximum number of items to return. The value must be between 1 and 1000.
    pub limit: Option<u32>,

    /// The page token representing the page at which to start retrieving results.
    pub page: Option<String>,

    /// The tenancy ID for cross-tenancy searches. If not specified, searches within
    /// the tenancy of the authenticated user.
    pub tenant_id: Option<String>,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about
    /// a particular request, please provide the request ID.
    pub opc_request_id: Option<String>,
}

impl SearchResourcesRequest {
    /// Create a new request with required fields
    pub fn new(required: SearchResourcesRequestRequired) -> Self {
        Self {
            search_details: required.search_details,
            limit: None,
            page: None,
            tenant_id: None,
            opc_request_id: None,
        }
    }

    /// Set the search details
    pub fn set_search_details(mut self, search_details: SearchDetails) -> Self {
        self.search_details = search_details;
        self
    }

    /// Set the maximum number of items to return
    pub fn set_limit(mut self, limit: Option<u32>) -> Self {
        self.limit = limit;
        self
    }

    /// Set the page token for pagination
    pub fn set_page(mut self, page: Option<String>) -> Self {
        self.page = page;
        self
    }

    /// Set the tenant ID
    pub fn set_tenant_id(mut self, tenant_id: Option<String>) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    /// Set the OPC request ID
    pub fn set_opc_request_id(mut self, opc_request_id: Option<String>) -> Self {
        self.opc_request_id = opc_request_id;
        self
    }

    /// Set the maximum number of items to return (1-1000)
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the page token for pagination
    pub fn with_page(mut self, page: impl Into<String>) -> Self {
        self.page = Some(page.into());
        self
    }

    /// Set the tenant ID for cross-tenancy searches
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Set the OPC request ID
    pub fn with_opc_request_id(mut self, id: impl Into<String>) -> Self {
        self.opc_request_id = Some(id.into());
        self
    }

    /// Convert this request's query parameters to a vector of key-value pairs.
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }

        if let Some(ref page) = self.page {
            params.push(("page".to_string(), page.clone()));
        }

        if let Some(ref tenant_id) = self.tenant_id {
            params.push(("tenantId".to_string(), tenant_id.clone()));
        }

        params
    }
}
