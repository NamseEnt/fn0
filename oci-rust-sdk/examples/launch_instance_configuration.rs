use oci_rust_sdk::{
    auth::ConfigFileAuthProvider,
    core::{
        self, LaunchInstanceDetails, LaunchInstanceDetailsRequired, LaunchInstanceRequest,
        LaunchInstanceRequestRequired, Retrier, region::Region,
    },
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OCI Launch Instance Example ===\n");

    // Initialize authentication from config file
    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    // Create core client
    let client = core::client(core::ClientConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(60),
        retry: Retrier::new(),
    })?;

    // Get configuration from environment variables
    let compartment_id = std::env::var("OCI_COMPARTMENT_ID")
        .expect("OCI_COMPARTMENT_ID environment variable must be set");
    let availability_domain = std::env::var("OCI_AVAILABILITY_DOMAIN")
        .expect("OCI_AVAILABILITY_DOMAIN environment variable must be set");
    let subnet_id =
        std::env::var("OCI_SUBNET_ID").expect("OCI_SUBNET_ID environment variable must be set");
    let image_id =
        std::env::var("OCI_IMAGE_ID").expect("OCI_IMAGE_ID environment variable must be set");

    println!("Configuration:");
    println!("  Compartment ID: {}", compartment_id);
    println!("  Availability Domain: {}", availability_domain);
    println!("  Subnet ID: {}", subnet_id);
    println!("  Image ID: {}\n", image_id);

    // Create launch instance details
    let launch_details = LaunchInstanceDetails::new(LaunchInstanceDetailsRequired {
        availability_domain: availability_domain.clone(),
        compartment_id: compartment_id.clone(),
    })
    .with_display_name("example-instance")
    .with_shape("VM.Standard.E4.Flex")
    .with_image_id(image_id);

    // Build the request
    let request = LaunchInstanceRequest::new(LaunchInstanceRequestRequired {
        launch_instance_details: launch_details,
    });

    // Launch the instance
    println!("Launching instance...");
    match client.launch_instance(request).await {
        Ok(response) => {
            println!("✓ Instance launched successfully!");
            println!("  Instance ID: {}", response.instance.id);
            if let Some(ref display_name) = response.instance.display_name {
                println!("  Display Name: {}", display_name);
            }
            println!("  Shape: {}", response.instance.shape);
            println!("  Lifecycle State: {:?}", response.instance.lifecycle_state);
            println!(
                "  Availability Domain: {}",
                response.instance.availability_domain
            );

            if let Some(work_request_id) = response.opc_work_request_id {
                println!("\n  Work Request ID: {}", work_request_id);
                println!("  Use GetWorkRequest to track the status of this operation.");
            }

            if let Some(request_id) = response.opc_request_id {
                println!("  Request ID: {}", request_id);
            }

            println!("\nNote: The instance will take a few minutes to reach the RUNNING state.");
            println!("Use 'list_instances' example to check the instance status.");
        }
        Err(e) => {
            eprintln!("✗ Error launching instance: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
