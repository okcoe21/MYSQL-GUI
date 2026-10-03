use sqlx::{Row, Executor};
use serde_json::Value;
use rand::seq::SliceRandom;
use rand::Rng;
use tokio::io::AsyncWriteExt;
use crate::state::AppState;
use crate::db::sanitize::{sanitize_identifier, escape_sql_string};
use crate::db::models::row_to_json;
use crate::db::query::split_sql_statements;

pub fn generate_mock_value(field_type: &str) -> String {
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

pub async fn generate_mock_data(
    state: &AppState,
    db: &str,
    table: &str,
    count: i64,
    blueprint: &Value,
) -> Result<String, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(table)?;
    
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
    
    Ok(format!("Successfully inserted {} rows into {}", inserted, table))
}

/// Formats a single value as an RFC 4180 compliant CSV cell, neutralizing formula injection.
///
/// Formula Injection Protection:
/// If the field begins with '=', '+', '-', or '@', it is prefixed with a single quote (')
/// so that spreadsheet programs treat it strictly as text.
///
/// RFC 4180 Quoting:
/// Fields containing double quotes, commas, newlines, or starting with a formula prefix
/// are enclosed in double quotes with embedded double quotes escaped as `""`.
pub fn format_csv_cell(val: &str) -> String {
    let starts_with_formula = val.starts_with('=')
        || val.starts_with('+')
        || val.starts_with('-')
        || val.starts_with('@');

    let neutralized = if starts_with_formula {
        format!("'{}", val)
    } else {
        val.to_string()
    };

    let needs_quotes = starts_with_formula
        || neutralized.contains(',')
        || neutralized.contains('"')
        || neutralized.contains('\n')
        || neutralized.contains('\r');

    if needs_quotes {
        format!("\"{}\"", neutralized.replace('"', "\"\""))
    } else {
        neutralized
    }
}

/// Formats a row of cells into an RFC 4180 CSV line with CRLF line ending.
pub fn format_csv_row(cells: &[String]) -> String {
    let formatted: Vec<String> = cells.iter().map(|c| format_csv_cell(c)).collect();
    format!("{}\r\n", formatted.join(","))
}

/// Restores the original value of a cell if it was neutralized during CSV export.
pub fn strip_formula_prefix(val: &str) -> &str {
    if (val.starts_with("'=") || val.starts_with("'+") || val.starts_with("'-") || val.starts_with("'@"))
        && val.len() > 1
    {
        &val[1..]
    } else {
        val
    }
}

/// Hand-crafted, zero-dependency RFC 4180 compliant CSV parser.
///
/// Parses CSV data into rows and fields, handling:
/// - Quoted fields enclosing commas, newlines (CRLF and LF), and escaped quotes (`""`).
/// - Unquoted fields with whitespace preserved.
/// - Empty fields and records.
/// - Unclosed quotes detection (returns Err).
pub fn parse_csv(input: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut current_record: Vec<String> = Vec::new();
    let mut current_field = String::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_quotes = false;
    let mut has_data = false;

    while i < len {
        let c = chars[i];

        if in_quotes {
            if c == '"' {
                if i + 1 < len && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 2;
                    continue;
                } else {
                    in_quotes = false;
                    i += 1;
                    continue;
                }
            } else {
                current_field.push(c);
                i += 1;
                continue;
            }
        }

        if c == '"' {
            if current_field.is_empty() {
                in_quotes = true;
                has_data = true;
                i += 1;
                continue;
            } else {
                current_field.push(c);
                i += 1;
                continue;
            }
        } else if c == ',' {
            current_record.push(current_field);
            current_field = String::new();
            has_data = true;
            i += 1;
            continue;
        } else if c == '\r' {
            if i + 1 < len && chars[i + 1] == '\n' {
                i += 1;
            }
            current_record.push(current_field);
            current_field = String::new();
            records.push(current_record);
            current_record = Vec::new();
            has_data = false;
            i += 1;
            continue;
        } else if c == '\n' {
            current_record.push(current_field);
            current_field = String::new();
            records.push(current_record);
            current_record = Vec::new();
            has_data = false;
            i += 1;
            continue;
        } else {
            current_field.push(c);
            has_data = true;
            i += 1;
        }
    }

    if in_quotes {
        return Err("Unclosed double quote in CSV input".to_string());
    }

    if has_data || !current_field.is_empty() || !current_record.is_empty() {
        current_record.push(current_field);
        records.push(current_record);
    }

    Ok(records)
}

/// Streams database export to disk in chunks of 500 rows, preventing memory bloat on large tables.
///
/// Ensures zero credentials or connection details are written to the file.
pub async fn export_database_stream(
    state: &AppState,
    db: &str,
    format: &str,
    include_structure: bool,
    include_data: bool,
    file_path: &std::path::Path,
) -> Result<(usize, u64), String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_db = sanitize_identifier(db)?;

    let file = tokio::fs::File::create(file_path)
        .await
        .map_err(|e| format!("Failed to create export file: {}", e))?;
    let mut writer = tokio::io::BufWriter::new(file);

    let show_tables_query = format!("SHOW TABLES FROM {}", sanitized_db);
    let table_rows = sqlx::query(&show_tables_query)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch tables: {}", e))?;
    let table_names: Vec<String> = table_rows.iter().map(|r| r.try_get(0).unwrap_or_default()).collect();

    let mut tables_exported = 0;

    if format == "sql" {
        let header = format!(
            "-- MySQL GUI SQL Dump\n-- Database: {}\n-- Generated at: {}\n\nSET FOREIGN_KEY_CHECKS=0;\n\n",
            sanitized_db,
            chrono::Utc::now().to_rfc3339()
        );
        writer.write_all(header.as_bytes()).await.map_err(|e| e.to_string())?;

        for table in &table_names {
            let sanitized_table = sanitize_identifier(table)?;

            if include_structure {
                let show_create_query = format!("SHOW CREATE TABLE {}.{}", sanitized_db, sanitized_table);
                if let Ok(create_res) = sqlx::query(&show_create_query).fetch_one(&mut *conn).await {
                    let create_sql: String = create_res.try_get("Create Table").unwrap_or_default();
                    let ddl = format!("DROP TABLE IF EXISTS {};\n{};\n\n", sanitized_table, create_sql);
                    writer.write_all(ddl.as_bytes()).await.map_err(|e| e.to_string())?;
                }
            }

            if include_data {
                let chunk_size: i64 = 500;
                let mut offset: i64 = 0;

                loop {
                    let select_query = format!(
                        "SELECT * FROM {}.{} LIMIT {} OFFSET {}",
                        sanitized_db, sanitized_table, chunk_size, offset
                    );
                    let rows = sqlx::query(&select_query)
                        .fetch_all(&mut *conn)
                        .await
                        .map_err(|e| e.to_string())?;

                    if rows.is_empty() {
                        break;
                    }

                    let col_data: Vec<Value> = rows.iter().map(row_to_json).collect();
                    let columns: Vec<String> = col_data[0].as_object().unwrap().keys().cloned().collect();
                    let sanitized_columns: Vec<String> = columns
                        .iter()
                        .map(|c| sanitize_identifier(c))
                        .collect::<Result<_, _>>()?;
                    let col_names = sanitized_columns.join(", ");

                    let mut values_str = Vec::new();
                    for row in &col_data {
                        let mut row_vals = Vec::new();
                        for col in &columns {
                            let val = row.get(col).unwrap_or(&Value::Null);
                            if val.is_null() {
                                row_vals.push("NULL".to_string());
                            } else if let Some(n) = val.as_i64() {
                                row_vals.push(n.to_string());
                            } else if let Some(f) = val.as_f64() {
                                row_vals.push(f.to_string());
                            } else if let Some(b) = val.as_bool() {
                                row_vals.push(if b { "1".to_string() } else { "0".to_string() });
                            } else {
                                let s = if let Some(st) = val.as_str() {
                                    st.to_string()
                                } else {
                                    val.to_string()
                                };
                                row_vals.push(format!("'{}'", escape_sql_string(&s)));
                            }
                        }
                        values_str.push(format!("({})", row_vals.join(", ")));
                    }

                    let insert_stmt = format!(
                        "INSERT INTO {} ({}) VALUES\n{};\n\n",
                        sanitized_table,
                        col_names,
                        values_str.join(",\n")
                    );
                    writer.write_all(insert_stmt.as_bytes()).await.map_err(|e| e.to_string())?;

                    let count = rows.len() as i64;
                    offset += count;
                    if count < chunk_size {
                        break;
                    }
                }
            }
            tables_exported += 1;
        }

        let footer = "SET FOREIGN_KEY_CHECKS=1;\n";
        writer.write_all(footer.as_bytes()).await.map_err(|e| e.to_string())?;
    } else if format == "csv" {
        for table in &table_names {
            let sanitized_table = sanitize_identifier(table)?;

            if include_data {
                if table_names.len() > 1 {
                    let banner = format!("# Table: {}\r\n", table);
                    writer.write_all(banner.as_bytes()).await.map_err(|e| e.to_string())?;
                }

                let chunk_size: i64 = 500;
                let mut offset: i64 = 0;
                let mut header_written = false;

                loop {
                    let select_query = format!(
                        "SELECT * FROM {}.{} LIMIT {} OFFSET {}",
                        sanitized_db, sanitized_table, chunk_size, offset
                    );
                    let rows = sqlx::query(&select_query)
                        .fetch_all(&mut *conn)
                        .await
                        .map_err(|e| e.to_string())?;

                    if rows.is_empty() {
                        break;
                    }

                    let col_data: Vec<Value> = rows.iter().map(row_to_json).collect();
                    let columns: Vec<String> = col_data[0].as_object().unwrap().keys().cloned().collect();

                    if !header_written {
                        let header_row = format_csv_row(&columns);
                        writer.write_all(header_row.as_bytes()).await.map_err(|e| e.to_string())?;
                        header_written = true;
                    }

                    for row in &col_data {
                        let row_vals: Vec<String> = columns
                            .iter()
                            .map(|col| {
                                let val = row.get(col).unwrap_or(&Value::Null);
                                if val.is_null() {
                                    String::new()
                                } else if let Some(st) = val.as_str() {
                                    st.to_string()
                                } else {
                                    val.to_string()
                                }
                            })
                            .collect();
                        let line = format_csv_row(&row_vals);
                        writer.write_all(line.as_bytes()).await.map_err(|e| e.to_string())?;
                    }

                    let count = rows.len() as i64;
                    offset += count;
                    if count < chunk_size {
                        break;
                    }
                }

                if table_names.len() > 1 {
                    writer.write_all(b"\r\n").await.map_err(|e| e.to_string())?;
                }
            }
            tables_exported += 1;
        }
    } else {
        let content = export_database(state, db, format, include_structure, include_data).await?;
        writer.write_all(content.as_bytes()).await.map_err(|e| e.to_string())?;
        tables_exported = table_names.len();
    }

    writer.flush().await.map_err(|e| e.to_string())?;
    let metadata = tokio::fs::metadata(file_path).await.map_err(|e| e.to_string())?;

    Ok((tables_exported, metadata.len()))
}

pub async fn export_database(
    state: &AppState,
    db: &str,
    format: &str,
    include_structure: bool,
    include_data: bool,
) -> Result<String, String> {
    let mut conn = state.get_connection(None).await?;
    let show_tables_query = format!("SHOW TABLES FROM {}", sanitize_identifier(db)?);
    let table_rows = sqlx::query(&show_tables_query)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let table_names: Vec<String> = table_rows.iter().map(|r| r.try_get(0).unwrap_or_default()).collect();
    
    if format == "sql" {
        let mut sql_dump = format!("-- MySQL GUI Dump\n-- Database: {}\n\n", sanitize_identifier(db)?);
        
        for table in table_names {
            let sanitized_table = sanitize_identifier(&table)?;
            
            if include_structure {
                let show_create_query = format!("SHOW CREATE TABLE {}.{}", sanitize_identifier(db)?, sanitized_table);
                let create_res = sqlx::query(&show_create_query)
                    .fetch_one(&mut *conn)
                    .await
                    .map_err(|e| e.to_string())?;
                let create_sql: String = create_res.try_get("Create Table").unwrap_or_default();
                sql_dump += &format!("DROP TABLE IF EXISTS {};\n{};\n\n", sanitized_table, create_sql);
            }
            
            if include_data {
                let select_all_query = format!("SELECT * FROM {}.{}", sanitize_identifier(db)?, sanitized_table);
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
                            } else if let Some(b) = val.as_bool() {
                                row_vals.push(if b { "1".to_string() } else { "0".to_string() });
                            } else {
                                let s = if let Some(st) = val.as_str() {
                                    st.to_string()
                                } else {
                                    val.to_string()
                                };
                                row_vals.push(format!("'{}'", escape_sql_string(&s)));
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
                let select_all_query = format!("SELECT * FROM {}.{}", sanitize_identifier(db)?, sanitize_identifier(&table)?);
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
                let select_all_query = format!("SELECT * FROM {}.{}", sanitize_identifier(db)?, sanitize_identifier(&table)?);
                let rows = sqlx::query(&select_all_query)
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|e| e.to_string())?;
                if !rows.is_empty() {
                    let col_data: Vec<Value> = rows.iter().map(row_to_json).collect();
                    let columns: Vec<String> = col_data[0].as_object().unwrap().keys().cloned().collect();
                    csv += &format!("# Table: {}\r\n", table);
                    csv += &format_csv_row(&columns);
                    for row in col_data {
                        let row_parts: Vec<String> = columns.iter().map(|col| {
                            let val = row.get(col).unwrap_or(&Value::Null);
                            if val.is_null() {
                                String::new()
                            } else {
                                val.as_str().map(|s| s.to_string()).unwrap_or_else(|| val.to_string())
                            }
                        }).collect();
                        csv += &format_csv_row(&row_parts);
                    }
                    csv += "\r\n";
                }
            }
        }
        Ok(csv)
    }
}

/// Executes SQL import statements sequentially or inside a transaction where allowed.
pub async fn import_sql(
    state: &AppState,
    db: &str,
    sql: &str,
) -> Result<(usize, Vec<String>), String> {
    import_sql_content(state, db, sql).await
}

/// Executes SQL import statements, using transactions when only DML statements are present.
pub async fn import_sql_content(
    state: &AppState,
    db: &str,
    sql: &str,
) -> Result<(usize, Vec<String>), String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let statements = split_sql_statements(sql);

    if statements.is_empty() {
        return Ok((0, Vec::new()));
    }

    // In MySQL, DDL statements cause an implicit commit. Only wrap pure DML in an explicit transaction.
    let has_ddl = statements.iter().any(|stmt| {
        let trimmed = stmt.trim_start();
        let upper = trimmed.to_uppercase();
        upper.starts_with("CREATE")
            || upper.starts_with("DROP")
            || upper.starts_with("ALTER")
            || upper.starts_with("TRUNCATE")
            || upper.starts_with("RENAME")
    });

    let use_tx = !has_ddl;

    if use_tx {
        let _ = sqlx::query("START TRANSACTION").execute(&mut *conn).await;
    }

    let mut success_count = 0;
    let mut errors = Vec::new();

    for stmt in &statements {
        match conn.execute(stmt.as_str()).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                let trunc = if stmt.len() > 100 {
                    format!("{}...", &stmt[0..100])
                } else {
                    stmt.clone()
                };
                errors.push(format!("{}: {}", trunc, e));
                if use_tx {
                    let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
                    return Ok((success_count, errors));
                }
            }
        }
    }

    if use_tx {
        let _ = sqlx::query("COMMIT").execute(&mut *conn).await;
    }

    Ok((success_count, errors))
}

/// Reads and imports a CSV file into the target table, enforcing a 50 MB size cap.
pub async fn import_csv_file(
    state: &AppState,
    db: &str,
    table: &str,
    file_path: &std::path::Path,
) -> Result<usize, String> {
    let metadata = tokio::fs::metadata(file_path)
        .await
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;

    const MAX_CSV_SIZE: u64 = 50 * 1024 * 1024;
    if metadata.len() > MAX_CSV_SIZE {
        return Err(format!(
            "CSV file size ({:.2} MB) exceeds maximum permitted limit of 50 MB",
            metadata.len() as f64 / (1024.0 * 1024.0)
        ));
    }

    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read CSV file: {}", e))?;

    import_csv_content(state, db, table, &content).await
}

/// Imports CSV content into the target table with bound parameters, validating column count.
pub async fn import_csv_content(
    state: &AppState,
    db: &str,
    table: &str,
    content: &str,
) -> Result<usize, String> {
    let sanitized_table = sanitize_identifier(table)?;
    let records = parse_csv(content)?;

    if records.is_empty() {
        return Err("CSV file is empty".to_string());
    }

    let headers = &records[0];
    if headers.is_empty() {
        return Err("CSV header row is empty".to_string());
    }

    let data_rows = &records[1..];
    if data_rows.is_empty() {
        return Ok(0);
    }

    let mut conn = state.get_connection(Some(db)).await?;

    // Introspect table columns to validate column count against table schema
    let col_query = "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION";
    let col_rows = sqlx::query(col_query)
        .bind(db)
        .bind(table)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| format!("Failed to introspect table columns: {}", e))?;

    let db_columns: Vec<String> = col_rows.iter().map(|r| r.try_get(0).unwrap_or_default()).collect();
    if db_columns.is_empty() {
        return Err(format!("Table '{}' does not exist or has no columns in database '{}'", table, db));
    }

    if headers.len() > db_columns.len() {
        return Err(format!(
            "Column count mismatch: CSV provides {} columns, but table '{}' only has {} columns",
            headers.len(), table, db_columns.len()
        ));
    }

    let insert_columns: Vec<String> = headers
        .iter()
        .map(|h| sanitize_identifier(h.trim()))
        .collect::<Result<_, _>>()?;

    let placeholders: Vec<&str> = (0..headers.len()).map(|_| "?").collect();
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        sanitized_table,
        insert_columns.join(", "),
        placeholders.join(", ")
    );

    sqlx::query("START TRANSACTION").execute(&mut *conn).await.map_err(|e| e.to_string())?;

    let mut inserted_count = 0;
    for (row_idx, row) in data_rows.iter().enumerate() {
        if row.len() == 1 && row[0].trim().is_empty() {
            continue;
        }

        if row.len() != headers.len() {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            return Err(format!(
                "Row {} has {} fields, expected {} matching header columns",
                row_idx + 2, row.len(), headers.len()
            ));
        }

        let mut query = sqlx::query(&insert_sql);
        for cell in row {
            let restored = strip_formula_prefix(cell);
            query = query.bind(restored);
        }

        match query.execute(&mut *conn).await {
            Ok(_) => {
                inserted_count += 1;
            }
            Err(e) => {
                let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
                return Err(format!("Failed to insert row {}: {}", row_idx + 2, e));
            }
        }
    }

    sqlx::query("COMMIT").execute(&mut *conn).await.map_err(|e| e.to_string())?;

    Ok(inserted_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_cell_plain() {
        assert_eq!(format_csv_cell("hello"), "hello");
        assert_eq!(format_csv_cell("user123"), "user123");
        assert_eq!(format_csv_cell(""), "");
    }

    #[test]
    fn test_csv_cell_with_comma() {
        assert_eq!(format_csv_cell("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn test_csv_cell_with_quotes() {
        assert_eq!(format_csv_cell("She said \"yes\""), "\"She said \"\"yes\"\"\"");
    }

    #[test]
    fn test_csv_cell_with_newline() {
        assert_eq!(format_csv_cell("line 1\nline 2"), "\"line 1\nline 2\"");
        assert_eq!(format_csv_cell("line 1\r\nline 2"), "\"line 1\r\nline 2\"");
    }

    #[test]
    fn test_csv_formula_neutralization_equals() {
        // Must neutralize formula injection starting with '='
        let payload = "=cmd|' /c calc'!A0";
        let formatted = format_csv_cell(payload);
        assert_eq!(formatted, "\"'=cmd|' /c calc'!A0\"");
        assert!(formatted.starts_with("\"'="));
    }

    #[test]
    fn test_csv_formula_neutralization_plus_minus_at() {
        assert_eq!(format_csv_cell("+12345"), "\"'+12345\"");
        assert_eq!(format_csv_cell("-456"), "\"'-456\"");
        assert_eq!(format_csv_cell("@SUM(A1:A10)"), "\"'@SUM(A1:A10)\"");
    }

    #[test]
    fn test_csv_row_formatting() {
        let cells = vec!["Alice".to_string(), "=SUM(1,2)".to_string(), "San Francisco, CA".to_string()];
        let row = format_csv_row(&cells);
        assert_eq!(row, "Alice,\"'=SUM(1,2)\",\"San Francisco, CA\"\r\n");
    }

    #[test]
    fn test_csv_parser_simple() {
        let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA\n";
        let parsed = parse_csv(csv).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], vec!["name", "age", "city"]);
        assert_eq!(parsed[1], vec!["Alice", "30", "NYC"]);
        assert_eq!(parsed[2], vec!["Bob", "25", "LA"]);
    }

    #[test]
    fn test_csv_parser_quoted_commas() {
        let csv = "name,address\n\"Smith, John\",\"123 Main St, Apt 4B\"\n";
        let parsed = parse_csv(csv).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[1], vec!["Smith, John", "123 Main St, Apt 4B"]);
    }

    #[test]
    fn test_csv_parser_embedded_newlines() {
        let csv = "id,notes\n1,\"First line\nSecond line\"\n2,Done\n";
        let parsed = parse_csv(csv).unwrap();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[1], vec!["1", "First line\nSecond line"]);
        assert_eq!(parsed[2], vec!["2", "Done"]);
    }

    #[test]
    fn test_csv_parser_escaped_quotes() {
        let csv = "id,quote\n1,\"He said \"\"Hello world\"\" to everyone\"\n";
        let parsed = parse_csv(csv).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[1][1], "He said \"Hello world\" to everyone");
    }

    #[test]
    fn test_csv_parser_empty_fields() {
        let csv = "a,,c\n,b,\n";
        let parsed = parse_csv(csv).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], vec!["a", "", "c"]);
        assert_eq!(parsed[1], vec!["", "b", ""]);
    }

    #[test]
    fn test_csv_parser_crlf_and_lf() {
        let csv_crlf = "a,b\r\n1,2\r\n";
        let parsed_crlf = parse_csv(csv_crlf).unwrap();
        assert_eq!(parsed_crlf.len(), 2);
        assert_eq!(parsed_crlf[1], vec!["1", "2"]);

        let csv_lf = "a,b\n1,2\n";
        let parsed_lf = parse_csv(csv_lf).unwrap();
        assert_eq!(parsed_lf.len(), 2);
        assert_eq!(parsed_lf[1], vec!["1", "2"]);
    }

    #[test]
    fn test_csv_parser_unclosed_quote_error() {
        let csv = "a,b\n1,\"unclosed quote string\n";
        assert!(parse_csv(csv).is_err());
    }

    #[test]
    fn test_csv_formula_round_trip() {
        let original_data = "=cmd|' /c calc'!A0";
        let formatted = format_csv_cell(original_data);
        assert_eq!(formatted, "\"'=cmd|' /c calc'!A0\"");

        let csv_content = format!("id,command\r\n1,{}\r\n", formatted);
        let parsed = parse_csv(&csv_content).unwrap();
        assert_eq!(parsed.len(), 2);
        let parsed_cell = &parsed[1][1];
        assert_eq!(parsed_cell, "'=cmd|' /c calc'!A0");

        let restored = strip_formula_prefix(parsed_cell);
        assert_eq!(restored, original_data);
    }

    #[test]
    fn test_sql_statement_splitter_semicolons_in_strings() {
        let sql = "INSERT INTO users (note) VALUES ('semi;colon;inside'); SELECT * FROM users;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "INSERT INTO users (note) VALUES ('semi;colon;inside')");
        assert_eq!(stmts[1], "SELECT * FROM users");
    }

    #[test]
    fn test_sql_statement_splitter_semicolons_in_double_quotes() {
        let sql = "INSERT INTO users (note) VALUES (\"semi;inside\"); SELECT 1;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "INSERT INTO users (note) VALUES (\"semi;inside\")");
        assert_eq!(stmts[1], "SELECT 1");
    }

    #[test]
    fn test_sql_statement_splitter_semicolons_in_line_comments() {
        let sql = "-- Line comment with ; inside\nSELECT 1;\n# Hash comment with ; here\nSELECT 2;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("SELECT 1"));
        assert!(stmts[1].contains("SELECT 2"));
    }

    #[test]
    fn test_sql_statement_splitter_semicolons_in_block_comments() {
        let sql = "/* Block comment with ; inside */ SELECT 1; /* another ; comment */ SELECT 2;";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("SELECT 1"));
        assert!(stmts[1].contains("SELECT 2"));
    }
}
