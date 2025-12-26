//! containerinstances service module
pub mod models;
pub mod requests;

// Re-export commonly used types
pub use models::*;
pub use requests::*;

use crate::auth::provider::AuthProvider;
use crate::core::{client::http_client::{OciClient, OciResponse}, region::Region, retry::Retrier, Result};
use std::sync::Arc;

/// Client configuration for container instances service
pub struct ClientConfig {
    pub auth_provider: Arc<dyn AuthProvider>,
    pub region: Region,
    pub timeout: std::time::Duration,
    pub retry: Retrier,
}

/// Create a new container instances client
pub fn client(config: ClientConfig) -> Result<ContainerInstancesClient> {
    let endpoint = format!("https://containerinstances.{}.oci.oraclecloud.com", config.region.id());
    let client = OciClient::new(config.auth_provider, endpoint)?
        .with_retrier(config.retry);

    Ok(ContainerInstancesClient { client })
}

/// Container Instances API client
pub struct ContainerInstancesClient {
    client: OciClient,
}

impl ContainerInstancesClient {
    /// Create a container instance
    pub async fn create_container_instance(
        &self,
        request: CreateContainerInstanceRequest,
    ) -> Result<CreateContainerInstanceResponse> {
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

        let path = format!("/20210410/containerInstances{}", query_string);
        let response = self
            .client
            .post(&path, Some(&request.create_container_instance_details))
            .await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(CreateContainerInstanceResponse {
            container_instance: response.body,
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// List container instances
    pub async fn list_container_instances(
        &self,
        request: ListContainerInstancesRequest,
    ) -> Result<ListContainerInstancesResponse> {
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

        let path = format!("/20210410/containerInstances{}", query_string);
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(ListContainerInstancesResponse {
            items: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// Get a container instance
    pub async fn get_container_instance(
        &self,
        request: GetContainerInstanceRequest,
    ) -> Result<GetContainerInstanceResponse> {
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

        let path = format!("/20210410/containerInstances/{}{}", request.container_instance_id, query_string);
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");

        Ok(GetContainerInstanceResponse {
            container_instance: response.body,
            opc_request_id,
        })
    }

    /// Delete a container instance
    pub async fn delete_container_instance(
        &self,
        request: DeleteContainerInstanceRequest,
    ) -> Result<DeleteContainerInstanceResponse> {
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

        let path = format!("/20210410/containerInstances/{}{}", request.container_instance_id, query_string);
        let response: OciResponse<serde_json::Value> = self.client.delete(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(DeleteContainerInstanceResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }
}

/// Response types
pub struct CreateContainerInstanceResponse {
    pub container_instance: ContainerInstance,
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct ListContainerInstancesResponse {
    pub items: Vec<ContainerInstanceSummary>,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct GetContainerInstanceResponse {
    pub container_instance: ContainerInstance,
    pub opc_request_id: Option<String>,
}

pub struct DeleteContainerInstanceResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}
