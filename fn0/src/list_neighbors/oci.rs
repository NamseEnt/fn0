use super::*;
use color_eyre::eyre::{self, Context};
use oci_rust_sdk::virtual_network::{Lifetime, ListPublicIpsRequest, Scope, VirtualNetwork};
use sonic_rs::JsonValueTrait;
use std::{str::FromStr, sync::Arc};

pub struct OciListNeighbors {
    worker_port: u16,
    oci_client: Arc<dyn VirtualNetwork>,
}

impl OciListNeighbors {
    pub fn new(worker_port: u16, oci_client: Arc<dyn VirtualNetwork>) -> Self {
        Self {
            worker_port,
            oci_client,
        }
    }

    async fn list_public_ips(
        &self,
        compartment_id: String,
    ) -> Result<Vec<std::net::IpAddr>, eyre::Error> {
        let mut ips = vec![];
        let mut next_page = None;

        loop {
            let result = self
                .oci_client
                .list_public_ips(ListPublicIpsRequest {
                    scope: Scope::AvailabilityDomain,
                    compartment_id: compartment_id.clone(),
                    limit: None,
                    page: next_page,
                    availability_domain: None,
                    lifetime: Some(Lifetime::Ephemeral),
                    public_ip_pool_id: None,
                })
                .await?;

            ips.extend(
                result
                    .items
                    .into_iter()
                    .map(|ip| IpAddr::from_str(&ip.ip_address).unwrap()),
            );
            next_page = result.opc_next_page;
            if next_page.is_none() {
                break;
            }
        }

        Ok(ips)
    }
}

impl ListNeighbors for OciListNeighbors {
    async fn list_neighbors(&self) -> Result<Vec<std::net::IpAddr>, eyre::Error> {
        let compartment_id = get_compartment_id()
            .await
            .context("failed to retrieve compartment ID from OCI metadata service")?;
        let public_ips = self
            .list_public_ips(compartment_id)
            .await
            .context("failed to list public IPs from OCI")?;

        let worker_port = self.worker_port;
        let futures = public_ips.into_iter().map(move |ip| async move {
            let is_worker = reqwest::get(format!("http://{ip}:{worker_port}/role"))
                .await?
                .text()
                .await?
                == "worker";
            Ok::<Option<IpAddr>, eyre::Error>(is_worker.then_some(ip))
        });
        let neighbors = futures::future::try_join_all(futures)
            .await
            .context("failed to check worker roles for neighbor IPs")?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(neighbors)
    }
}

async fn check_role_worker(ip: IpAddr, worker_port: u16) -> Result<bool, eyre::Error> {
    Ok(reqwest::get(format!("http://{}:{}/role", ip, worker_port))
        .await
        .context("failed to send HTTP request to worker role endpoint")?
        .text()
        .await
        .context("failed to read response body from worker role endpoint")?
        == "worker")
}

async fn get_compartment_id() -> Result<String, eyre::Error> {
    let text = reqwest::get("http://169.254.169.254/opc/v2/instance/")
        .await
        .context("failed to fetch OCI instance metadata")?
        .text()
        .await
        .context("failed to read OCI metadata response body")?;
    let object: sonic_rs::Object =
        sonic_rs::from_str(&text).context("failed to parse OCI metadata JSON")?;
    let compartment_id = object
        .get(&"compartmentId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("compartmentId not found in OCI metadata"))?;
    Ok(compartment_id.to_string())
}
