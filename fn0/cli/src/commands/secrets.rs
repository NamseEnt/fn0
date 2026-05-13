use crate::utils::credentials::{self, Credentials};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ENV_YAML_FILENAME: &str = "env.yaml";
const DEK_KEY: &str = "__dek";

pub async fn set(key: String, value: String) -> Result<()> {
    let creds = credentials::require()?;
    let env_path = env_yaml_path()?;
    let mut mapping = load_mapping(&env_path)?;

    let dek_ct = ensure_dek(&mut mapping, &creds).await?;

    let ciphertext = call_secrets_encrypt(&creds, &dek_ct, &value).await?;

    let mut entry = serde_yaml::Mapping::new();
    entry.insert(
        serde_yaml::Value::String("secret".to_string()),
        serde_yaml::Value::String(ciphertext),
    );
    mapping.insert(
        serde_yaml::Value::String(key.clone()),
        serde_yaml::Value::Mapping(entry),
    );

    save_mapping(&env_path, &mapping)?;
    println!("set {} in {}", key, env_path.display());
    Ok(())
}

pub async fn list() -> Result<()> {
    let env_path = env_yaml_path()?;
    if !env_path.exists() {
        println!("(no {} in this project)", env_path.display());
        return Ok(());
    }
    let mapping = load_mapping(&env_path)?;
    let mut names: Vec<&str> = mapping
        .keys()
        .filter_map(|k| k.as_str())
        .filter(|k| *k != DEK_KEY)
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no entries)");
        return Ok(());
    }
    for name in names {
        let kind = match mapping.get(serde_yaml::Value::String(name.to_string())) {
            Some(serde_yaml::Value::Mapping(m)) if m.contains_key("secret") => "secret",
            _ => "plain",
        };
        println!("{name}\t{kind}");
    }
    Ok(())
}

pub async fn unset(key: String) -> Result<()> {
    if key == DEK_KEY {
        return Err(eyre!("cannot unset {DEK_KEY}"));
    }
    let env_path = env_yaml_path()?;
    if !env_path.exists() {
        return Err(eyre!("no {} in this project", env_path.display()));
    }
    let mut mapping = load_mapping(&env_path)?;
    if mapping
        .remove(serde_yaml::Value::String(key.clone()))
        .is_none()
    {
        return Err(eyre!("{} not present in {}", key, env_path.display()));
    }
    save_mapping(&env_path, &mapping)?;
    println!("unset {} in {}", key, env_path.display());
    Ok(())
}

async fn ensure_dek(mapping: &mut serde_yaml::Mapping, creds: &Credentials) -> Result<String> {
    if let Some(serde_yaml::Value::Mapping(dek_map)) = mapping.get(DEK_KEY)
        && let Some(serde_yaml::Value::String(s)) = dek_map.get("encrypted")
    {
        return Ok(s.clone());
    }

    let ct = call_secrets_init(creds).await?;
    let mut dek_entry = serde_yaml::Mapping::new();
    dek_entry.insert(
        serde_yaml::Value::String("encrypted".to_string()),
        serde_yaml::Value::String(ct.clone()),
    );
    mapping.insert(
        serde_yaml::Value::String(DEK_KEY.to_string()),
        serde_yaml::Value::Mapping(dek_entry),
    );
    Ok(ct)
}

async fn call_secrets_init(creds: &Credentials) -> Result<String> {
    #[derive(Serialize)]
    struct Empty {}
    #[derive(Deserialize)]
    #[serde(tag = "t", rename_all_fields = "camelCase")]
    enum InitResp {
        Ok { encrypted_dek: String },
        Unauthorized,
        Error { message: String },
    }
    let resp: InitResp = post_action(creds, "secrets_init", &Empty {}).await?;
    match resp {
        InitResp::Ok { encrypted_dek } => Ok(encrypted_dek),
        InitResp::Unauthorized => Err(eyre!("unauthorized — `fn0 login` again")),
        InitResp::Error { message } => Err(eyre!("control error: {message}")),
    }
}

async fn call_secrets_encrypt(
    creds: &Credentials,
    encrypted_dek: &str,
    value: &str,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Req<'a> {
        encrypted_dek: &'a str,
        value: &'a str,
    }
    #[derive(Deserialize)]
    #[serde(tag = "t", rename_all_fields = "camelCase")]
    enum EncResp {
        Ok { ciphertext: String },
        Unauthorized,
        Error { message: String },
    }
    let resp: EncResp = post_action(
        creds,
        "secrets_encrypt",
        &Req {
            encrypted_dek,
            value,
        },
    )
    .await?;
    match resp {
        EncResp::Ok { ciphertext } => Ok(ciphertext),
        EncResp::Unauthorized => Err(eyre!("unauthorized — `fn0 login` again")),
        EncResp::Error { message } => Err(eyre!("control error: {message}")),
    }
}

async fn post_action<I, O>(creds: &Credentials, name: &str, body: &I) -> Result<O>
where
    I: Serialize,
    O: serde::de::DeserializeOwned,
{
    let url = format!(
        "{}/__forte_action/{}",
        creds.control_url.trim_end_matches('/'),
        name
    );
    let resp = reqwest::Client::new()
        .post(url)
        .bearer_auth(&creds.token)
        .json(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(eyre!("control returned {status}: {body}"));
    }
    let parsed = resp.json().await?;
    Ok(parsed)
}

fn env_yaml_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join(ENV_YAML_FILENAME))
}

fn load_mapping(p: &Path) -> Result<serde_yaml::Mapping> {
    if !p.exists() {
        return Ok(serde_yaml::Mapping::new());
    }
    let content = std::fs::read_to_string(p)?;
    if content.trim().is_empty() {
        return Ok(serde_yaml::Mapping::new());
    }
    let value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    match value {
        serde_yaml::Value::Mapping(m) => Ok(m),
        _ => Err(eyre!("{} must contain a mapping", p.display())),
    }
}

fn save_mapping(p: &Path, m: &serde_yaml::Mapping) -> Result<()> {
    let s = serde_yaml::to_string(&serde_yaml::Value::Mapping(m.clone()))?;
    std::fs::write(p, s)?;
    Ok(())
}
