use sqlx::Row;
use serde_json::Value;
use crate::state::AppState;
use crate::db::models::{row_to_json, ServerStatus, ServerMetrics, ServerMetricsResponse, UserInfo};

pub async fn server_status(state: &AppState) -> Result<(ServerStatus, Vec<Value>), String> {
    let mut conn = state.get_connection(None).await?;
    
    let proc_rows = sqlx::query("SHOW FULL PROCESSLIST")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    let processes: Vec<Value> = proc_rows.iter().map(row_to_json).collect();
    
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
    
    Ok((
        ServerStatus {
            uptime,
            threads_connected,
            threads_running,
            queries,
            slow_queries,
        },
        processes,
    ))
}

pub async fn server_metrics(state: &AppState) -> Result<ServerMetricsResponse, String> {
    let mut conn = state.get_connection(None).await?;
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
        },
    })
}

pub async fn server_slow_log(state: &AppState) -> Result<Vec<Value>, String> {
    let mut conn = state.get_connection(None).await?;
    let var_row = sqlx::query("SHOW VARIABLES LIKE 'log_output'")
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    let val: String = var_row.and_then(|r| r.try_get("Value").ok()).unwrap_or_default();
    if !val.contains("TABLE") {
        return Err("Slow query logging to table is not enabled. Run: SET GLOBAL log_output = 'TABLE'; SET GLOBAL slow_query_log = 'ON';".to_string());
    }
    
    let rows = sqlx::query("SELECT * FROM mysql.slow_log ORDER BY start_time DESC LIMIT 100")
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(rows.iter().map(row_to_json).collect())
}

pub async fn list_users(state: &AppState) -> Result<(Vec<UserInfo>, Option<String>), String> {
    let mut conn = state.get_connection(None).await?;
    match sqlx::query("SELECT user, host, account_locked from mysql.user ORDER BY user ASC")
        .fetch_all(&mut *conn)
        .await 
    {
        Ok(rows) => {
            let list = rows.iter().map(|row| {
                UserInfo {
                    user: row.try_get("user").unwrap_or_default(),
                    host: row.try_get("host").unwrap_or_default(),
                    account_locked: row.try_get("account_locked").unwrap_or_else(|_| "N".to_string()),
                }
            }).collect();
            Ok((list, None))
        }
        Err(_) => {
            let self_row = sqlx::query("SELECT USER()")
                .fetch_one(&mut *conn)
                .await
                .map_err(|e| e.to_string())?;
            let user_self: String = self_row.try_get(0).unwrap_or_default();
            let parts: Vec<&str> = user_self.split('@').collect();
            let user = parts.first().copied().unwrap_or("root").to_string();
            let host = parts.get(1).copied().unwrap_or("localhost").to_string();
            
            Ok((
                vec![UserInfo {
                    user,
                    host,
                    account_locked: "N".to_string(),
                }],
                Some("Limited visibility: no privileges to access mysql.user table directly.".to_string()),
            ))
        }
    }
}
