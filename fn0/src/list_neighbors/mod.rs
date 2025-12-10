use std::net::IpAddr;
use color_eyre::eyre;

pub mod oci;

pub trait ListNeighbors {
    async fn list_neighbors(&self) -> Result<Vec<IpAddr>, eyre::Error>;
}
