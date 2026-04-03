use super::*;
use crate::args::OciContainerInstanceHostProviderArgs;
use base64::Engine;
use oci_rust_sdk::auth::{SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use oci_rust_sdk::container_instances::*;
use oci_rust_sdk::core::{Retrier, region::Region};
use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Arc;
use super::oci_scaler::{OciScaler, ScaleAction};
use tracing::*;

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Clone)]
pub struct OciContainerInstanceHostProvider {
    container_instance_client: Arc<ContainerinstancesClient>,
    compartment_id: String,
    availability_domain: String,
    shape: String,
    ocpus: NonZeroUsize,
    memory_in_gbs: NonZeroUsize,
    subnet_id: String,
    image: String,
    envs: BTreeMap<String, String>,
    scaler: Arc<OciScaler>,
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

        let container_instance_client = Arc::new(
            oci_rust_sdk::container_instances::client(ClientConfig {
                auth_provider: Arc::new(auth_provider),
                region,
                timeout: DEFAULT_TIMEOUT,
                retry: Retrier::new(),
            })
            .unwrap(),
        );

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
            scaler: Arc::new(OciScaler::new()),
        }
    }

    async fn count_managed_instances(&self) -> color_eyre::Result<usize> {
        let mut count = 0;

        for state in ["CREATING", "UPDATING", "ACTIVE"] {
            let mut page = None;
            loop {
                let response = self
                    .container_instance_client
                    .list_container_instances(
                        ListContainerInstancesRequest::new(
                            ListContainerInstancesRequestRequired {
                                compartment_id: self.compartment_id.clone(),
                            },
                        )
                        .with_lifecycle_state(state)
                        .set_page(page),
                    )
                    .await?;

                count += response.items.len();

                if let Some(next_page) = response.opc_next_page {
                    page = Some(next_page);
                } else {
                    break;
                }
            }
        }

        Ok(count)
    }

    async fn launch_one(&self) -> color_eyre::Result<()> {
        let environment_variables: HashMap<String, String> = self
            .envs
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let create_container_instance_details = CreateContainerInstanceDetails::new(
            CreateContainerInstanceDetailsRequired {
                compartment_id: self.compartment_id.clone(),
                availability_domain: self.availability_domain.clone(),
                shape: self.shape.clone(),
                shape_config: CreateContainerInstanceShapeConfigDetails::new(
                    CreateContainerInstanceShapeConfigDetailsRequired {
                        ocpus: self.ocpus.get() as i64,
                    },
                )
                .with_memory_in_gbs(self.memory_in_gbs.get() as i64),
                containers: vec![CreateContainerDetails::new(CreateContainerDetailsRequired {
                    image_url: self.image.clone(),
                })
                .with_display_name("fn0-host")
                .set_environment_variables(Some(environment_variables))],
                vnics: vec![CreateContainerVnicDetails::new(
                    CreateContainerVnicDetailsRequired {
                        subnet_id: self.subnet_id.clone(),
                    },
                )
                .with_is_public_ip_assigned(true)],
            },
        )
        .with_container_restart_policy("ALWAYS");

        self.container_instance_client
            .create_container_instance(
                CreateContainerInstanceRequest::new(
                    CreateContainerInstanceRequestRequired {
                        create_container_instance_details,
                    },
                ),
            )
            .await?;

        Ok(())
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
                    ListContainerInstancesRequest::new(
                        ListContainerInstancesRequestRequired {
                            compartment_id: self.compartment_id.clone(),
                        },
                    )
                    .set_page(page),
                )
                .await?;

            response.items.into_iter().for_each(|instance| {
                let Some(ip_str) = instance.freeform_tags.as_ref().and_then(|tags| {
                    tags.get("public_ip").cloned()
                }) else {
                    error!("Failed to get public ip, id: {}", instance.id);
                    return;
                };

                let host = Host {
                    id: HostId::new(instance.id),
                    addr: ip_str,
                    port: 10000,
                    transport: HostTransport::Quic,
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
                DeleteContainerInstanceRequest::new(
                    DeleteContainerInstanceRequestRequired {
                        container_instance_id: host_id.to_string(),
                    },
                ),
            )
            .await?;
        Ok(())
    }

    async fn scale_to(&self, n: usize) -> color_eyre::Result<()> {
        let count = self.count_managed_instances().await?;

        match self.scaler.decide(count, n).await {
            ScaleAction::ScaleOut(deficit) => {
                info!(current = count, target = n, launching = deficit, "Scaling out");
                for _ in 0..deficit {
                    if let Err(err) = self.launch_one().await {
                        warn!(%err, "Failed to launch container instance");
                    }
                }
            }
            ScaleAction::ScaleIn(surplus) => {
                info!(current = count, target = n, terminating = surplus, "Scaling in");
                let hosts = self.list_hosts().await?;
                for host in hosts.into_iter().take(surplus) {
                    if let Err(err) = self.terminate(&host.id).await {
                        warn!(%err, "Failed to terminate container instance");
                    }
                }
            }
            ScaleAction::Locked | ScaleAction::NoOp => {}
        }

        Ok(())
    }
}
