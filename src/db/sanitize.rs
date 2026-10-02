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

/// Checks whether a query contains potentially destructive operations.
pub fn is_destructive(query: &str) -> bool {
    let upper = query.to_uppercase();
    upper.contains("DROP ") || upper.contains("DELETE ") || upper.contains("TRUNCATE ") || upper.contains("ALTER ")
}
