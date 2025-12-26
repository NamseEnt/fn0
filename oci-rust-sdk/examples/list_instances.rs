use oci_rust_sdk::{
    auth::ConfigFileAuthProvider,
    core::{self, ListInstancesRequest, ListInstancesRequestRequired, Retrier, region::Region},
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up authentication from default OCI config file
    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    // Create a Core client
    let client = core::client(core::ClientConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(30),
        retry: Retrier::new(),
    })?;

    // IMPORTANT: Replace this with your actual compartment OCID
    let compartment_id = std::env::var("OCI_COMPARTMENT_ID")
        .unwrap_or_else(|_| "ocid1.compartment.oc1..aaaaaaaxxxxx".to_string());

    println!("=== Example 1: List All Instances ===");
    let request = ListInstancesRequest::new(ListInstancesRequestRequired {
        compartment_id: compartment_id.clone(),
    })
    .with_limit(10);

    match client.list_instances(request).await {
        Ok(response) => {
            println!("Found {} instances", response.items.len());
            for instance in &response.items {
                println!(
                    "  - {}: {}",
                    instance.display_name.as_deref().unwrap_or("(unnamed)"),
                    instance.id
                );
                println!("    Shape: {}", instance.shape);
                println!("    State: {:?}", instance.lifecycle_state);
                println!("    AD: {}", instance.availability_domain);
                println!("    Created: {}", instance.time_created);
            }

            if let Some(next_page) = response.opc_next_page {
                println!("\nMore results available. Next page token: {}", next_page);
            }

            if let Some(request_id) = response.opc_request_id {
                println!("\nRequest ID: {}", request_id);
            }
        }
        Err(e) => {
            eprintln!("Error listing instances: {}", e);
        }
    }

    println!("\n=== Example 2: List Running Instances Only ===");
    let request = ListInstancesRequest::new(ListInstancesRequestRequired {
        compartment_id: compartment_id.clone(),
    })
    .with_lifecycle_state("RUNNING");

    match client.list_instances(request).await {
        Ok(response) => {
            println!("Found {} running instances", response.items.len());
            for instance in &response.items {
                println!(
                    "  - {}: {}",
                    instance.display_name.as_deref().unwrap_or("(unnamed)"),
                    instance.shape
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n=== Example 3: Filter by Display Name ===");
    let request = ListInstancesRequest::new(ListInstancesRequestRequired {
        compartment_id: compartment_id.clone(),
    })
    .with_display_name("my-instance");

    match client.list_instances(request).await {
        Ok(response) => {
            println!(
                "Found {} instances matching 'my-instance'",
                response.items.len()
            );
            for instance in &response.items {
                println!("  - {} ({:?})", instance.id, instance.lifecycle_state);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\n=== Example 4: Pagination Example ===");
    let mut page_token: Option<String> = None;
    let mut page_number = 1;
    let max_pages = 3;
    let mut total_instances = 0;

    loop {
        let mut request = ListInstancesRequest::new(ListInstancesRequestRequired {
            compartment_id: compartment_id.clone(),
        })
        .with_limit(2); // Small limit to demonstrate pagination

        if let Some(ref token) = page_token {
            request = request.with_page(token);
        }

        match client.list_instances(request).await {
            Ok(response) => {
                println!(
                    "\nPage {}: Found {} instances",
                    page_number,
                    response.items.len()
                );
                total_instances += response.items.len();

                for instance in &response.items {
                    println!(
                        "  - {}",
                        instance.display_name.as_deref().unwrap_or("(unnamed)")
                    );
                }

                // Check if there are more pages
                if let Some(next) = response.opc_next_page {
                    page_token = Some(next);
                    page_number += 1;

                    if page_number > max_pages {
                        println!(
                            "\nReached maximum pages ({}), stopping pagination demo",
                            max_pages
                        );
                        break;
                    }
                } else {
                    println!("\nNo more pages available");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    println!("\nTotal instances found: {}", total_instances);

    Ok(())
}
