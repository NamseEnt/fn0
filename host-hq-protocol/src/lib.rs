use bytes::Bytes;
use postcard::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum HqToHostDatagram {
    AdvertiseLatestDeploymentId { deployment_id: u64 },
}
#[derive(Clone, Serialize, Deserialize)]
pub enum HqToHostReliable {
    DeploymentUpdates {
        deployment_id: u64,
        code_id_and_versions: Vec<(u64, u64)>,
    },
    GracefulShutdown,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum HostToHq {
    NotifyHostStatus {
        timestamp: u64,
        deployment_id: u64,
        instances: u64,
    },
}

impl HqToHostDatagram {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(self)?.into())
    }
    pub fn from_bytes(bytes: Bytes) -> Result<Self> {
        postcard::from_bytes(&bytes)
    }
}
impl HqToHostReliable {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(self)?.into())
    }
    pub fn from_bytes(bytes: Bytes) -> Result<Self> {
        postcard::from_bytes(&bytes)
    }
}
impl HostToHq {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(self)?.into())
    }
    pub fn from_bytes(bytes: Bytes) -> Result<Self> {
        postcard::from_bytes(&bytes)
    }
}
