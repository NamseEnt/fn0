use crate::config::Config;
use color_eyre::{Result, eyre::eyre};
use std::io::Write;
use std::path::PathBuf;

pub async fn execute(project_id_override: Option<String>, yes: bool) -> Result<()> {
    let config_path = PathBuf::from("fn0.toml");
    let config = Config::load(&config_path).ok();

    let project_id = match project_id_override {
        Some(id) => id,
        None => config
            .as_ref()
            .and_then(|c| c.project_id.clone())
            .ok_or_else(|| {
                eyre!("no project_id in fn0.toml; pass --project-id to destroy a project by id.")
            })?,
    };

    if !yes {
        println!(
            "This permanently deletes project '{project_id}' and ALL of its resources:\n\
             routing, custom domain, deployed bundles, static assets, object storage, and its database."
        );
        print!("Type the project id to confirm: ");
        std::io::stdout().flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if answer.trim() != project_id {
            return Err(eyre!("confirmation did not match '{project_id}'; aborted."));
        }
    }

    fn0_deploy::delete_project(&project_id)
        .await
        .map_err(|e| eyre!(e))?;

    if let Some(mut config) = config
        && config.project_id.as_deref() == Some(project_id.as_str())
    {
        config.project_id = None;
        config.save(&config_path)?;
        println!("Removed project_id from fn0.toml (next `fn0 deploy` creates a new project)");
    }

    println!("Teardown of '{project_id}' enqueued; resources are being deleted.");
    Ok(())
}
