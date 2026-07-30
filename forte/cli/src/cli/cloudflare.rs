use anyhow::{Result, anyhow};
use std::path::PathBuf;

use super::project_config::read_project_id;

#[allow(clippy::too_many_arguments)]
pub async fn connect(
    project_dir: PathBuf,
    account_id: String,
    zone_id: String,
    api_token: Option<String>,
    zone_name: Option<String>,
    dataplane_access_key_id: Option<String>,
    dataplane_secret: Option<String>,
    purge_token: Option<String>,
) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    match (
        api_token,
        zone_name,
        dataplane_access_key_id,
        dataplane_secret,
        purge_token,
    ) {
        (Some(api_token), ..) => {
            fn0_deploy::cloudflare_connect_managed(&id, &account_id, &zone_id, &api_token).await
        }
        (None, Some(zone_name), Some(key_id), Some(secret), Some(purge)) => {
            fn0_deploy::cloudflare_connect_manual(
                &id,
                &account_id,
                &zone_id,
                &zone_name,
                &key_id,
                &secret,
                &purge,
            )
            .await
        }
        _ => Err(anyhow!(
            "pass --api-token to let the CLI provision and mint, or run `forte cloudflare \
             provision` first and pass --zone-name --dataplane-access-key-id --dataplane-secret \
             --purge-token"
        )),
    }
}

pub async fn provision(
    project_dir: PathBuf,
    account_id: String,
    zone_id: String,
    api_token: String,
) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::cloudflare_provision(&id, &account_id, &zone_id, &api_token).await
}

pub async fn status(project_dir: PathBuf) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::cloudflare_status(&id).await
}
