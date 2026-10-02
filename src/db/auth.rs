use sqlx::mysql::MySqlPoolOptions;
use crate::state::{AppState, ConnectionInfo};

pub async fn login(
    state: &AppState,
    host: &str,
    port: &str,
    user: &str,
    password: Option<&str>,
) -> Result<bool, String> {
    let port_num = port.parse::<u16>().map_err(|_| "Invalid port number. Port must be between 1 and 65535.".to_string())?;
    let pool_options = MySqlPoolOptions::new()
        .max_connections(5)
        .idle_timeout(std::time::Duration::from_secs(60));
        
    let url = format!(
        "mysql://{}:{}@{}:{}/",
        user,
        password.unwrap_or_default(),
        host,
        port_num
    );
    
    let pool = pool_options.connect(&url).await.map_err(|e| e.to_string())?;
    
    {
        let mut pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
        *pool_guard = Some(pool);
    }
    {
        let mut info_guard = state.info.lock().map_err(|e| e.to_string())?;
        *info_guard = Some(ConnectionInfo {
            host: host.to_string(),
            port: port_num,
            user: user.to_string(),
        });
    }
    state.set_current_db(None);
    
    Ok(true)
}

pub async fn logout(state: &AppState) -> Result<bool, String> {
    {
        let mut pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
        *pool_guard = None;
    }
    {
        let mut info_guard = state.info.lock().map_err(|e| e.to_string())?;
        *info_guard = None;
    }
    state.set_current_db(None);
    Ok(true)
}
