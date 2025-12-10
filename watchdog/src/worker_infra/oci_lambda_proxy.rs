use super::*;
use crate::*;
use futures::TryStreamExt;
use oci_rust_sdk::compute::*;
use std::{env, net::IpAddr, str::FromStr};

pub struct OciLambdaProxyWorkerInfra {
    proxy: ::oci_lambda_proxy::OciLambdaProxy,
    compartment_id: String,
    instance_configuration_id: String,
    availability_domain: String,
}

impl OciLambdaProxyWorkerInfra {
    pub async fn new() -> Self {
        let compartment_id =
            env::var("OCI_COMPARTMENT_ID").expect("env var OCI_COMPARTMENT_ID is not set");
        let instance_configuration_id = env::var("OCI_INSTANCE_CONFIGURATION_ID")
            .expect("env var OCI_INSTANCE_CONFIGURATION_ID is not set");
        let availability_domain = env::var("OCI_AVAILABILITY_DOMAIN")
            .expect("env var OCI_AVAILABILITY_DOMAIN is not set");
        let fn_name = env::var("OCI_LAMBDA_PROXY_FN_NAME")
            .expect("env var OCI_LAMBDA_PROXY_FN_NAME is not set");
        let proxy = ::oci_lambda_proxy::OciLambdaProxy::new(fn_name).await;
        Self {
            proxy,
            compartment_id,
            instance_configuration_id,
            availability_domain,
        }
    }
}

impl WorkerInfra for OciLambdaProxyWorkerInfra {
    fn get_worker_infos<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<WorkerInfos>> + 'a + Send>> {
        Box::pin(async move {
            let mut infos = vec![];
            let mut page = None;

            loop {
                println!("on loop top");
                let response = self
                    .proxy
                    .list_instances(ListInstancesRequest {
                        compartment_id: self.compartment_id.clone(),
                        limit: None,
                        page,
                        availability_domain: None,
                        capacity_reservation_id: None,
                        compute_cluster_id: None,
                        display_name: None,
                        sort_by: None,
                        sort_order: None,
                        lifecycle_state: None,
                    })
                    .await?;

                println!("got response with {} items", response.items.len());
                infos.extend(response.items.into_iter().map(|instance| WorkerInfo {
                    id: WorkerId(instance.id),
                    ip: instance.freeform_tags.and_then(|tags| {
                        let ip = tags.get("public_ip")?;
                        let Ok(ip) = IpAddr::from_str(ip) else {
                            panic!("Failed to parse IP address: {ip}");
                        };
                        Some(ip)
                    }),
                    instance_state: match instance.lifecycle_state {
                        LifecycleState::Provisioning | LifecycleState::Starting => {
                            WorkerInstanceState::Starting
                        }
                        LifecycleState::Running => WorkerInstanceState::Running,
                        LifecycleState::Stopping
                        | LifecycleState::Stopped
                        | LifecycleState::Terminating
                        | LifecycleState::Terminated => WorkerInstanceState::Terminating,
                        LifecycleState::Moving | LifecycleState::CreatingImage => unreachable!(),
                    },
                    instance_created: instance.time_created,
                }));

                println!("processed items, next page: {:?}", response.opc_next_page);
                if let Some(next_page) = response.opc_next_page {
                    page = Some(next_page);
                } else {
                    break;
                }
            }
            Ok(infos)
        })
    }

    fn terminate<'a>(
        &'a self,
        worker_id: &'a WorkerId,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>> {
        Box::pin(async move {
            self.proxy
                .terminate_instance(TerminateInstanceRequest {
                    instance_id: worker_id.0.clone(),
                    if_match: None,
                    preserve_boot_volume: Some(false),
                    preserve_data_volumes_created_at_launch: Some(false),
                })
                .await?;
            Ok(())
        })
    }

    fn launch_instances<'a>(
        &'a self,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>> {
        Box::pin(async move {
            futures::stream::iter(0..count)
                .map(|_| async move {
                    self.proxy
                        .launch_instance_configuration(LaunchInstanceConfigurationRequest {
                            instance_configuration_id: self.instance_configuration_id.clone(),
                            instance_configuration: InstanceConfigurationInstanceDetails::Compute(
                                ComputeInstanceDetails {
                                    launch_details: Some(InstanceConfigurationLaunchInstanceDetails {
                                        availability_domain: Some(self.availability_domain.clone()),
                                        compartment_id: Some(self.compartment_id.clone()),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                },
                            ),
                            opc_retry_token: None,
                        })
                        .await?;

                    color_eyre::eyre::Ok(())
                })
                .buffer_unordered(4)
                .try_collect::<Vec<()>>()
                .await?;

            Ok(())
        })
    }
}
