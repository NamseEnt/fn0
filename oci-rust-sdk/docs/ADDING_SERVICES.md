# Adding New OCI Services

This guide explains how to add new OCI services to the SDK following the established trait-based pattern.

## Overview

Each OCI service follows a consistent structure:
- A trait defining service operations
- A client struct implementing the trait
- Request and response models
- Module organization for discoverability

## Step-by-Step Guide

### 1. Create Service Module Structure

For a new service (e.g., Compute), create the following directory structure:

```
src/
  compute/
    mod.rs              - Module exports
    trait.rs            - Service trait definition
    client.rs           - Client implementation
    models/             - Response models
      mod.rs
      instance.rs
      image.rs
    requests/           - Request builders
      mod.rs
      list_instances.rs
      get_instance.rs
```

### 2. Define the Service Trait

Create `src/compute/trait.rs`:

```rust
use crate::core::Result;
use crate::compute::requests::*;
use crate::compute::models::*;

/// Trait defining operations for Compute service
///
/// This trait abstracts all Compute API operations, enabling
/// dependency injection and custom mock implementations.
pub trait Compute: Send + Sync {
    /// List compute instances
    async fn list_instances(
        &self,
        request: ListInstancesRequest,
    ) -> Result<ListInstancesResponse>;

    /// Get a specific compute instance
    async fn get_instance(
        &self,
        request: GetInstanceRequest,
    ) -> Result<GetInstanceResponse>;

    // Add more methods as needed
}
```

**Key Points:**
- Use `Send + Sync` bounds for async compatibility
- Import types from local modules
- Document each method
- Keep method signatures async

### 3. Create Request Models

Create `src/compute/requests/list_instances.rs`:

```rust
/// Request to list compute instances
#[derive(Debug, Clone, Default)]
pub struct ListInstancesRequest {
    pub compartment_id: Option<String>,
    pub availability_domain: Option<String>,
    pub limit: Option<i32>,
    pub page: Option<String>,
}

impl ListInstancesRequest {
    pub fn builder() -> ListInstancesRequestBuilder {
        ListInstancesRequestBuilder::default()
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(ref compartment_id) = self.compartment_id {
            params.push(("compartmentId".to_string(), compartment_id.clone()));
        }
        if let Some(ref availability_domain) = self.availability_domain {
            params.push(("availabilityDomain".to_string(), availability_domain.clone()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(ref page) = self.page {
            params.push(("page".to_string(), page.clone()));
        }

        params
    }
}

#[derive(Debug, Default)]
pub struct ListInstancesRequestBuilder {
    compartment_id: Option<String>,
    availability_domain: Option<String>,
    limit: Option<i32>,
    page: Option<String>,
}

impl ListInstancesRequestBuilder {
    pub fn compartment_id(mut self, compartment_id: impl Into<String>) -> Self {
        self.compartment_id = Some(compartment_id.into());
        self
    }

    pub fn availability_domain(mut self, availability_domain: impl Into<String>) -> Self {
        self.availability_domain = Some(availability_domain.into());
        self
    }

    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn page(mut self, page: impl Into<String>) -> Self {
        self.page = Some(page.into());
        self
    }

    pub fn build(self) -> ListInstancesRequest {
        ListInstancesRequest {
            compartment_id: self.compartment_id,
            availability_domain: self.availability_domain,
            limit: self.limit,
            page: self.page,
        }
    }
}
```

### 4. Create Response Models

Create `src/compute/models/instance.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Response containing list of instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInstancesResponse {
    pub instances: Vec<Instance>,
    #[serde(skip)]
    pub opc_request_id: Option<String>,
    #[serde(skip)]
    pub opc_next_page: Option<String>,
}

/// Represents a compute instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "compartmentId")]
    pub compartment_id: String,
    #[serde(rename = "lifecycleState")]
    pub lifecycle_state: String,
    // Add more fields as needed
}
```

### 5. Implement the Client

Create `src/compute/client.rs`:

```rust
use std::sync::Arc;
use crate::core::{OciHttpClient, Result, auth::AuthProvider, region::Region};
use crate::compute::{Compute, requests::*, models::*};

/// Client for Compute service operations
pub struct ComputeClient {
    http_client: OciHttpClient,
}

impl ComputeClient {
    /// Create a new Compute client
    pub fn new(
        auth_provider: Arc<dyn AuthProvider>,
        region: Region,
    ) -> Result<Self> {
        let endpoint = region.endpoint("iaas");  // Compute uses iaas endpoint
        let http_client = OciHttpClient::new(auth_provider, endpoint)?;
        Ok(Self { http_client })
    }
}

impl Compute for ComputeClient {
    async fn list_instances(
        &self,
        request: ListInstancesRequest,
    ) -> Result<ListInstancesResponse> {
        let query_params = request.to_query_params();
        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!(
                "?{}",
                query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}",
                        urlencoding::encode(k),
                        urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&")
            )
        };

        let path = format!("/instances{}", query_string);
        let oci_response = self.http_client.get::<Vec<Instance>>(&path).await?;

        Ok(ListInstancesResponse {
            instances: oci_response.body,
            opc_request_id: oci_response.get_header("opc-request-id"),
            opc_next_page: oci_response.get_header("opc-next-page"),
        })
    }

    async fn get_instance(
        &self,
        request: GetInstanceRequest,
    ) -> Result<GetInstanceResponse> {
        let path = format!("/instances/{}", request.instance_id);
        let oci_response = self.http_client.get::<Instance>(&path).await?;

        Ok(GetInstanceResponse {
            instance: oci_response.body,
            opc_request_id: oci_response.get_header("opc-request-id"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example mock for testing
    struct MockCompute;

    impl Compute for MockCompute {
        async fn list_instances(
            &self,
            _request: ListInstancesRequest,
        ) -> Result<ListInstancesResponse> {
            Ok(ListInstancesResponse {
                instances: vec![],
                opc_request_id: Some("test-id".to_string()),
                opc_next_page: None,
            })
        }

        async fn get_instance(
            &self,
            _request: GetInstanceRequest,
        ) -> Result<GetInstanceResponse> {
            Ok(GetInstanceResponse {
                instance: Instance {
                    id: "test-instance".to_string(),
                    display_name: Some("Test Instance".to_string()),
                    compartment_id: "test-compartment".to_string(),
                    lifecycle_state: "RUNNING".to_string(),
                },
                opc_request_id: Some("test-id".to_string()),
            })
        }
    }

    #[tokio::test]
    async fn test_with_mock() {
        let mock = MockCompute;
        let response = mock.list_instances(
            ListInstancesRequest::builder().build()
        ).await.unwrap();
        assert_eq!(response.instances.len(), 0);
    }
}
```

### 6. Set Up Module Exports

Create `src/compute/mod.rs`:

```rust
pub mod models;
pub mod requests;
pub mod client;
mod r#trait;

pub use client::ComputeClient;
pub use r#trait::Compute;
pub use models::*;
pub use requests::*;
```

### 7. Add to Top-Level Library

Update `src/lib.rs`:

```rust
#[cfg(feature = "compute")]
pub mod compute;

#[cfg(feature = "compute")]
pub use compute::{
    Compute,
    ComputeClient,
    models::*,
    requests::*,
};
```

### 8. Add Feature Flag

Update `Cargo.toml`:

```toml
[features]
default = []
os_management_hub = []
compute = []  # Add new feature
```

## Endpoint Configuration

Different services use different endpoint prefixes. Common ones:

- OS Management Hub: `"osmh"`
- Compute (IaaS): `"iaas"`
- Object Storage: `"objectstorage"`
- Identity: `"identity"`

Check OCI documentation for the correct endpoint prefix for your service.

## Query Parameter Naming

OCI APIs typically use camelCase for query parameters and JSON fields:
- `compartmentId` not `compartment_id`
- `displayName` not `display_name`

Use `#[serde(rename = "...")]` for struct fields but keep Rust field names as snake_case.

## Testing Your Service

1. **Unit Tests**: Add tests in `client.rs` using mock implementations
2. **Integration Tests**: Create tests in `tests/` directory
3. **Examples**: Add usage examples in `examples/`

Example integration test:

```rust
// tests/compute_integration.rs
#[cfg(feature = "compute")]
mod compute_tests {
    use std::sync::Arc;
    use oci_rust_sdk::compute::{Compute, ComputeClient, ListInstancesRequest};
    use oci_rust_sdk::core::{auth::ConfigFileAuthProvider, region::Region};

    #[tokio::test]
    #[ignore]  // Requires real OCI credentials
    async fn test_list_instances() {
        let auth = Arc::new(ConfigFileAuthProvider::from_default().unwrap());
        let client = ComputeClient::new(auth, Region::UsPhoenix1).unwrap();

        let request = ListInstancesRequest::builder()
            .compartment_id("ocid1.compartment.oc1..xxxxx")
            .limit(10)
            .build();

        let response = client.list_instances(request).await.unwrap();
        println!("Found {} instances", response.instances.len());
    }
}
```

## Checklist

When adding a new service, ensure:

- [ ] Trait defined with `Send + Sync` bounds
- [ ] Client struct implements the trait
- [ ] Request models use builder pattern
- [ ] Response models use serde for JSON deserialization
- [ ] Module exports are correct
- [ ] Feature flag added to Cargo.toml
- [ ] Top-level exports added to lib.rs
- [ ] Example tests showing mock usage
- [ ] Documentation for all public types
- [ ] Example usage in `examples/` directory

## Common Patterns

### Pagination

```rust
pub async fn list_all_instances<T: Compute>(
    client: &T,
    compartment_id: &str,
) -> Result<Vec<Instance>> {
    let mut all_instances = Vec::new();
    let mut page = None;

    loop {
        let request = ListInstancesRequest::builder()
            .compartment_id(compartment_id)
            .page(page.clone().unwrap_or_default())
            .build();

        let response = client.list_instances(request).await?;
        all_instances.extend(response.instances);

        if response.opc_next_page.is_none() {
            break;
        }
        page = response.opc_next_page;
    }

    Ok(all_instances)
}
```

### Error Handling

```rust
use crate::core::{Result, OciError};

impl Compute for ComputeClient {
    async fn delete_instance(
        &self,
        request: DeleteInstanceRequest,
    ) -> Result<()> {
        let path = format!("/instances/{}", request.instance_id);
        self.http_client.delete::<()>(&path).await?;
        Ok(())
    }
}
```

## See Also

- [Testing Guide](TESTING.md) - How to create mocks
- [examples/using_traits.rs](../examples/using_traits.rs) - Complete example
- [OCI API Documentation](https://docs.oracle.com/en-us/iaas/api/) - Official API docs
