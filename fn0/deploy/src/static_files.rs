use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

pub struct StaticFile {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub size: u64,
    pub content_type: &'static str,
}

pub fn collect_static_files(dir: &Path) -> Result<Vec<StaticFile>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    walk_collect(dir, dir, &mut out)?;
    Ok(out)
}

fn walk_collect(base: &Path, dir: &Path, out: &mut Vec<StaticFile>) -> Result<()> {
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
        let metadata = entry.metadata()?;
        let rel = path
            .strip_prefix(base)
            .map_err(|e| anyhow!("strip_prefix: {e}"))?
            .to_string_lossy()
            .replace('\\', "/");
        out.push(StaticFile {
            relative_path: rel,
            absolute_path: path.clone(),
            size: metadata.len(),
            content_type: content_type_for(&path),
        });
    }
    Ok(())
}

pub fn content_type_for(path: &Path) -> &'static str {
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
