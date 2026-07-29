use color_eyre::{Result, eyre::eyre};

pub async fn run(keys: Vec<String>, project: Option<String>) -> Result<()> {
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

    let urls = fn0_deploy::public_purge(&project_name, &keys)
        .await
        .map_err(|e| eyre!("{e}"))?;
    for url in &urls {
        println!("{url}");
    }
    println!("queued {} invalidation(s)", urls.len());
    Ok(())
}
