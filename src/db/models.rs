use sqlx::{mysql::MySqlRow, Row, Column, TypeInfo, ValueRef};
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDataResponse {
    pub success: bool,
    pub data: Vec<Value>,
    pub columns: Vec<String>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerStatus {
    pub uptime: i64,
    pub threads_connected: i64,
    pub threads_running: i64,
    pub queries: i64,
    pub slow_queries: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub questions: i64,
    pub threads_connected: i64,
    pub threads_running: i64,
    pub bytes_received: i64,
    pub bytes_sent: i64,
    pub innodb_total: i64,
    pub innodb_free: i64,
    pub slow_queries: i64,
    pub uptime: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetricsResponse {
    pub success: bool,
    pub timestamp: u64,
    pub metrics: ServerMetrics,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefInput {
    pub name: String,
    pub r#type: String,
    pub length: Option<String>,
    pub is_null: Option<bool>,
    pub is_primary: Option<bool>,
    pub is_auto_increment: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStats {
    pub table_name: String,
    pub row_count: i64,
    pub data_size: i64,
    pub engine: String,
    pub collation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectsSummary {
    pub success: bool,
    pub views: Vec<String>,
    pub procedures: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub success: bool,
    pub data: Option<Vec<Value>>,
    pub columns: Vec<String>,
    pub affected_rows: Option<u64>,
    pub error: Option<String>,
    pub message: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnInfo {
    pub field: String,
    pub r#type: String,
    pub null: String,
    pub key: String,
    pub default: Option<String>,
    pub extra: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user: String,
    pub host: String,
    pub account_locked: String,
}

pub fn row_to_json(row: &MySqlRow) -> Value {
    let mut map = serde_json::Map::new();
    for col in row.columns() {
        let name = col.name();
        let value = if row.try_get_raw(name).map(|v| v.is_null()).unwrap_or(true) {
            Value::Null
        } else {
            let type_name = col.type_info().name();
            if type_name.contains("INT") || type_name == "INTEGER" {
                if let Ok(val) = row.try_get::<i64, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u64, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<i32, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u32, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<i16, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u16, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<i8, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.try_get::<u8, _>(name) {
                    Value::Number(serde_json::Number::from(val))
                } else {
                    Value::Null
                }
            } else {
                match type_name {
                    "FLOAT" | "DOUBLE" | "DECIMAL" => {
                        if let Ok(val) = row.try_get::<f64, _>(name) {
                            if let Some(num) = serde_json::Number::from_f64(val) {
                                Value::Number(num)
                            } else {
                                Value::Null
                            }
                        } else {
                            Value::Null
                        }
                    }
                    "VARCHAR" | "CHAR" | "TEXT" | "LONGTEXT" | "MEDIUMTEXT" | "TINYTEXT" | "ENUM" | "SET" => {
                        if let Ok(val) = row.try_get::<String, _>(name) {
                            Value::String(val)
                        } else {
                            Value::Null
                        }
                    }
                    "DATE" | "DATETIME" | "TIMESTAMP" | "TIME" => {
                        if let Ok(val) = row.try_get::<chrono::NaiveDateTime, _>(name) {
                            Value::String(val.format("%Y-%m-%d %H:%M:%S").to_string())
                        } else if let Ok(val) = row.try_get::<chrono::NaiveDate, _>(name) {
                            Value::String(val.to_string())
                        } else if let Ok(val) = row.try_get::<String, _>(name) {
                            Value::String(val)
                        } else {
                            Value::Null
                        }
                    }
                    _ => {
                        if let Ok(val) = row.try_get::<String, _>(name) {
                            Value::String(val)
                        } else if let Ok(val) = row.try_get::<i64, _>(name) {
                            Value::Number(serde_json::Number::from(val))
                        } else if let Ok(val) = row.try_get::<f64, _>(name) {
                            if let Some(num) = serde_json::Number::from_f64(val) {
                                Value::Number(num)
                            } else {
                                Value::Null
                            }
                        } else if let Ok(val) = row.try_get::<Vec<u8>, _>(name) {
                            if let Ok(s) = String::from_utf8(val) {
                                Value::String(s)
                            } else {
                                Value::String("[Binary data]".to_string())
                            }
                        } else {
                            Value::Null
                        }
                    }
                }
            }
        };
        map.insert(name.to_string(), value);
    }
    Value::Object(map)
}
