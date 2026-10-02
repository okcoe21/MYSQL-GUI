use sqlx::{Row, Executor};
use serde_json::Value;
use rand::seq::SliceRandom;
use rand::Rng;
use crate::state::AppState;
use crate::db::sanitize::sanitize_identifier;
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

pub async fn import_sql(
    state: &AppState,
    db: &str,
    sql: &str,
) -> Result<(usize, Vec<String>), String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let statements = split_sql_statements(sql);
    
    let mut success_count = 0;
    let mut errors = Vec::new();
    
    for stmt in statements {
        match conn.execute(stmt.as_str()).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                let trunc = if stmt.len() > 100 { format!("{}...", &stmt[0..100]) } else { stmt.clone() };
                errors.push(format!("{}: {}", trunc, e));
            }
        }
    }
    
    Ok((success_count, errors))
}
