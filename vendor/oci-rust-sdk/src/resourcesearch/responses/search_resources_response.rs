use crate::resourcesearch::models::ResourceSummaryCollection;

/// Response from a search resources request.
#[derive(Debug, Clone)]
pub struct SearchResourcesResponse {
    /// The collection of resource summaries matching the search criteria.
    pub resource_summary_collection: ResourceSummaryCollection,

    /// Unique Oracle-assigned identifier for the request.
    pub opc_request_id: Option<String>,

    /// For pagination of a list of items. When paging through a list, if this header appears
    /// in the response, then a partial list might have been returned. Include this value as
    /// the page parameter for the subsequent GET request to get the next batch of items.
    pub opc_next_page: Option<String>,

    /// For pagination of a list of items. When paging through a list, if this header appears
    /// in the response, then a partial list might have been returned. Include this value as
    /// the page parameter for the subsequent GET request to get the previous batch of items.
    pub opc_previous_page: Option<String>,
}
