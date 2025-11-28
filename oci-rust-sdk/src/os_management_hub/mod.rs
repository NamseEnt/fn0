pub mod models;
pub mod requests;

pub use models::*;
pub use requests::*;

use crate::core::Result;
use std::future::Future;
use std::pin::Pin;

/// Trait defining operations for OS Management Hub service
///
/// This trait abstracts all OS Management Hub API operations, enabling
/// dependency injection and allowing users to create their own mock
/// implementations for testing.
///
/// # Real Implementation
///
/// `OciClient` implements this trait to provide real OCI API access:
///
/// ```no_run
/// use std::sync::Arc;
/// use oci_rust_sdk::core::{auth::ConfigFileAuthProvider, region::Region, OciClient};
/// use oci_rust_sdk::os_management_hub::{OsManagementHub, ListManagedInstancesRequest};
///
/// # async fn example() -> oci_rust_sdk::core::Result<()> {
/// let auth = Arc::new(ConfigFileAuthProvider::from_default()?);
/// let endpoint = Region::ApSeoul1.endpoint("osmh");
/// let client = OciClient::new(auth, endpoint)?;
///
/// // Use OciClient through the trait
/// let request = ListManagedInstancesRequest::builder().build();
/// let response = client.list_managed_instances(request).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Mock Implementation
///
/// Users can implement this trait for their own mock types:
///
/// ```
/// use std::pin::Pin;
/// use std::future::Future;
/// use oci_rust_sdk::core::Result;
/// use oci_rust_sdk::os_management_hub::{
///     OsManagementHub,
///     ListManagedInstancesRequest,
///     ListManagedInstancesResponse,
///     ManagedInstanceCollection,
/// };
///
/// struct MyMock;
///
/// impl OsManagementHub for MyMock {
///     fn list_managed_instances(
///         &self,
///         _request: ListManagedInstancesRequest,
///     ) -> Pin<Box<dyn Future<Output = Result<ListManagedInstancesResponse>> + Send + '_>> {
///         Box::pin(async {
///             Ok(ListManagedInstancesResponse {
///                 managed_instance_collection: ManagedInstanceCollection { items: vec![] },
///                 opc_request_id: Some("mock-id".to_string()),
///                 opc_next_page: None,
///                 opc_total_items: Some(0),
///             })
///         })
///     }
/// }
/// ```
///
/// # Using the Trait
///
/// Write functions that accept the trait instead of concrete types:
///
/// ```
/// async fn count_instances<T: OsManagementHub>(
///     service: &T,
///     compartment_id: &str,
/// ) -> Result<usize> {
///     let request = ListManagedInstancesRequest::builder()
///         .compartment_id(compartment_id)
///         .build();
///     let response = service.list_managed_instances(request).await?;
///     Ok(response.managed_instance_collection.items.len())
/// }
/// ```
pub trait OsManagementHub: Send + Sync {
    /// List managed instances
    ///
    /// Returns a list of managed instances that match the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters for listing managed instances
    ///
    /// # Errors
    ///
    /// Returns `OciError` if the request fails
    fn list_managed_instances(
        &self,
        request: ListManagedInstancesRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ListManagedInstancesResponse>> + Send + '_>>;

    // Future methods will be added here as the SDK expands:
    // async fn get_managed_instance(&self, request: GetManagedInstanceRequest) -> Result<GetManagedInstanceResponse>;
    // async fn update_managed_instance(&self, request: UpdateManagedInstanceRequest) -> Result<UpdateManagedInstanceResponse>;
    // async fn delete_managed_instance(&self, request: DeleteManagedInstanceRequest) -> Result<DeleteManagedInstanceResponse>;
}

/// Implementation of OsManagementHub trait for OciClient
impl OsManagementHub for crate::core::OciClient {
    fn list_managed_instances(
        &self,
        request: ListManagedInstancesRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ListManagedInstancesResponse>> + Send + '_>,
    > {
        Box::pin(async move {
            // Build query string
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

            let path = format!("/managedInstances{}", query_string);

            // Make GET request
            let oci_response = self.get::<ManagedInstanceCollection>(&path).await?;

            // Extract headers
            let opc_request_id = oci_response.get_header("opc-request-id");
            let opc_next_page = oci_response.get_header("opc-next-page");
            let opc_total_items = oci_response
                .get_header("opc-total-items")
                .and_then(|v| v.parse().ok());

            Ok(ListManagedInstancesResponse {
                managed_instance_collection: oci_response.body,
                opc_request_id,
                opc_next_page,
                opc_total_items,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example: Users can create their own mock by implementing the trait
    struct SimpleMock;

    impl OsManagementHub for SimpleMock {
        fn list_managed_instances(
            &self,
            _request: ListManagedInstancesRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<ListManagedInstancesResponse>> + Send + '_>,
        > {
            Box::pin(async {
                Ok(ListManagedInstancesResponse {
                    managed_instance_collection: ManagedInstanceCollection { items: vec![] },
                    opc_request_id: Some("test-id".to_string()),
                    opc_next_page: None,
                    opc_total_items: Some(0),
                })
            })
        }
    }

    // Business logic accepting trait - demonstrates dependency injection
    async fn process_instances<T: OsManagementHub>(service: &T) -> Result<usize> {
        let request = ListManagedInstancesRequest::builder().limit(10).build();
        let response = service.list_managed_instances(request).await?;
        Ok(response.managed_instance_collection.items.len())
    }

    #[tokio::test]
    async fn test_with_user_defined_mock() {
        let mock = SimpleMock;
        let count = process_instances(&mock).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_mock_returns_expected_headers() {
        let mock = SimpleMock;
        let request = ListManagedInstancesRequest::builder().build();
        let response = mock.list_managed_instances(request).await.unwrap();

        assert_eq!(response.opc_request_id, Some("test-id".to_string()));
        assert_eq!(response.opc_next_page, None);
        assert_eq!(response.opc_total_items, Some(0));
    }
}
