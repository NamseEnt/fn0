use forte_sdk::*;
use serde::Serialize;

#[derive(Serialize)]
struct InvokeBody<'a> {
    project_id: &'a str,
    task_name: &'a str,
    payload: serde_json::Value,
}

pub async fn enqueue(
    project_id: &str,
    task_name: &str,
    payload: serde_json::Value,
) -> anyhow::Result<()> {
    let invoke_queue_url = std::env::var("FN0_CONTROL_INVOKE_QUEUE_URL")
        .map_err(|_| anyhow::anyhow!("FN0_CONTROL_INVOKE_QUEUE_URL not set"))?;
    let body = serde_json::to_vec(&InvokeBody {
        project_id,
        task_name,
        payload,
    })?;
    let req = http::Request::builder()
        .method("POST")
        .uri(&invoke_queue_url)
        .header("content-type", "application/json")
        .body(body)?;
    let resp = http::Client::new().send(req).await?;
    if !resp.status().is_success() {
        anyhow::bail!("invoke queue status {}", resp.status());
    }
    Ok(())
}
