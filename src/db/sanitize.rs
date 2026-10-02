/// Validates and wraps a MySQL identifier (database, table, column) in backticks.
/// Prevents SQL injection via identifiers by ensuring they only contain valid characters.
pub fn sanitize_identifier(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Invalid identifier scope: name cannot be empty".to_string());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
        return Err(format!("Invalid identifier name: '{}'. Only alphanumeric, '_' and '$' are permitted.", name));
    }
    Ok(format!("`{}`", name))
}

/// Checks whether a query contains potentially destructive operations.
pub fn is_destructive(query: &str) -> bool {
    let upper = query.to_uppercase();
    upper.contains("DROP ") || upper.contains("DELETE ") || upper.contains("TRUNCATE ") || upper.contains("ALTER ")
}
