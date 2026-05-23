use anyhow::{Result, anyhow, bail};
use fn0_deploy::CronJob;
use serde::Deserialize;
use std::path::Path;
use syn::{Item, ItemStruct, ItemType, Type};

#[derive(Deserialize)]
struct CronYamlEntry {
    function: String,
    every_minutes: u32,
}

pub fn read_and_validate(project_dir: &Path) -> Result<Vec<CronJob>> {
    let cron_path = project_dir.join("cron.yaml");
    if !cron_path.exists() {
        return Ok(Vec::new());
    }

    let raw = std::fs::read_to_string(&cron_path).map_err(|e| anyhow!("read cron.yaml: {e}"))?;
    let entries: Vec<CronYamlEntry> =
        serde_yaml::from_str(&raw).map_err(|e| anyhow!("parse cron.yaml: {e}"))?;

    let mut jobs: Vec<CronJob> = Vec::with_capacity(entries.len());
    for entry in entries {
        if entry.every_minutes == 0 {
            bail!(
                "cron.yaml: function '{}' has every_minutes=0",
                entry.function
            );
        }
        let task_path = project_dir
            .join("rs/src/queue_task")
            .join(format!("{}.rs", entry.function));
        if !task_path.exists() {
            bail!(
                "cron.yaml: function '{}' has no rs/src/queue_task/{}.rs",
                entry.function,
                entry.function
            );
        }
        validate_unit_input(&task_path, &entry.function)?;
        jobs.push(CronJob {
            function: entry.function,
            every_minutes: entry.every_minutes,
        });
    }

    Ok(jobs)
}

fn validate_unit_input(task_file: &Path, function: &str) -> Result<()> {
    let content = std::fs::read_to_string(task_file)
        .map_err(|e| anyhow!("read {}: {e}", task_file.display()))?;
    let parsed =
        syn::parse_file(&content).map_err(|e| anyhow!("parse {}: {e}", task_file.display()))?;

    for item in parsed.items {
        match item {
            Item::Struct(ItemStruct { ident, fields, .. }) if ident == "Input" => {
                let is_unit = matches!(fields, syn::Fields::Unit)
                    || matches!(&fields, syn::Fields::Named(n) if n.named.is_empty())
                    || matches!(&fields, syn::Fields::Unnamed(u) if u.unnamed.is_empty());
                if !is_unit {
                    bail!(
                        "cron.yaml: queue_task '{function}' has non-unit Input; cron requires Input = ()"
                    );
                }
                return Ok(());
            }
            Item::Type(ItemType { ident, ty, .. }) if ident == "Input" => {
                let Type::Tuple(t) = ty.as_ref() else {
                    bail!(
                        "cron.yaml: queue_task '{function}' Input alias must be (); cron requires Input = ()"
                    );
                };
                if !t.elems.is_empty() {
                    bail!(
                        "cron.yaml: queue_task '{function}' Input alias must be (); cron requires Input = ()"
                    );
                }
                return Ok(());
            }
            _ => {}
        }
    }
    bail!(
        "cron.yaml: queue_task '{function}' has no Input definition; cron requires `pub type Input = ();` or unit struct"
    );
}
