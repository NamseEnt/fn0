use color_eyre::{eyre::eyre, Result};

pub async fn execute(new_name: &str) -> Result<()> {
    let mut config = crate::config::Config::load("fn0.toml")
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;

    let project_name = config
        .name
        .clone()
        .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))?;

    if project_name == new_name {
        return Err(eyre!(
            "new name '{}' is identical to current name",
            new_name
        ));
    }

    fn0_deploy::rename(&project_name, new_name)
        .await
        .map_err(|e| eyre!(e))?;

    config.name = Some(new_name.to_string());
    config.save("fn0.toml").map_err(|e| eyre!(e))?;
    println!("Updated fn0.toml: name = \"{}\"", new_name);

    Ok(())
}
