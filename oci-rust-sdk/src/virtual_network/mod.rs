pub mod models;
pub mod requests;

pub use models::*;
pub use requests::*;

use crate::core::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Trait defining operations for Virtual Network (Core) service
pub trait VirtualNetwork: Send + Sync {
    /// List public IPs in a compartment
    fn list_public_ips(
        &self,
        request: ListPublicIpsRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ListPublicIpsResponse>> + Send + '_>>;
}

/// Create a new Virtual Network (Core) client for the specified region.
///
/// Returns an `Arc<dyn VirtualNetwork>` that can be used to interact with the
/// Virtual Network (Core Services) API. This service handles networking resources
/// like VCNs, subnets, and public IPs.
///
/// # Arguments
///
/// * `auth_provider` - Authentication provider for signing requests
/// * `region` - OCI region where the service will be accessed
///
/// # Example
///
/// ```no_run
/// use oci_rust_sdk::virtual_network;
/// use oci_rust_sdk::core::{auth::ConfigFileAuthProvider, region::Region};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = Arc::new(ConfigFileAuthProvider::from_default()?);
/// let client = virtual_network::client(auth, Region::ApSeoul1)?;
///
/// let request = virtual_network::ListPublicIpsRequest::builder()
///     .compartment_id("ocid1.compartment.oc1..xxxxx")
///     .build();
/// let response = client.list_public_ips(request).await?;
/// # Ok(())
/// # }
/// ```
pub fn client(
    auth_provider: Arc<dyn crate::core::auth::AuthProvider>,
    region: crate::core::region::Region,
) -> Result<Arc<dyn VirtualNetwork>> {
    let endpoint = region.endpoint("iaas");
    let oci_client = crate::core::OciClient::new(auth_provider, endpoint)?;
    Ok(Arc::new(oci_client))
}

/// Implementation of VirtualNetwork trait for OciClient
impl VirtualNetwork for crate::core::OciClient {
    fn list_public_ips(
        &self,
        request: ListPublicIpsRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ListPublicIpsResponse>> + Send + '_>> {
        Box::pin(async move {
            // Build query string from request parameters
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

            // API version 20160918
            let path = format!("/20160918/publicIps{}", query_string);

            // Make GET request - API returns Vec<PublicIp> directly
            let oci_response = self.get::<Vec<PublicIp>>(&path).await?;

            // Extract pagination and request tracking headers
            let opc_request_id = oci_response.get_header("opc-request-id");
            let opc_next_page = oci_response.get_header("opc-next-page");

            Ok(ListPublicIpsResponse {
                items: oci_response.body,
                opc_request_id,
                opc_next_page,
            })
        })
    }
}
