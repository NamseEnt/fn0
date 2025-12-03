use crate::compute::models::*;

/// Request to list compute instances
#[derive(Debug, Clone)]
pub struct ListInstancesRequest {
    /// Required: The OCID of the compartment
    pub compartment_id: String,

    /// The name of the availability domain
    pub availability_domain: Option<String>,

    /// The OCID of the compute capacity reservation
    pub capacity_reservation_id: Option<String>,

    /// The OCID of the compute cluster
    pub compute_cluster_id: Option<String>,

    /// Filter to return only resources that match the given display name exactly
    pub display_name: Option<String>,

    /// For list pagination - maximum number of results per page
    pub limit: Option<u32>,

    /// For list pagination - the value of opc-next-page from previous response
    pub page: Option<String>,

    /// The field to sort by (TIMECREATED or DISPLAYNAME)
    pub sort_by: Option<SortBy>,

    /// The sort order to use (ASC or DESC)
    pub sort_order: Option<SortOrder>,

    /// Filter to only return resources that match the given lifecycle state
    pub lifecycle_state: Option<LifecycleState>,
}

impl ListInstancesRequest {
    /// Create a new builder for ListInstancesRequest
    pub fn builder(compartment_id: impl Into<String>) -> ListInstancesRequestBuilder {
        ListInstancesRequestBuilder {
            request: ListInstancesRequest {
                compartment_id: compartment_id.into(),
                availability_domain: None,
                capacity_reservation_id: None,
                compute_cluster_id: None,
                display_name: None,
                limit: None,
                page: None,
                sort_by: None,
                sort_order: None,
                lifecycle_state: None,
            },
        }
    }

    /// Convert request to query parameters
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        // Required parameters
        params.push(("compartmentId".to_string(), self.compartment_id.clone()));

        // Optional parameters
        if let Some(ref v) = self.availability_domain {
            params.push(("availabilityDomain".to_string(), v.clone()));
        }
        if let Some(ref v) = self.capacity_reservation_id {
            params.push(("capacityReservationId".to_string(), v.clone()));
        }
        if let Some(ref v) = self.compute_cluster_id {
            params.push(("computeClusterId".to_string(), v.clone()));
        }
        if let Some(ref v) = self.display_name {
            params.push(("displayName".to_string(), v.clone()));
        }
        if let Some(v) = self.limit {
            params.push(("limit".to_string(), v.to_string()));
        }
        if let Some(ref v) = self.page {
            params.push(("page".to_string(), v.clone()));
        }
        if let Some(v) = self.sort_by {
            params.push(("sortBy".to_string(), format!("{:?}", v).to_uppercase()));
        }
        if let Some(v) = self.sort_order {
            params.push(("sortOrder".to_string(), format!("{:?}", v).to_uppercase()));
        }
        if let Some(v) = self.lifecycle_state {
            params.push(("lifecycleState".to_string(), v.to_string()));
        }

        params
    }
}

/// Builder for ListInstancesRequest
#[derive(Debug)]
pub struct ListInstancesRequestBuilder {
    request: ListInstancesRequest,
}

impl ListInstancesRequestBuilder {
    pub fn availability_domain(mut self, availability_domain: impl Into<String>) -> Self {
        self.request.availability_domain = Some(availability_domain.into());
        self
    }

    pub fn capacity_reservation_id(mut self, capacity_reservation_id: impl Into<String>) -> Self {
        self.request.capacity_reservation_id = Some(capacity_reservation_id.into());
        self
    }

    pub fn compute_cluster_id(mut self, compute_cluster_id: impl Into<String>) -> Self {
        self.request.compute_cluster_id = Some(compute_cluster_id.into());
        self
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.request.display_name = Some(display_name.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.request.limit = Some(limit);
        self
    }

    pub fn page(mut self, page: impl Into<String>) -> Self {
        self.request.page = Some(page.into());
        self
    }

    pub fn sort_by(mut self, sort_by: SortBy) -> Self {
        self.request.sort_by = Some(sort_by);
        self
    }

    pub fn sort_order(mut self, sort_order: SortOrder) -> Self {
        self.request.sort_order = Some(sort_order);
        self
    }

    pub fn lifecycle_state(mut self, lifecycle_state: LifecycleState) -> Self {
        self.request.lifecycle_state = Some(lifecycle_state);
        self
    }

    pub fn build(self) -> ListInstancesRequest {
        self.request
    }
}

/// Response from listing instances
#[derive(Debug, Clone)]
pub struct ListInstancesResponse {
    /// The list of instances
    pub items: Vec<Instance>,

    /// Unique Oracle-assigned identifier for the request
    pub opc_request_id: Option<String>,

    /// For list pagination - token for next page
    pub opc_next_page: Option<String>,
}
