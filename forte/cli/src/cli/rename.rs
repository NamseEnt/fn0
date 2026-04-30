use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct ForteConfig {
    name: Option<String>,
}

pub async fn run(project_dir: PathBuf, new_name: String) -> Result<()> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;

    let project_name = config
        .name
        .ok_or_else(|| anyhow!("'name' field missing in Forte.toml"))?;

    if project_name == new_name {
        return Err(anyhow!(
            "new name '{}' is identical to current name",
            new_name
        ));
    }

    fn0_deploy::rename(&project_name, &new_name).await?;

    let updated = rewrite_name_in_toml(&content, &project_name, &new_name)?;
    std::fs::write(&config_path, updated)
        .map_err(|e| anyhow!("Failed to update Forte.toml: {}", e))?;
    println!(
        "Updated {}: name = \"{}\"",
        config_path.display(),
        new_name
    );

    Ok(())
}

fn rewrite_name_in_toml(content: &str, old_name: &str, new_name: &str) -> Result<String> {
    let mut out = String::with_capacity(content.len());
    let mut replaced = false;
    for line in content.lines() {
        if !replaced {
            if let Some(rewritten) = try_rewrite_name_line(line, old_name, new_name) {
                out.push_str(&rewritten);
                out.push('\n');
                replaced = true;
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    if !replaced {
        return Err(anyhow!(
            "could not locate `name = \"{}\"` line in Forte.toml",
            old_name
        ));
    }
    Ok(out)
}

fn try_rewrite_name_line(line: &str, old_name: &str, new_name: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("name") {
        return None;
    }
    let leading_ws_len = line.len() - trimmed.len();
    let after_key = trimmed.strip_prefix("name")?;
    let after_eq = after_key.trim_start().strip_prefix('=')?;
    let value_part = after_eq.trim_start();
    let (quote, expected) = if let Some(rest) = value_part.strip_prefix('"') {
        ('"', rest)
    } else if let Some(rest) = value_part.strip_prefix('\'') {
        ('\'', rest)
    } else {
        return None;
    };
    let end_idx = expected.find(quote)?;
    let current = &expected[..end_idx];
    if current != old_name {
        return None;
    }
    let leading_ws = &line[..leading_ws_len];
    Some(format!("{leading_ws}name = \"{new_name}\""))
}
