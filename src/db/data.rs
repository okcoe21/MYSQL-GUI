use sqlx::{Row, Column};
use serde_json::Value;
use crate::state::AppState;
use crate::db::sanitize::sanitize_identifier;
use crate::db::models::{row_to_json, GetDataResponse, PaginationInfo};

pub async fn get_data(
    state: &AppState,
    db: &str,
    table: &str,
    limit: i64,
    offset: i64,
    sort_col: Option<&str>,
    sort_order: Option<&str>,
) -> Result<GetDataResponse, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(table)?;
    
    let mut query = format!("SELECT * FROM {}", sanitized_table);
    
    if let Some(col) = sort_col {
        if !col.trim().is_empty() {
            let order = if sort_order.unwrap_or_default().eq_ignore_ascii_case("DESC") { "DESC" } else { "ASC" };
            let sanitized_col = sanitize_identifier(col)?;
            query += &format!(" ORDER BY {} {}", sanitized_col, order);
        }
    }
    
    query += " LIMIT ? OFFSET ?";
    
    let rows = sqlx::query(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let data: Vec<Value> = rows.iter().map(row_to_json).collect();
    
    let columns = if !rows.is_empty() {
        rows[0].columns().iter().map(|c| c.name().to_string()).collect()
    } else {
        // Fallback: fetch columns via DESCRIBE if no rows exist
        let desc_query = format!("DESCRIBE {}", sanitized_table);
        if let Ok(desc_rows) = sqlx::query(&desc_query).fetch_all(&mut *conn).await {
            desc_rows.iter().map(|r| r.try_get::<String, _>("Field").unwrap_or_default()).collect()
        } else {
            Vec::new()
        }
    };
    
    let count_query = format!("SELECT COUNT(*) as total FROM {}", sanitized_table);
    let count_row = sqlx::query(&count_query)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let total: i64 = count_row.try_get(0).unwrap_or(0);
    
    Ok(GetDataResponse {
        success: true,
        data,
        columns,
        pagination: PaginationInfo {
            total,
            limit,
            offset,
        },
    })
}

pub async fn insert_row(
    state: &AppState,
    db: &str,
    table: &str,
    data: &Value,
) -> Result<u64, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(table)?;
    
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
    Ok(result.last_insert_id())
}

#[allow(dead_code)]
pub async fn update_row(
    state: &AppState,
    db: &str,
    table: &str,
    column: &str,
    value: &Value,
    where_clause: &Value,
) -> Result<bool, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(table)?;
    let sanitized_col = sanitize_identifier(column)?;
    
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
    
    // Bind updated value
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

pub async fn delete_row(
    state: &AppState,
    db: &str,
    table: &str,
    where_clause: &Value,
) -> Result<u64, String> {
    let mut conn = state.get_connection(Some(db)).await?;
    let sanitized_table = sanitize_identifier(table)?;
    
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
    Ok(result.rows_affected())
}
