use std::sync::Mutex;
use sqlx::{MySqlPool, mysql::{MySqlPoolOptions, MySqlRow}, Executor, Row, Column, TypeInfo, ValueRef};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use rand::seq::SliceRandom;
use rand::Rng;

pub struct AppState {
    pub pool: Mutex<Option<MySqlPool>>,
    pub current_db: Mutex<Option<String>>,
}

// Structs for payload serialization/deserialization

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub success: bool,
    pub data: Option<Value>,
    pub columns: Vec<String>,
    pub affected_rows: Option<u64>,
    pub error: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct PaginationInfo {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize)]
pub struct GetDataResponse {
    pub success: bool,
    pub data: Value,
    pub columns: Vec<String>,
    pub pagination: PaginationInfo,
}

#[derive(Serialize)]
pub struct ServerStatusResponse {
    pub success: bool,
    pub processes: Value,
    pub status: ServerStatus,
}

#[derive(Serialize)]
pub struct ServerStatus {
    pub uptime: i64,
    pub threads_connected: i64,
    pub threads_running: i64,
    pub queries: i64,
    pub slow_queries: i64,
}

#[derive(Serialize)]
pub struct ServerMetricsResponse {
    pub success: bool,
    pub timestamp: u64,
    pub metrics: ServerMetrics,
}

#[derive(Serialize)]
pub struct ServerMetrics {
    pub questions: i64,
    pub threads_connected: i64,
    pub threads_running: i64,
    pub bytes_received: i64,
    pub bytes_sent: i64,
    pub innodb_total: i64,
    pub innodb_free: i64,
    pub slow_queries: i64,
    pub uptime: i64,
}

#[derive(Deserialize)]
pub struct ColumnDefInput {
    pub name: String,
    pub r#type: String,
    pub length: Option<String>,
    pub is_null: Option<bool>,
    pub is_primary: Option<bool>,
    pub is_auto_increment: Option<bool>,
}

#[derive(Serialize)]
pub struct DbStats {
    pub tableName: String,
    pub rowCount: i64,
    pub dataSize: i64,
    pub engine: String,
    pub collation: String,
}

#[derive(Serialize)]
pub struct ObjectsSummary {
    pub success: bool,
    pub views: Vec<String>,
    pub procedures: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub setup_required: Option<bool>,
    pub message: Option<String>,
}

// Helpers

fn sanitize_identifier(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Invalid identifier scope".to_string());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
        return Err(format!("Invalid identifier name: {}", name));
    }
    Ok(format!("`{}`", name))
}

fn is_destructive(query: &str) -> bool {
    let upper = query.to_uppercase();
    upper.contains("DROP ") || upper.contains("DELETE ") || upper.contains("TRUNCATE ") || upper.contains("ALTER ")
}

async fn get_connection(state: &AppState, db: Option<&str>) -> Result<sqlx::pool::PoolConnection<sqlx::MySql>, String> {
    let pool = {
        let pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
        pool_guard.as_ref().ok_or("Not logged in. No active database connection pool.")?.clone()
    };
    
    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
    
    if let Some(db_name) = db {
        let sanitized = sanitize_identifier(db_name)?;
        let use_query = format!("USE {}", sanitized);
        conn.execute(use_query.as_str()).await.map_err(|e| e.to_string())?;
    }
    
    Ok(conn)
}

fn row_to_json(row: &MySqlRow) -> Value {
    let mut map = serde_json::Map::new();
    for col in row.columns() {
        let name = col.name();
        let value = if row.try_get_raw(name).map(|v| v.is_null()).unwrap_or(true) {
            Value::Null
        } else {
            let type_name = col.type_info().name();
            if type_name.contains("INT") || type_name == "INTEGER" {
                if let Ok(val) = row.try_get::<i64, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u64, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<i32, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u32, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<i16, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u16, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<i8, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u8, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else {
                    Value::Null
                }
            } else {
                match type_name {
                    "FLOAT" | "DOUBLE" | "DECIMAL" => {
                        if let Ok(val) = row.try_get::<f64, _>(name) {
                            if let Some(num) = serde_json::Number::from_f64(val) {
                                Value::Number(num)
                            } else {
                                Value::Null
                            }
                        } else {
                            Value::Null
                        }
                    }
                    "VARCHAR" | "CHAR" | "TEXT" | "LONGTEXT" | "MEDIUMTEXT" | "TINYTEXT" | "ENUM" | "SET" => {
                        if let Ok(val) = row.try_get::<String, _>(name) {
                            Value::String(val)
                        } else {
                            Value::Null
                        }
                    }
                    "DATE" | "DATETIME" | "TIMESTAMP" | "TIME" => {
                        if let Ok(val) = row.try_get::<chrono::NaiveDateTime, _>(name) {
                            Value::String(val.format("%Y-%m-%d %H:%M:%S").to_string())
                        } else if let Ok(val) = row.try_get::<chrono::NaiveDate, _>(name) {
                            Value::String(val.to_string())
                        } else if let Ok(val) = row.try_get::<String, _>(name) {
                            Value::String(val)
                        } else {
                            Value::Null
                        }
                    }
                    _ => {
                        if let Ok(val) = row.try_get::<String, _>(name) {
                            Value::String(val)
                        } else if let Ok(val) = row.try_get::<i64, _>(name) {
                            Value::Number(serde_json::Number::from(val))
                        } else if let Ok(val) = row.try_get::<f64, _>(name) {
                            if let Some(num) = serde_json::Number::from_f64(val) {
                                Value::Number(num)
                            } else {
                                Value::Null
                            }
                        } else if let Ok(val) = row.try_get::<Vec<u8>, _>(name) {
                            if let Ok(s) = String::from_utf8(val) {
                                Value::String(s)
                            } else {
                                Value::String("[Binary data]".to_string())
                            }
                        } else {
                            Value::Null
                        }
                    }
                }
            }
        };
        map.insert(name.to_string(), value);
    }
    Value::Object(map)
}

fn split_sql_statements(sql: &str) -> Vec<String> {
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

fn generate_mock_value(field_type: &str) -> String {
    let mut rng = rand::thread_rng();
    match field_type {
        "name" => {
            let first_names = ["John", "Jane", "Michael", "Sarah", "Chris", "Emma", "David", "Olivia"];
            let last_names = ["Smith", "Johnson", "Brown", "Taylor", "Miller", "Wilson", "Moore"];
            format!("{} {}", first_names.choose(&mut rng).unwrap_or(&"John"), last_names.choose(&mut rng).unwrap_or(&"Smith"))
        }
        "email" => {
            let domains = ["example.com", "test.org", "gmail.com", "outlook.com"];
            let r: String = (0..7).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect();
            format!("{}@{}", r.to_lowercase(), domains.choose(&mut rng).unwrap_or(&"example.com"))
        }
        "phone" => {
            format!("+1-{}-{}-{}", rng.gen_range(100..1000), rng.gen_range(100..1000), rng.gen_range(1000..10000))
        }
        "date" => {
            let start = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
            let end = chrono::Utc::now().timestamp();
            let random_ts = rng.gen_range(start..end);
            let dt = chrono::DateTime::from_timestamp(random_ts, 0).unwrap().naive_utc();
            dt.date().to_string()
        }
        "datetime" => {
            let start = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
            let end = chrono::Utc::now().timestamp();
            let random_ts = rng.gen_range(start..end);
            let dt = chrono::DateTime::from_timestamp(random_ts, 0).unwrap().naive_utc();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        "integer" => {
            rng.gen_range(0..10000).to_string()
        }
        _ => {
            let words = ["lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit"];
            let chosen: Vec<&str> = (0..5).map(|_| *words.choose(&mut rng).unwrap_or(&"lorem")).collect();
            chosen.join(" ")
        }
    }
}

// Commands

#[tauri::command]
async fn cmd_auth_login(
    state: tauri::State<'_, AppState>,
    host: String,
    port: String,
    user: String,
    password: Option<String>,
) -> Result<bool, String> {
    let port_num = port.parse::<u16>().map_err(|_| "Invalid port number")?;
    let pool_options = MySqlPoolOptions::new()
        .max_connections(5)
        .idle_timeout(std::time::Duration::from_secs(60));
        
    let url = format!(
        "mysql://{}:{}@{}:{}/",
        user,
        password.clone().unwrap_or_default(),
        host,
        port_num
    );
    
    let pool = pool_options.connect(&url).await.map_err(|e| e.to_string())?;
    
    let mut pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    *pool_guard = Some(pool);
    
    Ok(true)
}

#[tauri::command]
async fn cmd_auth_logout(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let mut pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    *pool_guard = None;
    let mut db_guard = state.current_db.lock().map_err(|e| e.to_string())?;
    *db_guard = None;
    Ok(true)
}

#[tauri::command]
async fn cmd_list_databases(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut conn = get_connection(&state, None).await?;
    let rows = sqlx::query("SHOW DATABASES")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let list = rows.iter().map(|row| {
        let db_name: String = row.try_get(0).unwrap_or_default();
        db_name
    }).collect();
    
    Ok(list)
}

#[tauri::command]
async fn cmd_create_database(state: tauri::State<'_, AppState>, name: String) -> Result<bool, String> {
    let mut conn = get_connection(&state, None).await?;
    let sanitized = sanitize_identifier(&name)?;
    let query = format!("CREATE DATABASE {}", sanitized);
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn cmd_drop_database(
    state: tauri::State<'_, AppState>,
    name: String,
    confirmed: bool,
) -> Result<Value, String> {
    if !confirmed {
        return Ok(serde_json::json!({
            "success": false,
            "error": "CONFIRMATION_REQUIRED",
            "message": format!("Are you sure you want to DROP the entire database [{}]? All tables and data will be permanently deleted.", name)
        }));
    }
    
    let mut conn = get_connection(&state, None).await?;
    let sanitized = sanitize_identifier(&name)?;
    let query = format!("DROP DATABASE {}", sanitized);
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn cmd_list_tables(state: tauri::State<'_, AppState>, db: String) -> Result<Vec<String>, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let rows = sqlx::query("SHOW TABLES")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let list = rows.iter().map(|row| {
        let table_name: String = row.try_get(0).unwrap_or_default();
        table_name
    }).collect();
    
    Ok(list)
}

#[tauri::command]
async fn cmd_create_table(
    state: tauri::State<'_, AppState>,
    db: String,
    name: String,
    columns: Vec<ColumnDefInput>,
) -> Result<bool, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&name)?;
    
    let mut primary_keys = Vec::new();
    let mut col_defs = Vec::new();
    
    let valid_types = [
        "INT", "TINYINT", "SMALLINT", "MEDIUMINT", "BIGINT", "DECIMAL", "FLOAT", "DOUBLE", "REAL", "BIT", "BOOLEAN", "SERIAL",
        "DATE", "DATETIME", "TIMESTAMP", "TIME", "YEAR",
        "CHAR", "VARCHAR", "TINYTEXT", "TEXT", "MEDIUMTEXT", "LONGTEXT", "BINARY", "VARBINARY", "TINYBLOB", "BLOB", "MEDIUMBLOB", "LONGBLOB", "ENUM", "SET", "JSON"
    ];
    
    for col in columns {
        let col_name = sanitize_identifier(&col.name)?;
        let t_upper = col.r#type.to_uppercase();
        if !valid_types.contains(&t_upper.as_str()) {
            return Err(format!("Invalid data type: [{}]", col.r#type));
        }
        let length = col.length.map(|l| format!("({})", l)).unwrap_or_default();
        let is_null = if col.is_null.unwrap_or(true) { "NULL" } else { "NOT NULL" };
        let auto_inc = if col.is_auto_increment.unwrap_or(false) { "AUTO_INCREMENT" } else { "" };
        
        if col.is_primary.unwrap_or(false) {
            primary_keys.push(col_name.clone());
        }
        
        col_defs.push(format!("{} {}{} {} {}", col_name, t_upper, length, is_null, auto_inc).trim().to_string());
    }
    
    if !primary_keys.is_empty() {
        col_defs.push(format!("PRIMARY KEY ({})", primary_keys.join(", ")));
    }
    
    let query = format!("CREATE TABLE {} ({})", sanitized_table, col_defs.join(", "));
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn cmd_drop_table(
    state: tauri::State<'_, AppState>,
    db: String,
    name: String,
    confirmed: bool,
) -> Result<Value, String> {
    if !confirmed {
        return Ok(serde_json::json!({
            "success": false,
            "error": "CONFIRMATION_REQUIRED",
            "message": format!("Are you sure you want to DROP the table [{}]? This action cannot be undone.", name)
        }));
    }
    
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&name)?;
    let query = format!("DROP TABLE {}", sanitized_table);
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn cmd_get_data(
    state: tauri::State<'_, AppState>,
    db: String,
    table: String,
    limit: i64,
    offset: i64,
    sort_col: Option<String>,
    sort_order: Option<String>,
) -> Result<GetDataResponse, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&table)?;
    
    let mut query = format!("SELECT * FROM {}", sanitized_table);
    
    if let Some(col) = sort_col {
        let order = if sort_order.unwrap_or_default() == "DESC" { "DESC" } else { "ASC" };
        let sanitized_col = sanitize_identifier(&col)?;
        query += &format!(" ORDER BY {} {}", sanitized_col, order);
    }
    
    query += " LIMIT ? OFFSET ?";
    
    let rows = sqlx::query(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let data: Vec<Value> = rows.iter().map(row_to_json).collect();
    let columns = if !data.is_empty() {
        data[0].as_object().unwrap().keys().cloned().collect()
    } else {
        Vec::new()
    };
    
    let count_query = format!("SELECT COUNT(*) as total FROM {}", sanitized_table);
    let count_row = sqlx::query(&count_query)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let total: i64 = count_row.try_get(0).unwrap_or(0);
    
    Ok(GetDataResponse {
        success: true,
        data: Value::Array(data),
        columns,
        pagination: PaginationInfo {
            total,
            limit,
            offset,
        }
    })
}

#[tauri::command]
async fn cmd_insert_row(
    state: tauri::State<'_, AppState>,
    db: String,
    table: String,
    data: Value,
) -> Result<Value, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&table)?;
    
    let map = data.as_object().ok_or("Data must be a JSON object")?;
    let mut columns = Vec::new();
    let mut placeholders = Vec::new();
    
    for (k, _) in map {
        columns.push(sanitize_identifier(k)?);
        placeholders.push("?");
    }
    
    let query_str = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        sanitized_table,
        columns.join(", "),
        placeholders.join(", ")
    );
    
    let mut query = sqlx::query(&query_str);
    for (_, val) in map {
        if val.is_null() {
            query = query.bind(None::<String>);
        } else if let Some(s) = val.as_str() {
            query = query.bind(s.to_string());
        } else if let Some(n) = val.as_i64() {
            query = query.bind(n);
        } else if let Some(f) = val.as_f64() {
            query = query.bind(f);
        } else if let Some(b) = val.as_bool() {
            query = query.bind(b);
        } else {
            query = query.bind(val.to_string());
        }
    }
    
    let result = query.execute(&mut *conn).await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "insertId": result.last_insert_id(),
        "affectedRows": result.rows_affected()
    }))
}

#[tauri::command]
async fn cmd_update_row(
    state: tauri::State<'_, AppState>,
    db: String,
    table: String,
    column: String,
    value: Value,
    where_clause: Value,
) -> Result<bool, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&table)?;
    let sanitized_col = sanitize_identifier(&column)?;
    
    let where_map = where_clause.as_object().ok_or("Where-clause must be a JSON object")?;
    let mut where_parts = Vec::new();
    for (k, _) in where_map {
        where_parts.push(format!("{} = ?", sanitize_identifier(k)?));
    }
    
    let query_str = format!(
        "UPDATE {} SET {} = ? WHERE {} LIMIT 1",
        sanitized_table,
        sanitized_col,
        where_parts.join(" AND ")
    );
    
    let mut query = sqlx::query(&query_str);
    
    // Bind value to set
    if value.is_null() {
        query = query.bind(None::<String>);
    } else if let Some(s) = value.as_str() {
        query = query.bind(s.to_string());
    } else if let Some(n) = value.as_i64() {
        query = query.bind(n);
    } else if let Some(f) = value.as_f64() {
        query = query.bind(f);
    } else if let Some(b) = value.as_bool() {
        query = query.bind(b);
    } else {
        query = query.bind(value.to_string());
    }
    
    // Bind where clause values
    for (_, val) in where_map {
        if val.is_null() {
            query = query.bind(None::<String>);
        } else if let Some(s) = val.as_str() {
            query = query.bind(s.to_string());
        } else if let Some(n) = val.as_i64() {
            query = query.bind(n);
        } else if let Some(f) = val.as_f64() {
            query = query.bind(f);
        } else if let Some(b) = val.as_bool() {
            query = query.bind(b);
        } else {
            query = query.bind(val.to_string());
        }
    }
    
    query.execute(&mut *conn).await.map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn cmd_delete_row(
    state: tauri::State<'_, AppState>,
    db: String,
    table: String,
    where_clause: Value,
) -> Result<Value, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&table)?;
    
    let where_map = where_clause.as_object().ok_or("Where-clause must be a JSON object")?;
    let mut where_parts = Vec::new();
    for (k, _) in where_map {
        where_parts.push(format!("{} = ?", sanitize_identifier(k)?));
    }
    
    let query_str = format!(
        "DELETE FROM {} WHERE {} LIMIT 1",
        sanitized_table,
        where_parts.join(" AND ")
    );
    
    let mut query = sqlx::query(&query_str);
    for (_, val) in where_map {
        if val.is_null() {
            query = query.bind(None::<String>);
        } else if let Some(s) = val.as_str() {
            query = query.bind(s.to_string());
        } else if let Some(n) = val.as_i64() {
            query = query.bind(n);
        } else if let Some(f) = val.as_f64() {
            query = query.bind(f);
        } else if let Some(b) = val.as_bool() {
            query = query.bind(b);
        } else {
            query = query.bind(val.to_string());
        }
    }
    
    let result = query.execute(&mut *conn).await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": true,
        "affectedRows": result.rows_affected()
    }))
}

#[tauri::command]
async fn cmd_generate_mock_data(
    state: tauri::State<'_, AppState>,
    db: String,
    table: String,
    count: i64,
    blueprint: Value,
) -> Result<Value, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&table)?;
    
    let blueprint_map = blueprint.as_object().ok_or("Blueprint must be a JSON object")?;
    let columns: Vec<String> = blueprint_map.keys().cloned().collect();
    let sanitized_columns: Vec<String> = columns.iter().map(|c| sanitize_identifier(c)).collect::<Result<_, _>>()?;
    
    let batch_size = 100;
    let mut inserted = 0;
    
    for i in (0..count).step_by(batch_size) {
        let current_batch = std::cmp::min(batch_size as i64, count - i) as usize;
        let mut placeholders = Vec::new();
        let mut values = Vec::new();
        
        for _ in 0..current_batch {
            let mut row_placeholders = Vec::new();
            for col in &columns {
                let col_type = blueprint_map.get(col).and_then(|v| v.as_str()).unwrap_or("text");
                values.push(generate_mock_value(col_type));
                row_placeholders.push("?");
            }
            placeholders.push(format!("({})", row_placeholders.join(", ")));
        }
        
        let sql = format!(
            "INSERT INTO {} ({}) VALUES {}",
            sanitized_table,
            sanitized_columns.join(", "),
            placeholders.join(", ")
        );
        
        let mut query = sqlx::query(&sql);
        for val in values {
            query = query.bind(val);
        }
        query.execute(&mut *conn).await.map_err(|e| e.to_string())?;
        inserted += current_batch;
    }
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Successfully inserted {} rows into {}", inserted, table)
    }))
}

#[tauri::command]
async fn cmd_get_structure(
    state: tauri::State<'_, AppState>,
    db: String,
    table: String,
) -> Result<Value, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let sanitized_table = sanitize_identifier(&table)?;
    let query = format!("DESCRIBE {}", sanitized_table);
    let rows = sqlx::query(&query)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let data: Vec<Value> = rows.iter().map(row_to_json).collect();
    Ok(serde_json::json!({
        "success": true,
        "structure": data
    }))
}

#[tauri::command]
async fn cmd_get_relations(state: tauri::State<'_, AppState>, db: String) -> Result<Value, String> {
    let mut conn = get_connection(&state, None).await?;
    let query = "
        SELECT 
            TABLE_NAME, 
            COLUMN_NAME, 
            CONSTRAINT_NAME, 
            REFERENCED_TABLE_NAME, 
            REFERENCED_COLUMN_NAME 
        FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE 
        WHERE TABLE_SCHEMA = ? 
        AND REFERENCED_TABLE_NAME IS NOT NULL
    ";
    
    let rows = sqlx::query(query)
        .bind(db)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let list: Vec<Value> = rows.iter().map(row_to_json).collect();
    Ok(serde_json::json!({ "success": true, "relations": list }))
}

#[tauri::command]
async fn cmd_get_schema_suggestions(state: tauri::State<'_, AppState>, db: String) -> Result<Value, String> {
    let mut conn = get_connection(&state, None).await?;
    let query = "
        SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE 
        FROM INFORMATION_SCHEMA.COLUMNS 
        WHERE TABLE_SCHEMA = ?
        ORDER BY TABLE_NAME, ORDINAL_POSITION
    ";
    let rows = sqlx::query(query)
        .bind(db)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let mut tables = Vec::new();
    let mut columns = serde_json::Map::new();
    
    for row in rows {
        let table_name: String = row.try_get("TABLE_NAME").unwrap_or_default();
        let column_name: String = row.try_get("COLUMN_NAME").unwrap_or_default();
        let data_type: String = row.try_get("DATA_TYPE").unwrap_or_default();
        
        if !tables.contains(&table_name) {
            tables.push(table_name.clone());
        }
        
        let cols_entry = columns.entry(table_name).or_insert(Value::Array(Vec::new()));
        if let Value::Array(arr) = cols_entry {
            arr.push(serde_json::json!({
                "name": column_name,
                "type": data_type
            }));
        }
    }
    
    Ok(serde_json::json!({
        "tables": tables,
        "columns": Value::Object(columns)
    }))
}

#[tauri::command]
async fn cmd_get_objects(state: tauri::State<'_, AppState>, db: String) -> Result<ObjectsSummary, String> {
    let mut conn = get_connection(&state, None).await?;
    
    let view_rows = sqlx::query("SELECT TABLE_NAME as name FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = ?")
        .bind(&db)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default();
    let views: Vec<String> = view_rows.iter().map(|row| row.try_get(0).unwrap_or_default()).collect();
    
    let proc_rows = sqlx::query("SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'PROCEDURE'")
        .bind(&db)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default();
    let mut procedures: Vec<String> = proc_rows.iter().map(|row| row.try_get(0).unwrap_or_default()).collect();
    
    if procedures.is_empty() {
        if let Ok(fallback_rows) = sqlx::query(&format!("SHOW PROCEDURE STATUS WHERE Db = '{}'", db)).fetch_all(&mut *conn).await {
            procedures = fallback_rows.iter().map(|row| row.try_get("Name").unwrap_or_default()).collect();
        }
    }
    
    let func_rows = sqlx::query("SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'FUNCTION'")
        .bind(&db)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default();
    let mut functions: Vec<String> = func_rows.iter().map(|row| row.try_get(0).unwrap_or_default()).collect();
    
    if functions.is_empty() {
        if let Ok(fallback_rows) = sqlx::query(&format!("SHOW FUNCTION STATUS WHERE Db = '{}'", db)).fetch_all(&mut *conn).await {
            functions = fallback_rows.iter().map(|row| row.try_get("Name").unwrap_or_default()).collect();
        }
    }
    
    Ok(ObjectsSummary {
        success: true,
        views,
        procedures,
        functions
    })
}

#[tauri::command]
async fn cmd_server_status(state: tauri::State<'_, AppState>) -> Result<ServerStatusResponse, String> {
    let mut conn = get_connection(&state, None).await?;
    
    let proc_rows = sqlx::query("SHOW FULL PROCESSLIST")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let processes = Value::Array(proc_rows.iter().map(row_to_json).collect());
    
    let status_rows = sqlx::query("SHOW GLOBAL STATUS WHERE Variable_name IN ('Uptime', 'Threads_connected', 'Threads_running', 'Questions', 'Slow_queries')")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let mut uptime = 0;
    let mut threads_connected = 0;
    let mut threads_running = 0;
    let mut queries = 0;
    let mut slow_queries = 0;
    
    for row in status_rows {
        let name: String = row.try_get("Variable_name").unwrap_or_default();
        let val: String = row.try_get("Value").unwrap_or_default();
        let num: i64 = val.parse().unwrap_or(0);
        match name.as_str() {
            "Uptime" => uptime = num,
            "Threads_connected" => threads_connected = num,
            "Threads_running" => threads_running = num,
            "Questions" => queries = num,
            "Slow_queries" => slow_queries = num,
            _ => {}
        }
    }
    
    Ok(ServerStatusResponse {
        success: true,
        processes,
        status: ServerStatus {
            uptime,
            threads_connected,
            threads_running,
            queries,
            slow_queries,
        }
    })
}

#[tauri::command]
async fn cmd_server_metrics(state: tauri::State<'_, AppState>) -> Result<ServerMetricsResponse, String> {
    let mut conn = get_connection(&state, None).await?;
    let status_rows = sqlx::query("
        SHOW GLOBAL STATUS WHERE Variable_name IN (
            'Questions', 
            'Threads_connected', 
            'Threads_running', 
            'Bytes_received', 
            'Bytes_sent',
            'Innodb_buffer_pool_pages_total',
            'Innodb_buffer_pool_pages_free',
            'Slow_queries',
            'Uptime'
        )
    ")
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    
    let mut q = 0;
    let mut tc = 0;
    let mut tr = 0;
    let mut br = 0;
    let mut bs = 0;
    let mut it = 0;
    let mut ifr = 0;
    let mut sq = 0;
    let mut up = 0;
    
    for row in status_rows {
        let name: String = row.try_get("Variable_name").unwrap_or_default();
        let val: String = row.try_get("Value").unwrap_or_default();
        let num: i64 = val.parse().unwrap_or(0);
        match name.as_str() {
            "Questions" => q = num,
            "Threads_connected" => tc = num,
            "Threads_running" => tr = num,
            "Bytes_received" => br = num,
            "Bytes_sent" => bs = num,
            "Innodb_buffer_pool_pages_total" => it = num,
            "Innodb_buffer_pool_pages_free" => ifr = num,
            "Slow_queries" => sq = num,
            "Uptime" => up = num,
            _ => {}
        }
    }
    
    Ok(ServerMetricsResponse {
        success: true,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        metrics: ServerMetrics {
            questions: q,
            threads_connected: tc,
            threads_running: tr,
            bytes_received: br,
            bytes_sent: bs,
            innodb_total: it,
            innodb_free: ifr,
            slow_queries: sq,
            uptime: up,
        }
    })
}

#[tauri::command]
async fn cmd_server_slow_log(state: tauri::State<'_, AppState>) -> Result<Value, String> {
    let mut conn = get_connection(&state, None).await?;
    let var_row = sqlx::query("SHOW VARIABLES LIKE 'log_output'")
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let val: String = var_row.try_get("Value").unwrap_or_default();
    
    if !val.contains("TABLE") {
        return Ok(serde_json::json!({
            "success": false,
            "error": "Slow query logging to table is not enabled. Run: SET GLOBAL log_output = 'TABLE'; SET GLOBAL slow_query_log = 'ON';",
            "setup_required": true
        }));
    }
    
    let rows = sqlx::query("SELECT * FROM mysql.slow_log ORDER BY start_time DESC LIMIT 100")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let list: Vec<Value> = rows.iter().map(row_to_json).collect();
    Ok(serde_json::json!({ "success": true, "logs": list }))
}

#[tauri::command]
async fn cmd_execute_query(
    state: tauri::State<'_, AppState>,
    db: Option<String>,
    sql: String,
    confirmed: bool,
) -> Result<QueryResult, String> {
    if is_destructive(&sql) && !confirmed {
        return Ok(QueryResult {
            success: false,
            data: None,
            columns: Vec::new(),
            affected_rows: None,
            error: Some("DESTRUCTIVE_QUERY".into()),
            message: Some("This query contains potentially destructive operations (DROP, DELETE, TRUNCATE, ALTER). Are you sure you want to proceed?".into()),
        });
    }
    
    let mut conn = get_connection(&state, db.as_deref()).await?;
    
    let sql_upper = sql.trim().to_uppercase();
    if sql_upper.starts_with("SELECT") || sql_upper.starts_with("SHOW") || sql_upper.starts_with("DESCRIBE") || sql_upper.starts_with("EXPLAIN") {
        let rows = sqlx::query(&sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;
            
        let list: Vec<Value> = rows.iter().map(row_to_json).collect();
        let columns = if !list.is_empty() {
            list[0].as_object().unwrap().keys().cloned().collect()
        } else {
            Vec::new()
        };
        Ok(QueryResult {
            success: true,
            data: Some(Value::Array(list)),
            columns,
            affected_rows: None,
            error: None,
            message: None,
        })
    } else {
        let result = conn.execute(sql.as_str()).await.map_err(|e| e.to_string())?;
        Ok(QueryResult {
            success: true,
            data: None,
            columns: Vec::new(),
            affected_rows: Some(result.rows_affected()),
            error: None,
            message: None,
        })
    }
}

#[tauri::command]
async fn cmd_get_database_stats(state: tauri::State<'_, AppState>, db: String) -> Result<Value, String> {
    let mut conn = get_connection(&state, None).await?;
    let rows = sqlx::query("
        SELECT 
            TABLE_NAME as tableName, 
            TABLE_ROWS as rowCount, 
            DATA_LENGTH as dataSize, 
            ENGINE as engine,
            TABLE_COLLATION as collation
         FROM information_schema.tables 
         WHERE TABLE_SCHEMA = ?
    ")
    .bind(db)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    
    let list: Vec<Value> = rows.iter().map(row_to_json).collect();
    Ok(serde_json::json!({ "success": true, "stats": list }))
}

#[tauri::command]
async fn cmd_export(
    state: tauri::State<'_, AppState>,
    db: String,
    format: String,
    include_structure: bool,
    include_data: bool,
) -> Result<String, String> {
    let mut conn = get_connection(&state, None).await?;
    
    let show_tables_query = format!("SHOW TABLES FROM {}", sanitize_identifier(&db)?);
    let table_rows = sqlx::query(&show_tables_query)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let table_names: Vec<String> = table_rows.iter().map(|r| r.try_get(0).unwrap_or_default()).collect();
    
    if format == "sql" {
        let mut sql_dump = format!("-- MySQL GUI Dump\n-- Database: {}\n\n", sanitize_identifier(&db)?);
        
        for table in table_names {
            let sanitized_table = sanitize_identifier(&table)?;
            
            if include_structure {
                let show_create_query = format!("SHOW CREATE TABLE {}.{}", sanitize_identifier(&db)?, sanitized_table);
                let create_res = sqlx::query(&show_create_query)
                    .fetch_one(&mut *conn)
                    .await
                    .map_err(|e| e.to_string())?;
                let create_sql: String = create_res.try_get("Create Table").unwrap_or_default();
                sql_dump += &format!("DROP TABLE IF EXISTS {};\n{};\n\n", sanitized_table, create_sql);
            }
            
            if include_data {
                let select_all_query = format!("SELECT * FROM {}.{}", sanitize_identifier(&db)?, sanitized_table);
                let rows = sqlx::query(&select_all_query)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| e.to_string())?;
                    
                if !rows.is_empty() {
                    let col_data: Vec<Value> = rows.iter().map(row_to_json).collect();
                    let columns: Vec<String> = col_data[0].as_object().unwrap().keys().cloned().collect();
                    let sanitized_columns: Vec<String> = columns.iter().map(|c| sanitize_identifier(c)).collect::<Result<_, _>>()?;
                    let col_names = sanitized_columns.join(", ");
                    
                    let mut values_str = Vec::new();
                    for row in col_data {
                        let mut row_vals = Vec::new();
                        for col in &columns {
                            let val = row.get(col).unwrap_or(&Value::Null);
                            if val.is_null() {
                                row_vals.push("NULL".to_string());
                            } else if let Some(n) = val.as_i64() {
                                row_vals.push(n.to_string());
                            } else if let Some(f) = val.as_f64() {
                                row_vals.push(f.to_string());
                            } else {
                                let s = val.as_str().unwrap_or_default().replace('\'', "''");
                                row_vals.push(format!("'{}'", s));
                            }
                        }
                        values_str.push(format!("({})", row_vals.join(", ")));
                    }
                    
                    sql_dump += &format!("INSERT INTO {} ({}) VALUES\n{};\n\n", sanitized_table, col_names, values_str.join(",\n"));
                }
            }
        }
        Ok(sql_dump)
    } else if format == "json" {
        let mut full_data = serde_json::Map::new();
        for table in table_names {
            if include_data {
                let select_all_query = format!("SELECT * FROM {}.{}", sanitize_identifier(&db)?, sanitize_identifier(&table)?);
                let rows = sqlx::query(&select_all_query)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| e.to_string())?;
                let list: Vec<Value> = rows.iter().map(row_to_json).collect();
                full_data.insert(table, Value::Array(list));
            } else {
                full_data.insert(table, Value::Array(Vec::new()));
            }
        }
        Ok(serde_json::to_string_pretty(&Value::Object(full_data)).unwrap_or_default())
    } else {
        let mut csv = String::new();
        for table in table_names {
            if include_data {
                let select_all_query = format!("SELECT * FROM {}.{}", sanitize_identifier(&db)?, sanitize_identifier(&table)?);
                let rows = sqlx::query(&select_all_query)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| e.to_string())?;
                if !rows.is_empty() {
                    let col_data: Vec<Value> = rows.iter().map(row_to_json).collect();
                    let columns: Vec<String> = col_data[0].as_object().unwrap().keys().cloned().collect();
                    csv += &format!("Table: {}\n", table);
                    csv += &format!("{}\n", columns.join(","));
                    for row in col_data {
                        let row_parts: Vec<String> = columns.iter().map(|col| {
                            let val = row.get(col).unwrap_or(&Value::Null);
                            if val.is_null() {
                                "".to_string()
                            } else {
                                let s = if let Some(st) = val.as_str() { st.to_string() } else { val.to_string() };
                                format!("\"{}\"", s.replace('"', "\"\""))
                            }
                        }).collect();
                        csv += &format!("{}\n", row_parts.join(","));
                    }
                    csv += "\n";
                }
            }
        }
        Ok(csv)
    }
}

#[tauri::command]
async fn cmd_import(
    state: tauri::State<'_, AppState>,
    db: String,
    sql: String,
) -> Result<Value, String> {
    let mut conn = get_connection(&state, Some(&db)).await?;
    let statements = split_sql_statements(&sql);
    
    let mut success_count = 0;
    let mut errors = Vec::new();
    
    for stmt in statements {
        match conn.execute(stmt.as_str()).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                let trunc = if stmt.len() > 100 { format!("{}...", &stmt[0..100]) } else { stmt.clone() };
                errors.push(serde_json::json!({
                    "statement": trunc,
                    "error": e.to_string()
                }));
            }
        }
    }
    
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Executed {} statements.", success_count),
        "errors": if errors.is_empty() { Value::Null } else { Value::Array(errors) }
    }))
}

#[tauri::command]
async fn cmd_list_users(state: tauri::State<'_, AppState>) -> Result<Value, String> {
    let mut conn = get_connection(&state, None).await?;
    match sqlx::query("SELECT user, host, authentication_string, account_locked from mysql.user")
        .fetch_all(&mut *conn)
        .await 
    {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_json).collect();
            Ok(serde_json::json!({ "success": true, "users": list }))
        }
        Err(_) => {
            let self_row = sqlx::query("SELECT USER()")
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| e.to_string())?;
            let user_self: String = self_row.try_get(0).unwrap_or_default();
            let parts: Vec<&str> = user_self.split('@').collect();
            let user = parts.first().cloned().unwrap_or("root");
            let host = parts.get(1).cloned().unwrap_or("localhost");
            
            Ok(serde_json::json!({
                "success": true,
                "users": [{ "user": user, "host": host }],
                "note": "Limited visibility: could not access mysql.user table"
            }))
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub key: Option<String>,
    pub ollama_host: Option<String>,
    pub ollama_model: Option<String>,
}

#[tauri::command]
async fn cmd_save_llm_config(
    provider: String,
    key: Option<String>,
    ollama_host: Option<String>,
    ollama_model: Option<String>,
) -> Result<bool, String> {
    let entry = keyring::Entry::new("mysql-gui-tauri", "llm_config").map_err(|e| e.to_string())?;
    
    let mut final_key = key;
    if let Some(ref k) = final_key {
        if k.contains("...") || k == "****" {
            let password = entry.get_password().unwrap_or_default();
            if !password.is_empty() {
                if let Ok(old_config) = serde_json::from_str::<LLMConfig>(&password) {
                    final_key = old_config.key;
                }
            }
        }
    }

    let config = LLMConfig {
        provider,
        key: final_key,
        ollama_host,
        ollama_model,
    };
    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    entry.set_password(&json).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn cmd_get_llm_config() -> Result<Value, String> {
    let entry = keyring::Entry::new("mysql-gui-tauri", "llm_config").map_err(|e| e.to_string())?;
    let password = entry.get_password().unwrap_or_default();
    if password.is_empty() {
        return Ok(serde_json::json!({
            "provider": "anthropic",
            "key": "",
            "ollama_host": "http://localhost:11434",
            "ollama_model": "llama3"
        }));
    }
    let mut config: LLMConfig = serde_json::from_str(&password).map_err(|e| e.to_string())?;
    
    if let Some(ref k) = config.key {
        if k.len() > 8 {
            let masked = format!("{}...{}", &k[0..4], &k[k.len()-4..]);
            config.key = Some(masked);
        } else if !k.is_empty() {
            config.key = Some("****".to_string());
        }
    }
    
    Ok(serde_json::to_value(&config).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn cmd_get_llm_config_unmasked() -> Result<Value, String> {
    let entry = keyring::Entry::new("mysql-gui-tauri", "llm_config").map_err(|e| e.to_string())?;
    let password = entry.get_password().unwrap_or_default();
    if password.is_empty() {
        return Ok(serde_json::json!({
            "provider": "anthropic",
            "key": "",
            "ollama_host": "http://localhost:11434",
            "ollama_model": "llama3"
        }));
    }
    let config: LLMConfig = serde_json::from_str(&password).map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(&config).map_err(|e| e.to_string())?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            pool: Mutex::new(None),
            current_db: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            cmd_auth_login,
            cmd_auth_logout,
            cmd_list_databases,
            cmd_create_database,
            cmd_drop_database,
            cmd_list_tables,
            cmd_create_table,
            cmd_drop_table,
            cmd_get_data,
            cmd_insert_row,
            cmd_update_row,
            cmd_delete_row,
            cmd_generate_mock_data,
            cmd_get_structure,
            cmd_get_relations,
            cmd_get_schema_suggestions,
            cmd_get_objects,
            cmd_server_status,
            cmd_server_metrics,
            cmd_server_slow_log,
            cmd_execute_query,
            cmd_get_database_stats,
            cmd_export,
            cmd_import,
            cmd_list_users,
            cmd_save_llm_config,
            cmd_get_llm_config,
            cmd_get_llm_config_unmasked
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
