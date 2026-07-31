use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::cloudflare_provision::Provisioner;

#[derive(Serialize)]
struct DomainAddInput<'a> {
    project_id: &'a str,
    domain: &'a str,
    certificate_pem: Option<&'a str>,
    private_key_pem: Option<&'a str>,
    not_after_epoch_seconds: Option<i64>,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DomainAdd {
    Ok {
        origin_ip: String,
        needs_dns_record: bool,
    },
    NotLoggedIn,
    NotFound,
    InvalidDomain {
        message: String,
    },
    CertificateFailed {
        message: String,
    },
    DomainTaken {
        existing_project_id: String,
    },
    AlreadyHasDomain {
        current_domain: String,
    },
    InternalError,
}

/// Attaching a domain to a project on the owner's own Cloudflare account needs
/// an origin certificate, and only a token with `SSL and Certificates -> Edit`
/// can sign one. fn0 does not hold such a token by design, so the CLI signs the
/// certificate here and uploads it. `account_id`/`zone_id`/`api_token` are the
/// same ones used for `cloudflare connect`.
pub struct OriginCertificateRequest<'a> {
    pub account_id: &'a str,
    pub zone_id: &'a str,
    pub api_token: &'a str,
    /// `true` when the token can only create tokens, so a signing token has to
    /// be minted from it; `false` when it can sign directly.
    pub mint_signing_token: bool,
}

pub async fn domain_add(
    project_id: &str,
    domain: &str,
    certificate: Option<OriginCertificateRequest<'_>>,
) -> Result<()> {
    let creds = crate::credentials::require()?;

    let issued = match certificate {
        Some(request) => {
            println!("signing an origin certificate for {domain} (this runs locally)...");
            Some(
                Provisioner::new(
                    request.api_token.to_string(),
                    request.account_id.to_string(),
                    request.zone_id.to_string(),
                )
                .issue_origin_certificate(domain, request.mint_signing_token)
                .await?,
            )
        }
        None => None,
    };

    let client = reqwest::Client::new();
    let url = format!(
        "{}/__forte_action/domain_add",
        creds.control_url.trim_end_matches('/')
    );
    let resp = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainAddInput {
            project_id,
            domain,
            certificate_pem: issued.as_ref().map(|i| i.certificate_pem.as_str()),
            private_key_pem: issued.as_ref().map(|i| i.private_key_pem.as_str()),
            not_after_epoch_seconds: issued.as_ref().map(|i| i.not_after_epoch_seconds),
        })
        .send()
        .await?
        .error_for_status()?;
    let raw: DomainAdd = resp.json().await?;
    match raw {
        DomainAdd::Ok {
            origin_ip,
            needs_dns_record,
        } => {
            println!("domain '{domain}' attached to project '{project_id}'");
            if needs_dns_record {
                println!();
                println!("add this record in your Cloudflare dashboard, then you are done:");
                println!("    A   {domain}   {origin_ip}   (proxied / orange cloud)");
                println!();
                println!(
                    "it must stay proxied: the origin certificate is trusted by Cloudflare's \
                     edge only, so turning the record grey breaks the hostname."
                );
            } else {
                println!(
                    "Cloudflare hostname registration is queued; run `forte domain status` to check."
                );
            }
            Ok(())
        }
        DomainAdd::CertificateFailed { message } => Err(anyhow!("{message}")),
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
    /// A project on its owner's own Cloudflare account. Their edge holds the
    /// visitor-facing certificate, so there is no fn0-side DV status to report;
    /// what fn0 holds is the origin certificate the worker presents.
    SelfHosted {
        domain: String,
        origin_certificate_ready: bool,
        origin_certificate_expires_epoch_seconds: Option<i64>,
        origin_ip: String,
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
        DomainStatus::SelfHosted {
            domain,
            origin_certificate_ready,
            origin_certificate_expires_epoch_seconds,
            origin_ip,
        } => {
            println!("project '{project_id}' custom domain: {domain}");
            println!("served from your own Cloudflare account.");
            if origin_certificate_ready {
                match origin_certificate_expires_epoch_seconds
                    .and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0))
                {
                    Some(expires) => println!(
                        "origin certificate: held by fn0, expires {}",
                        expires.format("%Y-%m-%d")
                    ),
                    None => println!("origin certificate: held by fn0"),
                }
            } else {
                println!(
                    "origin certificate: missing — run `forte domain add {domain}` to issue one."
                );
            }
            if !origin_ip.is_empty() {
                println!("point a proxied A record at {origin_ip}.");
            }
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
