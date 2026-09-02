use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::cloudflare::CloudSetup;

#[derive(Serialize)]
struct DomainSetInput<'a> {
    project_id: &'a str,
    domain: &'a str,
    certificate_pem: &'a str,
    private_key_pem: &'a str,
    not_after_epoch_seconds: i64,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DomainSet {
    Ok {
        origin_hostname: String,
        replaced_domain: Option<String>,
    },
    NotLoggedIn,
    NotFound,
    InvalidDomain {
        message: String,
    },
    DomainTaken {
        existing_project_id: String,
    },
    InternalError,
}

/// What the caller has to tell the user once the domain is registered.
pub struct DomainOutcome {
    /// What the domain's proxied CNAME was pointed at.
    pub origin_hostname: String,
    pub replaced_domain: Option<String>,
}

/// Points a project at a domain, replacing whatever it answered on before.
///
/// The origin certificate is signed here rather than by fn0: only a token with
/// `SSL and Certificates -> Edit` can sign one, and fn0 holds no such token by
/// design.
///
/// The CORS allowlist on the project's buckets is the app's own origin, so it
/// is rewritten every time, even when the domain has not changed. Provisioning
/// already wrote it for the domain a project is set up with, so the first write
/// is redundant — but making it unconditional is what leaves a way to repair an
/// allowlist that drifted, and re-running with an unchanged domain is how a
/// user would expect to do that.
///
/// The DNS record is written last, once fn0 holds the certificate and the zone
/// carries the cache rule: until it exists the hostname resolves nowhere, so no
/// visitor can reach a domain that is only half set up.
pub async fn set_domain(setup: &CloudSetup<'_>) -> Result<DomainOutcome> {
    println!(
        "requesting an origin certificate through the Cloudflare setup broker for {}...",
        setup.domain
    );
    let issued = setup
        .broker
        .issue_origin_certificate(setup.project_id, setup.zone_id, setup.domain)
        .await?;

    let creds = crate::credentials::require()?;
    let url = format!(
        "{}/__forte_action/domain_set",
        creds.control_url.trim_end_matches('/')
    );
    let project_id = setup.project_id;
    let domain = setup.domain;
    let response: DomainSet = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainSetInput {
            project_id,
            domain,
            certificate_pem: &issued.certificate_pem,
            private_key_pem: &issued.private_key_pem,
            not_after_epoch_seconds: issued.not_after_epoch_seconds,
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let (origin_hostname, replaced_domain) = match response {
        DomainSet::Ok {
            origin_hostname,
            replaced_domain,
        } => (origin_hostname, replaced_domain),
        DomainSet::NotLoggedIn => {
            return Err(anyhow!("control rejected token; sign in again."));
        }
        DomainSet::NotFound => {
            return Err(anyhow!(
                "project '{project_id}' not found or not owned by you."
            ));
        }
        DomainSet::InvalidDomain { message } => {
            return Err(anyhow!("invalid domain: {message}"));
        }
        DomainSet::DomainTaken {
            existing_project_id,
        } => {
            return Err(anyhow!(
                "domain '{domain}' is already in use by project '{existing_project_id}'"
            ));
        }
        DomainSet::InternalError => {
            return Err(anyhow!("domain_set: server error; check fn0-control logs"));
        }
    };

    setup
        .broker
        .finalize_domain(
            project_id,
            setup.zone_id,
            setup.zone_name,
            setup.domain,
            &origin_hostname,
            replaced_domain.as_deref(),
        )
        .await?;

    Ok(DomainOutcome {
        origin_hostname,
        replaced_domain,
    })
}

#[derive(Serialize)]
struct DomainProjectInput<'a> {
    project_id: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
pub enum DomainStatus {
    /// The project has no registered domain, so the worker serves nothing for
    /// it.
    NoDomain,
    /// A project on its owner's own Cloudflare account. Their edge holds the
    /// visitor-facing certificate, so there is no fn0-side DV status to report;
    /// what fn0 holds is the origin certificate the worker presents.
    SelfHosted {
        domain: String,
        origin_certificate_ready: bool,
        origin_certificate_expires_epoch_seconds: Option<i64>,
        origin_hostname: String,
    },
    NotLoggedIn,
    NotFound,
    InternalError,
}

pub async fn fetch_domain_status(
    creds: &crate::credentials::Credentials,
    project_id: &str,
) -> Result<DomainStatus> {
    let url = format!(
        "{}/__forte_action/domain_status",
        creds.control_url.trim_end_matches('/')
    );
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DomainProjectInput { project_id })
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await?)
}
