use super::project_config;
use anyhow::{Result, anyhow, bail};
use fn0_deploy::{DocQueryCell, DocQueryOutcome, DocQueryStatement, DocQueryStatementResult};
use std::path::{Path, PathBuf};

pub async fn query(
    sql: String,
    raw_args: Vec<String>,
    project_dir: PathBuf,
    json: bool,
    timeout_seconds: u64,
) -> Result<()> {
    let args = raw_args
        .into_iter()
        .map(|raw_arg| serde_json::from_str(&raw_arg).unwrap_or(serde_json::Value::String(raw_arg)))
        .collect();
    run(
        &project_dir,
        vec![DocQueryStatement { sql, args }],
        json,
        timeout_seconds,
    )
    .await
}

pub async fn exec(
    file: PathBuf,
    project_dir: PathBuf,
    json: bool,
    timeout_seconds: u64,
) -> Result<()> {
    let sql_text = std::fs::read_to_string(&file)
        .map_err(|e| anyhow!("Failed to read {}: {}", file.display(), e))?;
    let statements: Vec<DocQueryStatement> = split_sql_statements(&sql_text)
        .into_iter()
        .map(|sql| DocQueryStatement { sql, args: vec![] })
        .collect();
    if statements.is_empty() {
        bail!("{} contains no SQL statements", file.display());
    }
    run(&project_dir, statements, json, timeout_seconds).await
}

async fn run(
    project_dir: &Path,
    statements: Vec<DocQueryStatement>,
    json: bool,
    timeout_seconds: u64,
) -> Result<()> {
    let project_id = project_config::read_project_id(project_dir)?;

    let outcome = fn0_deploy::doc_query(&project_id, &statements, timeout_seconds).await?;

    match outcome {
        DocQueryOutcome::Committed { statement_results } => {
            if json {
                println!("{}", serde_json::to_string_pretty(&statement_results)?);
            } else {
                print_statement_results(&statement_results);
            }
            Ok(())
        }
        DocQueryOutcome::RolledBack {
            failed_statement_index,
            error_message,
        } => {
            eprintln!(
                "rolled back: statement {} failed: {}",
                failed_statement_index + 1,
                error_message
            );
            Err(anyhow!("transaction rolled back; nothing was applied"))
        }
    }
}

fn print_statement_results(statement_results: &[DocQueryStatementResult]) {
    for (statement_index, statement_result) in statement_results.iter().enumerate() {
        if statement_results.len() > 1 {
            println!("-- statement {}", statement_index + 1);
        }
        if !statement_result.column_names.is_empty() {
            println!("{}", statement_result.column_names.join("\t"));
            for row in &statement_result.rows {
                let rendered: Vec<String> = row.iter().map(render_cell).collect();
                println!("{}", rendered.join("\t"));
            }
            if statement_result.rows_truncated {
                println!("(rows truncated; narrow the query or add LIMIT to see the rest)");
            }
        }
        println!(
            "-- affected: {}, rows read: {}, rows written: {}, {:.1}ms",
            statement_result.affected_row_count,
            statement_result.rows_read,
            statement_result.rows_written,
            statement_result.query_duration_ms,
        );
    }
    if statement_results.len() > 1 {
        let total_rows_read: u64 = statement_results.iter().map(|r| r.rows_read).sum();
        let total_rows_written: u64 = statement_results.iter().map(|r| r.rows_written).sum();
        println!("-- total rows read: {total_rows_read}, total rows written: {total_rows_written}");
    }
}

fn render_cell(cell: &DocQueryCell) -> String {
    match cell {
        DocQueryCell::Null => "NULL".to_string(),
        DocQueryCell::Integer(value) => value.to_string(),
        DocQueryCell::Float(value) => value.to_string(),
        DocQueryCell::Text(value) => value.clone(),
        DocQueryCell::Blob(bytes) => match std::str::from_utf8(bytes) {
            Ok(text) => text.to_string(),
            Err(_) => format!("<blob {} bytes>", bytes.len()),
        },
    }
}

fn split_sql_statements(sql_text: &str) -> Vec<String> {
    enum State {
        Normal,
        SingleQuote,
        DoubleQuote,
        Backtick,
        BracketIdentifier,
        LineComment,
        BlockComment,
    }

    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut state = State::Normal;
    let mut characters = sql_text.chars().peekable();

    while let Some(character) = characters.next() {
        match state {
            State::Normal => match character {
                ';' => {
                    let trimmed = current_statement.trim();
                    if !trimmed.is_empty() {
                        statements.push(trimmed.to_string());
                    }
                    current_statement.clear();
                    continue;
                }
                '\'' => state = State::SingleQuote,
                '"' => state = State::DoubleQuote,
                '`' => state = State::Backtick,
                '[' => state = State::BracketIdentifier,
                '-' if characters.peek() == Some(&'-') => state = State::LineComment,
                '/' if characters.peek() == Some(&'*') => state = State::BlockComment,
                _ => {}
            },
            State::SingleQuote => {
                if character == '\'' {
                    if characters.peek() == Some(&'\'') {
                        current_statement.push(characters.next().unwrap());
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::DoubleQuote => {
                if character == '"' {
                    if characters.peek() == Some(&'"') {
                        current_statement.push(characters.next().unwrap());
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::Backtick => {
                if character == '`' {
                    state = State::Normal;
                }
            }
            State::BracketIdentifier => {
                if character == ']' {
                    state = State::Normal;
                }
            }
            State::LineComment => {
                if character == '\n' {
                    state = State::Normal;
                }
            }
            State::BlockComment => {
                if character == '*' && characters.peek() == Some(&'/') {
                    current_statement.push(character);
                    current_statement.push(characters.next().unwrap());
                    state = State::Normal;
                    continue;
                }
            }
        }
        current_statement.push(character);
    }

    let trimmed = current_statement.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }
    statements
}

#[cfg(test)]
mod tests {
    use super::split_sql_statements;

    #[test]
    fn splits_on_semicolons() {
        let statements = split_sql_statements("SELECT 1; SELECT 2;");
        assert_eq!(statements, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn keeps_semicolons_inside_strings() {
        let statements =
            split_sql_statements("UPDATE docs SET data = 'a;b' WHERE pk = \"weird;name\";");
        assert_eq!(
            statements,
            vec!["UPDATE docs SET data = 'a;b' WHERE pk = \"weird;name\""]
        );
    }

    #[test]
    fn keeps_escaped_quotes_inside_strings() {
        let statements = split_sql_statements("SELECT 'it''s;fine'; SELECT 2;");
        assert_eq!(statements, vec!["SELECT 'it''s;fine'", "SELECT 2"]);
    }

    #[test]
    fn ignores_semicolons_inside_comments() {
        let statements =
            split_sql_statements("SELECT 1 -- trailing; comment\n; /* block; comment */ SELECT 2;");
        assert_eq!(
            statements,
            vec![
                "SELECT 1 -- trailing; comment",
                "/* block; comment */ SELECT 2"
            ]
        );
    }

    #[test]
    fn keeps_statement_without_trailing_semicolon() {
        let statements = split_sql_statements("SELECT 1;\nSELECT 2");
        assert_eq!(statements, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn drops_whitespace_only_statements() {
        let statements = split_sql_statements(" ;;\n;  SELECT 1;\n\n");
        assert_eq!(statements, vec!["SELECT 1"]);
    }
}
