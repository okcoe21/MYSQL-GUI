use sqlx::{Row, Column, Acquire};
use serde_json::Value;
use crate::state::AppState;
use crate::db::sanitize::sanitize_identifier;
use crate::db::models::{row_to_json, GetDataResponse, PaginationInfo, TableColumnInfo};

#[derive(Debug, Clone, PartialEq)]
pub enum BindValue {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[allow(dead_code)]
impl BindValue {
    pub fn is_null(&self) -> bool {
        matches!(self, BindValue::Null)
    }

    pub fn to_display_string(&self) -> String {
        match self {
            BindValue::Null => "NULL".to_string(),
            BindValue::String(s) => s.clone(),
            BindValue::Int(i) => i.to_string(),
            BindValue::Float(f) => f.to_string(),
            BindValue::Bool(b) => b.to_string(),
        }
    }
}

impl From<&Value> for BindValue {
    fn from(v: &Value) -> Self {
        match v {
            Value::Null => BindValue::Null,
            Value::Bool(b) => BindValue::Bool(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    BindValue::Int(i)
                } else if let Some(f) = n.as_f64() {
                    BindValue::Float(f)
                } else {
                    BindValue::String(n.to_string())
                }
            }
            Value::String(s) => BindValue::String(s.clone()),
            other => BindValue::String(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedUpdate {
    pub sql: String,
    pub select_sql: String,
    pub bindings: Vec<BindValue>,
    pub select_bindings: Vec<BindValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedInsert {
    pub sql: String,
    pub bindings: Vec<BindValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedDelete {
    pub sql: String,
    pub bindings: Vec<BindValue>,
}

fn bind_value<'a>(
    query: sqlx::query::Query<'a, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    val: &'a BindValue,
) -> sqlx::query::Query<'a, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match val {
        BindValue::Null => query.bind(None::<String>),
        BindValue::String(s) => query.bind(s.as_str()),
        BindValue::Int(i) => query.bind(*i),
        BindValue::Float(f) => query.bind(*f),
        BindValue::Bool(b) => query.bind(*b),
    }
}

pub fn build_update_query(
    table: &str,
    column: &str,
    new_value: BindValue,
    pk_values: &[(&str, BindValue)],
    columns_info: &[TableColumnInfo],
) -> Result<PreparedUpdate, String> {
    let sanitized_table = sanitize_identifier(table)?;
    let sanitized_col = sanitize_identifier(column)?;

    // 1. Column validation
    let target_col = columns_info
        .iter()
        .find(|c| c.field.eq_ignore_ascii_case(column))
        .ok_or_else(|| format!("Column '{}' does not exist in table '{}'", column, table))?;

    if target_col.is_read_only() {
        return Err(format!(
            "Column '{}' is not editable (generated, blob, binary, or spatial type)",
            column
        ));
    }

    // 2. Primary key validation
    let pri_cols: Vec<&TableColumnInfo> = columns_info.iter().filter(|c| c.is_primary()).collect();
    if pri_cols.is_empty() {
        return Err(format!(
            "Table '{}' has no primary key; cannot safely identify row for update",
            table
        ));
    }

    if let Some(bad_col) = pri_cols.iter().find(|c| c.has_unsupported_pk_type()) {
        return Err(format!(
            "Primary key column '{}' has unsupported type '{}' (FLOAT, DOUBLE, REAL, BLOB, BINARY); table is read-only",
            bad_col.field, bad_col.r#type
        ));
    }

    if pk_values.is_empty() {
        return Err("Refusing to update row without primary key condition".to_string());
    }

    if pk_values.len() != pri_cols.len() {
        return Err(format!(
            "Primary key count mismatch: expected {} PK column(s), got {}",
            pri_cols.len(),
            pk_values.len()
        ));
    }

    for pri in &pri_cols {
        if !pk_values.iter().any(|(name, _)| name.eq_ignore_ascii_case(&pri.field)) {
            return Err(format!("Missing primary key value for column '{}'", pri.field));
        }
    }

    // 3. Construct WHERE clause preserving primary key order
    let mut where_parts = Vec::new();
    let mut pk_bindings = Vec::new();

    for pri in &pri_cols {
        let sanitized_pk = sanitize_identifier(&pri.field)?;
        let (_, val) = pk_values
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&pri.field))
            .unwrap();
        where_parts.push(format!("{} = ?", sanitized_pk));
        pk_bindings.push(val.clone());
    }

    let where_clause = where_parts.join(" AND ");
    let select_sql = format!("SELECT 1 FROM {} WHERE {} FOR UPDATE", sanitized_table, where_clause);
    let sql = format!("UPDATE {} SET {} = ? WHERE {} LIMIT 1", sanitized_table, sanitized_col, where_clause);

    let mut bindings = vec![new_value];
    bindings.extend(pk_bindings.clone());

    Ok(PreparedUpdate {
        sql,
        select_sql,
        bindings,
        select_bindings: pk_bindings,
    })
}

pub fn build_insert_query(
    table: &str,
    fields: &[(&str, Option<BindValue>)],
    columns_info: &[TableColumnInfo],
) -> Result<PreparedInsert, String> {
    let sanitized_table = sanitize_identifier(table)?;

    let mut col_names = Vec::new();
    let mut placeholders = Vec::new();
    let mut bindings = Vec::new();

    for (col_name, opt_val) in fields {
        if let Some(col_info) = columns_info.iter().find(|c| c.field.eq_ignore_ascii_case(col_name)) {
            if col_info.is_generated() {
                return Err(format!("Cannot insert into generated column '{}'", col_info.field));
            }
        }

        // When opt_val is None, column is omitted to let MySQL apply DEFAULT or AUTO_INCREMENT
        if let Some(val) = opt_val {
            col_names.push(sanitize_identifier(col_name)?);
            placeholders.push("?");
            bindings.push(val.clone());
        }
    }

    let sql = if col_names.is_empty() {
        format!("INSERT INTO {} () VALUES ()", sanitized_table)
    } else {
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            sanitized_table,
            col_names.join(", "),
            placeholders.join(", ")
        )
    };

    Ok(PreparedInsert { sql, bindings })
}

pub fn build_delete_query(
    table: &str,
    pk_values: &[(&str, BindValue)],
    columns_info: &[TableColumnInfo],
) -> Result<PreparedDelete, String> {
    let sanitized_table = sanitize_identifier(table)?;

    let pri_cols: Vec<&TableColumnInfo> = columns_info.iter().filter(|c| c.is_primary()).collect();
    if pri_cols.is_empty() {
        return Err(format!(
            "Table '{}' has no primary key; cannot safely delete row",
            table
        ));
    }

    if let Some(bad_col) = pri_cols.iter().find(|c| c.has_unsupported_pk_type()) {
        return Err(format!(
            "Primary key column '{}' has unsupported type '{}' (FLOAT, DOUBLE, REAL, BLOB, BINARY); table is read-only",
            bad_col.field, bad_col.r#type
        ));
    }

    if pk_values.is_empty() {
        return Err("Refusing to delete row without primary key condition".to_string());
    }

    if pk_values.len() != pri_cols.len() {
        return Err(format!(
            "Primary key count mismatch: expected {} PK column(s), got {}",
            pri_cols.len(),
            pk_values.len()
        ));
    }

    for pri in &pri_cols {
        if !pk_values.iter().any(|(name, _)| name.eq_ignore_ascii_case(&pri.field)) {
            return Err(format!("Missing primary key value for column '{}'", pri.field));
        }
    }

    let mut where_parts = Vec::new();
    let mut bindings = Vec::new();

    for pri in &pri_cols {
        let sanitized_pk = sanitize_identifier(&pri.field)?;
        let (_, val) = pk_values
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&pri.field))
            .unwrap();
        where_parts.push(format!("{} = ?", sanitized_pk));
        bindings.push(val.clone());
    }

    let sql = format!("DELETE FROM {} WHERE {} LIMIT 1", sanitized_table, where_parts.join(" AND "));

    Ok(PreparedDelete { sql, bindings })
}

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
    fields: &[(&str, Option<BindValue>)],
    columns_info: &[TableColumnInfo],
) -> Result<u64, String> {
    let prepared = build_insert_query(table, fields, columns_info)?;
    let mut conn = state.get_connection(Some(db)).await?;

    let mut query = sqlx::query(&prepared.sql);
    for b in &prepared.bindings {
        query = bind_value(query, b);
    }

    let result = query.execute(&mut *conn).await.map_err(|e| e.to_string())?;
    Ok(result.last_insert_id())
}

pub async fn update_row(
    state: &AppState,
    db: &str,
    table: &str,
    column: &str,
    new_value: BindValue,
    pk_values: &[(&str, BindValue)],
    columns_info: &[TableColumnInfo],
) -> Result<bool, String> {
    let prepared = build_update_query(table, column, new_value, pk_values, columns_info)?;

    let mut conn = state.get_connection(Some(db)).await?;
    let mut tx = conn.begin().await.map_err(|e| e.to_string())?;

    // 1. Lock and verify row exists
    let mut select_q = sqlx::query(&prepared.select_sql);
    for b in &prepared.select_bindings {
        select_q = bind_value(select_q, b);
    }
    let row_exists = select_q
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if row_exists.is_none() {
        return Err("Row not found (may have been deleted or modified concurrently)".to_string());
    }

    // 2. Execute UPDATE
    let mut update_q = sqlx::query(&prepared.sql);
    for b in &prepared.bindings {
        update_q = bind_value(update_q, b);
    }
    update_q
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Commit transaction
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(true)
}

pub async fn delete_row(
    state: &AppState,
    db: &str,
    table: &str,
    pk_values: &[(&str, BindValue)],
    columns_info: &[TableColumnInfo],
) -> Result<u64, String> {
    let prepared = build_delete_query(table, pk_values, columns_info)?;
    let mut conn = state.get_connection(Some(db)).await?;

    let mut query = sqlx::query(&prepared.sql);
    for b in &prepared.bindings {
        query = bind_value(query, b);
    }

    let result = query.execute(&mut *conn).await.map_err(|e| e.to_string())?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_col(field: &str, col_type: &str, is_pri: bool, extra: &str, null: &str, default: Option<&str>) -> TableColumnInfo {
        TableColumnInfo {
            field: field.to_string(),
            r#type: col_type.to_string(),
            null: null.to_string(),
            key: if is_pri { "PRI".to_string() } else { "".to_string() },
            default: default.map(|s| s.to_string()),
            extra: extra.to_string(),
        }
    }

    #[test]
    fn test_composite_pk_missing_value() {
        let cols = vec![
            make_col("org_id", "int", true, "", "NO", None),
            make_col("user_id", "int", true, "", "NO", None),
            make_col("name", "varchar(100)", false, "", "YES", None),
        ];

        let pks = [("org_id", BindValue::Int(10))];
        let res = build_update_query("members", "name", BindValue::String("Bob".into()), &pks, &cols);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("Primary key count mismatch"), "Expected count mismatch error, got: {}", err);
    }

    #[test]
    fn test_pk_count_mismatch() {
        let cols = vec![
            make_col("id", "int", true, "auto_increment", "NO", None),
            make_col("name", "varchar(100)", false, "", "YES", None),
        ];

        let pks = [("id", BindValue::Int(1)), ("extra_pk", BindValue::Int(2))];
        let res = build_update_query("users", "name", BindValue::String("Bob".into()), &pks, &cols);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("Primary key count mismatch"), "Expected count mismatch, got: {}", err);
    }

    #[test]
    fn test_insert_use_default_omitting_column() {
        let cols = vec![
            make_col("id", "int", true, "auto_increment", "NO", None),
            make_col("name", "varchar(100)", false, "", "NO", None),
            make_col("role", "varchar(50)", false, "", "NO", Some("member")),
        ];

        let fields = [
            ("id", None), // Use default/auto_increment
            ("name", Some(BindValue::String("Alice".into()))),
            ("role", None), // Use default
        ];

        let res = build_insert_query("users", &fields, &cols).expect("Failed to build insert query");
        assert_eq!(res.sql, "INSERT INTO `users` (`name`) VALUES (?)");
        assert_eq!(res.bindings, vec![BindValue::String("Alice".into())]);
    }

    #[test]
    fn test_generated_column_rejected_on_update() {
        let cols = vec![
            make_col("id", "int", true, "auto_increment", "NO", None),
            make_col("first_name", "varchar(50)", false, "", "NO", None),
            make_col("full_name", "varchar(100)", false, "VIRTUAL GENERATED", "YES", None),
        ];

        let pks = [("id", BindValue::Int(1))];
        let res = build_update_query("users", "full_name", BindValue::String("Alice Smith".into()), &pks, &cols);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("is not editable"), "Expected not editable error, got: {}", err);
    }

    #[test]
    fn test_blob_and_spatial_columns_rejected_on_update() {
        let cols = vec![
            make_col("id", "int", true, "auto_increment", "NO", None),
            make_col("data_blob", "longblob", false, "", "YES", None),
            make_col("geo_point", "point", false, "", "YES", None),
        ];

        let pks = [("id", BindValue::Int(1))];
        let res_blob = build_update_query("tbl", "data_blob", BindValue::String("abc".into()), &pks, &cols);
        assert!(res_blob.is_err());
        assert!(res_blob.unwrap_err().contains("is not editable"));

        let res_geo = build_update_query("tbl", "geo_point", BindValue::String("POINT(1 1)".into()), &pks, &cols);
        assert!(res_geo.is_err());
        assert!(res_geo.unwrap_err().contains("is not editable"));
    }

    #[test]
    fn test_unsupported_pk_type_rejected() {
        let cols = vec![
            make_col("score", "float", true, "", "NO", None),
            make_col("label", "varchar(50)", false, "", "YES", None),
        ];

        let pks = [("score", BindValue::Float(9.5))];
        let res = build_update_query("scores", "label", BindValue::String("High".into()), &pks, &cols);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("unsupported type"), "Expected unsupported type error, got: {}", err);

        let del_res = build_delete_query("scores", &pks, &cols);
        assert!(del_res.is_err());
        assert!(del_res.unwrap_err().contains("unsupported type"));
    }

    #[test]
    fn test_no_pk_table_rejected() {
        let cols = vec![
            make_col("event", "varchar(50)", false, "", "YES", None),
            make_col("occurred_at", "datetime", false, "", "YES", None),
        ];

        let pks = [("event", BindValue::String("login".into()))];
        let update_res = build_update_query("logs", "event", BindValue::String("logout".into()), &pks, &cols);
        assert!(update_res.is_err());
        assert!(update_res.unwrap_err().contains("no primary key"));

        let del_res = build_delete_query("logs", &pks, &cols);
        assert!(del_res.is_err());
        assert!(del_res.unwrap_err().contains("no primary key"));
    }

    #[test]
    fn test_successful_update_query_building() {
        let cols = vec![
            make_col("id", "int", true, "auto_increment", "NO", None),
            make_col("email", "varchar(255)", false, "", "NO", None),
        ];

        let pks = [("id", BindValue::Int(42))];
        let res = build_update_query("users", "email", BindValue::String("dev@example.com".into()), &pks, &cols)
            .expect("Valid update query failed");

        assert_eq!(res.sql, "UPDATE `users` SET `email` = ? WHERE `id` = ? LIMIT 1");
        assert_eq!(res.select_sql, "SELECT 1 FROM `users` WHERE `id` = ? FOR UPDATE");
        assert_eq!(res.bindings, vec![BindValue::String("dev@example.com".into()), BindValue::Int(42)]);
        assert_eq!(res.select_bindings, vec![BindValue::Int(42)]);
    }

    #[test]
    fn test_successful_composite_pk_update() {
        let cols = vec![
            make_col("org_id", "int", true, "", "NO", None),
            make_col("user_id", "int", true, "", "NO", None),
            make_col("role", "varchar(50)", false, "", "NO", None),
        ];

        // Intentionally provide in reverse order to ensure column-order matching
        let pks = [("user_id", BindValue::Int(200)), ("org_id", BindValue::Int(100))];
        let res = build_update_query("memberships", "role", BindValue::String("admin".into()), &pks, &cols)
            .expect("Composite PK update failed");

        assert_eq!(res.sql, "UPDATE `memberships` SET `role` = ? WHERE `org_id` = ? AND `user_id` = ? LIMIT 1");
        assert_eq!(res.select_sql, "SELECT 1 FROM `memberships` WHERE `org_id` = ? AND `user_id` = ? FOR UPDATE");
        assert_eq!(res.bindings, vec![BindValue::String("admin".into()), BindValue::Int(100), BindValue::Int(200)]);
    }

    #[test]
    fn test_successful_delete_query_building() {
        let cols = vec![
            make_col("id", "int", true, "auto_increment", "NO", None),
            make_col("name", "varchar(50)", false, "", "YES", None),
        ];

        let pks = [("id", BindValue::Int(77))];
        let res = build_delete_query("users", &pks, &cols).expect("Delete query failed");
        assert_eq!(res.sql, "DELETE FROM `users` WHERE `id` = ? LIMIT 1");
        assert_eq!(res.bindings, vec![BindValue::Int(77)]);
    }
}
