use anyhow::{Result, bail};
use libsql_hrana::proto::*;
use serde::Deserialize;

pub struct TursoQueue {
    pipeline_url: String,
    auth_token: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct QueueTask {
    pub id: String,
    pub task_name: String,
    pub payload: String,
    pub retry_count: i64,
    pub max_retries: i64,
}

#[derive(Deserialize)]
struct PipelineResponse {
    #[allow(dead_code)]
    baton: Option<String>,
    results: Vec<PipelineResult>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum PipelineResult {
    #[serde(rename = "ok")]
    Ok { response: PipelineOkResponse },
    #[serde(rename = "error")]
    Error { error: PipelineError },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum PipelineOkResponse {
    #[serde(rename = "execute")]
    Execute { result: ExecuteResult },
    #[serde(rename = "close")]
    Close,
}

#[derive(Deserialize)]
struct PipelineError {
    message: String,
}

#[derive(Deserialize)]
struct ExecuteResult {
    rows: Vec<Vec<serde_json::Value>>,
    affected_row_count: u64,
}

impl TursoQueue {
    pub fn new(url: &str, auth_token: &str) -> Self {
        let pipeline_url = if let Some(stripped) = url.strip_prefix("libsql://") {
            format!("https://{}/v2/pipeline", stripped)
        } else {
            format!("{}/v2/pipeline", url.trim_end_matches('/'))
        };

        Self {
            pipeline_url,
            auth_token: auth_token.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn execute_pipeline(&self, requests: Vec<StreamRequest>) -> Result<PipelineResponse> {
        let body = PipelineReqBody {
            baton: None,
            requests,
        };

        let mut req = self.client.post(&self.pipeline_url);

        if !self.auth_token.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.auth_token));
        }

        let response = req
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&body)?)
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("Turso request failed with status: {}", response.status());
        }

        let resp: PipelineResponse = response.json().await?;
        Ok(resp)
    }

    fn make_stmt(sql: &str, args: Vec<Value>, want_rows: bool) -> StreamRequest {
        StreamRequest::Execute(ExecuteStreamReq {
            stmt: Stmt {
                sql: Some(sql.to_string()),
                sql_id: None,
                args,
                named_args: vec![],
                want_rows: Some(want_rows),
                replication_index: None,
            },
        })
    }

    fn text(s: &str) -> Value {
        Value::Text {
            value: s.to_string().into(),
        }
    }

    fn extract_execute_result(resp: &PipelineResponse) -> Result<Option<&ExecuteResult>> {
        for result in &resp.results {
            match result {
                PipelineResult::Ok { response } => match response {
                    PipelineOkResponse::Execute { result } => return Ok(Some(result)),
                    PipelineOkResponse::Close => continue,
                },
                PipelineResult::Error { error } => {
                    bail!("Turso error: {}", error.message);
                }
            }
        }
        Ok(None)
    }

    fn row_to_string(val: &serde_json::Value) -> Option<String> {
        match val {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) if arr.len() == 2 => {
                arr[1].as_str().map(|s| s.to_string())
            }
            serde_json::Value::Object(map) => map
                .get("value")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        }
    }

    fn row_to_i64(val: &serde_json::Value) -> Option<i64> {
        match val {
            serde_json::Value::Number(n) => n.as_i64(),
            serde_json::Value::String(s) => s.parse().ok(),
            serde_json::Value::Object(map) => map.get("value").and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_i64())
            }),
            _ => None,
        }
    }

    pub async fn create_tables(&self) -> Result<()> {
        let resp = self
            .execute_pipeline(vec![
                Self::make_stmt(
                    "CREATE TABLE IF NOT EXISTS __forte_queue (id TEXT PRIMARY KEY, task_name TEXT NOT NULL, payload TEXT NOT NULL, status TEXT NOT NULL, retry_count INTEGER NOT NULL, max_retries INTEGER NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
                    vec![],
                    false,
                ),
                Self::make_stmt(
                    "CREATE TABLE IF NOT EXISTS __forte_dead_queue (id TEXT PRIMARY KEY, task_name TEXT NOT NULL, payload TEXT NOT NULL, error_message TEXT NOT NULL, retry_count INTEGER NOT NULL, created_at TEXT NOT NULL, died_at TEXT NOT NULL)",
                    vec![],
                    false,
                ),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for result in &resp.results {
            if let PipelineResult::Error { error } = result {
                bail!("Create table error: {}", error.message);
            }
        }

        Ok(())
    }

    pub async fn claim_task(&self) -> Result<Option<QueueTask>> {
        let now = chrono::Utc::now().to_rfc3339();
        let resp = self
            .execute_pipeline(vec![
                Self::make_stmt(
                    "WITH target AS (SELECT id FROM __forte_queue WHERE status='pending' OR (status='processing' AND updated_at < datetime('now', '-60 seconds')) LIMIT 1) UPDATE __forte_queue SET status='processing', updated_at=? WHERE id IN (SELECT id FROM target) RETURNING *",
                    vec![Self::text(&now)],
                    true,
                ),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        let Some(result) = Self::extract_execute_result(&resp)? else {
            return Ok(None);
        };

        let Some(row) = result.rows.first() else {
            return Ok(None);
        };

        if row.len() < 6 {
            return Ok(None);
        }

        Ok(Some(QueueTask {
            id: Self::row_to_string(&row[0]).unwrap_or_default(),
            task_name: Self::row_to_string(&row[1]).unwrap_or_default(),
            payload: Self::row_to_string(&row[2]).unwrap_or_default(),
            retry_count: Self::row_to_i64(&row[4]).unwrap_or(0),
            max_retries: Self::row_to_i64(&row[5]).unwrap_or(3),
        }))
    }

    pub async fn delete_task(&self, id: &str) -> Result<()> {
        let resp = self
            .execute_pipeline(vec![
                Self::make_stmt(
                    "DELETE FROM __forte_queue WHERE id = ?",
                    vec![Self::text(id)],
                    false,
                ),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for result in &resp.results {
            if let PipelineResult::Error { error } = result {
                bail!("Delete task error: {}", error.message);
            }
        }

        Ok(())
    }

    pub async fn retry_task(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let resp = self
            .execute_pipeline(vec![
                Self::make_stmt(
                    "UPDATE __forte_queue SET status='pending', retry_count=retry_count+1, updated_at=? WHERE id=?",
                    vec![Self::text(&now), Self::text(id)],
                    false,
                ),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for result in &resp.results {
            if let PipelineResult::Error { error } = result {
                bail!("Retry task error: {}", error.message);
            }
        }

        Ok(())
    }

    pub async fn move_to_dead_queue(&self, task: &QueueTask, error_msg: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let resp = self
            .execute_pipeline(vec![
                Self::make_stmt(
                    "INSERT INTO __forte_dead_queue (id, task_name, payload, error_message, retry_count, created_at, died_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    vec![
                        Self::text(&task.id),
                        Self::text(&task.task_name),
                        Self::text(&task.payload),
                        Self::text(error_msg),
                        Value::Integer { value: task.retry_count + 1 },
                        Self::text(&now),
                        Self::text(&now),
                    ],
                    false,
                ),
                Self::make_stmt(
                    "DELETE FROM __forte_queue WHERE id = ?",
                    vec![Self::text(&task.id)],
                    false,
                ),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for result in &resp.results {
            if let PipelineResult::Error { error } = result {
                bail!("Move to dead queue error: {}", error.message);
            }
        }

        Ok(())
    }

    pub async fn dead_queue_count(&self) -> Result<Vec<(String, u64)>> {
        let resp = self
            .execute_pipeline(vec![
                Self::make_stmt(
                    "SELECT task_name, COUNT(*) as count FROM __forte_dead_queue GROUP BY task_name",
                    vec![],
                    true,
                ),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        let Some(result) = Self::extract_execute_result(&resp)? else {
            return Ok(vec![]);
        };

        let mut counts = Vec::new();
        for row in &result.rows {
            if row.len() >= 2 {
                let task_name = Self::row_to_string(&row[0]).unwrap_or_default();
                let count = Self::row_to_i64(&row[1]).unwrap_or(0) as u64;
                counts.push((task_name, count));
            }
        }

        Ok(counts)
    }

    pub async fn flush_dead_queue(&self, task_name: Option<&str>) -> Result<u64> {
        let now = chrono::Utc::now().to_rfc3339();

        let (select_sql, insert_sql, delete_sql) = match task_name {
            Some(name) => (
                format!("SELECT id, task_name, payload, created_at FROM __forte_dead_queue WHERE task_name = '{}'", name),
                "INSERT INTO __forte_queue (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) SELECT id, task_name, payload, 'pending', 0, 3, created_at, ? FROM __forte_dead_queue WHERE task_name = ?".to_string(),
                "DELETE FROM __forte_dead_queue WHERE task_name = ?".to_string(),
            ),
            None => (
                "SELECT id, task_name, payload, created_at FROM __forte_dead_queue".to_string(),
                "INSERT INTO __forte_queue (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) SELECT id, task_name, payload, 'pending', 0, 3, created_at, ? FROM __forte_dead_queue".to_string(),
                "DELETE FROM __forte_dead_queue".to_string(),
            ),
        };

        let _ = select_sql;

        let (insert_args, delete_args) = match task_name {
            Some(name) => (
                vec![Self::text(&now), Self::text(name)],
                vec![Self::text(name)],
            ),
            None => (vec![Self::text(&now)], vec![]),
        };

        let resp = self
            .execute_pipeline(vec![
                Self::make_stmt(&insert_sql, insert_args, false),
                Self::make_stmt(&delete_sql, delete_args, false),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        let mut flushed = 0u64;
        for result in &resp.results {
            match result {
                PipelineResult::Ok { response } => {
                    if let PipelineOkResponse::Execute { result } = response
                        && result.affected_row_count > 0
                        && flushed == 0
                    {
                        flushed = result.affected_row_count;
                    }
                }
                PipelineResult::Error { error } => {
                    bail!("Flush dead queue error: {}", error.message);
                }
            }
        }

        Ok(flushed)
    }
}
