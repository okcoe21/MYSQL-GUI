use sqlx::{Row, Executor};
use crate::state::AppState;
use crate::db::sanitize::sanitize_identifier;
use crate::db::models::{ColumnDefInput, TableColumnInfo};

pub async fn list_tables(state: &AppState, db: &str) -> Result<Vec<String>, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let rows = sqlx::query("SHOW TABLES")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let list = rows.iter().map(|row| {
        row.try_get::<String, _>(0).unwrap_or_default()
    }).collect();
    
    Ok(list)
}

#[allow(dead_code)]
pub async fn create_table(
    state: &AppState,
    db: &str,
    name: &str,
    columns: &[ColumnDefInput],
) -> Result<bool, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(name)?;
    
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
        let length = col.length.as_deref().filter(|l| !l.is_empty()).map(|l| format!("({})", l)).unwrap_or_default();
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

pub async fn drop_table(state: &AppState, db: &str, name: &str) -> Result<bool, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(name)?;
    let query = format!("DROP TABLE {}", sanitized_table);
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(true)
}

pub async fn truncate_table(state: &AppState, db: &str, name: &str) -> Result<bool, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(name)?;
    let query = format!("TRUNCATE TABLE {}", sanitized_table);
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(true)
}

pub async fn get_structure(state: &AppState, db: &str, table: &str) -> Result<Vec<TableColumnInfo>, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(table)?;
    let query = format!("DESCRIBE {}", sanitized_table);
    let rows = sqlx::query(&query)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let cols = rows.iter().map(|row| {
        TableColumnInfo {
            field: row.try_get("Field").unwrap_or_default(),
            r#type: row.try_get("Type").unwrap_or_default(),
            null: row.try_get("Null").unwrap_or_default(),
            key: row.try_get("Key").unwrap_or_default(),
            default: row.try_get("Default").ok(),
            extra: row.try_get("Extra").unwrap_or_default(),
        }
    }).collect();
    
    Ok(cols)
}
