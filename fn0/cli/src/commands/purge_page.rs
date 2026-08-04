use color_eyre::{Result, eyre::eyre};

pub async fn run(paths: Vec<String>, project: Option<String>) -> Result<()> {
    let project_name = match project {
        Some(project) => project,
        None => {
            let config = crate::config::Config::load("fn0.toml").map_err(|_| {
                eyre!("fn0.toml not found. Pass --project <name> or run in a project directory.")
            })?;
            config
                .name
                .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))?
        }
    };

    let purged = fn0_deploy::static_page_purge(&project_name, &paths)
        .await
        .map_err(|e| eyre!("{e}"))?;
    for path in &purged {
        println!("{path}");
    }
    println!("queued {} invalidation(s)", purged.len());
    Ok(())
}
