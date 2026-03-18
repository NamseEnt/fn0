use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

fn tools_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".forte").join("bin"))
}

fn target_triple() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        (os, arch) => anyhow::bail!("Unsupported platform: {os}-{arch}"),
    }
}

pub async fn ensure_github_tool(
    name: &str,
    version: &str,
    download_url: &str,
    binary_name: &str,
) -> Result<PathBuf> {
    let dir = tools_dir()?;
    let path = dir.join(format!("{}-{}", name, version));

    if path.exists() {
        return Ok(path);
    }

    std::fs::create_dir_all(&dir)?;

    println!("Downloading {} {}...", name, version);

    let response = reqwest::get(download_url).await?.error_for_status()?;
    let bytes = response.bytes().await?;

    let temp_archive = dir.join(format!("{}-temp.tar.xz", name));
    std::fs::write(&temp_archive, &bytes)?;

    let temp_extract = dir.join(format!("{}-temp_extract", name));
    std::fs::create_dir_all(&temp_extract)?;

    let status = Command::new("tar")
        .arg("xf")
        .arg(&temp_archive)
        .arg("-C")
        .arg(&temp_extract)
        .status()
        .context("Failed to run tar")?;

    std::fs::remove_file(&temp_archive)?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&temp_extract);
        anyhow::bail!("Failed to extract {} archive", name);
    }

    let extracted = find_binary(&temp_extract, binary_name)?;
    std::fs::rename(&extracted, &path)?;
    let _ = std::fs::remove_dir_all(&temp_extract);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("{} {} ready.", name, version);
    Ok(path)
}

fn find_binary(dir: &Path, binary_name: &str) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_binary(&path, binary_name) {
                return Ok(found);
            }
        } else if path.file_name().is_some_and(|n| n == binary_name) {
            return Ok(path);
        }
    }
    anyhow::bail!("{} binary not found in archive", binary_name)
}

pub fn fn0_release_url(package: &str, version: &str) -> Result<String> {
    let triple = target_triple()?;
    Ok(format!(
        "https://github.com/NamseEnt/fn0/releases/download/{package}-v{version}/{package}-{triple}.tar.xz"
    ))
}
