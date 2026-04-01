use super::*;
use crate::args::OciComputeVmHostProviderArgs;
use base64::Engine;
use oci_rust_sdk::auth::{SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use oci_rust_sdk::core::{self, region::Region, *};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Arc;

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Clone)]
pub struct OciComputeVmHostProvider {
    core_client: Arc<core::CoreClient>,
    compartment_id: String,
    availability_domain: String,
    shape: String,
    ocpus: NonZeroUsize,
    memory_in_gbs: NonZeroUsize,
    subnet_id: String,
    image_id: String,
    worker_image_url: String,
    envs: BTreeMap<String, String>,
}

impl OciComputeVmHostProvider {
    pub fn new(args: OciComputeVmHostProviderArgs) -> Self {
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

        let core_client = Arc::new(
            core::client(core::ClientConfig {
                auth_provider: Arc::new(auth_provider),
                region,
                timeout: DEFAULT_TIMEOUT,
                retry: core::Retrier::new(),
            })
            .unwrap(),
        );

        Self {
            core_client,
            compartment_id: args.compartment_id,
            availability_domain: args.availability_domain,
            shape: args.shape,
            ocpus: args.ocpus,
            memory_in_gbs: args.memory_in_gbs,
            subnet_id: args.subnet_id,
            image_id: args.image_id,
            worker_image_url: args.worker_image_url,
            envs: args.envs,
        }
    }

    fn build_cloud_init(&self) -> String {
        let env_flags: String = self
            .envs
            .iter()
            .map(|(k, v)| format!("-e {}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");

        format!(
            r#"#!/bin/bash
podman pull {image}
podman run -d --restart=always --network=host --name fn0-worker {env_flags} {image}
"#,
            image = self.worker_image_url,
            env_flags = env_flags,
        )
    }
}

impl HostProvide for OciComputeVmHostProvider {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>> {
        let mut page = None;
        let mut hosts = Vec::new();

        loop {
            let mut request = ListInstancesRequest::new(ListInstancesRequestRequired {
                compartment_id: self.compartment_id.clone(),
            })
            .with_lifecycle_state("RUNNING");

            if let Some(p) = page {
                request = request.with_page(p);
            }

            let response = self.core_client.list_instances(request).await?;

            for instance in &response.items {
                let tags = instance.freeform_tags.as_ref();
                let is_managed = tags
                    .and_then(|t| t.get("managed_by"))
                    .is_some_and(|v| v == "fn0-hq");

                if !is_managed {
                    continue;
                }

                let vnic_response = self
                    .core_client
                    .list_vnic_attachments(
                        ListVnicAttachmentsRequest::new(ListVnicAttachmentsRequestRequired {
                            compartment_id: self.compartment_id.clone(),
                        })
                        .with_instance_id(&instance.id),
                    )
                    .await?;

                for attachment in &vnic_response.items {
                    let Some(vnic_id) = &attachment.vnic_id else {
                        continue;
                    };

                    let vnic_response = self
                        .core_client
                        .get_vnic(GetVnicRequest::new(GetVnicRequestRequired {
                            vnic_id: vnic_id.clone(),
                        }))
                        .await?;

                    if let Some(public_ip) = &vnic_response.vnic.public_ip {
                        hosts.push(Host {
                            id: HostId::new(instance.id.clone()),
                            addr: public_ip.clone(),
                            port: 10000,
                        });
                    }
                }
            }

            if let Some(next_page) = response.opc_next_page {
                page = Some(next_page);
            } else {
                break;
            }
        }

        Ok(hosts)
    }

    async fn terminate(&self, host_id: &HostId) -> color_eyre::Result<()> {
        self.core_client
            .terminate_instance(
                TerminateInstanceRequest::new(TerminateInstanceRequestRequired {
                    instance_id: host_id.to_string(),
                })
                .with_preserve_boot_volume(false),
            )
            .await?;
        Ok(())
    }

    async fn launch_instance(&self) -> color_eyre::Result<()> {
        let cloud_init = self.build_cloud_init();
        let user_data = base64::engine::general_purpose::STANDARD.encode(cloud_init.as_bytes());

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("user_data".to_string(), user_data);

        let mut freeform_tags = std::collections::HashMap::new();
        freeform_tags.insert("managed_by".to_string(), "fn0-hq".to_string());

        let source_details = InstanceSourceViaImageDetails::new(
            InstanceSourceViaImageDetailsRequired {
                source_type: "image".to_string(),
            },
        )
        .with_image_id(&self.image_id);

        let launch_details = LaunchInstanceDetails::new(LaunchInstanceDetailsRequired {
            availability_domain: self.availability_domain.clone(),
            compartment_id: self.compartment_id.clone(),
        })
        .with_shape(&self.shape)
        .with_source_details(source_details)
        .with_create_vnic_details(
            CreateVnicDetails::new()
                .with_subnet_id(&self.subnet_id)
                .with_assign_public_ip(true),
        )
        .with_shape_config(
            LaunchInstanceShapeConfigDetails::new()
                .with_ocpus(self.ocpus.get() as i64)
                .with_memory_in_gbs(self.memory_in_gbs.get() as i64),
        )
        .set_metadata(Some(metadata))
        .set_freeform_tags(Some(freeform_tags));

        self.core_client
            .launch_instance(LaunchInstanceRequest::new(LaunchInstanceRequestRequired {
                launch_instance_details: launch_details,
            }))
            .await?;

        Ok(())
    }
}
