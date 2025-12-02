pub mod models;
pub mod requests;

pub use models::*;
pub use requests::*;

use crate::core::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Trait defining operations for OS Management Hub service
///
/// This trait abstracts all OS Management Hub API operations, enabling
/// dependency injection and allowing users to create their own mock
/// implementations for testing.
///
/// # Real Implementation
///
/// Use the `client()` function to create a real client:
///
/// ```no_run
/// use std::sync::Arc;
/// use oci_rust_sdk::core::{auth::ConfigFileAuthProvider, region::Region};
/// use oci_rust_sdk::os_management_hub;
///
/// # async fn example() -> oci_rust_sdk::core::Result<()> {
/// let auth = Arc::new(ConfigFileAuthProvider::from_default()?);
/// let client = os_management_hub::client(auth, Region::ApSeoul1)?;
///
/// // Use the client through the trait
/// let request = os_management_hub::ListManagedInstancesRequest::builder().build();
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

/// Create a new OS Management Hub client for the specified region.
///
/// Returns an `Arc<dyn OsManagementHub>` that can be used to interact with the
/// OS Management Hub service. The client is configured with the provided
/// authentication and region.
///
/// # Arguments
///
/// * `auth_provider` - Authentication provider for signing requests
/// * `region` - OCI region where the service will be accessed
///
/// # Example
///
/// ```no_run
/// use oci_rust_sdk::os_management_hub;
/// use oci_rust_sdk::core::{auth::ConfigFileAuthProvider, region::Region};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = Arc::new(ConfigFileAuthProvider::from_default()?);
/// let client = os_management_hub::client(auth, Region::ApSeoul1)?;
///
/// let request = os_management_hub::ListManagedInstancesRequest::builder()
///     .compartment_id("ocid1.compartment.oc1..xxxxx")
///     .build();
/// let response = client.list_managed_instances(request).await?;
/// # Ok(())
/// # }
/// ```
pub fn client(
    auth_provider: Arc<dyn crate::core::auth::AuthProvider>,
    region: crate::core::region::Region,
) -> Result<Arc<dyn OsManagementHub>> {
    let endpoint = region.endpoint("osmh");
    let oci_client = crate::core::OciClient::new(auth_provider, endpoint)?;
    Ok(Arc::new(oci_client))
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
