//! `forte cloud init` — gives a project an identity, a Cloudflare account to
//! live in, and the domain it answers on.
//!
//! It is interactive on purpose. The inputs are a Cloudflare API token and a
//! choice between two trust models, and neither belongs on a command line: a
//! token passed as an argument lands in shell history and in `ps`, and the
//! choice is one a user should be shown rather than expected to know.
//!
//! It is also the only command that can change a project's domain, because
//! doing so means signing a new origin certificate and that needs the same
//! token. Re-running it on a project that is already set up asks for the token
//! and the new domain and skips everything already done, so `forte deploy`
//! never has to prompt for anything.

use anyhow::{Result, anyhow};
use inquire::{Confirm, Password, PasswordDisplayMode, Select, Text};
use std::path::PathBuf;

use super::project_config::{read_optional_project_id, write_cloud_config};
use fn0_deploy::cloudflare_provision::{
    ConnectCredentials, ProvisionedResources, ReachableZone, ZoneDiscovery,
};
use fn0_deploy::{
    CloudSetup, CloudflareConnection, connect_with_own_credentials, expected_resources,
    fetch_cloudflare_connection, provision_and_connect, provision_only, set_domain,
};

/// How the user wants to hand the setup the permissions it needs.
#[derive(Clone, Copy, PartialEq)]
enum SetupStyle {
    /// One token carrying `API Tokens -> Edit`. Everything else is minted from
    /// it and revoked on the way out.
    Managed,
    /// A token that can provision but cannot create tokens. The user makes the
    /// three long-lived credentials by hand.
    SelfMade,
}

impl SetupStyle {
    fn mints_its_own_tokens(self) -> bool {
        self == Self::Managed
    }
}

pub async fn init(project_dir: PathBuf) -> Result<()> {
    let creds = fn0_deploy::credentials::load()?.ok_or_else(|| {
        anyhow!(
            "not signed in. Run `forte login` first (credentials at {}).",
            fn0_deploy::credentials::path()
                .map(|path| path.display().to_string())
                .unwrap_or_default()
        )
    })?;

    let registered = read_optional_project_id(&project_dir)?;
    let connection = match &registered {
        Some(project_id) => fetch_cloudflare_connection(project_id).await?,
        None => CloudflareConnection::NotConnected,
    };
    if let CloudflareConnection::NotFound = connection {
        return Err(anyhow!(
            "Forte.toml names project '{}', which does not exist under your account.",
            registered.unwrap_or_default()
        ));
    }
    let connected_zone_name = match &connection {
        CloudflareConnection::Connected { zone_name } => Some(zone_name.clone()),
        _ => None,
    };

    println!();
    match &connected_zone_name {
        Some(zone_name) => {
            println!("This project is already connected to {zone_name}.");
            println!("Continuing will change the domain it answers on.");
        }
        None => {
            println!("fn0 runs this project's storage, CDN and domain on your own");
            println!("Cloudflare account. Nothing is stored on fn0's.");
        }
    }
    println!();

    let style = ask_setup_style()?;
    let api_token = Password::new("Cloudflare API token")
        .with_display_mode(PasswordDisplayMode::Masked)
        .without_confirmation()
        .with_help_message("not saved anywhere, and never sent to fn0")
        .prompt()?;

    println!("reading the zones this token can reach...");
    let zones = ZoneDiscovery::new(api_token.clone())
        .list(style.mints_its_own_tokens())
        .await?;
    let zone = pick_zone(zones, connected_zone_name.as_deref())?;
    let domain = ask_domain(&zone)?;

    let project_id = match registered {
        Some(project_id) => project_id,
        None => {
            let project_name = ask_project_name()?;
            let project_id = fn0_deploy::ensure_project_id(
                &reqwest::Client::new(),
                &creds.control_url,
                &creds.token,
                &project_name,
                &mut None,
            )
            .await?;
            println!();
            println!("  created project {project_name} ({project_id})");
            project_id
        }
    };
    write_cloud_config(&project_dir, &project_id, &domain)?;

    let setup = CloudSetup {
        project_id: &project_id,
        account_id: &zone.account_id,
        zone_id: &zone.zone_id,
        api_token: &api_token,
        mint_from_setup_token: style.mints_its_own_tokens(),
        domain: &domain,
    };

    if connected_zone_name.is_none() {
        println!("  provisioning your Cloudflare account (this runs locally)...");
        match style {
            SetupStyle::Managed => {
                let resources = provision_and_connect(&setup).await?;
                print_resources(&resources);
                println!("  minted two bucket-scoped R2 tokens and a purge-only token");
            }
            SetupStyle::SelfMade => {
                let resources = provision_only(&setup).await?;
                print_resources(&resources);
                let credentials = ask_own_credentials(&setup, &zone.zone_name)?;
                connect_with_own_credentials(&setup, &resources, &credentials).await?;
            }
        }
        println!("  connected");
    }

    let outcome = set_domain(&setup).await?;
    match &outcome.replaced_domain {
        Some(replaced) => println!("  domain {domain} registered, replacing {replaced}"),
        None => println!("  domain {domain} registered"),
    }

    println!();
    println!("One thing left. Add this record in your Cloudflare dashboard:");
    println!();
    println!(
        "    CNAME   {domain}   {}   (proxied / orange cloud)",
        outcome.origin_hostname
    );
    println!();
    println!("It must stay proxied: the origin certificate is trusted by Cloudflare's");
    println!("edge only, so turning the record grey breaks the hostname.");
    println!();
    println!("Then: forte deploy");
    if style.mints_its_own_tokens() {
        println!();
        println!("The token you just used was not sent to fn0 and is no longer needed —");
        println!("you can delete it in the Cloudflare dashboard.");
    }
    Ok(())
}

fn ask_setup_style() -> Result<SetupStyle> {
    const MANAGED: &str = "Let fn0 set it up  -  one token, but it can create tokens, so it is account-wide until you delete it";
    const SELF_MADE: &str = "I create every credential myself  -  more steps, but nothing you hand over can widen itself";
    let choice = Select::new(
        "How should the Cloudflare setup be done?",
        vec![MANAGED, SELF_MADE],
    )
    .prompt()?;
    Ok(if choice == MANAGED {
        SetupStyle::Managed
    } else {
        SetupStyle::SelfMade
    })
}

fn pick_zone(zones: Vec<ReachableZone>, must_be: Option<&str>) -> Result<ReachableZone> {
    if zones.is_empty() {
        return Err(anyhow!(
            "that token cannot see any zone. fn0 serves a project from a domain in a zone you \
             own, so there is nowhere to put it."
        ));
    }
    if let Some(zone_name) = must_be {
        return zones
            .into_iter()
            .find(|zone| zone.zone_name == zone_name)
            .ok_or_else(|| {
                anyhow!(
                    "this project lives in {zone_name}, which that token cannot see. Moving a \
                     project to another zone is not supported."
                )
            });
    }
    let labels: Vec<String> = zones
        .iter()
        .map(|zone| format!("{}  ({})", zone.zone_name, zone.account_name))
        .collect();
    let choice = Select::new("Which zone should this project live in?", labels.clone()).prompt()?;
    let picked = labels
        .iter()
        .position(|label| label == &choice)
        .expect("Select returns one of the options it was given");
    Ok(zones
        .into_iter()
        .nth(picked)
        .expect("index came from zones"))
}

fn ask_domain(zone: &ReachableZone) -> Result<String> {
    let domain = Text::new("Domain this app answers on")
        .with_help_message(&format!("a hostname in {}", zone.zone_name))
        .prompt()?
        .trim()
        .to_ascii_lowercase();
    if domain != zone.zone_name && !domain.ends_with(&format!(".{}", zone.zone_name)) {
        return Err(anyhow!(
            "'{domain}' is not inside {}. The record that points it at fn0 has to be created in \
             that zone.",
            zone.zone_name
        ));
    }
    Ok(domain)
}

fn ask_project_name() -> Result<String> {
    let name = Text::new("Project name")
        .with_help_message("a label in the control plane, not the domain")
        .prompt()?
        .trim()
        .to_string();
    if name.is_empty() {
        return Err(anyhow!("project name cannot be empty"));
    }
    Ok(name)
}

fn ask_own_credentials(setup: &CloudSetup<'_>, zone_name: &str) -> Result<ConnectCredentials> {
    let resources = expected_resources(setup.project_id, zone_name);
    println!();
    println!("Now create the three credentials fn0 will keep. None can be minted for");
    println!("you here, because the token you chose deliberately cannot create tokens.");
    println!();
    println!("The first two are separate on purpose: only the worker one is handed to");
    println!("the fleet, and only the frontend-asset one is used by the GC that deletes.");
    println!("Scoping them apart is what keeps either from reaching the other's bucket.");
    println!();
    println!("1. R2 -> Manage API Tokens -> Create API token");
    println!("     permission: Object Read & Write");
    println!("     apply to specific buckets:");
    println!("       {}", resources.private_object_storage_bucket);
    println!("       {}", resources.public_object_storage_bucket);
    println!("       {}", resources.rendered_html_cache_bucket);
    println!();
    println!("2. R2 -> Manage API Tokens -> Create API token");
    println!("     permission: Object Read & Write");
    println!("     apply to specific buckets:");
    println!("       {}", resources.frontend_asset_bucket);
    println!();
    println!("3. My Profile -> API Tokens -> Create Custom Token");
    println!("     permission: Zone -> Cache Purge -> Purge");
    println!("     zone resources: {zone_name} only");
    println!();
    if !Confirm::new("Created all three?")
        .with_default(false)
        .prompt()?
    {
        return Err(anyhow!(
            "stopped before connecting. The buckets are provisioned; re-run `forte cloud init` \
             once the credentials exist."
        ));
    }

    Ok(ConnectCredentials {
        worker_access_key_id: plain("Access Key ID from 1")?,
        worker_secret: secret("Secret Access Key from 1")?,
        frontend_asset_access_key_id: plain("Access Key ID from 2")?,
        frontend_asset_secret: secret("Secret Access Key from 2")?,
        purge_token: secret("Purge token from 3")?,
    })
}

fn plain(prompt: &str) -> Result<String> {
    Ok(Text::new(prompt).prompt()?.trim().to_string())
}

fn secret(prompt: &str) -> Result<String> {
    Ok(Password::new(prompt)
        .with_display_mode(PasswordDisplayMode::Masked)
        .without_confirmation()
        .prompt()?
        .trim()
        .to_string())
}

fn print_resources(resources: &ProvisionedResources) {
    println!("  zone     {}", resources.zone_name);
    println!("  buckets  {}", resources.private_object_storage_bucket);
    println!("           {}", resources.public_object_storage_bucket);
    println!("           {}", resources.frontend_asset_bucket);
    println!("           {}", resources.rendered_html_cache_bucket);
    println!("  assets   https://{}", resources.frontend_asset_hostname);
    println!(
        "  public   https://{}",
        resources.public_object_storage_hostname
    );
}
