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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_destructive_comments_and_whitespace() {
        assert!(is_destructive("  /*x*/ DROP\nTABLE t"));
        assert!(is_destructive("/*comment*/TRUNCATE TABLE users"));
        assert!(is_destructive("-- test\nDELETE FROM users;"));
        assert!(is_destructive("ALTER TABLE orders ADD col INT"));
        assert!(is_destructive("GRANT ALL ON *.* TO 'user'@'%'"));
        assert!(is_destructive("REVOKE ALL ON *.* FROM 'user'@'%'"));
        assert!(is_destructive("UPDATE users SET active = 1"));
        assert!(!is_destructive("UPDATE users SET active = 1 WHERE id = 5"));
        assert!(!is_destructive("SELECT * FROM users WHERE note = 'DROP TABLE'"));
    }
}
