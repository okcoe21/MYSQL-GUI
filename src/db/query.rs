use std::time::Instant;
use sqlx::{Executor, Column, Row};
use serde_json::Value;
use crate::state::AppState;
use crate::db::sanitize::is_destructive;
use crate::db::models::{row_to_json, QueryResult};

pub fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_backtick = false;
    let mut in_escape = false;
    let mut chars = sql.chars().peekable();
    
    while let Some(c) = chars.next() {
        if in_escape {
            current.push(c);
            in_escape = false;
            continue;
        }
        
        match c {
            '\\' => {
                current.push(c);
                if in_single_quote || in_double_quote {
                    in_escape = true;
                }
            }
            '\'' if !in_double_quote && !in_backtick => {
                in_single_quote = !in_single_quote;
                current.push(c);
            }
            '"' if !in_single_quote && !in_backtick => {
                in_double_quote = !in_double_quote;
                current.push(c);
            }
            '`' if !in_single_quote && !in_double_quote => {
                in_backtick = !in_backtick;
                current.push(c);
            }
            '-' if !in_single_quote && !in_double_quote && !in_backtick => {
                if let Some(&'-') = chars.peek() {
                    chars.next();
                    current.push('-');
                    current.push('-');
                    while let Some(nc) = chars.next() {
                        current.push(nc);
                        if nc == '\n' {
                            break;
                        }
                    }
                } else {
                    current.push(c);
                }
            }
            '#' if !in_single_quote && !in_double_quote && !in_backtick => {
                current.push(c);
                while let Some(nc) = chars.next() {
                    current.push(nc);
                    if nc == '\n' {
                        break;
                    }
                }
            }
            '/' if !in_single_quote && !in_double_quote && !in_backtick => {
                if let Some(&'*') = chars.peek() {
                    chars.next();
                    current.push('/');
                    current.push('*');
                    while let Some(nc) = chars.next() {
                        current.push(nc);
                        if nc == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                current.push('/');
                                break;
                            }
                        }
                    }
                } else {
                    current.push(c);
                }
            }
            ';' if !in_single_quote && !in_double_quote && !in_backtick => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }
    
    statements.into_iter().filter(|s| {
        let mut text = s.clone();
        while let Some(start) = text.find("/*") {
            if let Some(end) = text[start..].find("*/") {
                text.replace_range(start..start + end + 2, "");
            } else {
                break;
            }
        }
        text = text.lines()
            .map(|line| {
                let trimmed_line = line.trim();
                if trimmed_line.starts_with("--") || trimmed_line.starts_with("#") {
                    ""
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
            
        !text.trim().is_empty()
    }).collect()
}

pub async fn execute_query(
    state: &AppState,
    db: Option<&str>,
    sql: &str,
    confirmed: bool,
) -> Result<QueryResult, String> {
    if is_destructive(sql) && !confirmed {
        return Ok(QueryResult {
            success: false,
            data: None,
            columns: Vec::new(),
            affected_rows: None,
            error: Some("DESTRUCTIVE_QUERY".into()),
            message: Some("This query contains potentially destructive operations (DROP, DELETE, TRUNCATE, ALTER). Please confirm execution.".into()),
            execution_time_ms: 0,
        });
    }
    
    let mut conn = state.get_connection(db).await?;
    let start = Instant::now();
    
    let sql_upper = sql.trim().to_uppercase();
    if sql_upper.starts_with("SELECT") 
        || sql_upper.starts_with("SHOW") 
        || sql_upper.starts_with("DESCRIBE") 
        || sql_upper.starts_with("EXPLAIN") 
    {
        let rows = sqlx::query(sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;
            
        let elapsed = start.elapsed().as_millis() as u64;
        let data: Vec<Value> = rows.iter().map(row_to_json).collect();
        let columns = if !rows.is_empty() {
            rows[0].columns().iter().map(|c| c.name().to_string()).collect()
        } else {
            Vec::new()
        };
        
        Ok(QueryResult {
            success: true,
            data: Some(data),
            columns,
            affected_rows: None,
            error: None,
            message: None,
            execution_time_ms: elapsed,
        })
    } else {
        let result = conn.execute(sql).await.map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(QueryResult {
            success: true,
            data: None,
            columns: Vec::new(),
            affected_rows: Some(result.rows_affected()),
            error: None,
            message: Some(format!("Query executed successfully (affected {} rows)", result.rows_affected())),
            execution_time_ms: elapsed,
        })
    }
}
