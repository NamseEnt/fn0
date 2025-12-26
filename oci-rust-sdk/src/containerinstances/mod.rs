//! containerinstances service module
pub mod models;
pub mod requests;

// Re-export commonly used types
pub use models::*;
pub use requests::*;

use crate::auth::provider::AuthProvider;
use crate::core::{
    Result,
    client::http_client::{OciClient, OciResponse},
    region::Region,
    retry::Retrier,
};
use serde_json;
use std::sync::Arc;

/// Client configuration for containerinstances service
pub struct ClientConfig {
    pub auth_provider: Arc<dyn AuthProvider>,
    pub region: Region,
    pub timeout: std::time::Duration,
    pub retry: Retrier,
}

/// Create a new containerinstances client
pub fn client(config: ClientConfig) -> Result<ContainerinstancesClient> {
    let endpoint = format!(
        "https://containerinstances.{}.oci.oraclecloud.com",
        config.region.id()
    );
    let client = OciClient::new(config.auth_provider, endpoint)?.with_retrier(config.retry);

    Ok(ContainerinstancesClient { client })
}

/// Containerinstances API client
pub struct ContainerinstancesClient {
    client: OciClient,
}

impl ContainerinstancesClient {
    /// changeContainerInstanceCompartment
    pub async fn change_container_instance_compartment(
        &self,
        request: ChangeContainerInstanceCompartmentRequest,
    ) -> Result<ChangeContainerInstanceCompartmentResponse> {
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

        let path = format!(
            "/20210410/containerInstances/{}/actions/changeCompartment{}",
            request.container_instance_id, query_string
        );
        let response: OciResponse<serde_json::Value> = self
            .client
            .post(
                &path,
                Some(&request.change_container_instance_compartment_details),
            )
            .await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(ChangeContainerInstanceCompartmentResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// createContainerInstance
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

    /// deleteContainerInstance
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

        let path = format!(
            "/20210410/containerInstances/{}{}",
            request.container_instance_id, query_string
        );
        let response: OciResponse<serde_json::Value> = self.client.delete(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(DeleteContainerInstanceResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// getContainer
    pub async fn get_container(
        &self,
        request: GetContainerRequest,
    ) -> Result<GetContainerResponse> {
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

        let path = format!(
            "/20210410/containers/{}{}",
            request.container_id, query_string
        );
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(GetContainerResponse {
            container: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// getContainerInstance
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

        let path = format!(
            "/20210410/containerInstances/{}{}",
            request.container_instance_id, query_string
        );
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(GetContainerInstanceResponse {
            container_instance: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// getWorkRequest
    pub async fn get_work_request(
        &self,
        request: GetWorkRequestRequest,
    ) -> Result<GetWorkRequestResponse> {
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

        let path = format!(
            "/20210410/workRequests/{}{}",
            request.work_request_id, query_string
        );
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(GetWorkRequestResponse {
            work_request: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// listContainerInstanceShapes
    pub async fn list_container_instance_shapes(
        &self,
        request: ListContainerInstanceShapesRequest,
    ) -> Result<ListContainerInstanceShapesResponse> {
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

        let path = format!("/20210410/containerInstanceShapes{}", query_string);
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(ListContainerInstanceShapesResponse {
            items: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// listContainerInstances
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

    /// listContainers
    pub async fn list_containers(
        &self,
        request: ListContainersRequest,
    ) -> Result<ListContainersResponse> {
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

        let path = format!("/20210410/containers{}", query_string);
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(ListContainersResponse {
            items: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// listWorkRequestErrors
    pub async fn list_work_request_errors(
        &self,
        request: ListWorkRequestErrorsRequest,
    ) -> Result<ListWorkRequestErrorsResponse> {
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

        let path = format!(
            "/20210410/workRequests/{}/errors{}",
            request.work_request_id, query_string
        );
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(ListWorkRequestErrorsResponse {
            items: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// listWorkRequestLogs
    pub async fn list_work_request_logs(
        &self,
        request: ListWorkRequestLogsRequest,
    ) -> Result<ListWorkRequestLogsResponse> {
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

        let path = format!(
            "/20210410/workRequests/{}/logs{}",
            request.work_request_id, query_string
        );
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(ListWorkRequestLogsResponse {
            items: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// listWorkRequests
    pub async fn list_work_requests(
        &self,
        request: ListWorkRequestsRequest,
    ) -> Result<ListWorkRequestsResponse> {
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

        let path = format!("/20210410/workRequests{}", query_string);
        let response = self.client.get(&path).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_next_page = response.get_header("opc-next-page");

        Ok(ListWorkRequestsResponse {
            items: response.body,
            opc_request_id,
            opc_next_page,
        })
    }

    /// restartContainerInstance
    pub async fn restart_container_instance(
        &self,
        request: RestartContainerInstanceRequest,
    ) -> Result<RestartContainerInstanceResponse> {
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

        let path = format!(
            "/20210410/containerInstances/{}/actions/restart{}",
            request.container_instance_id, query_string
        );
        let response: OciResponse<serde_json::Value> =
            self.client.post(&path, None::<&serde_json::Value>).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(RestartContainerInstanceResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// retrieveLogs
    pub async fn retrieve_logs(
        &self,
        request: RetrieveLogsRequest,
    ) -> Result<RetrieveLogsResponse> {
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

        let path = format!(
            "/20210410/containers/{}/actions/retrieveLogs{}",
            request.container_id, query_string
        );
        let response: OciResponse<serde_json::Value> =
            self.client.post(&path, None::<&serde_json::Value>).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(RetrieveLogsResponse {
            value: response.body,
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// startContainerInstance
    pub async fn start_container_instance(
        &self,
        request: StartContainerInstanceRequest,
    ) -> Result<StartContainerInstanceResponse> {
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

        let path = format!(
            "/20210410/containerInstances/{}/actions/start{}",
            request.container_instance_id, query_string
        );
        let response: OciResponse<serde_json::Value> =
            self.client.post(&path, None::<&serde_json::Value>).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(StartContainerInstanceResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// stopContainerInstance
    pub async fn stop_container_instance(
        &self,
        request: StopContainerInstanceRequest,
    ) -> Result<StopContainerInstanceResponse> {
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

        let path = format!(
            "/20210410/containerInstances/{}/actions/stop{}",
            request.container_instance_id, query_string
        );
        let response: OciResponse<serde_json::Value> =
            self.client.post(&path, None::<&serde_json::Value>).await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(StopContainerInstanceResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// updateContainer
    pub async fn update_container(
        &self,
        request: UpdateContainerRequest,
    ) -> Result<UpdateContainerResponse> {
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

        let path = format!(
            "/20210410/containers/{}{}",
            request.container_id, query_string
        );
        let response: OciResponse<serde_json::Value> = self
            .client
            .put(&path, Some(&request.update_container_details))
            .await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(UpdateContainerResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }

    /// updateContainerInstance
    pub async fn update_container_instance(
        &self,
        request: UpdateContainerInstanceRequest,
    ) -> Result<UpdateContainerInstanceResponse> {
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

        let path = format!(
            "/20210410/containerInstances/{}{}",
            request.container_instance_id, query_string
        );
        let response: OciResponse<serde_json::Value> = self
            .client
            .put(&path, Some(&request.update_container_instance_details))
            .await?;

        let opc_request_id = response.get_header("opc-request-id");
        let opc_work_request_id = response.get_header("opc-work-request-id");

        Ok(UpdateContainerInstanceResponse {
            opc_request_id,
            opc_work_request_id,
        })
    }
}

/// Response types
pub struct ChangeContainerInstanceCompartmentResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct CreateContainerInstanceResponse {
    pub container_instance: ContainerInstance,
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct DeleteContainerInstanceResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct GetContainerResponse {
    pub container: Container,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct GetContainerInstanceResponse {
    pub container_instance: ContainerInstance,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct GetWorkRequestResponse {
    pub work_request: WorkRequest,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct ListContainerInstanceShapesResponse {
    pub items: Vec<ContainerInstanceShapeSummary>,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct ListContainerInstancesResponse {
    pub items: Vec<ContainerInstanceSummary>,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct ListContainersResponse {
    pub items: Vec<ContainerSummary>,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct ListWorkRequestErrorsResponse {
    pub items: Vec<WorkRequestError>,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct ListWorkRequestLogsResponse {
    pub items: Vec<WorkRequestLogEntry>,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct ListWorkRequestsResponse {
    pub items: Vec<WorkRequestSummary>,
    pub opc_request_id: Option<String>,
    pub opc_next_page: Option<String>,
}

pub struct RestartContainerInstanceResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct RetrieveLogsResponse {
    pub value: serde_json::Value,
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct StartContainerInstanceResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct StopContainerInstanceResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct UpdateContainerResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}

pub struct UpdateContainerInstanceResponse {
    pub opc_request_id: Option<String>,
    pub opc_work_request_id: Option<String>,
}
