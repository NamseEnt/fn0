# Testing with Trait-Based Mocking

This guide shows how to create custom mock implementations of OCI SDK services for testing.

## Overview

The OCI Rust SDK uses traits to abstract service operations, enabling you to create your own mock implementations for testing without any external mocking libraries.

## Basic Mock Implementation

To create a mock, simply implement the service trait:

```rust
use oci_rust_sdk::core::Result;
use oci_rust_sdk::os_management_hub::{
    OsManagementHub,
    ListManagedInstancesRequest,
    ListManagedInstancesResponse,
    ManagedInstanceCollection,
};

struct MyMock;

impl OsManagementHub for MyMock {
    async fn list_managed_instances(
        &self,
        _request: ListManagedInstancesRequest,
    ) -> Result<ListManagedInstancesResponse> {
        Ok(ListManagedInstancesResponse {
            managed_instance_collection: ManagedInstanceCollection { items: vec![] },
            opc_request_id: Some("mock-id".to_string()),
            opc_next_page: None,
            opc_total_items: Some(0),
        })
    }
}
```

## Using Mocks in Tests

### Example 1: Simple Mock

```rust
#[tokio::test]
async fn test_my_feature() {
    let mock = MyMock;

    // Use the mock just like the real client
    let request = ListManagedInstancesRequest::builder().build();
    let response = mock.list_managed_instances(request).await.unwrap();

    assert_eq!(response.managed_instance_collection.items.len(), 0);
}
```

### Example 2: Configurable Mock

Create mocks that return different responses based on configuration:

```rust
struct ConfigurableMock {
    instance_count: usize,
    should_fail: bool,
}

impl ConfigurableMock {
    fn new(instance_count: usize) -> Self {
        Self { instance_count, should_fail: false }
    }

    fn with_error(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl OsManagementHub for ConfigurableMock {
    async fn list_managed_instances(
        &self,
        _request: ListManagedInstancesRequest,
    ) -> Result<ListManagedInstancesResponse> {
        if self.should_fail {
            return Err(OciError::RequestFailed("Simulated error".to_string()));
        }

        // Generate mock instances based on configuration
        let items = (0..self.instance_count)
            .map(|i| ManagedInstanceSummary {
                id: format!("instance-{}", i),
                display_name: Some(format!("Instance {}", i)),
                // ... other fields
            })
            .collect();

        Ok(ListManagedInstancesResponse {
            managed_instance_collection: ManagedInstanceCollection { items },
            opc_request_id: Some("mock-id".to_string()),
            opc_next_page: None,
            opc_total_items: Some(self.instance_count),
        })
    }
}

#[tokio::test]
async fn test_with_configurable_mock() {
    // Test success case
    let mock = ConfigurableMock::new(5);
    let response = mock.list_managed_instances(
        ListManagedInstancesRequest::builder().build()
    ).await.unwrap();
    assert_eq!(response.managed_instance_collection.items.len(), 5);

    // Test error case
    let error_mock = ConfigurableMock::new(0).with_error();
    let result = error_mock.list_managed_instances(
        ListManagedInstancesRequest::builder().build()
    ).await;
    assert!(result.is_err());
}
```

### Example 3: Request Verification Mock

Create mocks that verify requests were made correctly:

```rust
use std::sync::{Arc, Mutex};

struct VerifyingMock {
    calls: Arc<Mutex<Vec<ListManagedInstancesRequest>>>,
}

impl VerifyingMock {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn verify_called(&self, expected_count: usize) {
        let calls = self.calls.lock().unwrap();
        assert_eq!(calls.len(), expected_count,
            "Expected {} calls but got {}", expected_count, calls.len());
    }

    fn verify_compartment_id(&self, expected: &str) {
        let calls = self.calls.lock().unwrap();
        assert_eq!(calls[0].compartment_id.as_deref(), Some(expected));
    }
}

impl OsManagementHub for VerifyingMock {
    async fn list_managed_instances(
        &self,
        request: ListManagedInstancesRequest,
    ) -> Result<ListManagedInstancesResponse> {
        // Record the call
        self.calls.lock().unwrap().push(request);

        // Return mock response
        Ok(ListManagedInstancesResponse {
            managed_instance_collection: ManagedInstanceCollection { items: vec![] },
            opc_request_id: Some("mock-id".to_string()),
            opc_next_page: None,
            opc_total_items: Some(0),
        })
    }
}

#[tokio::test]
async fn test_request_verification() {
    let mock = VerifyingMock::new();

    let request = ListManagedInstancesRequest::builder()
        .compartment_id("ocid1.compartment.oc1..xxxxx")
        .build();

    mock.list_managed_instances(request).await.unwrap();

    mock.verify_called(1);
    mock.verify_compartment_id("ocid1.compartment.oc1..xxxxx");
}
```

## Dependency Injection Pattern

Write your application code to accept traits instead of concrete types:

```rust
struct MyService<T: OsManagementHub> {
    os_mgmt: Arc<T>,
}

impl<T: OsManagementHub> MyService<T> {
    fn new(os_mgmt: Arc<T>) -> Self {
        Self { os_mgmt }
    }

    async fn get_instance_count(&self, compartment_id: &str) -> Result<usize> {
        let request = ListManagedInstancesRequest::builder()
            .compartment_id(compartment_id)
            .build();

        let response = self.os_mgmt.list_managed_instances(request).await?;
        Ok(response.managed_instance_collection.items.len())
    }
}

// In production code
let real_client = OsManagementHubClient::new(auth, region)?;
let service = MyService::new(Arc::new(real_client));

// In test code
let mock = MyMock;
let service = MyService::new(Arc::new(mock));
```

## Using Trait Objects

For runtime polymorphism, use trait objects:

```rust
fn create_service(use_mock: bool) -> Box<dyn OsManagementHub> {
    if use_mock {
        Box::new(MyMock)
    } else {
        Box::new(OsManagementHubClient::new(auth, region).unwrap())
    }
}

async fn use_service(service: &dyn OsManagementHub) -> Result<()> {
    let request = ListManagedInstancesRequest::builder().build();
    let response = service.list_managed_instances(request).await?;
    println!("Got {} instances", response.managed_instance_collection.items.len());
    Ok(())
}
```

## Testing Error Handling

Test how your code handles errors by returning errors from mocks:

```rust
struct ErrorMock;

impl OsManagementHub for ErrorMock {
    async fn list_managed_instances(
        &self,
        _request: ListManagedInstancesRequest,
    ) -> Result<ListManagedInstancesResponse> {
        Err(OciError::RequestFailed("Network error".to_string()))
    }
}

#[tokio::test]
async fn test_error_handling() {
    let mock = ErrorMock;
    let result = mock.list_managed_instances(
        ListManagedInstancesRequest::builder().build()
    ).await;

    assert!(result.is_err());
    match result {
        Err(OciError::RequestFailed(msg)) => {
            assert_eq!(msg, "Network error");
        }
        _ => panic!("Expected RequestFailed error"),
    }
}
```

## Best Practices

1. **Keep Mocks Simple**: Only mock the behavior you need for the specific test
2. **Use Builder Pattern**: Create helper methods to construct common mock responses
3. **Verify Calls**: Track and verify that methods were called with expected parameters
4. **Test Both Success and Failure**: Create mocks that can simulate errors
5. **Use Type Parameters**: Write generic code that accepts the trait, not concrete types
6. **Document Mock Behavior**: Clearly document what each mock does

## Advanced: Using External Mocking Libraries (Optional)

While not required, you can use external libraries like `mockall` for more sophisticated mocking:

```toml
[dev-dependencies]
mockall = "0.13"
```

However, the trait-based approach in this SDK is designed to work without external dependencies, giving you full control over mock behavior.

## See Also

- [examples/using_traits.rs](../examples/using_traits.rs) - Complete working examples
- [Adding Services Guide](ADDING_SERVICES.md) - How to add new services with the same pattern
