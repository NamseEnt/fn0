use anyhow::Result;
use std::path::PathBuf;

use super::project_config::read_project_id;

pub async fn run(project_dir: PathBuf, print: bool) -> Result<()> {
    let project_id = read_project_id(&project_dir)?;
    let resolved = fn0_deploy::resolve_app_url(&project_id).await?;

    if let Some(note) = &resolved.note {
        eprintln!("note: {note}");
    }
    let Some(url) = resolved.url else {
        anyhow::bail!("project '{project_id}' has no URL to open yet");
    };
    println!("{url}");

    if !print {
        open::that_detached(&url)?;
    }
    Ok(())
}
