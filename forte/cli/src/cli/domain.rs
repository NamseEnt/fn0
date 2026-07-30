use anyhow::Result;
use std::path::PathBuf;

use super::project_config::read_project_id;

pub async fn add(
    project_dir: PathBuf,
    domain: String,
    account_id: Option<String>,
    zone_id: Option<String>,
    api_token: Option<String>,
) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    // All three or none: a project on its own Cloudflare account needs an
    // origin certificate signed here, and fn0 holds no token that can sign one.
    let certificate = match (&account_id, &zone_id, &api_token) {
        (Some(account_id), Some(zone_id), Some(api_token)) => {
            Some(fn0_deploy::OriginCertificateRequest {
                account_id,
                zone_id,
                api_token,
            })
        }
        (None, None, None) => None,
        _ => {
            return Err(anyhow::anyhow!(
                "--account-id, --zone-id and --api-token go together; pass all three to attach a \
                 domain to a project on your own Cloudflare account, or none for a project on the \
                 fn0 platform account"
            ));
        }
    };
    fn0_deploy::domain_add(&id, &domain, certificate).await
}

pub async fn remove(project_dir: PathBuf) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::domain_remove(&id).await
}

pub async fn status(project_dir: PathBuf) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::domain_status(&id).await
}
