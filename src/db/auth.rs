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

/// Pure function mapping the require_ssl boolean toggle to MySQL SSL mode.
/// When require_ssl is true, returns MySqlSslMode::Required.
/// When false, defaults to MySqlSslMode::Preferred.
pub fn resolve_ssl_mode(require_ssl: bool) -> MySqlSslMode {
    if require_ssl {
        MySqlSslMode::Required
    } else {
        MySqlSslMode::Preferred
    }
}

/// Pure function mapping a connection error string and require_ssl flag into a user-facing error message.
/// Ensures no credentials leak, and maps SSL-requirement failures to a clear instruction.
pub fn map_login_error(error_msg: &str, require_ssl: bool, password: Option<&str>) -> String {
    let mut scrubbed = error_msg.to_string();
    if let Some(pwd) = password {
        if !pwd.is_empty() {
            scrubbed = scrubbed.replace(pwd, "******");
        }
    }

    if require_ssl {
        let lower = scrubbed.to_lowercase();
        if lower.contains("does not support ssl")
            || lower.contains("does not support tls")
            || lower.contains("tls is not supported")
            || lower.contains("ssl is not supported")
            || lower.contains("not supported by server")
            || lower.contains("server does not support")
            || lower.contains("tls error")
            || lower.contains("ssl error")
            || lower.contains("handshake failure")
            || lower.contains("handshake failed")
            || lower.contains("tls connection error")
            || lower.contains("ssl connection error")
        {
            return "Server does not support SSL. Turn off Require SSL to connect unencrypted.".to_string();
        }
    }

    scrubbed
}

pub async fn login(
    state: &AppState,
    host: &str,
    port: &str,
    user: &str,
    password: Option<&str>,
    require_ssl: bool,
) -> Result<ConnectionResult, String> {
    login_with_db(state, host, port, user, password, None, resolve_ssl_mode(require_ssl)).await
}

pub async fn login_with_db(
    state: &AppState,
    host: &str,
    port: &str,
    user: &str,
    password: Option<&str>,
    database: Option<&str>,
    ssl_mode: MySqlSslMode,
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
        .ssl_mode(ssl_mode);

    if let Some(pwd) = password {
        connect_options = connect_options.password(pwd);
    }

    if let Some(db) = database {
        if !db.trim().is_empty() {
            connect_options = connect_options.database(db);
        }
    }

    let is_ssl_required = matches!(ssl_mode, MySqlSslMode::Required);
    let pool = pool_options.connect_with(connect_options).await.map_err(|e| {
        let raw_err = e.to_string();
        let mapped = map_login_error(&raw_err, is_ssl_required, password);
        if mapped == "Server does not support SSL. Turn off Require SSL to connect unencrypted." {
            mapped
        } else {
            format!("Failed to connect to MySQL host '{}:{}': {}", host, port_num, mapped)
        }
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
            is_encrypted,
        });
    }
    state.set_is_encrypted(is_encrypted);
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
    state.set_is_encrypted(false);
    state.set_current_db(None);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_ssl_mode() {
        assert!(matches!(resolve_ssl_mode(true), MySqlSslMode::Required));
        assert!(matches!(resolve_ssl_mode(false), MySqlSslMode::Preferred));
    }

    #[test]
    fn test_map_login_error_ssl_required_unsupported() {
        let err1 = "server does not support TLS";
        assert_eq!(
            map_login_error(err1, true, None),
            "Server does not support SSL. Turn off Require SSL to connect unencrypted."
        );

        let err2 = "The server does not support SSL connections";
        assert_eq!(
            map_login_error(err2, true, None),
            "Server does not support SSL. Turn off Require SSL to connect unencrypted."
        );

        let err3 = "TLS handshake failed: unexpected message";
        assert_eq!(
            map_login_error(err3, true, None),
            "Server does not support SSL. Turn off Require SSL to connect unencrypted."
        );
    }

    #[test]
    fn test_map_login_error_ssl_preferred_not_intercepted() {
        let err = "server does not support TLS";
        assert_eq!(map_login_error(err, false, None), "server does not support TLS");
    }

    #[test]
    fn test_map_login_error_preserves_unrelated_errors() {
        let err = "Access denied for user 'root'@'localhost'";
        assert_eq!(map_login_error(err, true, None), "Access denied for user 'root'@'localhost'");
        assert_eq!(map_login_error(err, false, None), "Access denied for user 'root'@'localhost'");
    }

    #[test]
    fn test_map_login_error_redacts_password() {
        let err = "Failed auth with password secretP@ss999 on port 3306";
        let mapped = map_login_error(err, false, Some("secretP@ss999"));
        assert!(!mapped.contains("secretP@ss999"));
        assert!(mapped.contains("******"));
    }
}
