use anyhow::{Result, anyhow};
use std::path::PathBuf;

use super::project_config::{read_cloud_config, write_cloud_config};
use fn0_deploy::cloudflare_provision::{ProvisionedResources, ReachableZone, ZoneDiscovery};
use fn0_deploy::{
    CloudSetup, CloudflareConnection, DomainStatus, fetch_cloudflare_connection,
    provision_and_connect, set_domain,
};

pub async fn init(
    project_dir: PathBuf,
    requested_project_name: Option<String>,
    requested_zone: Option<String>,
) -> Result<()> {
    let api_token = read_cloudflare_token()?;
    let config = read_cloud_config(&project_dir)?;
    let has_project_id = config.project_id.is_some();
    let project_name = resolve_setting(
        requested_project_name,
        config.project_name,
        "--project-name",
        has_project_id,
    )?;
    validate_project_name(&project_name)?;
    let zone_name = resolve_setting(requested_zone, config.zone, "--zone", has_project_id)?;
    validate_zone_name(&zone_name)?;
    let domain = derive_domain(&project_name, &zone_name)?;
    if has_project_id
        && let Some(stored_domain) = config.domain.as_deref()
        && stored_domain != domain
    {
        return Err(anyhow!(
            "Forte.toml domain '{stored_domain}' does not match the derived domain '{domain}'."
        ));
    }

    let creds = fn0_deploy::credentials::load()?.ok_or_else(|| {
        anyhow!(
            "not signed in. Run `forte login` first (credentials at {}).",
            fn0_deploy::credentials::path()
                .map(|path| path.display().to_string())
                .unwrap_or_default()
        )
    })?;

    println!("reading the zones this token can reach...");
    let zones = ZoneDiscovery::new(api_token.clone()).list().await?;
    let zone = resolve_zone(zones, &zone_name)?;

    let mut project_id = config.project_id;
    let project_id = fn0_deploy::ensure_project_id(
        &reqwest::Client::new(),
        &creds.control_url,
        &creds.token,
        &project_name,
        &mut project_id,
    )
    .await?;
    write_cloud_config(
        &project_dir,
        &project_id,
        &project_name,
        &zone_name,
        &domain,
    )?;

    let connection = if has_project_id {
        fetch_cloudflare_connection(&project_id).await?
    } else {
        CloudflareConnection::NotConnected
    };
    validate_existing_connection(&creds, &project_id, &connection, &zone, &domain).await?;

    let setup = CloudSetup {
        project_id: &project_id,
        account_id: &zone.account_id,
        zone_id: &zone.zone_id,
        api_token: &api_token,
        domain: &domain,
    };

    println!("  enabling Cloudflare WebSockets...");
    setup.ensure_websockets().await?;
    println!("  Cloudflare WebSockets enabled");

    match connection {
        CloudflareConnection::NotConnected => {
            println!("  provisioning your Cloudflare account (this runs locally)...");
            let resources = provision_and_connect(&setup).await?;
            print_resources(&resources);
            println!("  minted two bucket-scoped R2 tokens and a purge-only token");
            println!("  connected");
        }
        CloudflareConnection::Connected { .. } => {
            println!("  Cloudflare account already connected");
        }
        CloudflareConnection::NotFound => {
            return Err(anyhow!(
                "project '{project_id}' not found or not owned by you."
            ));
        }
    }

    let outcome = set_domain(&setup).await?;
    match &outcome.replaced_domain {
        Some(replaced) => println!("  domain {domain} registered, replacing {replaced}"),
        None => println!("  domain {domain} registered"),
    }

    println!(
        "  CNAME {domain} -> {} written to your zone (proxied / orange cloud)",
        outcome.origin_hostname
    );
    println!();
    println!("It must stay proxied: the origin certificate is trusted by Cloudflare's");
    println!("edge only, so turning the record grey breaks the hostname.");
    println!();
    println!("Next: forte deploy");
    Ok(())
}

fn read_cloudflare_token() -> Result<String> {
    let token = std::env::var("CLOUDFLARE_API_TOKEN").map_err(|_| {
        anyhow!("CLOUDFLARE_API_TOKEN is required. Set it before running `forte cloud init`.")
    })?;
    let trimmed_token = token.trim();
    if trimmed_token.is_empty() {
        return Err(anyhow!(
            "CLOUDFLARE_API_TOKEN cannot be empty. Set it before running `forte cloud init`."
        ));
    }
    Ok(trimmed_token.to_string())
}

fn resolve_setting(
    requested_value: Option<String>,
    stored_value: Option<String>,
    option_name: &str,
    has_project_id: bool,
) -> Result<String> {
    match (requested_value, stored_value) {
        (Some(requested_value), Some(stored_value)) => {
            if requested_value != stored_value {
                return Err(anyhow!(
                    "Forte.toml value '{stored_value}' does not match {option_name} '{requested_value}'."
                ));
            }
            Ok(stored_value)
        }
        (Some(requested_value), None) => Ok(requested_value),
        (None, Some(stored_value)) => Ok(stored_value),
        (None, None) if has_project_id => Err(anyhow!(
            "Forte.toml is missing {option_name}. Pass {option_name} to continue without interactive input."
        )),
        (None, None) => Err(anyhow!("{option_name} is required for a new project.")),
    }
}

fn validate_project_name(project_name: &str) -> Result<()> {
    if project_name.is_empty() || project_name.len() > 63 {
        return Err(anyhow!(
            "project name '{project_name}' must be 1 to 63 ASCII characters"
        ));
    }
    let Some(first_character) = project_name.bytes().next() else {
        return Err(anyhow!(
            "project name '{project_name}' must start with a letter or digit"
        ));
    };
    let Some(last_character) = project_name.bytes().last() else {
        return Err(anyhow!(
            "project name '{project_name}' must end with a letter or digit"
        ));
    };
    if !is_dns_label_edge_character(first_character) || !is_dns_label_edge_character(last_character)
    {
        return Err(anyhow!(
            "project name '{project_name}' must start and end with a lowercase letter or digit"
        ));
    }
    if project_name
        .bytes()
        .any(|character| !is_dns_label_character(character))
    {
        return Err(anyhow!(
            "project name '{project_name}' must contain only lowercase letters, digits, and hyphens"
        ));
    }
    Ok(())
}

fn is_dns_label_character(character: u8) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'-'
}

fn is_dns_label_edge_character(character: u8) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit()
}

fn validate_zone_name(zone_name: &str) -> Result<()> {
    if zone_name.is_empty() || zone_name.chars().any(|character| character.is_whitespace()) {
        return Err(anyhow!("zone name must be a non-empty hostname"));
    }
    Ok(())
}

fn derive_domain(project_name: &str, zone_name: &str) -> Result<String> {
    let domain = format!("{project_name}.{zone_name}");
    if domain.len() > 253 {
        return Err(anyhow!(
            "derived domain '{domain}' exceeds the 253-character DNS hostname limit"
        ));
    }
    Ok(domain)
}

fn resolve_zone(zones: Vec<ReachableZone>, zone_name: &str) -> Result<ReachableZone> {
    let matching_zones: Vec<ReachableZone> = zones
        .into_iter()
        .filter(|zone| zone.zone_name == zone_name)
        .collect();
    match matching_zones.len() {
        0 => Err(anyhow!(
            "Cloudflare zone '{zone_name}' was not found or is not accessible to CLOUDFLARE_API_TOKEN."
        )),
        1 => matching_zones
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Cloudflare zone '{zone_name}' could not be resolved")),
        _ => {
            let account_names: Vec<&str> = matching_zones
                .iter()
                .map(|zone| zone.account_name.as_str())
                .collect();
            Err(anyhow!(
                "Cloudflare zone '{zone_name}' exists in multiple accounts: {}",
                account_names.join(", ")
            ))
        }
    }
}

async fn validate_existing_connection(
    creds: &fn0_deploy::credentials::Credentials,
    project_id: &str,
    connection: &CloudflareConnection,
    zone: &ReachableZone,
    domain: &str,
) -> Result<()> {
    let CloudflareConnection::Connected { zone_name } = connection else {
        if let CloudflareConnection::NotFound = connection {
            return Err(anyhow!(
                "project '{project_id}' not found or not owned by you."
            ));
        }
        return Ok(());
    };

    if zone_name != &zone.zone_name {
        return Err(anyhow!(
            "project '{project_id}' is connected to zone '{zone_name}', not requested zone '{}'. Reconnecting is not supported.",
            zone.zone_name
        ));
    }

    match fn0_deploy::fetch_domain_status(creds, project_id).await? {
        DomainStatus::SelfHosted {
            domain: live_domain,
            ..
        } if live_domain != domain => Err(anyhow!(
            "project '{project_id}' is configured for domain '{live_domain}', not '{domain}'. Reconfiguration is not supported by `forte cloud init`."
        )),
        DomainStatus::NotLoggedIn => Err(anyhow!("control rejected token; sign in again.")),
        DomainStatus::NotFound => Err(anyhow!(
            "project '{project_id}' not found or not owned by you."
        )),
        DomainStatus::InternalError => Err(anyhow!(
            "domain_status: server error; check fn0-control logs"
        )),
        _ => Ok(()),
    }
}

fn print_resources(resources: &ProvisionedResources) {
    println!("  zone     {}", resources.zone_name);
    println!("  buckets  {}", resources.private_object_storage_bucket);
    println!("           {}", resources.public_object_storage_bucket);
    println!("           {}", resources.frontend_asset_bucket);
    println!("  assets   https://{}", resources.frontend_asset_hostname);
    println!(
        "  public   https://{}",
        resources.public_object_storage_hostname
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_dns_label_project_names() {
        assert!(validate_project_name("my-app").is_ok());
        assert!(validate_project_name("a1").is_ok());
        let maximum_length_name = "a".repeat(63);
        assert!(validate_project_name(&maximum_length_name).is_ok());
    }

    #[test]
    fn rejects_invalid_dns_label_project_names() {
        for project_name in ["", "-my-app", "my-app-", "My-app", "my_app", "my.app"] {
            assert!(validate_project_name(project_name).is_err());
        }
        let too_long_name = "a".repeat(64);
        assert!(validate_project_name(&too_long_name).is_err());
    }

    #[test]
    fn derives_domain_from_project_name_and_zone() {
        assert_eq!(
            derive_domain("my-app", "example.com").unwrap(),
            "my-app.example.com"
        );
    }

    #[test]
    fn resolves_only_the_exact_zone_name() {
        let resolved_zone = resolve_zone(
            vec![ReachableZone {
                zone_id: "zone-id".to_string(),
                zone_name: "example.com".to_string(),
                account_id: "account-id".to_string(),
                account_name: "account".to_string(),
            }],
            "example.com",
        )
        .unwrap();
        assert_eq!(resolved_zone.zone_id, "zone-id");
        assert!(resolve_zone(Vec::new(), "other.example.com").is_err());
    }
}
