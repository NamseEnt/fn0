use crate::server;
use anyhow::Result;
use std::path::Path;

pub async fn flush(project_dir: &Path, task_name: Option<&str>) -> Result<()> {
    let env_vars = server::load_env_file(project_dir);
    let turso_url = env_vars
        .iter()
        .find(|(k, _)| k == "TURSO_URL")
        .map(|(_, v)| v.as_str())
        .unwrap_or("http://127.0.0.1:8080");
    let auth_token = env_vars
        .iter()
        .find(|(k, _)| k == "TURSO_AUTH_TOKEN")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");

    let queue = fn0::turso_queue::TursoQueue::new(turso_url, auth_token);
    let flushed = queue.flush_dead_queue(task_name).await?;
    println!("Flushed {} tasks from dead queue", flushed);

    Ok(())
}
