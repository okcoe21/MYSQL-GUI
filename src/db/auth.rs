use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::Row;
use crate::state::{AppState, ConnectionInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionResult {
    pub is_encrypted: bool,
}

impl std::ops::Deref for ConnectionResult {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.is_encrypted
    }
}

impl From<bool> for ConnectionResult {
    fn from(is_encrypted: bool) -> Self {
        Self { is_encrypted }
    }
}

#[allow(dead_code)]
pub type LoginResult = ConnectionResult;

pub async fn login(
    state: &AppState,
    host: &str,
    port: &str,
    user: &str,
    password: Option<&str>,
) -> Result<ConnectionResult, String> {
    login_with_db(state, host, port, user, password, None).await
}

pub async fn login_with_db(
    state: &AppState,
    host: &str,
    port: &str,
    user: &str,
    password: Option<&str>,
    database: Option<&str>,
) -> Result<ConnectionResult, String> {
    let port_num = port
        .parse::<u16>()
        .map_err(|_| "Invalid port number. Port must be between 1 and 65535.".to_string())?;

    let pool_options = MySqlPoolOptions::new()
        .max_connections(5)
        .idle_timeout(std::time::Duration::from_secs(60));

    let mut connect_options = MySqlConnectOptions::new()
        .host(host)
        .port(port_num)
        .username(user)
        .ssl_mode(MySqlSslMode::Preferred);

    if let Some(pwd) = password {
        connect_options = connect_options.password(pwd);
    }

    if let Some(db) = database {
        if !db.trim().is_empty() {
            connect_options = connect_options.database(db);
        }
    }

    let pool = pool_options.connect_with(connect_options).await.map_err(|e| {
        let mut msg = e.to_string();
        if let Some(pwd) = password {
            if !pwd.is_empty() {
                msg = msg.replace(pwd, "******");
            }
        }
        format!("Failed to connect to MySQL host '{}:{}': {}", host, port_num, msg)
    })?;

    // Check whether the established session is encrypted with SSL/TLS
    let is_encrypted = {
        if let Ok(mut conn) = pool.acquire().await {
            let row = sqlx::query("SHOW STATUS LIKE 'Ssl_cipher'")
                .fetch_optional(&mut *conn)
                .await
                .ok()
                .flatten();
            row.and_then(|r| r.try_get::<String, _>(1).or_else(|_| r.try_get::<String, _>("Value")).ok())
                .map(|val| !val.trim().is_empty())
                .unwrap_or(false)
        } else {
            false
        }
    };

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
    state.set_current_db(database.map(|s| s.to_string()));

    Ok(ConnectionResult { is_encrypted })
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
