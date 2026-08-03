//! Resolves the public URL a deployed project is served at.
//!
//! There is no default subdomain. A project answers on the custom domain its
//! owner registered in their own Cloudflare zone and on nothing else, so until
//! one is attached and its origin certificate is held, the project has no URL
//! to report rather than a fallback one.

use anyhow::Result;

use crate::domain::{DomainStatus, fetch_domain_status};

pub struct ResolvedAppUrl {
    pub url: Option<String>,
    pub note: Option<String>,
}

impl ResolvedAppUrl {
    fn unresolved(note: String) -> Self {
        Self {
            url: None,
            note: Some(note),
        }
    }
}

pub async fn resolve_app_url(project_id: &str) -> Result<ResolvedAppUrl> {
    let Some(creds) = crate::credentials::load()? else {
        return Ok(ResolvedAppUrl::unresolved(
            "not signed in, so the project's domain could not be checked".to_string(),
        ));
    };

    Ok(match fetch_domain_status(&creds, project_id).await {
        // The owner's own edge terminates the visitor connection, so the domain
        // serves as soon as their DNS points at the origin — there is no fn0-side
        // DV state to wait on. The origin certificate is what fn0 must hold.
        Ok(DomainStatus::SelfHosted {
            domain,
            origin_certificate_ready: true,
            ..
        }) => ResolvedAppUrl {
            url: Some(format!("https://{domain}")),
            note: None,
        },
        Ok(DomainStatus::SelfHosted { domain, .. }) => ResolvedAppUrl::unresolved(format!(
            "'{domain}' has no origin certificate yet; run `forte cloud init`"
        )),
        Ok(DomainStatus::NotConfigured) => {
            ResolvedAppUrl::unresolved("no domain is attached; run `forte cloud init`".to_string())
        }
        Ok(DomainStatus::NotLoggedIn) => ResolvedAppUrl::unresolved(
            "control rejected the saved token; run `forte login` again".to_string(),
        ),
        Ok(DomainStatus::NotFound) => ResolvedAppUrl::unresolved(format!(
            "project '{project_id}' was not found under your account; nothing may be deployed there"
        )),
        Ok(DomainStatus::InternalError) => {
            ResolvedAppUrl::unresolved("control failed to report the domain".to_string())
        }
        Err(e) => {
            ResolvedAppUrl::unresolved(format!("could not reach control to check the domain: {e}"))
        }
    })
}
