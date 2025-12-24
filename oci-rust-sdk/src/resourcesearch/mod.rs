//! resourcesearch service module
pub mod models;
pub mod requests;

// Re-export commonly used types
pub use models::*;
pub use requests::*;

use crate::auth::provider::AuthProvider;
use crate::core::{client::http_client::OciClient, region::Region, retry::Retrier, Result};
use std::sync::Arc;

/// Client configuration for resource search service
pub struct ClientConfig {
    pub auth_provider: Arc<dyn AuthProvider>,
    pub region: Region,
    pub timeout: std::time::Duration,
    pub retry: Retrier,
}

/// Create a new resource search client
pub fn client(config: ClientConfig) -> Result<ResourceSearchClient> {
    let endpoint = format!("https://query.{}.oci.oraclecloud.com", config.region.id());
    let client = OciClient::new(config.auth_provider, endpoint)?
        .with_retrier(config.retry);

    Ok(ResourceSearchClient { client })
}

/// Resource Search API client
pub struct ResourceSearchClient {
    client: OciClient,
}

impl ResourceSearchClient {
    /// Search for resources
    pub async fn search_resources(
        &self,
        request: SearchResourcesRequest,
    ) -> Result<SearchResourcesResponse> {
        let query_params = request.to_query_params();
        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!(
                "?{}",
                query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&")
            )
        };

        let path = format!("/20180409/resources{}", query_string);
        let response = self
            .client
            .post(&path, Some(&request.search_details))
            .await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");
        let opc_previous_page = response.get_header("opc-previous-page");

        Ok(SearchResourcesResponse {
            resource_summary_collection: response.body,
            opc_request_id,
            opc_next_page,
            opc_previous_page,
        })
    }
}
