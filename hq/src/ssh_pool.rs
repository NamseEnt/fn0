use color_eyre::eyre::{Result, eyre};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::ssh::SshClient;

const EXEC_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Clone)]
pub struct SshPool {
    inner: Arc<Inner>,
}

struct Inner {
    user: String,
    private_key_pem: String,
    conns: DashMap<String, Arc<SshClient>>,
}

impl SshPool {
    pub fn new(user: String, private_key_pem: String) -> Self {
        Self {
            inner: Arc::new(Inner {
                user,
                private_key_pem,
                conns: DashMap::new(),
            }),
        }
    }

    pub async fn exec(&self, addr: &str, command: &str) -> Result<(i32, String)> {
        let client = self.get_or_connect(addr).await?;
        let exec_fut = client.exec(command);
        match timeout(EXEC_TIMEOUT, exec_fut).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(err)) => {
                self.invalidate(addr);
                Err(err)
            }
            Err(_) => {
                self.invalidate(addr);
                Err(eyre!("SSH exec timed out after {:?} on {}", EXEC_TIMEOUT, addr))
            }
        }
    }

    pub fn invalidate(&self, addr: &str) {
        self.inner.conns.remove(addr);
    }

    async fn get_or_connect(&self, addr: &str) -> Result<Arc<SshClient>> {
        if let Some(client) = self.inner.conns.get(addr) {
            return Ok(client.clone());
        }
        let client =
            SshClient::connect(addr, &self.inner.user, &self.inner.private_key_pem).await?;
        let client = Arc::new(client);
        self.inner
            .conns
            .insert(addr.to_string(), client.clone());
        Ok(client)
    }
}
