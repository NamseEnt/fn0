use bytes::Bytes;
use postcard::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum HqToHostDatagram {
    AdvertiseLatestDeploymentId { deployment_id: u64 },
}
#[derive(Clone, Serialize, Deserialize)]
pub enum CodeDeployment {
    Deploy {
        subdomain: String,
        code_id: u64,
        code_version: u64,
    },
    Undeploy {
        subdomain: String,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum HqToHostReliable {
    DeploymentUpdates {
        deployment_id: u64,
        codes: Vec<CodeDeployment>,
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

#[derive(Clone, Serialize, Deserialize)]
pub enum WsHqToHost {
    Datagram(HqToHostDatagram),
    Reliable(HqToHostReliable),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WsHostToHq {
    Datagram(HostToHq),
}

impl WsHqToHost {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(self)?.into())
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        postcard::from_bytes(bytes)
    }
}

impl WsHostToHq {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(postcard::to_stdvec(self)?.into())
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        postcard::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_deployment_deploy_roundtrip() {
        let msg = CodeDeployment::Deploy {
            subdomain: "test-app".to_string(),
            code_id: 42,
            code_version: 3,
        };
        let bytes = postcard::to_stdvec(&msg).unwrap();
        let decoded: CodeDeployment = postcard::from_bytes(&bytes).unwrap();
        match decoded {
            CodeDeployment::Deploy {
                subdomain,
                code_id,
                code_version,
            } => {
                assert_eq!(subdomain, "test-app");
                assert_eq!(code_id, 42);
                assert_eq!(code_version, 3);
            }
            _ => panic!("expected Deploy"),
        }
    }

    #[test]
    fn code_deployment_undeploy_roundtrip() {
        let msg = CodeDeployment::Undeploy {
            subdomain: "test-app".to_string(),
        };
        let bytes = postcard::to_stdvec(&msg).unwrap();
        let decoded: CodeDeployment = postcard::from_bytes(&bytes).unwrap();
        match decoded {
            CodeDeployment::Undeploy { subdomain } => {
                assert_eq!(subdomain, "test-app");
            }
            _ => panic!("expected Undeploy"),
        }
    }

    #[test]
    fn reliable_message_with_mixed_deployments() {
        let msg = HqToHostReliable::DeploymentUpdates {
            deployment_id: 5,
            codes: vec![
                CodeDeployment::Deploy {
                    subdomain: "a".to_string(),
                    code_id: 1,
                    code_version: 1,
                },
                CodeDeployment::Undeploy {
                    subdomain: "b".to_string(),
                },
            ],
        };
        let bytes = msg.to_bytes().unwrap();
        let decoded = HqToHostReliable::from_bytes(bytes).unwrap();
        match decoded {
            HqToHostReliable::DeploymentUpdates {
                deployment_id,
                codes,
            } => {
                assert_eq!(deployment_id, 5);
                assert_eq!(codes.len(), 2);
                assert!(matches!(&codes[0], CodeDeployment::Deploy { subdomain, .. } if subdomain == "a"));
                assert!(matches!(&codes[1], CodeDeployment::Undeploy { subdomain } if subdomain == "b"));
            }
            _ => panic!("expected DeploymentUpdates"),
        }
    }
}
