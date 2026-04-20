use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::build::{BuildOptions, run_build};

#[derive(Deserialize)]
struct ForteConfig {
    name: Option<String>,
}

const BASE_PLACEHOLDER: &str = "/__FORTE_BASE__/";

#[derive(Deserialize)]
struct DeployStartResponse {
    presigned_url: String,
    deploy_job_id: String,
    subdomain: String,
    code_id: u64,
    build_id: String,
    static_base_url: String,
}

#[derive(Deserialize)]
struct DeployFinishResponse {
    generation: u64,
}

#[derive(Deserialize)]
struct DeployStatusResponse {
    delivered: bool,
    hosts_total: usize,
    hosts_at_target: usize,
    hosts_pending: Vec<String>,
    hosts_quarantined: Vec<String>,
}

#[derive(Serialize)]
struct DeployR2SignRequest<'a> {
    github_token: &'a str,
    subdomain: &'a str,
    build_id: &'a str,
    files: Vec<DeployR2SignFile>,
}

#[derive(Serialize)]
struct DeployR2SignFile {
    path: String,
    content_type: Option<&'static str>,
}

#[derive(Deserialize)]
struct DeployR2SignResponse {
    uploads: Vec<DeployR2SignUpload>,
}

#[derive(Deserialize)]
struct DeployR2SignUpload {
    path: String,
    presigned_url: String,
}

pub async fn run(project_dir: PathBuf) -> Result<()> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;

    let project_name = config
        .name
        .ok_or_else(|| anyhow!("'name' field missing in Forte.toml"))?;

    run_build(BuildOptions {
        project_dir: project_dir.clone(),
    })
    .await?;

    let github_token = fn0_deploy::get_github_token().await?;
    let client = reqwest::Client::new();

    println!("Requesting deploy start...");
    let start: DeployStartResponse = client
        .post(format!("{}/deploy/start", fn0_deploy::HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "project_name": project_name,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Deploy start failed: {}", e))?
        .json()
        .await?;

    println!("Subdomain: {}.fn0.dev", start.subdomain);
    println!("Build id:  {}", start.build_id);
    println!("Static base URL: {}", start.static_base_url);

    rewrite_placeholders(&project_dir, &start.static_base_url)?;

    let dist_dir = project_dir.join("dist");
    regenerate_server_js(&project_dir, &dist_dir)?;

    let assets = collect_static_assets(&project_dir.join("fe/dist"))?;
    println!("Uploading {} static assets to R2...", assets.len());
    upload_assets_to_r2(
        &client,
        &github_token,
        &start.subdomain,
        &start.build_id,
        &assets,
    )
    .await?;

    let bundle_path = dist_dir.join("bundle.raw.tar");
    fn0_deploy::create_raw_bundle_forte(&dist_dir, &bundle_path)?;

    println!("Uploading worker bundle...");
    let bundle_bytes = std::fs::read(&bundle_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", bundle_path.display(), e))?;
    client
        .put(&start.presigned_url)
        .header("content-type", "application/x-tar")
        .body(bundle_bytes)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Bundle upload failed: {}", e))?;

    let env_content = fn0_deploy::read_env_content(&project_dir)?;

    println!("Requesting deploy finish...");
    let finish: DeployFinishResponse = client
        .post(format!("{}/deploy/finish", fn0_deploy::HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "deploy_job_id": start.deploy_job_id,
            "subdomain": start.subdomain,
            "code_id": start.code_id,
            "build_id": start.build_id,
            "env": env_content,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Deploy finish failed: {}", e))?
        .json()
        .await?;

    println!(
        "Waiting for rollout to all workers (generation {})...",
        finish.generation
    );

    let poll_interval = std::time::Duration::from_secs(2);
    let timeout = std::time::Duration::from_secs(300);
    let poll_start = std::time::Instant::now();
    let mut last_progress: Option<(usize, usize)> = None;

    loop {
        let status: DeployStatusResponse = client
            .get(format!(
                "{}/deploy/status?generation={}",
                fn0_deploy::HQ_URL,
                finish.generation
            ))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| anyhow!("Deploy status failed: {}", e))?
            .json()
            .await?;

        let progress = (status.hosts_at_target, status.hosts_total);
        if last_progress != Some(progress) {
            println!("  {}/{} hosts ready", progress.0, progress.1);
            last_progress = Some(progress);
        }

        if status.delivered {
            break;
        }

        if poll_start.elapsed() > timeout {
            return Err(anyhow!(
                "Deploy rollout timed out after {}s. pending={:?} quarantined={:?}",
                timeout.as_secs(),
                status.hosts_pending,
                status.hosts_quarantined
            ));
        }

        tokio::time::sleep(poll_interval).await;
    }

    println!("Deploy complete!");
    Ok(())
}

fn rewrite_placeholders(project_dir: &Path, static_base_url: &str) -> Result<()> {
    let fe_dist = project_dir.join("fe/dist");
    if !fe_dist.exists() {
        anyhow::bail!("fe/dist not found; did vite build fail?");
    }
    let mut count = 0usize;
    rewrite_dir_recursive(&fe_dist, static_base_url, &mut count)?;
    println!("[dist] Rewrote base placeholder in {count} file(s)");
    Ok(())
}

fn rewrite_dir_recursive(dir: &Path, target: &str, count: &mut usize) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            rewrite_dir_recursive(&path, target, count)?;
            continue;
        }
        if !is_text_asset(&path) {
            continue;
        }
        let bytes = std::fs::read(&path)?;
        let Ok(text) = std::str::from_utf8(&bytes) else {
            continue;
        };
        if !text.contains(BASE_PLACEHOLDER) {
            continue;
        }
        let rewritten = text.replace(BASE_PLACEHOLDER, target);
        std::fs::write(&path, rewritten)?;
        *count += 1;
    }
    Ok(())
}

fn is_text_asset(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(
        ext,
        "js" | "mjs" | "cjs" | "css" | "html" | "htm" | "json" | "svg" | "map"
    )
}

fn regenerate_server_js(project_dir: &Path, dist_dir: &Path) -> Result<()> {
    let src = project_dir.join("fe/dist/ssr/server.js");
    let dst = dist_dir.join("server.js");
    std::fs::copy(&src, &dst)
        .with_context(|| format!("copy {} -> {}", src.display(), dst.display()))?;
    Ok(())
}

struct AssetFile {
    relative_path: String,
    absolute_path: PathBuf,
    content_type: &'static str,
}

fn collect_static_assets(fe_dist: &Path) -> Result<Vec<AssetFile>> {
    let mut out = Vec::new();
    if !fe_dist.exists() {
        return Ok(out);
    }
    walk_collect(fe_dist, fe_dist, &mut out)?;
    Ok(out)
}

fn walk_collect(base: &Path, dir: &Path, out: &mut Vec<AssetFile>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|s| s.to_str()) == Some("ssr")
                && path.parent() == Some(base)
            {
                continue;
            }
            walk_collect(base, &path, out)?;
            continue;
        }
        let rel = path
            .strip_prefix(base)
            .map_err(|e| anyhow!("strip_prefix: {e}"))?
            .to_string_lossy()
            .replace('\\', "/");
        let content_type = content_type_for(&path);
        out.push(AssetFile {
            relative_path: rel,
            absolute_path: path.clone(),
            content_type,
        });
    }
    Ok(())
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") | Some("cjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("pdf") => "application/pdf",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        _ => "application/octet-stream",
    }
}

async fn upload_assets_to_r2(
    client: &reqwest::Client,
    github_token: &str,
    subdomain: &str,
    build_id: &str,
    assets: &[AssetFile],
) -> Result<()> {
    if assets.is_empty() {
        return Ok(());
    }

    let files: Vec<DeployR2SignFile> = assets
        .iter()
        .map(|a| DeployR2SignFile {
            path: a.relative_path.clone(),
            content_type: Some(a.content_type),
        })
        .collect();

    let sign_req = DeployR2SignRequest {
        github_token,
        subdomain,
        build_id,
        files,
    };

    let sign: DeployR2SignResponse = client
        .post(format!("{}/deploy/r2/sign", fn0_deploy::HQ_URL))
        .json(&sign_req)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("Deploy r2/sign failed: {}", e))?
        .json()
        .await?;

    let mut url_for_path: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for u in sign.uploads {
        url_for_path.insert(u.path, u.presigned_url);
    }

    let mut tasks = futures::stream::FuturesUnordered::new();
    for asset in assets {
        let url = url_for_path
            .remove(&asset.relative_path)
            .ok_or_else(|| anyhow!("No presigned URL returned for {}", asset.relative_path))?;
        let bytes = std::fs::read(&asset.absolute_path)
            .map_err(|e| anyhow!("read {}: {}", asset.absolute_path.display(), e))?;
        let client = client.clone();
        let content_type = asset.content_type;
        let relative = asset.relative_path.clone();
        tasks.push(async move {
            let resp = client
                .put(&url)
                .header("content-type", content_type)
                .body(bytes)
                .send()
                .await
                .map_err(|e| anyhow!("R2 PUT failed for {}: {}", relative, e))?;
            resp.error_for_status()
                .map_err(|e| anyhow!("R2 PUT HTTP error for {}: {}", relative, e))?;
            Ok::<_, anyhow::Error>(())
        });
    }

    use futures::StreamExt;
    while let Some(result) = tasks.next().await {
        result?;
    }

    Ok(())
}
