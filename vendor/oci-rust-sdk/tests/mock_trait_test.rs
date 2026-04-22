use async_trait::async_trait;
use chrono::Utc;
use mockall::mock;
use oci_rust_sdk::containerinstances::*;
use oci_rust_sdk::core::Result;

mock! {
    pub ContainerinstancesClient {}

    #[async_trait]
    impl ContainerinstancesApi for ContainerinstancesClient {
        async fn change_container_instance_compartment(
            &self,
            request: ChangeContainerInstanceCompartmentRequest,
        ) -> Result<ChangeContainerInstanceCompartmentResponse>;

        async fn create_container_instance(
            &self,
            request: CreateContainerInstanceRequest,
        ) -> Result<CreateContainerInstanceResponse>;

        async fn delete_container_instance(
            &self,
            request: DeleteContainerInstanceRequest,
        ) -> Result<DeleteContainerInstanceResponse>;

        async fn get_container(
            &self,
            request: GetContainerRequest,
        ) -> Result<GetContainerResponse>;

        async fn get_container_instance(
            &self,
            request: GetContainerInstanceRequest,
        ) -> Result<GetContainerInstanceResponse>;

        async fn get_work_request(
            &self,
            request: GetWorkRequestRequest,
        ) -> Result<GetWorkRequestResponse>;

        async fn list_container_instance_shapes(
            &self,
            request: ListContainerInstanceShapesRequest,
        ) -> Result<ListContainerInstanceShapesResponse>;

        async fn list_container_instances(
            &self,
            request: ListContainerInstancesRequest,
        ) -> Result<ListContainerInstancesResponse>;

        async fn list_containers(
            &self,
            request: ListContainersRequest,
        ) -> Result<ListContainersResponse>;

        async fn list_work_request_errors(
            &self,
            request: ListWorkRequestErrorsRequest,
        ) -> Result<ListWorkRequestErrorsResponse>;

        async fn list_work_request_logs(
            &self,
            request: ListWorkRequestLogsRequest,
        ) -> Result<ListWorkRequestLogsResponse>;

        async fn list_work_requests(
            &self,
            request: ListWorkRequestsRequest,
        ) -> Result<ListWorkRequestsResponse>;

        async fn restart_container_instance(
            &self,
            request: RestartContainerInstanceRequest,
        ) -> Result<RestartContainerInstanceResponse>;

        async fn retrieve_logs(
            &self,
            request: RetrieveLogsRequest,
        ) -> Result<RetrieveLogsResponse>;

        async fn start_container_instance(
            &self,
            request: StartContainerInstanceRequest,
        ) -> Result<StartContainerInstanceResponse>;

        async fn stop_container_instance(
            &self,
            request: StopContainerInstanceRequest,
        ) -> Result<StopContainerInstanceResponse>;

        async fn update_container(
            &self,
            request: UpdateContainerRequest,
        ) -> Result<UpdateContainerResponse>;

        async fn update_container_instance(
            &self,
            request: UpdateContainerInstanceRequest,
        ) -> Result<UpdateContainerInstanceResponse>;
    }
}

async fn count_instances(
    client: &dyn ContainerinstancesApi,
    compartment_id: &str,
) -> Result<usize> {
    let request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.to_string(),
    });
    let response = client.list_container_instances(request).await?;
    Ok(response.items.len())
}

#[tokio::test]
async fn test_mock_list_container_instances() {
    let mut mock = MockContainerinstancesClient::new();

    mock.expect_list_container_instances()
        .returning(|_| {
            Ok(ListContainerInstancesResponse {
                items: vec![
                    ContainerInstanceSummary::new(ContainerInstanceSummaryRequired {
                        id: "ocid1.containerinstance.oc1..aaa".to_string(),
                        display_name: "test-instance-1".to_string(),
                        compartment_id: "ocid1.compartment.oc1..test".to_string(),
                        availability_domain: "AD-1".to_string(),
                        lifecycle_state: "ACTIVE".to_string(),
                        shape: "CI.Standard.E4.Flex".to_string(),
                        shape_config: ContainerInstanceShapeConfig::new(
                            ContainerInstanceShapeConfigRequired {
                                ocpus: 1,
                                memory_in_gbs: 1,
                                processor_description: "AMD EPYC".to_string(),
                                networking_bandwidth_in_gbps: 1,
                            },
                        ),
                        container_count: 1,
                        container_restart_policy: "ALWAYS".to_string(),
                        time_created: Utc::now(),
                    }),
                ],
                opc_request_id: Some("test-request-id".to_string()),
                opc_next_page: None,
            })
        });

    let count = count_instances(&mock, "ocid1.compartment.oc1..test").await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_mock_empty_list() {
    let mut mock = MockContainerinstancesClient::new();

    mock.expect_list_container_instances()
        .returning(|_| {
            Ok(ListContainerInstancesResponse {
                items: vec![],
                opc_request_id: None,
                opc_next_page: None,
            })
        });

    let count = count_instances(&mock, "ocid1.compartment.oc1..test").await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_mock_delete_container_instance() {
    let mut mock = MockContainerinstancesClient::new();

    mock.expect_delete_container_instance()
        .returning(|req| {
            assert_eq!(req.container_instance_id, "ocid1.containerinstance.oc1..delete-me");
            Ok(DeleteContainerInstanceResponse {
                opc_request_id: Some("delete-req-id".to_string()),
                opc_work_request_id: Some("work-req-id".to_string()),
            })
        });

    let request = DeleteContainerInstanceRequest::new(DeleteContainerInstanceRequestRequired {
        container_instance_id: "ocid1.containerinstance.oc1..delete-me".to_string(),
    });
    let response = mock.delete_container_instance(request).await.unwrap();
    assert_eq!(response.opc_work_request_id, Some("work-req-id".to_string()));
}

#[tokio::test]
async fn test_mock_with_dyn_trait() {
    let mut mock = MockContainerinstancesClient::new();

    mock.expect_list_container_instances()
        .times(2)
        .returning(|req| {
            Ok(ListContainerInstancesResponse {
                items: vec![
                    ContainerInstanceSummary::new(ContainerInstanceSummaryRequired {
                        id: format!("instance-in-{}", req.compartment_id),
                        display_name: "test".to_string(),
                        compartment_id: req.compartment_id,
                        availability_domain: "AD-1".to_string(),
                        lifecycle_state: "ACTIVE".to_string(),
                        shape: "CI.Standard.E4.Flex".to_string(),
                        shape_config: ContainerInstanceShapeConfig::new(
                            ContainerInstanceShapeConfigRequired {
                                ocpus: 1,
                                memory_in_gbs: 1,
                                processor_description: "AMD".to_string(),
                                networking_bandwidth_in_gbps: 1,
                            },
                        ),
                        container_count: 1,
                        container_restart_policy: "ALWAYS".to_string(),
                        time_created: Utc::now(),
                    }),
                ],
                opc_request_id: None,
                opc_next_page: None,
            })
        });

    let client: &dyn ContainerinstancesApi = &mock;

    let resp1 = client
        .list_container_instances(
            ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
                compartment_id: "compartment-a".to_string(),
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp1.items[0].id, "instance-in-compartment-a");

    let resp2 = client
        .list_container_instances(
            ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
                compartment_id: "compartment-b".to_string(),
            }),
        )
        .await
        .unwrap();
    assert_eq!(resp2.items[0].id, "instance-in-compartment-b");
}
