use sqlx::{Row, Executor};
use crate::state::AppState;
use crate::db::sanitize::sanitize_identifier;
use crate::db::models::DbStats;

pub async fn list_databases(state: &AppState) -> Result<Vec<String>, String> {
    let mut conn = state.get_connection(None).await?;
    let rows = sqlx::query("SHOW DATABASES")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let list: Vec<String> = rows.iter().map(|row| {
        row.try_get::<String, _>(0).unwrap_or_default()
    }).collect();
    
    Ok(list)
}

pub async fn create_database(state: &AppState, name: &str) -> Result<bool, String> {
    let mut conn = state.get_connection(None).await?;
    let sanitized = sanitize_identifier(name)?;
    let query = format!("CREATE DATABASE {}", sanitized);
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(true)
}

pub async fn drop_database(state: &AppState, name: &str) -> Result<bool, String> {
    let mut conn = state.get_connection(None).await?;
    let sanitized = sanitize_identifier(name)?;
    let query = format!("DROP DATABASE {}", sanitized);
    conn.execute(query.as_str()).await.map_err(|e| e.to_string())?;
    Ok(true)
}

pub async fn get_database_stats(state: &AppState, db: &str) -> Result<Vec<DbStats>, String> {
    let mut conn = state.get_connection(None).await?;
    let rows = sqlx::query("
        SELECT 
            TABLE_NAME as tableName, 
            COALESCE(TABLE_ROWS, 0) as rowCount, 
            COALESCE(DATA_LENGTH, 0) as dataSize, 
            COALESCE(ENGINE, '') as engine,
            COALESCE(TABLE_COLLATION, '') as collation
         FROM information_schema.tables 
         WHERE TABLE_SCHEMA = ?
         ORDER BY TABLE_NAME ASC
    ")
    .bind(db)
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    
    let stats = rows.iter().map(|row| {
        DbStats {
            table_name: row.try_get("tableName").unwrap_or_default(),
            row_count: row.try_get("rowCount").unwrap_or(0),
            data_size: row.try_get("dataSize").unwrap_or(0),
            engine: row.try_get("engine").unwrap_or_default(),
            collation: row.try_get("collation").unwrap_or_default(),
        }
    }).collect();
    
    Ok(stats)
}
