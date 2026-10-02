/// Validates and wraps a MySQL identifier (database, table, column) in backticks.
/// Prevents SQL injection via identifiers by ensuring they only contain valid characters.
pub fn sanitize_identifier(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Invalid identifier scope: name cannot be empty".to_string());
    }
    if name.chars().count() > 64 {
        return Err(format!("Invalid identifier: '{}' exceeds MySQL maximum length of 64 characters", name));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
        return Err(format!("Invalid identifier name: '{}'. Only alphanumeric, '_' and '$' are permitted.", name));
    }
    Ok(format!("`{}`", name))
}

/// Escapes a string literal for safe inclusion inside MySQL single quotes ('...').
/// Escapes backslashes first, then single quotes, followed by NUL (\0), \n, \r, and \x1a (Ctrl+Z).
pub fn escape_sql_string(val: &str) -> String {
    let mut escaped = String::with_capacity(val.len() + 16);
    for c in val.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("''"),
            '\0' => escaped.push_str("\\0"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\x1a' => escaped.push_str("\\Z"),
            _ => escaped.push(c),
        }
    }
    escaped
}

/// Validates a column length or type parameter (e.g. "255", "10,2", or "'val1','val2'").
/// Rejects any unexpected characters, injection payloads, or unclosed quotes.
pub fn validate_column_length(len: &str) -> Result<String, String> {
    let trimmed = len.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    // Case 1: digits or precision with comma (e.g., "10", "10,2", "10, 2")
    let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
    if !parts.is_empty() && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
        return Ok(parts.join(", "));
    }

    // Case 2: Quoted ENUM or SET value list (e.g., 'active','inactive')
    if trimmed.starts_with('\'') {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;
        let mut chars = trimmed.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\'' => {
                    if in_quote {
                        if let Some(&'\'') = chars.peek() {
                            chars.next();
                            current.push('\'');
                        } else {
                            in_quote = false;
                            values.push(format!("'{}'", escape_sql_string(&current)));
                            current.clear();
                        }
                    } else {
                        in_quote = true;
                    }
                }
                ',' if !in_quote => {
                    // separator
                }
                c if !in_quote => {
                    if !c.is_whitespace() {
                        return Err(format!("Invalid character outside quote in ENUM/SET values: '{}'", c));
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if in_quote {
            return Err("Unclosed quote in ENUM/SET length specifier".to_string());
        }

        if !values.is_empty() {
            return Ok(values.join(", "));
        }
    }

    Err(format!(
        "Invalid length or value specifier: '{}'. Must be numeric (e.g. 255), precision (e.g. 10,2), or quoted ENUM/SET values.",
        trimmed
    ))
}

/// Client-side UX safeguard to prompt the user for confirmation before executing
/// potentially high-impact or destructive operations.
///
/// NOTE: This is purely a UX guard to prevent accidental data loss, NOT a security
/// sandbox or access-control boundary. The SQL editor intentionally allows execution
/// of arbitrary user SQL.
pub fn is_destructive(query: &str) -> bool {
    let mut s = query;
    // Strip leading whitespace and comments (-- line comments, # line comments, /* block comments */)
    loop {
        s = s.trim_start();
        if s.starts_with("--") {
            if let Some(idx) = s.find('\n') {
                s = &s[idx + 1..];
                continue;
            } else {
                return false;
            }
        }
        if s.starts_with('#') {
            if let Some(idx) = s.find('\n') {
                s = &s[idx + 1..];
                continue;
            } else {
                return false;
            }
        }
        if s.starts_with("/*") {
            if let Some(idx) = s.find("*/") {
                s = &s[idx + 2..];
                continue;
            } else {
                return false;
            }
        }
        break;
    }

    // Extract the first word/keyword using word boundaries
    let first_word: String = s.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    let first_upper = first_word.to_uppercase();

    match first_upper.as_str() {
        "DROP" | "TRUNCATE" | "DELETE" | "ALTER" | "GRANT" | "REVOKE" => true,
        "UPDATE" => {
            // Check if UPDATE statement has no WHERE clause
            let upper = s.to_uppercase();
            let has_where = upper
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|token| token == "WHERE");
            !has_where
        }
        _ => false,
    }
}

/// Whitelists the SQL sort direction, defaulting to "ASC" if not "DESC" (case-insensitive).
#[allow(dead_code)]
pub fn sanitize_sort_direction(order: Option<&str>) -> &'static str {
    match order {
        Some(s) if s.trim().eq_ignore_ascii_case("DESC") => "DESC",
        _ => "ASC",
    }
}

/// Clamps limit and offset to safe boundaries to prevent negative values or excessive allocations.
#[allow(dead_code)]
pub fn clamp_limit_offset(limit: i64, offset: i64) -> (i64, i64) {
    let clamped_limit = if limit <= 0 { 50 } else { limit.min(10_000) };
    let clamped_offset = offset.max(0);
    (clamped_limit, clamped_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier_valid_plain() {
        assert_eq!(sanitize_identifier("users").unwrap(), "`users`");
        assert_eq!(sanitize_identifier("order_items_2026").unwrap(), "`order_items_2026`");
        assert_eq!(sanitize_identifier("$user_data").unwrap(), "`$user_data`");
    }

    #[test]
    fn test_sanitize_identifier_rejects_empty() {
        assert!(sanitize_identifier("").is_err());
    }

    #[test]
    fn test_sanitize_identifier_rejects_spaces() {
        assert!(sanitize_identifier("name with spaces").is_err());
        assert!(sanitize_identifier(" users").is_err());
        assert!(sanitize_identifier("users ").is_err());
    }

    #[test]
    fn test_sanitize_identifier_rejects_embedded_backtick() {
        assert!(sanitize_identifier("users`injection").is_err());
        assert!(sanitize_identifier("`users`").is_err());
    }

    #[test]
    fn test_sanitize_identifier_rejects_nul_byte() {
        assert!(sanitize_identifier("users\0table").is_err());
        assert!(sanitize_identifier("\0").is_err());
    }

    #[test]
    fn test_sanitize_identifier_length_boundary() {
        let name_64 = "a".repeat(64);
        assert_eq!(sanitize_identifier(&name_64).unwrap(), format!("`{}`", name_64));

        let name_65 = "a".repeat(65);
        assert!(sanitize_identifier(&name_65).is_err());
    }

    #[test]
    fn test_sanitize_identifier_rejects_injection_attempt() {
        assert!(sanitize_identifier("users`; DROP TABLE x;--").is_err());
        assert!(sanitize_identifier("users OR 1=1").is_err());
        assert!(sanitize_identifier("users'--").is_err());
    }

    #[test]
    fn test_sanitize_sort_direction() {
        assert_eq!(sanitize_sort_direction(Some("DESC")), "DESC");
        assert_eq!(sanitize_sort_direction(Some("desc")), "DESC");
        assert_eq!(sanitize_sort_direction(Some("  desc  ")), "DESC");
        assert_eq!(sanitize_sort_direction(Some("ASC")), "ASC");
        assert_eq!(sanitize_sort_direction(Some("asc")), "ASC");
        assert_eq!(sanitize_sort_direction(None), "ASC");
        assert_eq!(sanitize_sort_direction(Some("invalid; DROP TABLE users;")), "ASC");
    }

    #[test]
    fn test_clamp_limit_offset() {
        assert_eq!(clamp_limit_offset(50, 0), (50, 0));
        assert_eq!(clamp_limit_offset(100, 200), (100, 200));
        assert_eq!(clamp_limit_offset(-5, -10), (50, 0));
        assert_eq!(clamp_limit_offset(0, -1), (50, 0));
        assert_eq!(clamp_limit_offset(50_000, 10), (10_000, 10));
    }

    #[test]
    fn test_escape_sql_string_special_chars() {
        assert_eq!(escape_sql_string("hello"), "hello");
        assert_eq!(escape_sql_string("it's a test"), "it''s a test");
        assert_eq!(escape_sql_string("path\\to\\file"), "path\\\\to\\\\file");
        assert_eq!(escape_sql_string("null\0byte"), "null\\0byte");
        assert_eq!(escape_sql_string("new\nline\rret"), "new\\nline\\rret");
        assert_eq!(escape_sql_string("ctrl\x1a_end"), "ctrl\\Z_end");
        assert_eq!(escape_sql_string("slash\\'quote"), "slash\\\\''quote");
    }

    #[test]
    fn test_validate_column_length_numeric_and_precision() {
        assert_eq!(validate_column_length("255").unwrap(), "255");
        assert_eq!(validate_column_length("10,2").unwrap(), "10, 2");
        assert_eq!(validate_column_length("10, 2").unwrap(), "10, 2");
        assert_eq!(validate_column_length("").unwrap(), "");
        assert_eq!(validate_column_length("   ").unwrap(), "");
    }

    #[test]
    fn test_validate_column_length_enum_set() {
        assert_eq!(
            validate_column_length("'small','medium','large'").unwrap(),
            "'small', 'medium', 'large'"
        );
        assert_eq!(
            validate_column_length("'it''s','other'").unwrap(),
            "'it''s', 'other'"
        );
        assert!(validate_column_length("'unclosed").is_err());
        assert!(validate_column_length("'valid', invalid").is_err());
    }

    #[test]
    fn test_validate_column_length_rejects_injections() {
        assert!(validate_column_length("10) DEFAULT 0; DROP TABLE x; --").is_err());
        assert!(validate_column_length("255; SELECT *").is_err());
        assert!(validate_column_length("10 OR 1=1").is_err());
    }

    #[test]
    fn test_is_destructive_comments_and_whitespace() {
        assert!(is_destructive("  /*x*/ DROP\nTABLE t"));
        assert!(is_destructive("/*comment*/TRUNCATE TABLE users"));
        assert!(is_destructive("-- test\nDELETE FROM users;"));
        assert!(is_destructive("ALTER TABLE orders ADD col INT"));
        assert!(is_destructive("GRANT ALL ON *.* TO 'user'@'%'"));
        assert!(is_destructive("REVOKE ALL ON *.* FROM 'user'@'%'"));
        assert!(!is_destructive("SELECT * FROM users WHERE note = 'DROP TABLE'"));
    }

    #[test]
    fn test_is_destructive_update_where() {
        assert!(is_destructive("UPDATE users SET active = 1"));
        assert!(!is_destructive("UPDATE users SET active = 1 WHERE id = 5"));
        assert!(!is_destructive("UPDATE users SET note = 'no where clause' WHERE user_id = 10"));
    }
}
