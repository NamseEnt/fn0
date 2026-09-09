use anyhow::{Result, anyhow};
use std::path::PathBuf;

use super::project_config::{
    CloudConfig, clear_cloud_config, read_cloud_config, write_origin_hostname,
};
use fn0_deploy::{BrokerClient, DomainStatus, ReachableZone, credentials::Credentials};

struct TeardownContext {
    broker: BrokerClient,
    zone: ReachableZone,
    zone_name: String,
    app_hostname: String,
}

pub async fn run(project_dir: PathBuf, yes: bool, delete_buckets: bool) -> Result<()> {
    let config = read_cloud_config(&project_dir)?;
    let project_id = config
        .project_id
        .clone()
        .ok_or_else(|| anyhow!("'project_id' field missing in Forte.toml. Nothing to destroy."))?;

    if !yes {
        let bucket_line = if delete_buckets {
            "\n             --delete-buckets: the three R2 buckets are deleted too, contents and all."
        } else {
            ""
        };
        println!(
            "This permanently deletes project '{project_id}' and ALL of its resources:\n\
             routing, custom domain, deployed bundles, static assets, object storage, and its database.{bucket_line}"
        );
        let answer = inquire::Text::new("Type the project id to confirm:").prompt()?;
        if answer.trim() != project_id {
            return Err(anyhow!(
                "confirmation did not match '{project_id}'; aborted."
            ));
        }
    }

    let origin_hostname = expected_origin_hostname(&config, &project_id).await?;
    if let Some(origin_hostname) = origin_hostname.as_deref() {
        write_origin_hostname(&project_dir, origin_hostname)?;
    }
    let teardown_context = prepare_teardown(&config).await?;

    fn0_deploy::delete_project_if_present(&project_id).await?;
    fn0_deploy::wait_for_project_teardown(&project_id).await?;

    if let Some(teardown_context) = teardown_context {
        teardown_cloudflare(
            teardown_context,
            &project_id,
            origin_hostname.as_deref(),
            delete_buckets,
        )
        .await?;
    }

    clear_cloud_config(&project_dir)?;
    println!(
        "Removed cloud configuration from Forte.toml (next `forte deploy` creates a new project)"
    );
    println!("Teardown of '{project_id}' completed.");
    Ok(())
}

async fn prepare_teardown(config: &CloudConfig) -> Result<Option<TeardownContext>> {
    let (Some(zone_name), Some(app_hostname)) = (config.zone.as_deref(), config.domain.as_deref())
    else {
        return Ok(None);
    };
    let creds = fn0_deploy::credentials::require()?;
    let broker = load_broker(config, &creds)?.ok_or_else(|| {
        anyhow!(
            "Cloudflare is configured but the setup broker is missing; run `forte cloud init` or restore the broker settings before destroying the project"
        )
    })?;
    let zone = broker.resolve_zone(zone_name).await?;
    Ok(Some(TeardownContext {
        broker,
        zone,
        zone_name: zone_name.to_string(),
        app_hostname: app_hostname.to_string(),
    }))
}

async fn teardown_cloudflare(
    context: TeardownContext,
    project_id: &str,
    origin_hostname: Option<&str>,
    delete_buckets: bool,
) -> Result<()> {
    println!("cleaning up the project's Cloudflare resources through the setup broker...");
    let outcome = context
        .broker
        .teardown_project(
            project_id,
            &context.zone.zone_id,
            &context.zone_name,
            &context.app_hostname,
            origin_hostname,
            delete_buckets,
        )
        .await?;
    println!(
        "  DNS record, bucket custom domains, origin certificate, and minted tokens cleaned up{}",
        if delete_buckets {
            "; bucket deletion requested"
        } else {
            ""
        }
    );
    for note in &outcome.notes {
        println!("  note: {note}");
    }
    if !outcome.pending.is_empty() {
        for pending in &outcome.pending {
            println!("  pending: {pending}");
        }
        return Err(anyhow!(
            "Cloudflare teardown is incomplete; rerun `forte destroy` after the pending resources finish clearing"
        ));
    }
    Ok(())
}

async fn expected_origin_hostname(
    config: &CloudConfig,
    project_id: &str,
) -> Result<Option<String>> {
    if config.zone.is_none() || config.domain.is_none() {
        return Ok(None);
    }
    if let Some(origin_hostname) = config.origin_hostname.as_deref() {
        return Ok(Some(origin_hostname.to_string()));
    }
    let creds = fn0_deploy::credentials::require()?;
    let configured_domain = config.domain.as_deref().unwrap_or_default();
    match fn0_deploy::fetch_domain_status(&creds, project_id).await? {
        DomainStatus::SelfHosted {
            domain,
            origin_hostname,
            ..
        } if domain == configured_domain && !origin_hostname.is_empty() => {
            Ok(Some(origin_hostname))
        }
        DomainStatus::NoDomain => Ok(None),
        DomainStatus::SelfHosted { .. } => Ok(None),
        DomainStatus::NotLoggedIn => Err(anyhow!("control rejected token; run `fn0 login` again.")),
        DomainStatus::NotFound => Ok(None),
        DomainStatus::InternalError => Err(anyhow!(
            "domain_status: server error; check fn0-control logs"
        )),
    }
}

fn load_broker(config: &CloudConfig, creds: &Credentials) -> Result<Option<BrokerClient>> {
    match (
        config.cloudflare_account_id.clone(),
        config.cloudflare_broker_url.clone(),
    ) {
        (Some(account_id), Some(broker_url)) => Ok(Some(BrokerClient::new(
            broker_url,
            creds.control_url.clone(),
            creds.token.clone(),
            account_id,
        )?)),
        (None, None) => match fn0_deploy::load_broker_settings()? {
            Some(settings) => Ok(Some(BrokerClient::new(
                settings.broker_url,
                creds.control_url.clone(),
                creds.token.clone(),
                settings.account_id,
            )?)),
            None => Ok(None),
        },
        _ => Err(anyhow!(
            "Forte.toml must contain both cloudflare_account_id and cloudflare_broker_url"
        )),
    }
}
