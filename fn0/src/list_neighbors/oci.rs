use super::*;
use oci_rust_sdk::virtual_network::{Lifetime, ListPublicIpsRequest, Scope};
use sonic_rs::JsonValueTrait;
use std::{str::FromStr, sync::Arc};

pub struct OciListNeighbors {
    worker_port: u16,
    oci_client: Arc<dyn oci_rust_sdk::VirtualNetwork>,
}

impl OciListNeighbors {
    pub fn new(worker_port: u16, oci_client: Arc<dyn oci_rust_sdk::VirtualNetwork>) -> Self {
        Self {
            worker_port,
            oci_client,
        }
    }

    async fn list_public_ips(
        &self,
        compartment_id: String,
    ) -> Result<Vec<std::net::IpAddr>, anyhow::Error> {
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
    async fn list_neighbors(&self) -> Result<Vec<std::net::IpAddr>, anyhow::Error> {
        let compartment_id = get_compartment_id().await?;
        let public_ips = self.list_public_ips(compartment_id).await?;

        let worker_port = self.worker_port;
        let futures = public_ips.into_iter().map(move |ip| async move {
            let is_worker = reqwest::get(format!("http://{ip}:{worker_port}/role"))
                .await?
                .text()
                .await?
                == "worker";
            Ok::<Option<IpAddr>, anyhow::Error>(is_worker.then_some(ip))
        });
        let neighbors = futures::future::try_join_all(futures)
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(neighbors)
    }
}

async fn check_role_worker(ip: IpAddr, worker_port: u16) -> Result<bool, anyhow::Error> {
    Ok(reqwest::get(format!("http://{}:{}/role", ip, worker_port))
        .await?
        .text()
        .await?
        == "worker")
}

async fn get_compartment_id() -> Result<String, anyhow::Error> {
    let text = reqwest::get("http://169.254.169.254/opc/v2/instance/")
        .await?
        .text()
        .await?;
    let object: sonic_rs::Object = sonic_rs::from_str(&text)?;
    let compartment_id = object.get(&"compartmentId").unwrap().as_str().unwrap();
    Ok(compartment_id.to_string())
}
