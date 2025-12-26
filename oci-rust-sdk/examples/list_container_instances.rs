use oci_rust_sdk::{
    containerinstances::{
        self, ClientConfig as ContainerInstancesConfig, ContainerInstanceLifecycleState,
        ListContainerInstancesRequest, ListContainerInstancesRequestRequired,
    },
    auth::ConfigFileAuthProvider,
    core::{region::Region, retry::Retrier},
};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    let client = containerinstances::client(ContainerInstancesConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(30),
        retry: Retrier::new(),
    })?;

    let compartment_id = std::env::var("OCI_COMPARTMENT_ID")
        .unwrap_or_else(|_| "ocid1.compartment.oc1..aaaaaaaxxxxx".to_string());

    println!("=== Example 1: List All Container Instances ===");
    let request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.clone(),
    })
    .with_limit(10);

    match client.list_container_instances(request).await {
        Ok(response) => {
            println!("Found {} container instances", response.items.len());
            for instance in &response.items {
                println!("  - {}: {}", instance.display_name, instance.id);
                println!("    Shape: {}", instance.shape);
                println!("    State: {:?}", instance.lifecycle_state);
                println!("    Containers: {}", instance.container_count);
            }
        }
        Err(e) => {
            eprintln!("Error listing container instances: {}", e);
        }
    }

    println!("\n=== Example 2: List Active Container Instances ===");
    let request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.clone(),
    })
    .with_lifecycle_state(ContainerInstanceLifecycleState::Active.to_string());

    match client.list_container_instances(request).await {
        Ok(response) => {
            println!("Found {} active container instances", response.items.len());
            for instance in &response.items {
                println!("  - {}: {}", instance.display_name, instance.shape);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
