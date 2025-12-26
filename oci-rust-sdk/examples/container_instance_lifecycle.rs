use oci_rust_sdk::{
    auth::ConfigFileAuthProvider,
    containerinstances::{
        self, ContainerInstanceLifecycleState, CreateContainerDetails,
        CreateContainerDetailsRequired, CreateContainerInstanceDetails,
        CreateContainerInstanceDetailsRequired, CreateContainerInstanceRequest,
        CreateContainerInstanceRequestRequired, CreateContainerInstanceShapeConfigDetails,
        CreateContainerInstanceShapeConfigDetailsRequired, CreateContainerVnicDetails,
        CreateContainerVnicDetailsRequired, DeleteContainerInstanceRequest,
        DeleteContainerInstanceRequestRequired, GetContainerInstanceRequest,
        GetContainerInstanceRequestRequired, ListContainerInstancesRequest,
        ListContainerInstancesRequestRequired, RestartContainerInstanceRequest,
        RestartContainerInstanceRequestRequired, StartContainerInstanceRequest,
        StartContainerInstanceRequestRequired, StopContainerInstanceRequest,
        StopContainerInstanceRequestRequired,
    },
    core::{Retrier, region::Region},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OCI Container Instance Lifecycle Example ===\n");

    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    let client = containerinstances::client(containerinstances::ClientConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(60),
        retry: Retrier::new(),
    })?;

    let compartment_id = std::env::var("OCI_COMPARTMENT_ID")
        .expect("OCI_COMPARTMENT_ID environment variable must be set");
    let subnet_id =
        std::env::var("OCI_SUBNET_ID").expect("OCI_SUBNET_ID environment variable must be set");
    let availability_domain = std::env::var("OCI_AVAILABILITY_DOMAIN")
        .expect("OCI_AVAILABILITY_DOMAIN environment variable must be set");

    println!("Configuration:");
    println!("  Compartment ID: {}", compartment_id);
    println!("  Subnet ID: {}", subnet_id);
    println!("  Availability Domain: {}\n", availability_domain);

    println!("=== Step 1: Create Container Instance ===");

    let mut env_vars = HashMap::new();
    env_vars.insert("EXAMPLE_VAR".to_string(), "example_value".to_string());
    env_vars.insert("ENVIRONMENT".to_string(), "test".to_string());

    let container = CreateContainerDetails::new(CreateContainerDetailsRequired {
        image_url: "nginx:latest".to_string(),
    })
    .with_display_name("nginx-container")
    .with_environment_variables(env_vars);

    let vnic = CreateContainerVnicDetails::new(CreateContainerVnicDetailsRequired {
        subnet_id: subnet_id.clone(),
    })
    .with_display_name("primary-vnic")
    .with_hostname_label("nginx-host")
    .with_is_public_ip_assigned(true);

    let shape_config = CreateContainerInstanceShapeConfigDetails::new(
        CreateContainerInstanceShapeConfigDetailsRequired { ocpus: 1 },
    )
    .with_memory_in_gbs(4);

    let create_details =
        CreateContainerInstanceDetails::new(CreateContainerInstanceDetailsRequired {
            compartment_id: compartment_id.clone(),
            availability_domain: availability_domain.clone(),
            shape: "CI.Standard.E4.Flex".to_string(),
            shape_config,
            containers: vec![container],
            vnics: vec![vnic],
        })
        .with_display_name("example-container-instance")
        .with_graceful_shutdown_timeout_in_seconds(30)
        .with_container_restart_policy("ALWAYS");

    let create_request =
        CreateContainerInstanceRequest::new(CreateContainerInstanceRequestRequired {
            create_container_instance_details: create_details,
        });

    let container_instance_id = match client.create_container_instance(create_request).await {
        Ok(response) => {
            println!("✓ Container instance created successfully!");
            println!("  ID: {}", response.container_instance.id);
            println!(
                "  Display Name: {}",
                response.container_instance.display_name
            );
            println!("  State: {:?}", response.container_instance.lifecycle_state);

            if let Some(work_request_id) = response.opc_work_request_id {
                println!("  Work Request ID: {}", work_request_id);
            }

            response.container_instance.id.clone()
        }
        Err(e) => {
            eprintln!("✗ Error creating container instance: {}", e);
            return Err(e.into());
        }
    };

    println!("\n=== Step 2: Wait for Container Instance to Become Active ===");
    println!("Polling container instance status...");

    let mut poll_count = 0;
    let max_polls = 60;

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        poll_count += 1;

        let get_request = GetContainerInstanceRequest::new(GetContainerInstanceRequestRequired {
            container_instance_id: container_instance_id.clone(),
        });

        match client.get_container_instance(get_request).await {
            Ok(response) => {
                println!(
                    "  Poll {}: State = {:?}",
                    poll_count, response.container_instance.lifecycle_state
                );

                match response.container_instance.lifecycle_state {
                    ContainerInstanceLifecycleState::Active => {
                        println!("\n✓ Container instance is now ACTIVE!");
                        println!("  VNICs: {}", response.container_instance.vnics.len());
                        break;
                    }
                    ContainerInstanceLifecycleState::Failed => {
                        eprintln!("\n✗ Container instance creation FAILED!");
                        if let Some(ref details) = response.container_instance.lifecycle_details {
                            eprintln!("  Details: {}", details);
                        }
                        return Err("Container instance creation failed".into());
                    }
                    _ => {
                        if poll_count >= max_polls {
                            eprintln!(
                                "\n✗ Timeout waiting for container instance to become active"
                            );
                            return Err("Timeout".into());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error polling status: {}", e);
            }
        }
    }

    println!("\n=== Step 3: Stop Container Instance ===");
    let stop_request = StopContainerInstanceRequest::new(StopContainerInstanceRequestRequired {
        container_instance_id: container_instance_id.clone(),
    });

    match client.stop_container_instance(stop_request).await {
        Ok(response) => {
            println!("✓ Stop request sent successfully!");
            if let Some(work_request_id) = response.opc_work_request_id {
                println!("  Work Request ID: {}", work_request_id);
            }
        }
        Err(e) => {
            eprintln!("✗ Error stopping container instance: {}", e);
        }
    }

    // Wait a bit for the stop operation
    tokio::time::sleep(Duration::from_secs(10)).await;

    println!("\n=== Step 4: Start Container Instance ===");
    let start_request = StartContainerInstanceRequest::new(StartContainerInstanceRequestRequired {
        container_instance_id: container_instance_id.clone(),
    });

    match client.start_container_instance(start_request).await {
        Ok(response) => {
            println!("✓ Start request sent successfully!");
            if let Some(work_request_id) = response.opc_work_request_id {
                println!("  Work Request ID: {}", work_request_id);
            }
        }
        Err(e) => {
            eprintln!("✗ Error starting container instance: {}", e);
        }
    }

    // Wait a bit for the start operation
    tokio::time::sleep(Duration::from_secs(10)).await;

    println!("\n=== Step 5: Restart Container Instance ===");
    let restart_request =
        RestartContainerInstanceRequest::new(RestartContainerInstanceRequestRequired {
            container_instance_id: container_instance_id.clone(),
        });

    match client.restart_container_instance(restart_request).await {
        Ok(response) => {
            println!("✓ Restart request sent successfully!");
            if let Some(work_request_id) = response.opc_work_request_id {
                println!("  Work Request ID: {}", work_request_id);
            }
        }
        Err(e) => {
            eprintln!("✗ Error restarting container instance: {}", e);
        }
    }

    println!("\n=== Step 6: List All Container Instances ===");
    let list_request = ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
        compartment_id: compartment_id.clone(),
    });

    match client.list_container_instances(list_request).await {
        Ok(response) => {
            println!("Total container instances: {}", response.items.len());
            for instance in &response.items {
                println!("  - {} ({})", instance.display_name, instance.id);
                println!("    State: {:?}", instance.lifecycle_state);
            }
        }
        Err(e) => {
            eprintln!("Error listing: {}", e);
        }
    }

    println!("\n=== Step 7: Delete Container Instance ===");
    println!("Deleting container instance: {}", container_instance_id);

    let delete_request =
        DeleteContainerInstanceRequest::new(DeleteContainerInstanceRequestRequired {
            container_instance_id: container_instance_id.clone(),
        });

    match client.delete_container_instance(delete_request).await {
        Ok(response) => {
            println!("✓ Delete request sent successfully!");
            if let Some(work_request_id) = response.opc_work_request_id {
                println!("  Work Request ID: {}", work_request_id);
            }
        }
        Err(e) => {
            eprintln!("✗ Error deleting container instance: {}", e);
            return Err(e.into());
        }
    }

    println!("\n=== Step 8: Wait for Container Instance to be Deleted ===");
    poll_count = 0;

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        poll_count += 1;

        let list_request =
            ListContainerInstancesRequest::new(ListContainerInstancesRequestRequired {
                compartment_id: compartment_id.clone(),
            })
            .with_display_name("example-container-instance");

        match client.list_container_instances(list_request).await {
            Ok(response) => {
                if let Some(instance) = response
                    .items
                    .iter()
                    .find(|i| i.id == container_instance_id)
                {
                    println!(
                        "  Poll {}: State = {:?}",
                        poll_count, instance.lifecycle_state
                    );

                    if instance.lifecycle_state == "DELETED" {
                        println!("\n✓ Container instance has been DELETED!");
                        break;
                    }
                } else {
                    println!("\n✓ Container instance no longer appears in list (deleted)!");
                    break;
                }

                if poll_count >= max_polls {
                    eprintln!("\n✗ Timeout waiting for deletion");
                    return Err("Timeout".into());
                }
            }
            Err(e) => {
                eprintln!("Error polling status: {}", e);
            }
        }
    }

    println!("\n=== Lifecycle Complete ===");
    println!("✓ Successfully demonstrated complete container instance lifecycle!");
    println!("  Operations performed:");
    println!("    - Create instance");
    println!("    - Stop instance");
    println!("    - Start instance");
    println!("    - Restart instance");
    println!("    - Delete instance");

    Ok(())
}
