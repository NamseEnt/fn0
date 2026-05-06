use crate::common::cloudflare_saas::CloudflareSaasClient;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub domain: String,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    let client = CloudflareSaasClient::from_env()?;
    client.create_or_adopt_hostname(&input.domain).await?;
    Ok(())
}
