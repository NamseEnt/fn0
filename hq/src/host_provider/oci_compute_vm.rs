use super::oci_scaler::{OciScaler, ScaleAction};
use super::*;
use crate::args::OciComputeVmHostProviderArgs;
use base64::Engine;
use oci_rust_sdk::auth::{SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use oci_rust_sdk::core::{self, region::Region, *};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Arc;
use tracing::*;

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Clone)]
pub struct OciComputeVmHostProvider {
    core_client: Arc<dyn CoreApi>,
    compartment_id: String,
    availability_domain: String,
    shape: String,
    ocpus: NonZeroUsize,
    memory_in_gbs: NonZeroUsize,
    subnet_id: String,
    image_id: String,
    worker_image_url: String,
    envs: BTreeMap<String, String>,
    scaler: Arc<OciScaler>,
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

        let core_client: Arc<dyn CoreApi> = Arc::new(
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
            scaler: Arc::new(OciScaler::new()),
        }
    }

    #[cfg(test)]
    pub fn new_with_client(client: Arc<dyn CoreApi>) -> Self {
        Self {
            core_client: client,
            compartment_id: "test-compartment".to_string(),
            availability_domain: "test-ad".to_string(),
            shape: "VM.Standard.A1.Flex".to_string(),
            ocpus: NonZeroUsize::new(1).unwrap(),
            memory_in_gbs: NonZeroUsize::new(6).unwrap(),
            subnet_id: "test-subnet".to_string(),
            image_id: "test-image".to_string(),
            worker_image_url: "test-image:latest".to_string(),
            envs: BTreeMap::new(),
            scaler: Arc::new(OciScaler::new()),
        }
    }

    fn build_cloud_init(&self) -> String {
        let env_flags: String = self
            .envs
            .iter()
            .map(|(k, v)| {
                if v.contains('\n') {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(v.as_bytes());
                    format!("-e {k}_BASE64={encoded}")
                } else {
                    format!("-e {k}={v}")
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        format!(
            r#"#!/bin/bash
systemctl disable --now firewalld 2>/dev/null || true
podman pull {image}
podman run -d --restart=always --network=host --name fn0-worker {env_flags} {image}
"#,
            image = self.worker_image_url,
            env_flags = env_flags,
        )
    }

    async fn count_managed_instances(&self) -> color_eyre::Result<usize> {
        let mut count = 0;

        for state in ["PROVISIONING", "STARTING", "RUNNING"] {
            let mut page = None;
            loop {
                let mut request = ListInstancesRequest::new(ListInstancesRequestRequired {
                    compartment_id: self.compartment_id.clone(),
                })
                .with_lifecycle_state(state);

                if let Some(p) = page {
                    request = request.with_page(p);
                }

                let response = self.core_client.list_instances(request).await?;

                for instance in &response.items {
                    let is_managed = instance
                        .freeform_tags
                        .as_ref()
                        .and_then(|t| t.get("managed_by"))
                        .is_some_and(|v| v == "fn0-hq");

                    if is_managed {
                        count += 1;
                    }
                }

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
                .with_assign_public_ip(true)
                .with_assign_ipv6_ip(true),
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

                    let vnic = &vnic_response.vnic;
                    let Some(addr) = vnic
                        .ipv6_addresses
                        .as_ref()
                        .and_then(|addrs| addrs.first())
                        .cloned()
                    else {
                        warn!(instance_id = %instance.id, "No IPv6 address, skipping");
                        continue;
                    };

                    hosts.push(Host {
                        id: HostId::new(instance.id.clone()),
                        addr,
                        port: 10000,
                        transport: HostTransport::Quic,
                        dns_addr: vnic.public_ip.clone(),
                    });
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

    async fn scale_to(&self, n: usize) -> color_eyre::Result<()> {
        let count = self.count_managed_instances().await?;

        match self.scaler.decide(count, n).await {
            ScaleAction::ScaleOut(deficit) => {
                info!(current = count, target = n, launching = deficit, "Scaling out");
                for _ in 0..deficit {
                    if let Err(err) = self.launch_one().await {
                        warn!(%err, "Failed to launch instance");
                    }
                }
            }
            ScaleAction::ScaleIn(surplus) => {
                info!(current = count, target = n, terminating = surplus, "Scaling in");
                let hosts = self.list_hosts().await?;
                for host in hosts.into_iter().take(surplus) {
                    if let Err(err) = self.terminate(&host.id).await {
                        warn!(%err, "Failed to terminate instance");
                    }
                }
            }
            ScaleAction::Locked | ScaleAction::NoOp => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mockall::mock;
    use oci_rust_sdk::core::models::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    mock! {
        pub TestCoreClient {}

        #[async_trait::async_trait]
        impl CoreApi for TestCoreClient {
            async fn list_instances(&self, request: ListInstancesRequest) -> core::Result<ListInstancesResponse>;
            async fn launch_instance(&self, request: LaunchInstanceRequest) -> core::Result<LaunchInstanceResponse>;
            async fn get_instance(&self, request: GetInstanceRequest) -> core::Result<GetInstanceResponse>;
            async fn terminate_instance(&self, request: TerminateInstanceRequest) -> core::Result<TerminateInstanceResponse>;
            async fn list_vnic_attachments(&self, request: ListVnicAttachmentsRequest) -> core::Result<ListVnicAttachmentsResponse>;
            async fn get_vnic(&self, request: GetVnicRequest) -> core::Result<GetVnicResponse>;
            async fn list_public_ips(&self, request: ListPublicIpsRequest) -> core::Result<ListPublicIpsResponse>;
        }
    }

    fn make_managed_instance(id: &str) -> Instance {
        let mut tags = HashMap::new();
        tags.insert("managed_by".to_string(), "fn0-hq".to_string());
        Instance::new(InstanceRequired {
            availability_domain: "ad".to_string(),
            compartment_id: "test-compartment".to_string(),
            id: id.to_string(),
            lifecycle_state: InstanceLifecycleState::Running,
            region: "test-region".to_string(),
            shape: "VM.Standard.A1.Flex".to_string(),
            time_created: Utc::now(),
        })
        .set_freeform_tags(Some(tags))
    }

    fn empty_list_response() -> core::Result<ListInstancesResponse> {
        Ok(ListInstancesResponse::new(ListInstancesResponseRequired {
            items: vec![],
        }))
    }

    fn list_response_with(instances: Vec<Instance>) -> core::Result<ListInstancesResponse> {
        Ok(ListInstancesResponse::new(ListInstancesResponseRequired {
            items: instances,
        }))
    }

    fn make_launch_response() -> core::Result<LaunchInstanceResponse> {
        Ok(LaunchInstanceResponse::new(
            LaunchInstanceResponseRequired {
                instance: make_managed_instance("new"),
            },
        ))
    }

    fn make_terminate_response() -> core::Result<TerminateInstanceResponse> {
        Ok(TerminateInstanceResponse::new(
            TerminateInstanceResponseRequired {
                opc_request_id: "req-id".to_string(),
            },
        ))
    }

    fn make_vnic_attachments_response(instance_id: &str) -> core::Result<ListVnicAttachmentsResponse> {
        Ok(ListVnicAttachmentsResponse::new(
            ListVnicAttachmentsResponseRequired {
                opc_next_page: String::new(),
                opc_request_id: "req-id".to_string(),
                items: vec![VnicAttachment::new(VnicAttachmentRequired {
                    availability_domain: "ad".to_string(),
                    compartment_id: "test-compartment".to_string(),
                    id: format!("vnic-attach-{instance_id}"),
                    instance_id: instance_id.to_string(),
                    lifecycle_state: VnicAttachmentLifecycleState::Attached,
                    time_created: Utc::now(),
                })
                .with_vnic_id(format!("vnic-{instance_id}"))],
            },
        ))
    }

    fn make_get_vnic_response(vnic_id: &str) -> core::Result<GetVnicResponse> {
        Ok(GetVnicResponse::new(GetVnicResponseRequired {
            etag: "etag".to_string(),
            opc_request_id: "req-id".to_string(),
            vnic: Vnic::new(VnicRequired {
                availability_domain: "ad".to_string(),
                compartment_id: "test-compartment".to_string(),
                id: vnic_id.to_string(),
                lifecycle_state: VnicLifecycleState::Available,
                time_created: Utc::now(),
            })
            .with_public_ip("1.2.3.4")
            .with_ipv6_addresses(vec!["2603:c023:b:1600::1".to_string()]),
        }))
    }

    #[tokio::test]
    async fn scale_to_launches_when_below_target() {
        let mut mock = MockTestCoreClient::new();
        let launch_count = Arc::new(AtomicUsize::new(0));
        let lc = launch_count.clone();

        mock.expect_list_instances()
            .returning(|_| empty_list_response());
        mock.expect_launch_instance()
            .returning(move |_| { lc.fetch_add(1, Ordering::SeqCst); make_launch_response() });

        let provider = OciComputeVmHostProvider::new_with_client(Arc::new(mock));
        provider.scale_to(2).await.unwrap();

        assert_eq!(launch_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn scale_to_noop_when_at_target() {
        let mut mock = MockTestCoreClient::new();
        let launch_count = Arc::new(AtomicUsize::new(0));
        let lc = launch_count.clone();

        mock.expect_list_instances().returning(|req| {
            if req.lifecycle_state.as_deref() == Some("RUNNING") {
                list_response_with(vec![make_managed_instance("i1"), make_managed_instance("i2")])
            } else {
                empty_list_response()
            }
        });
        mock.expect_launch_instance()
            .returning(move |_| { lc.fetch_add(1, Ordering::SeqCst); make_launch_response() });

        let provider = OciComputeVmHostProvider::new_with_client(Arc::new(mock));
        provider.scale_to(2).await.unwrap();

        assert_eq!(launch_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn scale_to_locked_after_launch() {
        let mut mock = MockTestCoreClient::new();
        let launch_count = Arc::new(AtomicUsize::new(0));
        let lc = launch_count.clone();

        mock.expect_list_instances()
            .returning(|_| empty_list_response());
        mock.expect_launch_instance()
            .returning(move |_| { lc.fetch_add(1, Ordering::SeqCst); make_launch_response() });

        let provider = OciComputeVmHostProvider::new_with_client(Arc::new(mock));

        provider.scale_to(2).await.unwrap();
        assert_eq!(launch_count.load(Ordering::SeqCst), 2);

        provider.scale_to(2).await.unwrap();
        assert_eq!(launch_count.load(Ordering::SeqCst), 2, "Should be locked, no additional launches");

        provider.scale_to(2).await.unwrap();
        assert_eq!(launch_count.load(Ordering::SeqCst), 2, "Still locked");
    }

    #[tokio::test]
    async fn scale_to_unlocks_when_instances_appear() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let cc = call_count.clone();
        let launch_count = Arc::new(AtomicUsize::new(0));
        let lc = launch_count.clone();

        let mut mock = MockTestCoreClient::new();

        mock.expect_list_instances().returning(move |req| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            let state = req.lifecycle_state.as_deref().unwrap_or("");
            if n >= 3 && state == "PROVISIONING" {
                list_response_with(vec![make_managed_instance("i1"), make_managed_instance("i2")])
            } else {
                empty_list_response()
            }
        });
        mock.expect_launch_instance()
            .returning(move |_| { lc.fetch_add(1, Ordering::SeqCst); make_launch_response() });

        let provider = OciComputeVmHostProvider::new_with_client(Arc::new(mock));

        provider.scale_to(2).await.unwrap();
        assert_eq!(launch_count.load(Ordering::SeqCst), 2);

        provider.scale_to(2).await.unwrap();
        assert_eq!(launch_count.load(Ordering::SeqCst), 2, "count=2 matches target, no more launches");
    }

    #[tokio::test]
    async fn scale_to_terminates_when_above_target() {
        let terminate_count = Arc::new(AtomicUsize::new(0));
        let tc = terminate_count.clone();

        let mut mock = MockTestCoreClient::new();

        mock.expect_list_instances().returning(|req| {
            if req.lifecycle_state.as_deref() == Some("RUNNING") {
                list_response_with(vec![
                    make_managed_instance("i1"),
                    make_managed_instance("i2"),
                    make_managed_instance("i3"),
                    make_managed_instance("i4"),
                ])
            } else {
                empty_list_response()
            }
        });
        mock.expect_list_vnic_attachments().returning(|req| {
            let id = req.instance_id.clone().unwrap_or_default();
            make_vnic_attachments_response(&id)
        });
        mock.expect_get_vnic().returning(|req| {
            make_get_vnic_response(&req.vnic_id)
        });
        mock.expect_terminate_instance()
            .returning(move |_| { tc.fetch_add(1, Ordering::SeqCst); make_terminate_response() });

        let provider = OciComputeVmHostProvider::new_with_client(Arc::new(mock));
        provider.scale_to(2).await.unwrap();

        assert_eq!(terminate_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn scale_to_does_not_double_launch_on_repeated_calls() {
        let launch_count = Arc::new(AtomicUsize::new(0));
        let lc = launch_count.clone();

        let mut mock = MockTestCoreClient::new();

        mock.expect_list_instances()
            .returning(|_| empty_list_response());
        mock.expect_launch_instance()
            .returning(move |_| { lc.fetch_add(1, Ordering::SeqCst); make_launch_response() });

        let provider = OciComputeVmHostProvider::new_with_client(Arc::new(mock));

        for _ in 0..10 {
            provider.scale_to(2).await.unwrap();
        }

        assert_eq!(launch_count.load(Ordering::SeqCst), 2, "Should launch exactly 2 despite 10 calls");
    }
}
