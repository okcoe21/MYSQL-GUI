use sqlx::Row;
use serde_json::Value;
use crate::state::AppState;
use crate::db::models::{row_to_json, ObjectsSummary};

pub async fn get_objects(state: &AppState, db: &str) -> Result<ObjectsSummary, String> {
    let mut conn = state.get_connection(None).await?;
    
    let view_rows = sqlx::query("SELECT TABLE_NAME as name FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = ?")
        .bind(db)
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_default();
    let views: Vec<String> = view_rows.iter().map(|row| row.try_get(0).unwrap_or_default()).collect();
    
    let proc_rows = sqlx::query("SELECT ROUTINE_NAME as name FROM INFORMATION_SCHEMA.ROUTINES WHERE ROUTINE_SCHEMA = ? AND ROUTINE_TYPE = 'PROCEDURE'")
        .bind(db)
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
        .bind(db)
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
        functions,
    })
}

pub async fn get_relations(state: &AppState, db: &str) -> Result<Vec<Value>, String> {
    let mut conn = state.get_connection(None).await?;
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
    Ok(list)
}

#[allow(dead_code)]
pub async fn get_schema_suggestions(state: &AppState, db: &str) -> Result<(Vec<String>, serde_json::Map<String, Value>), String> {
    let mut conn = state.get_connection(None).await?;
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
    
    Ok((tables, columns))
}
