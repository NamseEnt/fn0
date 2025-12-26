//! Core Services Module
//!
//! Contains models for Oracle Cloud Infrastructure Core Services:
//! - Compute (instances, images, shapes)
//! - VirtualNetwork (VCNs, subnets, security lists)
//! - Blockstorage (volumes, backups)

pub mod client;
pub mod error;
pub mod models;
pub mod region;
pub mod requests;
pub mod responses;
pub mod retry;

// Re-exports for convenience
pub use error::{OciError, Result};
pub use models::*;
pub use region::Region;
pub use requests::*;
pub use responses::*;
pub use retry::{Retrier, RetryConfiguration};

use crate::auth::provider::AuthProvider;
use client::http_client::OciClient;
use std::sync::Arc;

/// Client configuration for core services
pub struct ClientConfig {
    pub auth_provider: Arc<dyn AuthProvider>,
    pub region: Region,
    pub timeout: std::time::Duration,
    pub retry: Retrier,
}

/// Create a new core services client
pub fn client(config: ClientConfig) -> Result<CoreClient> {
    let endpoint = format!("https://iaas.{}.oraclecloud.com", config.region.id());
    let client = OciClient::new(config.auth_provider, endpoint)?.with_retrier(config.retry);

    Ok(CoreClient { client })
}

/// Core Services API client
pub struct CoreClient {
    client: OciClient,
}

impl CoreClient {
    /// List compute instances in a compartment
    pub async fn list_instances(
        &self,
        request: ListInstancesRequest,
    ) -> Result<ListInstancesResponse> {
        let mut query_params = vec![("compartmentId".to_string(), request.compartment_id.clone())];

        if let Some(av_domain) = &request.availability_domain {
            query_params.push(("availabilityDomain".to_string(), av_domain.clone()));
        }
        if let Some(display_name) = &request.display_name {
            query_params.push(("displayName".to_string(), display_name.clone()));
        }
        if let Some(limit) = request.limit {
            query_params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(page) = &request.page {
            query_params.push(("page".to_string(), page.clone()));
        }
        if let Some(lifecycle_state) = &request.lifecycle_state {
            query_params.push(("lifecycleState".to_string(), lifecycle_state.clone()));
        }

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

        let path = format!("/20160918/instances{}", query_string);
        let response = self.client.get(&path).await?;

        Ok(ListInstancesResponse {
            opc_next_page: response.get_header("opc-next-page"),
            opc_request_id: response.get_header("opc-request-id"),
            items: response.body,
        })
    }

    /// Launch a new compute instance
    pub async fn launch_instance(
        &self,
        request: LaunchInstanceRequest,
    ) -> Result<LaunchInstanceResponse> {
        let path = "/20160918/instances".to_string();
        let response = self
            .client
            .post(&path, Some(&request.launch_instance_details))
            .await?;

        Ok(LaunchInstanceResponse {
            etag: response.get_header("etag"),
            opc_request_id: response.get_header("opc-request-id"),
            opc_work_request_id: response.get_header("opc-work-request-id"),
            instance: response.body,
        })
    }

    /// Get details of a specific compute instance
    pub async fn get_instance(&self, request: GetInstanceRequest) -> Result<GetInstanceResponse> {
        let path = format!("/20160918/instances/{}", request.instance_id);
        let response = self.client.get(&path).await?;

        Ok(GetInstanceResponse {
            etag: response.get_header("etag"),
            opc_request_id: response.get_header("opc-request-id"),
            instance: response.body,
        })
    }

    /// List public IPs in a scope
    pub async fn list_public_ips(
        &self,
        request: ListPublicIpsRequest,
    ) -> Result<ListPublicIpsResponse> {
        let mut query_params = vec![("scope".to_string(), format!("{:?}", request.scope))];
        query_params.push(("compartmentId".to_string(), request.compartment_id.clone()));

        if let Some(limit) = request.limit {
            query_params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(page) = &request.page {
            query_params.push(("page".to_string(), page.clone()));
        }
        if let Some(availability_domain) = &request.availability_domain {
            query_params.push((
                "availabilityDomain".to_string(),
                availability_domain.clone(),
            ));
        }
        if let Some(lifetime) = &request.lifetime {
            query_params.push(("lifetime".to_string(), format!("{:?}", lifetime)));
        }

        let query_string = format!(
            "?{}",
            query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&")
        );

        let path = format!("/20160918/publicIps{}", query_string);
        let response = self.client.get(&path).await?;

        Ok(ListPublicIpsResponse {
            opc_next_page: response.get_header("opc-next-page"),
            opc_request_id: response.get_header("opc-request-id"),
            items: response.body,
        })
    }
}
