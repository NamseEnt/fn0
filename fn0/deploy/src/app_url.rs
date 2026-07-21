//! Resolves the public URL a deployed project is served at.
//!
//! A project is always reachable at `{project_id}.{control host}`, because the
//! worker routes on the first host label. A custom domain, when attached and
//! active, is preferred over that default subdomain.

use anyhow::{Result, anyhow};

use crate::domain::{
    CloudflareStatus, DomainStatus, fetch_domain_status, format_cloudflare_status,
};

pub struct ResolvedAppUrl {
    pub url: String,
    pub note: Option<String>,
}

pub fn default_app_url(control_url: &str, project_id: &str) -> Result<String> {
    let mut url = reqwest::Url::parse(control_url)
        .map_err(|e| anyhow!("control URL '{control_url}' is not a valid URL: {e}"))?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("control URL '{control_url}' has no host"))?;
    let app_host = format!("{project_id}.{host}");
    url.set_host(Some(&app_host))
        .map_err(|e| anyhow!("could not build app host '{app_host}': {e}"))?;
    url.set_path("");
    Ok(url.to_string().trim_end_matches('/').to_string())
}

pub async fn resolve_app_url(project_id: &str) -> Result<ResolvedAppUrl> {
    let creds = crate::credentials::load()?;
    let control_url = creds
        .as_ref()
        .map(|c| c.control_url.clone())
        .unwrap_or_else(crate::credentials::default_control_url);
    let default_url = default_app_url(&control_url, project_id)?;

    let Some(creds) = creds else {
        return Ok(ResolvedAppUrl {
            url: default_url,
            note: Some("not signed in, so a custom domain could not be checked".to_string()),
        });
    };

    let fallback = |note: String| ResolvedAppUrl {
        url: default_url.clone(),
        note: Some(note),
    };

    Ok(match fetch_domain_status(&creds, project_id).await {
        Ok(DomainStatus::Configured {
            domain,
            cloudflare_status: CloudflareStatus::Active,
        }) => ResolvedAppUrl {
            url: format!("https://{domain}"),
            note: None,
        },
        Ok(DomainStatus::Configured {
            domain,
            cloudflare_status,
        }) => fallback(format!(
            "custom domain '{domain}' is not serving yet ({}), so the default subdomain is used",
            format_cloudflare_status(&cloudflare_status)
        )),
        Ok(DomainStatus::NotConfigured) => ResolvedAppUrl {
            url: default_url,
            note: None,
        },
        Ok(DomainStatus::NotLoggedIn) => {
            fallback("control rejected the saved token; run `forte login` again".to_string())
        }
        Ok(DomainStatus::NotFound) => fallback(format!(
            "project '{project_id}' was not found under your account; nothing may be deployed there"
        )),
        Ok(DomainStatus::InternalError) => {
            fallback("control failed to report the custom domain".to_string())
        }
        Err(e) => fallback(format!(
            "could not reach control to check a custom domain: {e}"
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_subdomain_under_the_control_host() {
        assert_eq!(
            default_app_url("https://fn0.dev", "abc123").unwrap(),
            "https://abc123.fn0.dev"
        );
    }

    #[test]
    fn keeps_scheme_and_port_of_a_self_hosted_control() {
        assert_eq!(
            default_app_url("http://localhost:3000", "abc123").unwrap(),
            "http://abc123.localhost:3000"
        );
    }

    #[test]
    fn drops_a_trailing_path_on_the_control_url() {
        assert_eq!(
            default_app_url("https://fn0.dev/", "abc123").unwrap(),
            "https://abc123.fn0.dev"
        );
    }
}
