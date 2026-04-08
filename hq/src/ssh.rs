use color_eyre::eyre::{Result, eyre};
use russh::*;
use std::sync::Arc;

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &keys::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct SshClient {
    session: client::Handle<ClientHandler>,
}

impl SshClient {
    pub async fn connect(addr: &str, user: &str, private_key_pem: &str) -> Result<Self> {
        let key_pair = keys::decode_secret_key(private_key_pem, None)
            .map_err(|e| eyre!("Failed to decode SSH private key: {e}"))?;

        let config = client::Config::default();
        let mut session =
            client::connect(Arc::new(config), (addr, 22), ClientHandler)
                .await
                .map_err(|e| eyre!("SSH connect to {addr} failed: {e}"))?;

        let key_with_hash = keys::PrivateKeyWithHashAlg::new(
            Arc::new(key_pair),
            None,
        );

        let auth_result = session
            .authenticate_publickey(user, key_with_hash)
            .await
            .map_err(|e| eyre!("SSH auth failed: {e}"))?;

        match auth_result {
            client::AuthResult::Success => {}
            _ => return Err(eyre!("SSH authentication rejected by {addr}")),
        }

        Ok(Self { session })
    }

    pub async fn exec(&self, command: &str) -> Result<(i32, String)> {
        let mut channel = self
            .session
            .channel_open_session()
            .await
            .map_err(|e| eyre!("Failed to open SSH channel: {e}"))?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| eyre!("Failed to exec command: {e}"))?;

        let mut output = String::new();
        let mut exit_status = 0i32;

        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { data }) => {
                    output.push_str(&String::from_utf8_lossy(&data));
                }
                Some(ChannelMsg::ExtendedData { data, ext }) => {
                    if ext == 1 {
                        output.push_str(&String::from_utf8_lossy(&data));
                    }
                }
                Some(ChannelMsg::ExitStatus { exit_status: code }) => {
                    exit_status = code as i32;
                }
                Some(ChannelMsg::Eof) | None => break,
                _ => {}
            }
        }

        Ok((exit_status, output))
    }

    pub async fn close(self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await
            .map_err(|e| eyre!("SSH disconnect failed: {e}"))?;
        Ok(())
    }
}
