use crate::credentials;
use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};

pub struct DocQueryStatement {
    pub sql: String,
    pub args: Vec<serde_json::Value>,
}

pub enum DocQueryOutcome {
    Committed {
        statement_results: Vec<DocQueryStatementResult>,
    },
    RolledBack {
        failed_statement_index: usize,
        error_message: String,
    },
}

#[derive(Serialize)]
pub struct DocQueryStatementResult {
    pub column_names: Vec<String>,
    pub rows: Vec<Vec<DocQueryCell>>,
    pub rows_truncated: bool,
    pub affected_row_count: u64,
    pub rows_read: u64,
    pub rows_written: u64,
    pub query_duration_ms: f64,
}

#[derive(Serialize)]
pub enum DocQueryCell {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(Serialize)]
struct DocQueryInput<'a> {
    project_id: &'a str,
    statements: Vec<DocQueryInputStatement<'a>>,
}

#[derive(Serialize)]
struct DocQueryInputStatement<'a> {
    sql: &'a str,
    args: &'a [serde_json::Value],
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum DocQueryResponse {
    Committed {
        statement_results: Vec<WireStatementResult>,
    },
    RolledBack {
        failed_statement_index: usize,
        error_message: String,
    },
    NotLoggedIn,
    NotFound,
    Forbidden,
    TooManyStatements {
        max: usize,
    },
    InvalidArgument {
        statement_index: usize,
        argument_index: usize,
        reason: String,
    },
    InternalError {
        reason: String,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireStatementResult {
    column_names: Vec<String>,
    rows: Vec<Vec<WireCell>>,
    rows_truncated: bool,
    affected_row_count: u64,
    rows_read: u64,
    rows_written: u64,
    query_duration_ms: f64,
}

#[derive(Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
enum WireCell {
    Null,
    Integer { value: i64 },
    Float { value: f64 },
    Text { value: String },
    Blob { base64: String },
}

pub async fn doc_query(
    project_id: &str,
    statements: &[DocQueryStatement],
    timeout_secs: u64,
) -> Result<DocQueryOutcome> {
    let creds = credentials::require()?;

    let url = format!(
        "{}/__forte_action/doc_query",
        creds.control_url.trim_end_matches('/')
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;

    let raw: DocQueryResponse = client
        .post(&url)
        .bearer_auth(&creds.token)
        .json(&DocQueryInput {
            project_id,
            statements: statements
                .iter()
                .map(|statement| DocQueryInputStatement {
                    sql: &statement.sql,
                    args: &statement.args,
                })
                .collect(),
        })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("doc query control call failed: {e}"))?
        .json()
        .await?;

    match raw {
        DocQueryResponse::Committed { statement_results } => Ok(DocQueryOutcome::Committed {
            statement_results: statement_results
                .into_iter()
                .map(parse_statement_result)
                .collect::<Result<Vec<_>>>()?,
        }),
        DocQueryResponse::RolledBack {
            failed_statement_index,
            error_message,
        } => Ok(DocQueryOutcome::RolledBack {
            failed_statement_index,
            error_message,
        }),
        DocQueryResponse::NotLoggedIn => {
            Err(anyhow!("control rejected token; run `forte login` again."))
        }
        DocQueryResponse::NotFound => Err(anyhow!("project '{project_id}' not found.")),
        DocQueryResponse::Forbidden => Err(anyhow!(
            "project '{project_id}' is not owned by the signed-in user."
        )),
        DocQueryResponse::TooManyStatements { max } => Err(anyhow!(
            "too many statements in one request; the limit is {max}."
        )),
        DocQueryResponse::InvalidArgument {
            statement_index,
            argument_index,
            reason,
        } => Err(anyhow!(
            "invalid argument {argument_index} of statement {statement_index}: {reason}"
        )),
        DocQueryResponse::InternalError { reason } => {
            Err(anyhow!("control doc_query internal error: {reason}"))
        }
    }
}

fn parse_statement_result(wire: WireStatementResult) -> Result<DocQueryStatementResult> {
    let rows = wire
        .rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|cell| {
                    Ok(match cell {
                        WireCell::Null => DocQueryCell::Null,
                        WireCell::Integer { value } => DocQueryCell::Integer(value),
                        WireCell::Float { value } => DocQueryCell::Float(value),
                        WireCell::Text { value } => DocQueryCell::Text(value),
                        WireCell::Blob { base64 } => DocQueryCell::Blob(
                            STANDARD
                                .decode(&base64)
                                .map_err(|e| anyhow!("blob cell base64 decode: {e}"))?,
                        ),
                    })
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(DocQueryStatementResult {
        column_names: wire.column_names,
        rows,
        rows_truncated: wire.rows_truncated,
        affected_row_count: wire.affected_row_count,
        rows_read: wire.rows_read,
        rows_written: wire.rows_written,
        query_duration_ms: wire.query_duration_ms,
    })
}
