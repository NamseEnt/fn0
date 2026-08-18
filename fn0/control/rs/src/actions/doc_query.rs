//! Runs owner-submitted SQL against a project's doc-db, through control,
//! so the platform sees the engine's `rows_read` / `rows_written` counters
//! for every statement — the numbers a future quota system will charge.
//! Users never receive Turso credentials; control speaks hrana on their
//! behalf with the worker-grade group token.

use crate::common::auth;
use crate::docs::*;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const MAX_STATEMENTS_PER_REQUEST: usize = 100;
const RESPONSE_ROWS_BYTE_BUDGET: usize = 2 * 1024 * 1024;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub statements: Vec<InputStatement>,
}

#[derive(Deserialize)]
pub struct InputStatement {
    pub sql: String,
    pub args: Vec<serde_json::Value>,
}

#[derive(Serialize)]
pub enum Output {
    Committed {
        statement_results: Vec<StatementResult>,
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

#[derive(Serialize)]
pub struct StatementResult {
    pub column_names: Vec<String>,
    pub rows: Vec<Vec<CellValue>>,
    pub rows_truncated: bool,
    pub affected_row_count: u64,
    pub rows_read: u64,
    pub rows_written: u64,
    pub query_duration_ms: f64,
}

#[derive(Serialize)]
pub enum CellValue {
    Null,
    Integer { value: i64 },
    Float { value: f64 },
    Text { value: String },
    Blob { base64: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let db = doc_db::turso();

    let project = match (ProjectDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return Output::NotFound,
        Err(e) => {
            tracing::error!(?e, "doc_query ProjectDocGet");
            return Output::InternalError {
                reason: format!("ProjectDocGet: {e}"),
            };
        }
    };

    if project.owner_github_id != user.github_id {
        return Output::Forbidden;
    }

    if req.body.statements.len() > MAX_STATEMENTS_PER_REQUEST {
        return Output::TooManyStatements {
            max: MAX_STATEMENTS_PER_REQUEST,
        };
    }

    let mut statements = Vec::with_capacity(req.body.statements.len());
    for (statement_index, statement) in req.body.statements.iter().enumerate() {
        let mut args = Vec::with_capacity(statement.args.len());
        for (argument_index, argument) in statement.args.iter().enumerate() {
            match json_argument_to_hrana_value(argument) {
                Ok(value) => args.push(value),
                Err(reason) => {
                    return Output::InvalidArgument {
                        statement_index,
                        argument_index,
                        reason,
                    };
                }
            }
        }
        statements.push(doc_db::RawStatement {
            sql: statement.sql.clone(),
            args,
        });
    }

    let group_token = match std::env::var("FN0_TURSO_GROUP_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            return Output::InternalError {
                reason: "FN0_TURSO_GROUP_TOKEN not set".to_string(),
            };
        }
    };
    let host_suffix = match std::env::var("FN0_TURSO_DB_HOST_SUFFIX") {
        Ok(suffix) => suffix,
        Err(_) => {
            return Output::InternalError {
                reason: "FN0_TURSO_DB_HOST_SUFFIX not set".to_string(),
            };
        }
    };

    let project_db = doc_db::turso_with_config(
        format!(
            "https://{project_id}{host_suffix}",
            project_id = req.body.project_id
        ),
        group_token,
    );

    let outcome = match project_db.execute_raw_transactional(&statements).await {
        Ok(outcome) => outcome,
        Err(e) => {
            tracing::error!(?e, "doc_query execute_raw_transactional");
            return Output::InternalError {
                reason: format!("execute_raw_transactional: {e}"),
            };
        }
    };

    match outcome {
        doc_db::RawTransactionOutcome::Committed { statement_results } => {
            let total_rows_read: u64 = statement_results.iter().map(|r| r.rows_read).sum();
            let total_rows_written: u64 = statement_results.iter().map(|r| r.rows_written).sum();
            tracing::info!(
                project_id = %req.body.project_id,
                total_rows_read,
                total_rows_written,
                "doc_query usage"
            );
            Output::Committed {
                statement_results: render_statement_results(statement_results),
            }
        }
        doc_db::RawTransactionOutcome::RolledBack {
            failed_statement_index,
            error_message,
        } => Output::RolledBack {
            failed_statement_index,
            error_message,
        },
    }
}

fn json_argument_to_hrana_value(argument: &serde_json::Value) -> Result<doc_db::Value, String> {
    match argument {
        serde_json::Value::Null => Ok(doc_db::Value::Null),
        serde_json::Value::String(text) => Ok(doc_db::text_value(text.clone())),
        serde_json::Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                Ok(doc_db::integer_value(integer))
            } else if let Some(float) = number.as_f64() {
                Ok(doc_db::Value::Float { value: float })
            } else {
                Err("number does not fit in i64 or f64".to_string())
            }
        }
        serde_json::Value::Bool(_) => {
            Err("booleans are not supported; SQLite has no boolean type, use 0 or 1".to_string())
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            Err("only null, numbers, and strings are supported as arguments".to_string())
        }
    }
}

fn render_statement_results(
    statement_results: Vec<doc_db::RawStatementResult>,
) -> Vec<StatementResult> {
    let mut remaining_budget = RESPONSE_ROWS_BYTE_BUDGET;
    statement_results
        .into_iter()
        .map(|statement_result| {
            let mut rows = Vec::with_capacity(statement_result.rows.len());
            let mut rows_truncated = false;
            for row in statement_result.rows {
                let row_cost: usize = row.iter().map(cell_byte_cost).sum();
                if row_cost > remaining_budget {
                    rows_truncated = true;
                    break;
                }
                remaining_budget -= row_cost;
                rows.push(row.into_iter().map(hrana_value_to_cell).collect());
            }
            StatementResult {
                column_names: statement_result.column_names,
                rows,
                rows_truncated,
                affected_row_count: statement_result.affected_row_count,
                rows_read: statement_result.rows_read,
                rows_written: statement_result.rows_written,
                query_duration_ms: statement_result.query_duration_ms,
            }
        })
        .collect()
}

fn cell_byte_cost(value: &doc_db::Value) -> usize {
    match value {
        doc_db::Value::None | doc_db::Value::Null => 24,
        doc_db::Value::Integer { .. } | doc_db::Value::Float { .. } => 24,
        doc_db::Value::Text { value } => value.len() + 24,
        doc_db::Value::Blob { value } => value.len() / 3 * 4 + 24,
    }
}

fn hrana_value_to_cell(value: doc_db::Value) -> CellValue {
    match value {
        doc_db::Value::None | doc_db::Value::Null => CellValue::Null,
        doc_db::Value::Integer { value } => CellValue::Integer { value },
        doc_db::Value::Float { value } => CellValue::Float { value },
        doc_db::Value::Text { value } => CellValue::Text {
            value: value.to_string(),
        },
        doc_db::Value::Blob { value } => CellValue::Blob {
            base64: STANDARD.encode(&value),
        },
    }
}
