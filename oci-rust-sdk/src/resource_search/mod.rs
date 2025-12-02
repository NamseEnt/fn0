pub mod models;
pub mod requests;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::Result;
pub use models::*;
pub use requests::*;

/// Trait for Resource Search service operations.
///
/// The Resource Search service allows you to search for resources across your tenancy using
/// either structured queries or free text search.
///
/// # Example
///
/// ```no_run
/// use oci_rust_sdk::{
///     core::{auth::ConfigFileAuthProvider, region::Region},
///     resource_search::{
///         self, SearchResourcesRequest, SearchDetails,
///         StructuredSearchDetails, MatchingContextType,
///     },
/// };
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = Arc::new(ConfigFileAuthProvider::from_default()?);
/// let client = resource_search::client(auth, Region::ApSeoul1)?;
///
/// let search_details = SearchDetails::Structured(StructuredSearchDetails {
///     query: "query instance resources".to_string(),
///     matching_context_type: Some(MatchingContextType::Highlights),
/// });
///
/// let request = SearchResourcesRequest::builder(search_details)
///     .limit(100)
///     .build();
///
/// let response = client.search_resources(request).await?;
///
/// for resource in &response.resource_summary_collection.items {
///     println!("{}: {}", resource.resource_type, resource.identifier);
/// }
/// # Ok(())
/// # }
/// ```
pub trait ResourceSearch: Send + Sync {
    /// Search for resources in your cloud network.
    ///
    /// # Arguments
    ///
    /// * `request` - The search request containing search criteria and options
    ///
    /// # Returns
    ///
    /// Returns a `SearchResourcesResponse` containing the collection of resources
    /// that match the search criteria.
    fn search_resources(
        &self,
        request: SearchResourcesRequest,
    ) -> Pin<Box<dyn Future<Output = Result<SearchResourcesResponse>> + Send + '_>>;
}

/// Create a new Resource Search client for the specified region.
///
/// Returns an `Arc<dyn ResourceSearch>` that can be used to search for resources
/// across your OCI tenancy. The Resource Search service uses a special endpoint
/// pattern (`query.{region}.oci.oraclecloud.com`).
///
/// # Arguments
///
/// * `auth_provider` - Authentication provider for signing requests
/// * `region` - OCI region where the service will be accessed
///
/// # Example
///
/// ```no_run
/// use oci_rust_sdk::resource_search;
/// use oci_rust_sdk::core::{auth::ConfigFileAuthProvider, region::Region};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = Arc::new(ConfigFileAuthProvider::from_default()?);
/// let client = resource_search::client(auth, Region::ApSeoul1)?;
///
/// let search_details = resource_search::SearchDetails::Structured(
///     resource_search::StructuredSearchDetails {
///         query: "query instance resources".to_string(),
///         matching_context_type: Some(resource_search::MatchingContextType::Highlights),
///     }
/// );
/// let request = resource_search::SearchResourcesRequest::builder(search_details).build();
/// let response = client.search_resources(request).await?;
/// # Ok(())
/// # }
/// ```
pub fn client(
    auth_provider: Arc<dyn crate::core::auth::AuthProvider>,
    region: crate::core::region::Region,
) -> Result<Arc<dyn ResourceSearch>> {
    let endpoint = region.query_endpoint();
    let oci_client = crate::core::OciClient::new(auth_provider, endpoint)?;
    Ok(Arc::new(oci_client))
}

impl ResourceSearch for crate::core::OciClient {
    fn search_resources(
        &self,
        request: SearchResourcesRequest,
    ) -> Pin<Box<dyn Future<Output = Result<SearchResourcesResponse>> + Send + '_>> {
        Box::pin(async move {
            // Build query string from query parameters
            let query_params = request.to_query_params();
            let query_string = if query_params.is_empty() {
                String::new()
            } else {
                format!(
                    "?{}",
                    query_params
                        .iter()
                        .map(|(k, v)| format!(
                            "{}={}",
                            urlencoding::encode(k),
                            urlencoding::encode(v)
                        ))
                        .collect::<Vec<_>>()
                        .join("&")
                )
            };

            // Build the full path including API version
            let path = format!("/20180409/resources{}", query_string);

            // Make POST request with search_details as body
            let oci_response = self
                .post::<SearchDetails, ResourceSummaryCollection>(&path, Some(&request.search_details))
                .await?;

            // Extract response headers
            let opc_request_id = oci_response.get_header("opc-request-id");
            let opc_next_page = oci_response.get_header("opc-next-page");
            let opc_previous_page = oci_response.get_header("opc-previous-page");

            Ok(SearchResourcesResponse {
                resource_summary_collection: oci_response.body,
                opc_request_id,
                opc_next_page,
                opc_previous_page,
            })
        })
    }
}
