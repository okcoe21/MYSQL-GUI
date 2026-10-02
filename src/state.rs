use std::sync::{Arc, Mutex};
use sqlx::{MySqlPool, Executor};
use crate::db::sanitize::sanitize_identifier;

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub user: String,
}

pub struct AppState {
    pub pool: Mutex<Option<MySqlPool>>,
    pub current_db: Mutex<Option<String>>,
    pub info: Mutex<Option<ConnectionInfo>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            pool: Mutex::new(None),
            current_db: Mutex::new(None),
            info: Mutex::new(None),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_pool(&self) -> Result<MySqlPool, String> {
        let guard = self.pool.lock().map_err(|e| e.to_string())?;
        guard.as_ref().cloned().ok_or_else(|| "Not connected. No active database connection pool.".to_string())
    }

    pub async fn get_connection(&self, db: Option<&str>) -> Result<sqlx::pool::PoolConnection<sqlx::MySql>, String> {
        let pool = self.get_pool()?;
        let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
        if let Some(db_name) = db {
            let sanitized = sanitize_identifier(db_name)?;
            let use_query = format!("USE {}", sanitized);
            conn.execute(use_query.as_str()).await.map_err(|e| e.to_string())?;
        }
        Ok(conn)
    }

    pub fn set_current_db(&self, db: Option<String>) {
        if let Ok(mut guard) = self.current_db.lock() {
            *guard = db;
        }
    }

    #[allow(dead_code)]
    pub fn get_current_db(&self) -> Option<String> {
        self.current_db.lock().ok().and_then(|guard| guard.clone())
    }

    #[allow(dead_code)]
    pub fn get_connection_info(&self) -> Option<ConnectionInfo> {
        self.info.lock().ok().and_then(|guard| guard.clone())
    }
}

pub type SharedState = Arc<AppState>;
