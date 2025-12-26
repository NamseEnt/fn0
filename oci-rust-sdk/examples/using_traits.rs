//! Example demonstrating how to work with the OCI Rust SDK
//!
//! This example shows different patterns for using the SDK,
//! including error handling, pagination, and request building.

use oci_rust_sdk::{
    auth::ConfigFileAuthProvider,
    containerinstances::{
        self, ListContainerInstancesRequest, ListContainerInstancesRequestRequired,
    },
    core::{Result, Retrier, region::Region},
};
use std::sync::Arc;
use std::time::Duration;

/// Demonstrates proper error handling with the SDK
async fn list_with_error_handling(compartment_id: &str) -> Result<usize> {
    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    let client = containerinstances::client(containerinstances::ClientConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(30),
        retry: Retrier::new(),
    })?;

    let request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.to_string(),
    })
    .with_limit(10);

    let response = client.list_container_instances(request).await?;

    Ok(response.items.len())
}

/// Demonstrates pagination pattern
async fn list_all_with_pagination(compartment_id: &str) -> Result<Vec<String>> {
    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    let client = containerinstances::client(containerinstances::ClientConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(30),
        retry: Retrier::new(),
    })?;

    let mut all_ids = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let mut request =
            ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
                compartment_id: compartment_id.to_string(),
            })
            .with_limit(10);

        if let Some(ref token) = page_token {
            request = request.with_page(token);
        }

        let response = client.list_container_instances(request).await?;

        // Collect IDs from this page
        for instance in &response.items {
            all_ids.push(instance.id.clone());
        }

        // Check if there are more pages
        if let Some(next) = response.opc_next_page {
            page_token = Some(next);
        } else {
            break;
        }
    }

    Ok(all_ids)
}

/// Demonstrates request builder pattern
fn demonstrate_request_building(compartment_id: &str) {
    // Pattern 1: Minimal request
    let _minimal = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.to_string(),
    });

    // Pattern 2: Request with optional parameters
    let _with_options = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.to_string(),
    })
    .with_limit(20)
    .with_display_name("my-instance");

    // Pattern 3: Conditional building
    let mut request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.to_string(),
    });

    let filter_name = Some("production");
    if let Some(name) = filter_name {
        request = request.with_display_name(name);
    }

    // Use the request to demonstrate it's valid
    let _ = request.compartment_id;

    println!("✓ Request building patterns demonstrated");
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== OCI Rust SDK Usage Patterns ===\n");

    let test_compartment = std::env::var("OCI_COMPARTMENT_ID")
        .unwrap_or_else(|_| "ocid1.compartment.oc1..test".to_string());

    // Example 1: Basic usage with error handling
    println!("Example 1: Error Handling Pattern");
    println!("-----------------------------------");

    match list_with_error_handling(&test_compartment).await {
        Ok(count) => {
            println!("✓ Found {} container instances", count);
        }
        Err(e) => {
            println!(
                "✗ Error (this is expected if credentials are not configured): {}",
                e
            );
        }
    }

    // Example 2: Request building patterns
    println!("\nExample 2: Request Building Patterns");
    println!("-------------------------------------");
    demonstrate_request_building(&test_compartment);

    // Example 3: Pagination (only if credentials are available)
    println!("\nExample 3: Pagination Pattern");
    println!("-----------------------------");

    match list_all_with_pagination(&test_compartment).await {
        Ok(ids) => {
            println!("✓ Collected {} total instances across all pages", ids.len());
            for (i, id) in ids.iter().take(3).enumerate() {
                println!("  {}. {}", i + 1, id);
            }
            if ids.len() > 3 {
                println!("  ... and {} more", ids.len() - 3);
            }
        }
        Err(e) => {
            println!("✗ Error (expected without valid credentials): {}", e);
        }
    }

    println!("\n=== Examples Complete ===");
    println!("\nKey Patterns:");
    println!("  1. Use Result<T> for error handling");
    println!("  2. Build requests with new(Required).with_*() pattern");
    println!("  3. Handle pagination with opc_next_page");
    println!("  4. Wrap auth providers in Arc for shared ownership");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_building() {
        // Test that request building doesn't panic
        let compartment_id = "test-compartment";
        demonstrate_request_building(compartment_id);
    }

    #[test]
    fn test_required_fields() {
        let compartment_id = "test-compartment";
        let request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
            compartment_id: compartment_id.to_string(),
        });

        assert_eq!(request.compartment_id, compartment_id);
        assert_eq!(request.limit, None);
        assert_eq!(request.page, None);
    }

    #[test]
    fn test_optional_fields() {
        let request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
            compartment_id: "test".to_string(),
        })
        .with_limit(50)
        .with_display_name("my-instance");

        assert_eq!(request.limit, Some(50));
        assert_eq!(request.display_name, Some("my-instance".to_string()));
    }
}
