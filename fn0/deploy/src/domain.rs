use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct DomainAddInput<'a> {
    project_id: &'a str,
    domain: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DomainAdd {
    Ok,
    NotLoggedIn,
    NotFound,
    InvalidDomain { message: String },
    DomainTaken { existing_project_id: String },
    AlreadyHasDomain { current_domain: String },
    InternalError,
}

pub async fn domain_add(project_id: &str, domain: &str) -> Result<()> {
    let creds = crate::credentials::require()?;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/domain_add",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainAddInput { project_id, domain })
        .send()
        .await?
        .error_for_status()?;
    let raw: DomainAdd = resp.json().await?;
    match raw {
        DomainAdd::Ok => {
            println!("domain '{domain}' attached to project '{project_id}'");
            println!(
                "Cloudflare hostname registration is queued; run `fn0 domain status` to check."
            );
            Ok(())
        }
        DomainAdd::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DomainAdd::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        DomainAdd::InvalidDomain { message } => Err(anyhow!("invalid domain: {message}")),
        DomainAdd::DomainTaken {
            existing_project_id,
        } => Err(anyhow!(
            "domain '{domain}' already in use by project '{existing_project_id}'"
        )),
        DomainAdd::AlreadyHasDomain { current_domain } => Err(anyhow!(
            "project '{project_id}' already has domain '{current_domain}'; remove it first"
        )),
        DomainAdd::InternalError => {
            Err(anyhow!("domain_add: server error; check fn0-control logs"))
        }
    }
}

#[derive(Serialize)]
struct DomainProjectInput<'a> {
    project_id: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DomainRemove {
    Ok { removed_domain: String },
    NotLoggedIn,
    NotFound,
    NoDomain,
    InternalError,
}

pub async fn domain_remove(project_id: &str) -> Result<()> {
    let creds = crate::credentials::require()?;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/domain_remove",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainProjectInput { project_id })
        .send()
        .await?
        .error_for_status()?;
    let raw: DomainRemove = resp.json().await?;
    match raw {
        DomainRemove::Ok { removed_domain } => {
            println!("domain '{removed_domain}' detached from project '{project_id}'");
            println!("Cloudflare hostname removal is queued.");
            Ok(())
        }
        DomainRemove::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DomainRemove::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        DomainRemove::NoDomain => Err(anyhow!(
            "no custom domain attached to project '{project_id}'."
        )),
        DomainRemove::InternalError => Err(anyhow!(
            "domain_remove: server error; check fn0-control logs"
        )),
    }
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
pub enum DomainStatus {
    NotConfigured,
    Configured {
        domain: String,
        cloudflare_status: CloudflareStatus,
    },
    NotLoggedIn,
    NotFound,
    InternalError,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
pub enum CloudflareStatus {
    Active,
    Pending,
    Missing,
    Other { value: String },
}

pub async fn fetch_domain_status(
    creds: &crate::credentials::Credentials,
    project_id: &str,
) -> Result<DomainStatus> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/domain_status",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainProjectInput { project_id })
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await?)
}

pub async fn domain_status(project_id: &str) -> Result<()> {
    let creds = crate::credentials::require()?;
    match fetch_domain_status(&creds, project_id).await? {
        DomainStatus::NotConfigured => {
            println!("project '{project_id}' has no custom domain configured.");
            Ok(())
        }
        DomainStatus::Configured {
            domain,
            cloudflare_status,
        } => {
            println!("project '{project_id}' custom domain: {domain}");
            println!(
                "cloudflare status: {}",
                format_cloudflare_status(&cloudflare_status)
            );
            Ok(())
        }
        DomainStatus::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DomainStatus::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        DomainStatus::InternalError => Err(anyhow!(
            "domain_status: server error; check fn0-control logs"
        )),
    }
}

pub(crate) fn format_cloudflare_status(status: &CloudflareStatus) -> String {
    match status {
        CloudflareStatus::Active => "active".to_string(),
        CloudflareStatus::Pending => "pending (waiting for DV verification)".to_string(),
        CloudflareStatus::Missing => {
            "missing on Cloudflare (registration may still be in progress)".to_string()
        }
        CloudflareStatus::Other { value } => format!("other: {value}"),
    }
}
