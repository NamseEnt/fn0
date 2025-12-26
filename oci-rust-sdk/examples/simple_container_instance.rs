use oci_rust_sdk::{
    containerinstances::{
        self, ClientConfig as ContainerInstancesConfig, CreateContainerDetails,
        CreateContainerDetailsRequired, CreateContainerInstanceDetails,
        CreateContainerInstanceDetailsRequired, CreateContainerInstanceShapeConfigDetails,
        CreateContainerInstanceShapeConfigDetailsRequired, CreateContainerVnicDetails,
        CreateContainerVnicDetailsRequired, CreateContainerInstanceRequest,
        CreateContainerInstanceRequestRequired,
    },
    auth::ConfigFileAuthProvider,
    core::{region::Region, retry::Retrier},
};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    let client = containerinstances::client(ContainerInstancesConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(60),
        retry: Retrier::new(),
    })?;

    let compartment_id =
        std::env::var("OCI_COMPARTMENT_ID").expect("OCI_COMPARTMENT_ID must be set");
    let subnet_id = std::env::var("OCI_SUBNET_ID").expect("OCI_SUBNET_ID must be set");
    let availability_domain =
        std::env::var("OCI_AVAILABILITY_DOMAIN").expect("OCI_AVAILABILITY_DOMAIN must be set");

    println!("Creating a simple nginx container instance...\n");

    let mut env_vars = HashMap::new();
    env_vars.insert("APP_NAME".to_string(), "my-app".to_string());

    let container = CreateContainerDetails::new(CreateContainerDetailsRequired {
        image_url: "nginx:alpine".to_string(),
    })
    .with_display_name("nginx")
    .with_environment_variables(env_vars);

    let vnic = CreateContainerVnicDetails::new(CreateContainerVnicDetailsRequired { subnet_id })
        .with_display_name("main-vnic")
        .with_is_public_ip_assigned(true);

    let shape_config = CreateContainerInstanceShapeConfigDetails::new(
        CreateContainerInstanceShapeConfigDetailsRequired { ocpus: 1 },
    )
    .with_memory_in_gbs(4);

    let create_details = CreateContainerInstanceDetails::new(
        CreateContainerInstanceDetailsRequired {
            compartment_id,
            availability_domain,
            shape: "CI.Standard.E4.Flex".to_string(),
            shape_config,
            containers: vec![container],
            vnics: vec![vnic],
        },
    )
    .with_display_name("my-nginx-instance")
    .with_graceful_shutdown_timeout_in_seconds(30)
    .with_container_restart_policy("ALWAYS");

    let request = CreateContainerInstanceRequest::new(CreateContainerInstanceRequestRequired {
        create_container_instance_details: create_details,
    });

    match client.create_container_instance(request).await {
        Ok(response) => {
            println!("Container instance created successfully!");
            println!("ID: {}", response.container_instance.id);
            println!("Display Name: {}", response.container_instance.display_name);
        }
        Err(e) => {
            eprintln!("Error creating container instance: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
