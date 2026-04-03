use super::*;
use base64::Engine;
use crate::args::DedicatedHostProviderArgs;
use doc_db::DocDb;
use russh::client;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct DedicatedHostProvider {
    doc_db: DocDb,
    ssh_user: String,
    ssh_private_key_pem: String,
    worker_image: String,
    envs: BTreeMap<String, String>,
}

impl DedicatedHostProvider {
    pub fn new(args: DedicatedHostProviderArgs, doc_db: DocDb) -> Self {
        let ssh_private_key_pem = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(&args.ssh_private_key_base64)
                .expect("Failed to decode ssh_private_key_base64"),
        )
        .expect("SSH private key is not valid UTF-8");

        Self {
            doc_db,
            ssh_user: args.ssh_user,
            ssh_private_key_pem,
            worker_image: args.worker_image,
            envs: args.envs,
        }
    }

    fn build_docker_run_command(&self) -> String {
        let env_flags: String = self
            .envs
            .iter()
            .map(|(k, v)| format!("-e {}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");

        format!(
            "docker pull {image} && \
             docker stop fn0-worker 2>/dev/null; \
             docker rm fn0-worker 2>/dev/null; \
             docker run -d --name fn0-worker --network=host --restart=unless-stopped {env_flags} {image}",
            image = self.worker_image,
        )
    }

    async fn ssh_exec(&self, addr: &str, port: u16, command: &str) -> color_eyre::Result<()> {
        let key_pair = russh_keys::decode_secret_key(&self.ssh_private_key_pem, None)?;
        let config = Arc::new(client::Config::default());
        let handler = SshHandler;

        let mut session =
            client::connect(config, (addr, port), handler).await?;

        if !session
            .authenticate_publickey(&self.ssh_user, Arc::new(key_pair))
            .await?
        {
            return Err(color_eyre::eyre::eyre!(
                "SSH public key authentication failed for {}@{}:{}",
                self.ssh_user,
                addr,
                port
            ));
        }

        let mut channel = session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut exit_status = None;
        while let Some(msg) = channel.wait().await {
            match msg {
                russh::ChannelMsg::ExitStatus { exit_status: code } => {
                    exit_status = Some(code);
                }
                _ => {}
            }
        }

        session.disconnect(russh::Disconnect::ByApplication, "", "").await?;

        match exit_status {
            Some(0) => Ok(()),
            Some(code) => Err(color_eyre::eyre::eyre!(
                "SSH command exited with status {}",
                code
            )),
            None => Err(color_eyre::eyre::eyre!("SSH command did not return exit status")),
        }
    }
}

struct SshHandler;

#[async_trait::async_trait]
impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl HostProvide for DedicatedHostProvider {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>> {
        let records = self.doc_db.list_dedicated_hosts().await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to list dedicated hosts: {}", e))?;

        Ok(records
            .into_iter()
            .map(|(id, record)| {
                let transport = match record.transport {
                    doc_db::DwsTransport::Quic => HostTransport::Quic,
                    doc_db::DwsTransport::WebSocket => HostTransport::WebSocket,
                };
                Host {
                    id: HostId::new(id),
                    addr: record.addr,
                    port: record.port,
                    transport,
                }
            })
            .collect())
    }

    async fn terminate(&self, host_id: &HostId) -> color_eyre::Result<()> {
        let records = self.doc_db.list_dedicated_hosts().await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to list dedicated hosts: {}", e))?;

        let record = records
            .iter()
            .find(|(id, _)| id == host_id.as_ref())
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Dedicated host {} not found in doc-db", host_id)
            })?;

        let command = self.build_docker_run_command();
        self.ssh_exec(&record.1.addr, 22, &command).await
    }

    async fn scale_to(&self, _n: usize) -> color_eyre::Result<()> {
        Ok(())
    }
}
