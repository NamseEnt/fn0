use std::net::IpAddr;

pub mod oci;

pub trait ListNeighbors {
    async fn list_neighbors(&self) -> Result<Vec<IpAddr>, anyhow::Error>;
}
