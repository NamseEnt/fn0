use super::*;
use crate::args::OciContainerInstanceHostProviderArgs;
use base64::Engine;
use oci_rust_sdk::container_instances::*;
use oci_rust_sdk::core::{
    RetryConfig,
    auth::{SimpleAuthProvider, SimpleAuthProviderRequiredFields},
    region::Region,
};
use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroUsize;
use std::{net::IpAddr, str::FromStr, sync::Arc};

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Clone)]
pub struct OciContainerInstanceHostProvider {
    container_instance_client: Arc<dyn oci_rust_sdk::container_instances::ContainerInstance>,
    compartment_id: String,
    availability_domain: String,
    shape: String,
    ocpus: NonZeroUsize,
    memory_in_gbs: NonZeroUsize,
    subnet_id: String,
    image: String,
    envs: BTreeMap<String, String>,
}

impl OciContainerInstanceHostProvider {
    pub fn new(args: OciContainerInstanceHostProviderArgs) -> Self {
        let region = Region::from_str(&args.region)
            .unwrap_or_else(|_| panic!("Invalid region: {}", args.region));

        let auth_provider = SimpleAuthProvider::builder(SimpleAuthProviderRequiredFields {
            tenancy: args.tenancy_id,
            user: args.user_id,
            fingerprint: args.fingerprint,
            private_key: String::from_utf8_lossy(
                &base64::engine::general_purpose::STANDARD
                    .decode(args.private_key_base64)
                    .unwrap(),
            )
            .to_string(),
        })
        .region(region)
        .build();

        let container_instance_client =
            oci_rust_sdk::container_instances::client(oci_rust_sdk::core::ClientConfig {
                auth_provider,
                region,
                timeout: DEFAULT_TIMEOUT,
                retry: RetryConfig::no_retry(),
            })
            .unwrap();

        Self {
            container_instance_client,
            compartment_id: args.compartment_id,
            availability_domain: args.availability_domain,
            shape: args.shape,
            ocpus: args.ocpus,
            memory_in_gbs: args.memory_in_gbs,
            subnet_id: args.subnet_id,
            image: args.image,
            envs: args.envs,
        }
    }
}

impl HostProvide for OciContainerInstanceHostProvider {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>> {
        let mut page = None;
        let mut hosts = Vec::new();

        loop {
            let response = self
                .container_instance_client
                .list_container_instances(
                    ListContainerInstancesRequest::builder(
                        ListContainerInstancesRequestRequiredFields {
                            compartment_id: self.compartment_id.clone(),
                        },
                    )
                    .set_page(page)
                    .build(),
                )
                .await?;

            response.items.into_iter().for_each(|instance| {
                let Some(ip) = instance.freeform_tags.as_ref().and_then(|tags| {
                    let ip_str = tags.get("public_ip")?;
                    IpAddr::from_str(ip_str).ok()
                }) else {
                    error!("Failed to get public ip, id: {}", instance.id);
                    return;
                };

                let host = Host {
                    id: HostId::new(instance.id),
                    ip,
                };
                hosts.push(host);
            });

            if let Some(next_page) = response.opc_next_page {
                page = Some(next_page);
            } else {
                break;
            }
        }

        Ok(hosts)
    }

    async fn terminate(&self, host_id: &HostId) -> color_eyre::Result<()> {
        self.container_instance_client
            .delete_container_instance(
                DeleteContainerInstanceRequest::builder(
                    DeleteContainerInstanceRequestRequiredFields {
                        container_instance_id: host_id.to_string(),
                    },
                )
                .build(),
            )
            .await?;
        Ok(())
    }

    async fn launch_instance(&self) -> color_eyre::Result<()> {
        let environment_variables: HashMap<String, String> = self
            .envs
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let create_container_instance_details = CreateContainerInstanceDetails {
            compartment_id: self.compartment_id.clone(),
            availability_domain: self.availability_domain.clone(),
            shape: self.shape.clone(),
            shape_config: CreateContainerInstanceShapeConfigDetails {
                ocpus: self.ocpus.get() as f32,
                memory_in_gbs: self.memory_in_gbs.get() as f32,
            },
            containers: vec![CreateContainerDetails {
                image_url: self.image.clone(),
                display_name: Some("fn0-host".to_string()),
                command: None,
                arguments: None,
                environment_variables: Some(environment_variables),
                resource_config: None,
            }],
            vnics: vec![CreateContainerVnicDetails {
                subnet_id: self.subnet_id.clone(),
                display_name: None,
                hostname_label: None,
                is_public_ip_assigned: Some(true),
                skip_source_dest_check: None,
                nsg_ids: None,
                private_ip: None,
                freeform_tags: None,
                defined_tags: None,
            }],
            display_name: None,
            fault_domain: None,
            graceful_shutdown_timeout_in_seconds: None,
            container_restart_policy: Some("ALWAYS".to_string()),
            freeform_tags: None,
            defined_tags: None,
        };

        self.container_instance_client
            .create_container_instance(
                CreateContainerInstanceRequest::builder(
                    CreateContainerInstanceRequestRequiredFields {
                        create_container_instance_details,
                    },
                )
                .build(),
            )
            .await?;

        Ok(())
    }
}
