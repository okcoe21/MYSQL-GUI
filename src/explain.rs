/// Rule-based, offline SQL Query Explainer.
///
/// Translates SQL statements into plain English explanations, breaks down clauses,
/// and flags potentially dangerous operations (destructive updates, mass deletes,
/// unconstrained SELECT *, drops, truncates) without requiring any network or database connection.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    pub summary: String,
    pub lines: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub upper: String,
    pub is_quoted: bool,
}

/// Strips single-line (`--`, `#`) and block (`/* ... */`) comments outside of string literals.
/// Replaces stripped comments with a space to prevent token merging.
pub fn strip_comments(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_backtick = false;

    while i < len {
        let c = chars[i];

        if in_single_quote {
            result.push(c);
            if c == '\\' && i + 1 < len {
                i += 1;
                result.push(chars[i]);
            } else if c == '\'' {
                if i + 1 < len && chars[i + 1] == '\'' {
                    i += 1;
                    result.push('\'');
                } else {
                    in_single_quote = false;
                }
            }
            i += 1;
            continue;
        }

        if in_double_quote {
            result.push(c);
            if c == '\\' && i + 1 < len {
                i += 1;
                result.push(chars[i]);
            } else if c == '"' {
                if i + 1 < len && chars[i + 1] == '"' {
                    i += 1;
                    result.push('"');
                } else {
                    in_double_quote = false;
                }
            }
            i += 1;
            continue;
        }

        if in_backtick {
            result.push(c);
            if c == '`' {
                if i + 1 < len && chars[i + 1] == '`' {
                    i += 1;
                    result.push('`');
                } else {
                    in_backtick = false;
                }
            }
            i += 1;
            continue;
        }

        // Outside quotes
        if c == '\'' {
            in_single_quote = true;
            result.push(c);
            i += 1;
        } else if c == '"' {
            in_double_quote = true;
            result.push(c);
            i += 1;
        } else if c == '`' {
            in_backtick = true;
            result.push(c);
            i += 1;
        } else if c == '-' && i + 1 < len && chars[i + 1] == '-' {
            i += 2;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            result.push(' ');
        } else if c == '#' {
            i += 1;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            result.push(' ');
        } else if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2; // skip */
            } else {
                i = len;
            }
            result.push(' ');
        } else {
            result.push(c);
            i += 1;
        }
    }

    result
}

/// Splits a SQL string on semicolons outside of quotes into individual statement strings.
pub fn split_statements(sql: &str) -> Vec<String> {
    let mut stmts = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_backtick = false;

    while i < len {
        let c = chars[i];

        if in_single_quote {
            current.push(c);
            if c == '\\' && i + 1 < len {
                i += 1;
                current.push(chars[i]);
            } else if c == '\'' {
                if i + 1 < len && chars[i + 1] == '\'' {
                    i += 1;
                    current.push('\'');
                } else {
                    in_single_quote = false;
                }
            }
            i += 1;
            continue;
        }

        if in_double_quote {
            current.push(c);
            if c == '\\' && i + 1 < len {
                i += 1;
                current.push(chars[i]);
            } else if c == '"' {
                if i + 1 < len && chars[i + 1] == '"' {
                    i += 1;
                    current.push('"');
                } else {
                    in_double_quote = false;
                }
            }
            i += 1;
            continue;
        }

        if in_backtick {
            current.push(c);
            if c == '`' {
                if i + 1 < len && chars[i + 1] == '`' {
                    i += 1;
                    current.push('`');
                } else {
                    in_backtick = false;
                }
            }
            i += 1;
            continue;
        }

        if c == '\'' {
            in_single_quote = true;
            current.push(c);
            i += 1;
        } else if c == '"' {
            in_double_quote = true;
            current.push(c);
            i += 1;
        } else if c == '`' {
            in_backtick = true;
            current.push(c);
            i += 1;
        } else if c == ';' {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                stmts.push(trimmed.to_string());
            }
            current.clear();
            i += 1;
        } else {
            current.push(c);
            i += 1;
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        stmts.push(trimmed.to_string());
    }

    stmts
}

/// Tokenizes a single SQL statement into a sequence of Tokens.
pub fn tokenize(stmt: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = stmt.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Single-quoted string literal
        if c == '\'' {
            let mut s = String::new();
            s.push('\'');
            i += 1;
            while i < len {
                let sc = chars[i];
                s.push(sc);
                if sc == '\\' && i + 1 < len {
                    i += 1;
                    s.push(chars[i]);
                } else if sc == '\'' {
                    if i + 1 < len && chars[i + 1] == '\'' {
                        i += 1;
                        s.push('\'');
                    } else {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            let upper = s.to_uppercase();
            tokens.push(Token {
                text: s,
                upper,
                is_quoted: true,
            });
            continue;
        }

        // Double-quoted string literal
        if c == '"' {
            let mut s = String::new();
            s.push('"');
            i += 1;
            while i < len {
                let sc = chars[i];
                s.push(sc);
                if sc == '\\' && i + 1 < len {
                    i += 1;
                    s.push(chars[i]);
                } else if sc == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        i += 1;
                        s.push('"');
                    } else {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            let upper = s.to_uppercase();
            tokens.push(Token {
                text: s,
                upper,
                is_quoted: true,
            });
            continue;
        }

        // Backtick quoted identifier: strip surrounding backticks for natural readability
        if c == '`' {
            let mut s = String::new();
            i += 1;
            while i < len {
                let sc = chars[i];
                if sc == '`' {
                    if i + 1 < len && chars[i + 1] == '`' {
                        i += 1;
                        s.push('`');
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    s.push(sc);
                }
                i += 1;
            }
            let upper = s.to_uppercase();
            tokens.push(Token {
                text: s,
                upper,
                is_quoted: false,
            });
            continue;
        }

        // Multi-character operators
        if i + 1 < len {
            let two: String = chars[i..i + 2].iter().collect();
            if matches!(two.as_str(), "!=" | "<>" | ">=" | "<=" | ":=") {
                tokens.push(Token {
                    text: two.clone(),
                    upper: two,
                    is_quoted: false,
                });
                i += 2;
                continue;
            }
        }

        // Single-character punctuation and operators
        if matches!(c, '=' | '>' | '<' | '(' | ')' | ',' | '*' | '+' | '-' | '/' | ';') {
            let s = c.to_string();
            tokens.push(Token {
                text: s.clone(),
                upper: s,
                is_quoted: false,
            });
            i += 1;
            continue;
        }

        // Words / identifiers / numbers: alphanumeric, '_', '$', '.'
        if c.is_alphanumeric() || c == '_' || c == '$' || c == '.' {
            let mut s = String::new();
            while i < len
                && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$' || chars[i] == '.')
            {
                s.push(chars[i]);
                i += 1;
            }
            let upper = s.to_uppercase();
            tokens.push(Token {
                text: s,
                upper,
                is_quoted: false,
            });
            continue;
        }

        // Any other character
        let s = c.to_string();
        tokens.push(Token {
            text: s.clone(),
            upper: s,
            is_quoted: false,
        });
        i += 1;
    }

    tokens
}

/// Splits tokens by comma `,` at parenthesis depth 0.
fn split_by_comma(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let mut depth: usize = 0;

    for t in tokens {
        if !t.is_quoted && t.text == "(" {
            depth += 1;
        } else if !t.is_quoted && t.text == ")" {
            depth = depth.saturating_sub(1);
        }

        if !t.is_quoted && t.text == "," && depth == 0 {
            parts.push(current);
            current = Vec::new();
        } else {
            current.push(t.clone());
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Converts a slice of tokens to a string, respecting natural spacing around parentheses and commas.
fn tokens_to_string(tokens: &[Token]) -> String {
    let mut out = String::new();
    for (idx, t) in tokens.iter().enumerate() {
        if idx == 0 {
            out.push_str(&t.text);
        } else {
            let prev = &tokens[idx - 1];
            if t.text == "," || t.text == ";" || t.text == ")" || prev.text == "(" {
                out.push_str(&t.text);
            } else {
                out.push(' ');
                out.push_str(&t.text);
            }
        }
    }
    out
}

/// Formats a SQL type token sequence, ensuring no space before parentheses (e.g. VARCHAR(50)).
fn format_type_string(tokens: &[Token]) -> String {
    tokens_to_string(tokens).replace(" (", "(")
}

/// Formats a list of English phrases: "A", "A and B", "A, B and C".
fn format_list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let head = items[..items.len() - 1].join(", ");
            format!("{}, and {}", head, items.last().unwrap())
        }
    }
}

/// Translates SQL condition operators into plain English.
pub fn format_condition(tokens: &[Token]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let len = tokens.len();
    let mut i = 0;

    while i < len {
        let t = &tokens[i];
        let next = tokens.get(i + 1);
        let next2 = tokens.get(i + 2);

        // IS NOT NULL / IS NULL
        if !t.is_quoted && t.upper == "IS" {
            if let (Some(n1), Some(n2)) = (next, next2) {
                if !n1.is_quoted && n1.upper == "NOT" && !n2.is_quoted && n2.upper == "NULL" {
                    parts.push("is not empty".to_string());
                    i += 3;
                    continue;
                }
            }
            if let Some(n1) = next {
                if !n1.is_quoted && n1.upper == "NULL" {
                    parts.push("is empty".to_string());
                    i += 2;
                    continue;
                }
            }
        }

        // NOT LIKE / NOT IN
        if !t.is_quoted && t.upper == "NOT" {
            if let Some(n1) = next {
                if !n1.is_quoted && n1.upper == "LIKE" {
                    parts.push("does not match pattern".to_string());
                    i += 2;
                    continue;
                }
                if !n1.is_quoted && n1.upper == "IN" {
                    parts.push("is not one of".to_string());
                    i += 2;
                    continue;
                }
            }
        }

        if !t.is_quoted {
            match t.upper.as_str() {
                "=" => parts.push("equals".to_string()),
                "!=" | "<>" => parts.push("is not".to_string()),
                ">" => parts.push("is greater than".to_string()),
                "<" => parts.push("is less than".to_string()),
                ">=" => parts.push("at least".to_string()),
                "<=" => parts.push("at most".to_string()),
                "LIKE" => parts.push("matches pattern".to_string()),
                "IN" => parts.push("is one of".to_string()),
                "BETWEEN" => parts.push("is between".to_string()),
                "AND" => parts.push("and".to_string()),
                "OR" => parts.push("or".to_string()),
                _ => parts.push(t.text.clone()),
            }
        } else {
            parts.push(t.text.clone());
        }

        i += 1;
    }

    // Join with natural spacing
    let mut out = String::new();
    for (idx, p) in parts.iter().enumerate() {
        if idx == 0 {
            out.push_str(p);
        } else {
            let prev = &parts[idx - 1];
            if p == "," || p == ")" || prev == "(" {
                out.push_str(p);
            } else {
                out.push(' ');
                out.push_str(p);
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
enum ClauseKind {
    From,
    Join(String),
    Where,
    GroupBy,
    Having,
    OrderBy,
    Limit,
}

#[derive(Debug, Clone)]
struct ClauseBoundary {
    kind: ClauseKind,
    start: usize,
    len: usize,
}

/// Handles SELECT statement explanation.
fn handle_select(tokens: &[Token]) -> Explanation {
    let mut boundaries: Vec<ClauseBoundary> = Vec::new();
    let mut depth: usize = 0;
    let mut has_subquery = false;

    let len = tokens.len();
    let mut i = 1; // start after SELECT

    while i < len {
        let t = &tokens[i];
        let next = tokens.get(i + 1);
        let next2 = tokens.get(i + 2);

        if !t.is_quoted && t.text == "(" {
            depth += 1;
            // Check if this begins a subquery
            if let Some(n) = next {
                if !n.is_quoted && n.upper == "SELECT" {
                    has_subquery = true;
                }
            }
            i += 1;
            continue;
        }

        if !t.is_quoted && t.text == ")" {
            depth = depth.saturating_sub(1);
            i += 1;
            continue;
        }

        if depth == 0 && !t.is_quoted {
            if t.upper == "FROM" {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::From,
                    start: i,
                    len: 1,
                });
                i += 1;
                continue;
            }

            // JOINs
            if t.upper == "JOIN" || t.upper == "STRAIGHT_JOIN" {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::Join("joined".to_string()),
                    start: i,
                    len: 1,
                });
                i += 1;
                continue;
            }

            if t.upper == "INNER" && next.map_or(false, |n| !n.is_quoted && n.upper == "JOIN") {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::Join("inner joined".to_string()),
                    start: i,
                    len: 2,
                });
                i += 2;
                continue;
            }

            if t.upper == "CROSS" && next.map_or(false, |n| !n.is_quoted && n.upper == "JOIN") {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::Join("cross joined".to_string()),
                    start: i,
                    len: 2,
                });
                i += 2;
                continue;
            }

            if t.upper == "LEFT" {
                if next.map_or(false, |n| !n.is_quoted && n.upper == "OUTER")
                    && next2.map_or(false, |n| !n.is_quoted && n.upper == "JOIN")
                {
                    boundaries.push(ClauseBoundary {
                        kind: ClauseKind::Join("left joined".to_string()),
                        start: i,
                        len: 3,
                    });
                    i += 3;
                    continue;
                }
                if next.map_or(false, |n| !n.is_quoted && n.upper == "JOIN") {
                    boundaries.push(ClauseBoundary {
                        kind: ClauseKind::Join("left joined".to_string()),
                        start: i,
                        len: 2,
                    });
                    i += 2;
                    continue;
                }
            }

            if t.upper == "RIGHT" {
                if next.map_or(false, |n| !n.is_quoted && n.upper == "OUTER")
                    && next2.map_or(false, |n| !n.is_quoted && n.upper == "JOIN")
                {
                    boundaries.push(ClauseBoundary {
                        kind: ClauseKind::Join("right joined".to_string()),
                        start: i,
                        len: 3,
                    });
                    i += 3;
                    continue;
                }
                if next.map_or(false, |n| !n.is_quoted && n.upper == "JOIN") {
                    boundaries.push(ClauseBoundary {
                        kind: ClauseKind::Join("right joined".to_string()),
                        start: i,
                        len: 2,
                    });
                    i += 2;
                    continue;
                }
            }

            if t.upper == "WHERE" {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::Where,
                    start: i,
                    len: 1,
                });
                i += 1;
                continue;
            }

            if t.upper == "GROUP" && next.map_or(false, |n| !n.is_quoted && n.upper == "BY") {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::GroupBy,
                    start: i,
                    len: 2,
                });
                i += 2;
                continue;
            }

            if t.upper == "HAVING" {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::Having,
                    start: i,
                    len: 1,
                });
                i += 1;
                continue;
            }

            if t.upper == "ORDER" && next.map_or(false, |n| !n.is_quoted && n.upper == "BY") {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::OrderBy,
                    start: i,
                    len: 2,
                });
                i += 2;
                continue;
            }

            if t.upper == "LIMIT" {
                boundaries.push(ClauseBoundary {
                    kind: ClauseKind::Limit,
                    start: i,
                    len: 1,
                });
                i += 1;
                continue;
            }
        }

        i += 1;
    }

    // Sort boundaries by start index
    boundaries.sort_by_key(|b| b.start);

    // Columns: tokens from index 1 (or 2 if DISTINCT) to first boundary
    let col_start = if tokens.len() > 1 && !tokens[1].is_quoted && tokens[1].upper == "DISTINCT" {
        2
    } else {
        1
    };
    let is_distinct = col_start == 2;
    let col_end = boundaries.first().map_or(tokens.len(), |b| b.start);

    let col_tokens = if col_start < col_end {
        &tokens[col_start..col_end]
    } else {
        &[]
    };

    let col_parts = split_by_comma(col_tokens);
    let mut has_star = false;
    let mut formatted_cols_list: Vec<String> = Vec::new();

    for cp in &col_parts {
        let text = tokens_to_string(cp);
        if text == "*" {
            has_star = true;
            formatted_cols_list.push("all columns".to_string());
        } else if text.ends_with(".*") {
            has_star = true;
            let table = &text[..text.len() - 2];
            formatted_cols_list.push(format!("all columns from {}", table));
        } else {
            formatted_cols_list.push(text);
        }
    }

    let formatted_cols = if formatted_cols_list.is_empty() {
        "all columns".to_string()
    } else {
        format_list(&formatted_cols_list)
    };

    // Extract clause segments
    let mut table_name: Option<String> = None;
    let mut join_descriptions: Vec<String> = Vec::new();
    let mut where_desc: Option<String> = None;
    let mut group_desc: Option<String> = None;
    let mut having_desc: Option<String> = None;
    let mut order_desc: Option<String> = None;
    let mut limit_desc: Option<String> = None;
    let mut has_limit = false;

    for (idx, b) in boundaries.iter().enumerate() {
        let seg_start = b.start + b.len;
        let seg_end = if idx + 1 < boundaries.len() {
            boundaries[idx + 1].start
        } else {
            tokens.len()
        };
        let seg_tokens = if seg_start <= seg_end {
            &tokens[seg_start..seg_end]
        } else {
            &[]
        };

        match &b.kind {
            ClauseKind::From => {
                let from_tables = split_by_comma(seg_tokens);
                let table_names: Vec<String> = from_tables
                    .iter()
                    .filter_map(|p| p.first().map(|t| t.text.clone()))
                    .collect();
                if !table_names.is_empty() {
                    table_name = Some(table_names.join(", "));
                }
            }
            ClauseKind::Join(join_type) => {
                let on_idx = seg_tokens.iter().position(|t| !t.is_quoted && t.upper == "ON");
                let (j_table_tokens, j_cond_tokens) = if let Some(pos) = on_idx {
                    (&seg_tokens[..pos], &seg_tokens[pos + 1..])
                } else {
                    (seg_tokens, &[][..])
                };
                let j_table = j_table_tokens.first().map_or("table", |t| t.text.as_str());
                if !j_cond_tokens.is_empty() {
                    let cond = format_condition(j_cond_tokens);
                    join_descriptions.push(format!("{} with {} on {}", join_type, j_table, cond));
                } else {
                    join_descriptions.push(format!("{} with {}", join_type, j_table));
                }
            }
            ClauseKind::Where => {
                if !seg_tokens.is_empty() {
                    where_desc = Some(format_condition(seg_tokens));
                }
            }
            ClauseKind::GroupBy => {
                if !seg_tokens.is_empty() {
                    let group_parts = split_by_comma(seg_tokens);
                    let grp_strs: Vec<String> = group_parts.iter().map(|p| tokens_to_string(p)).collect();
                    group_desc = Some(grp_strs.join(", "));
                }
            }
            ClauseKind::Having => {
                if !seg_tokens.is_empty() {
                    having_desc = Some(format_condition(seg_tokens));
                }
            }
            ClauseKind::OrderBy => {
                if !seg_tokens.is_empty() {
                    let order_parts = split_by_comma(seg_tokens);
                    let mut order_items = Vec::new();
                    for op in order_parts {
                        if let Some(last) = op.last() {
                            if !last.is_quoted && last.upper == "DESC" {
                                let col = tokens_to_string(&op[..op.len() - 1]);
                                order_items.push(format!("{} descending", col));
                                continue;
                            } else if !last.is_quoted && last.upper == "ASC" {
                                let col = tokens_to_string(&op[..op.len() - 1]);
                                order_items.push(format!("{} ascending", col));
                                continue;
                            }
                        }
                        order_items.push(tokens_to_string(&op));
                    }
                    order_desc = Some(order_items.join(" then "));
                }
            }
            ClauseKind::Limit => {
                has_limit = true;
                if !seg_tokens.is_empty() {
                    let l_str = tokens_to_string(seg_tokens);
                    // Check for OFFSET keyword or comma
                    let offset_pos = seg_tokens.iter().position(|t| !t.is_quoted && t.upper == "OFFSET");
                    let comma_pos = seg_tokens.iter().position(|t| !t.is_quoted && t.text == ",");

                    if let Some(pos) = offset_pos {
                        let count = tokens_to_string(&seg_tokens[..pos]);
                        let offset = tokens_to_string(&seg_tokens[pos + 1..]);
                        let row_word = if count == "1" { "row" } else { "rows" };
                        limit_desc = Some(format!("limited to {} {} (offset {})", count, row_word, offset));
                    } else if let Some(pos) = comma_pos {
                        let offset = tokens_to_string(&seg_tokens[..pos]);
                        let count = tokens_to_string(&seg_tokens[pos + 1..]);
                        let row_word = if count == "1" { "row" } else { "rows" };
                        limit_desc = Some(format!("limited to {} {} (offset {})", count, row_word, offset));
                    } else {
                        let row_word = if l_str == "1" { "row" } else { "rows" };
                        limit_desc = Some(format!("limited to {} {}", l_str, row_word));
                    }
                }
            }
        }
    }

    // Build plain English summary and structured breakdown lines
    let mut parts: Vec<String> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    lines.push("Operation: SELECT".to_string());

    let distinct_prefix = if is_distinct { "distinct " } else { "" };

    if let Some(ref tbl) = table_name {
        parts.push(format!("Fetches {}{} from the {} table", distinct_prefix, formatted_cols, tbl));
        lines.push(format!("Target table: {}", tbl));
        lines.push(format!("Columns: {}{}", distinct_prefix, formatted_cols));
    } else {
        parts.push(format!("Evaluates expression: {}{}", distinct_prefix, formatted_cols));
        lines.push(format!("Expression: {}{}", distinct_prefix, formatted_cols));
    }

    if !join_descriptions.is_empty() {
        parts.push(format_list(&join_descriptions));
        for j in &join_descriptions {
            lines.push(format!("Join: {}", j));
        }
    }

    if let Some(ref w) = where_desc {
        parts.push(format!("filtered where {}", w));
        lines.push(format!("Filter: {}", w));
    }

    if let Some(ref g) = group_desc {
        parts.push(format!("grouped by {}", g));
        lines.push(format!("Group by: {}", g));
    }

    if let Some(ref h) = having_desc {
        parts.push(format!("having {}", h));
        lines.push(format!("Having: {}", h));
    }

    if let Some(ref o) = order_desc {
        parts.push(format!("sorted by {}", o));
        lines.push(format!("Sort: {}", o));
    }

    if let Some(ref l) = limit_desc {
        parts.push(l.clone());
        lines.push(format!("Limit: {}", l));
    }

    if has_subquery {
        lines.push("Note: Contains nested subquery".to_string());
    }

    // Warning check: SELECT * without LIMIT on table query
    if table_name.is_some() && has_star && !has_limit {
        warnings.push("SELECT * without LIMIT may return a very large result set and degrade server performance.".to_string());
    }

    let summary = parts.join(", ") + ".";

    Explanation {
        summary,
        lines,
        warnings,
    }
}

/// Handles INSERT statement explanation.
fn handle_insert(tokens: &[Token]) -> Explanation {
    let mut lines = Vec::new();
    lines.push("Operation: INSERT".to_string());

    let into_idx = tokens.iter().position(|t| !t.is_quoted && t.upper == "INTO");
    let table = if let Some(idx) = into_idx {
        tokens.get(idx + 1).map_or("table", |t| t.text.as_str())
    } else {
        tokens.get(1).map_or("table", |t| t.text.as_str())
    };

    lines.push(format!("Target table: {}", table));

    let values_idx = tokens.iter().position(|t| !t.is_quoted && t.upper == "VALUES");

    let extract_paren_tokens = |start: usize, end: usize| -> Vec<String> {
        let open_idx = tokens[start..end].iter().position(|t| !t.is_quoted && t.text == "(");
        if let Some(rel_open) = open_idx {
            let open = start + rel_open;
            let mut depth = 1;
            let mut close = end;
            for j in open + 1..end {
                if !tokens[j].is_quoted && tokens[j].text == "(" {
                    depth += 1;
                } else if !tokens[j].is_quoted && tokens[j].text == ")" {
                    depth -= 1;
                    if depth == 0 {
                        close = j;
                        break;
                    }
                }
            }
            if close > open + 1 {
                let inner = &tokens[open + 1..close];
                return split_by_comma(inner).iter().map(|p| tokens_to_string(p)).collect();
            }
        }
        Vec::new()
    };

    let mut summary = format!("Inserts a new row into {} with the provided values.", table);

    if let Some(v_idx) = values_idx {
        let cols = extract_paren_tokens(0, v_idx);
        let vals = extract_paren_tokens(v_idx, tokens.len());

        if !cols.is_empty() && !vals.is_empty() && cols.len() == vals.len() {
            let assignments: Vec<String> = cols
                .iter()
                .zip(vals.iter())
                .map(|(c, v)| format!("{} to {}", c, v))
                .collect();
            let assign_str = assignments.join(", ");
            summary = format!("Inserts a new row into {} setting {}.", table, assign_str);
            lines.push(format!("Fields: {}", assign_str));
        } else if !cols.is_empty() {
            lines.push(format!("Columns: {}", cols.join(", ")));
        }
    }

    Explanation {
        summary,
        lines,
        warnings: vec![],
    }
}

/// Handles UPDATE statement explanation.
fn handle_update(tokens: &[Token]) -> Explanation {
    let mut lines = Vec::new();
    let mut warnings = Vec::new();
    lines.push("Operation: UPDATE".to_string());

    let table = tokens.get(1).map_or("table", |t| t.text.as_str());
    lines.push(format!("Target table: {}", table));

    let set_idx = tokens.iter().position(|t| !t.is_quoted && t.upper == "SET");
    let where_idx = tokens.iter().position(|t| !t.is_quoted && t.upper == "WHERE");

    let assignments_str = if let Some(s_idx) = set_idx {
        let end_idx = where_idx.unwrap_or(tokens.len());
        let set_tokens = if s_idx + 1 < end_idx {
            &tokens[s_idx + 1..end_idx]
        } else {
            &[]
        };
        let parts = split_by_comma(set_tokens);
        let assign_parts: Vec<String> = parts
            .iter()
            .map(|p| {
                if let Some(eq_idx) = p.iter().position(|t| !t.is_quoted && t.text == "=") {
                    let col = tokens_to_string(&p[..eq_idx]);
                    let val = tokens_to_string(&p[eq_idx + 1..]);
                    format!("{} to {}", col, val)
                } else {
                    tokens_to_string(p)
                }
            })
            .collect();
        assign_parts.join(", ")
    } else {
        "values".to_string()
    };

    lines.push(format!("Assignments: {}", assignments_str));

    let summary = if let Some(w_idx) = where_idx {
        let w_tokens = &tokens[w_idx + 1..];
        let condition = format_condition(w_tokens);
        lines.push(format!("Filter: {}", condition));
        format!("Updates rows in {}, setting {}, where {}.", table, assignments_str, condition)
    } else {
        lines.push("Filter: None (affects ALL rows)".to_string());
        warnings.push(format!(
            "UPDATE statement has no WHERE clause and will modify EVERY row in table '{}'.",
            table
        ));
        format!("Updates ALL rows in {}, setting {}.", table, assignments_str)
    };

    Explanation {
        summary,
        lines,
        warnings,
    }
}

/// Handles DELETE statement explanation.
fn handle_delete(tokens: &[Token]) -> Explanation {
    let mut lines = Vec::new();
    let mut warnings = Vec::new();
    lines.push("Operation: DELETE".to_string());

    let from_idx = tokens.iter().position(|t| !t.is_quoted && t.upper == "FROM");
    let table = if let Some(idx) = from_idx {
        tokens.get(idx + 1).map_or("table", |t| t.text.as_str())
    } else {
        tokens.get(1).map_or("table", |t| t.text.as_str())
    };

    lines.push(format!("Target table: {}", table));

    let where_idx = tokens.iter().position(|t| !t.is_quoted && t.upper == "WHERE");

    let summary = if let Some(w_idx) = where_idx {
        let w_tokens = &tokens[w_idx + 1..];
        let condition = format_condition(w_tokens);
        lines.push(format!("Filter: {}", condition));
        format!("Deletes rows from {} where {}.", table, condition)
    } else {
        lines.push("Filter: None (affects ALL rows)".to_string());
        warnings.push(format!(
            "DELETE statement has no WHERE clause and will permanently delete EVERY row from table '{}'.",
            table
        ));
        format!("Deletes ALL rows from {}.", table)
    };

    Explanation {
        summary,
        lines,
        warnings,
    }
}

/// Handles CREATE TABLE statement explanation.
fn handle_create_table(tokens: &[Token]) -> Explanation {
    let mut lines = Vec::new();
    lines.push("Operation: CREATE TABLE".to_string());

    let tbl_pos = tokens.iter().position(|t| !t.is_quoted && t.upper == "TABLE");
    let mut table = "table";
    if let Some(pos) = tbl_pos {
        let mut curr = pos + 1;
        if curr < tokens.len() && !tokens[curr].is_quoted && tokens[curr].upper == "IF" {
            curr += 1;
            if curr < tokens.len() && !tokens[curr].is_quoted && tokens[curr].upper == "NOT" {
                curr += 1;
            }
            if curr < tokens.len() && !tokens[curr].is_quoted && tokens[curr].upper == "EXISTS" {
                curr += 1;
            }
        }
        if let Some(t) = tokens.get(curr) {
            table = &t.text;
        }
    }

    lines.push(format!("Table: {}", table));

    let open_idx = tokens.iter().position(|t| !t.is_quoted && t.text == "(");
    let mut col_descriptions = Vec::new();

    if let Some(open) = open_idx {
        let close = tokens.iter().rposition(|t| !t.is_quoted && t.text == ")").unwrap_or(tokens.len());
        if close > open + 1 {
            let inner = &tokens[open + 1..close];
            let col_parts = split_by_comma(inner);

            for cp in col_parts {
                if cp.is_empty() {
                    continue;
                }
                let col_name = &cp[0].text;
                let upper_name = &cp[0].upper;

                // Skip table-level constraints
                if matches!(upper_name.as_str(), "CONSTRAINT" | "PRIMARY" | "FOREIGN" | "KEY" | "UNIQUE" | "CHECK") {
                    continue;
                }

                let mut type_tokens = Vec::new();
                for t in &cp[1..] {
                    if !t.is_quoted && matches!(
                        t.upper.as_str(),
                        "NOT" | "NULL" | "PRIMARY" | "KEY" | "AUTO_INCREMENT" | "AUTOINCREMENT"
                            | "DEFAULT" | "UNIQUE" | "REFERENCES" | "CHECK" | "COMMENT"
                    ) {
                        break;
                    }
                    type_tokens.push(t.clone());
                }

                let type_str = format_type_string(&type_tokens);
                if !type_str.is_empty() {
                    col_descriptions.push(format!("{} ({})", col_name, type_str));
                } else {
                    col_descriptions.push(col_name.clone());
                }
            }
        }
    }

    let summary = if !col_descriptions.is_empty() {
        lines.push(format!("Columns: {}", col_descriptions.join(", ")));
        format!(
            "Creates a new table named {} with {} columns: {}.",
            table,
            col_descriptions.len(),
            col_descriptions.join(", ")
        )
    } else {
        format!("Creates a new table named {}.", table)
    };

    Explanation {
        summary,
        lines,
        warnings: vec![],
    }
}

/// Handles DROP statement explanation.
fn handle_drop(tokens: &[Token]) -> Explanation {
    let mut lines = Vec::new();
    let mut warnings = Vec::new();
    lines.push("Operation: DROP".to_string());

    let raw_type = tokens.get(1).map_or("OBJECT", |t| t.upper.as_str());
    let obj_type = match raw_type {
        "TABLE" => "table",
        "DATABASE" | "SCHEMA" => "database",
        "VIEW" => "view",
        "INDEX" => "index",
        "PROCEDURE" => "stored procedure",
        "FUNCTION" => "function",
        "TRIGGER" => "trigger",
        _ => "object",
    };

    let mut name_idx = 2;
    if tokens.get(name_idx).map_or(false, |t| !t.is_quoted && t.upper == "IF") {
        name_idx += 1;
        if tokens.get(name_idx).map_or(false, |t| !t.is_quoted && t.upper == "EXISTS") {
            name_idx += 1;
        }
    }

    let name = tokens.get(name_idx).map_or("unknown", |t| t.text.as_str());

    lines.push(format!("Object type: {}", obj_type));
    lines.push(format!("Object name: {}", name));

    warnings.push(format!(
        "DROP permanently destroys {} '{}' and all associated data. This action cannot be undone.",
        obj_type, name
    ));

    Explanation {
        summary: format!("Permanently drops the {} named {}. This cannot be undone.", obj_type, name),
        lines,
        warnings,
    }
}

/// Handles ALTER TABLE statement explanation.
fn handle_alter_table(tokens: &[Token]) -> Explanation {
    let mut lines = Vec::new();
    let mut warnings = Vec::new();
    lines.push("Operation: ALTER TABLE".to_string());

    let table = tokens.get(2).map_or("table", |t| t.text.as_str());
    lines.push(format!("Target table: {}", table));

    let mut add_idx = None;
    let mut drop_idx = None;
    let mut modify_idx = None;
    let mut change_idx = None;

    for (i, t) in tokens.iter().enumerate().skip(3) {
        if !t.is_quoted {
            match t.upper.as_str() {
                "ADD" if add_idx.is_none() => add_idx = Some(i),
                "DROP" if drop_idx.is_none() => drop_idx = Some(i),
                "MODIFY" if modify_idx.is_none() => modify_idx = Some(i),
                "CHANGE" if change_idx.is_none() => change_idx = Some(i),
                _ => {}
            }
        }
    }

    let summary = if let Some(idx) = add_idx {
        let mut col_start = idx + 1;
        if tokens.get(col_start).map_or(false, |t| !t.is_quoted && t.upper == "COLUMN") {
            col_start += 1;
        }
        let col = tokens.get(col_start).map_or("column", |t| t.text.as_str());
        let type_tokens = if col_start + 1 < tokens.len() {
            &tokens[col_start + 1..]
        } else {
            &[]
        };
        let type_str = format_type_string(type_tokens);
        let type_label = if type_str.is_empty() { "new" } else { &type_str };
        lines.push(format!("Action: Add column {} ({})", col, type_label));
        format!("Adds a {} column named {} to {}.", type_label, col, table)
    } else if let Some(idx) = drop_idx {
        let mut col_start = idx + 1;
        if tokens.get(col_start).map_or(false, |t| !t.is_quoted && t.upper == "COLUMN") {
            col_start += 1;
        }
        let col = tokens.get(col_start).map_or("column", |t| t.text.as_str());
        lines.push(format!("Action: Drop column {}", col));
        warnings.push(format!(
            "Dropping column '{}' from table '{}' permanently deletes all data stored in that column.",
            col, table
        ));
        format!("Removes the column {} from {}.", col, table)
    } else if let Some(idx) = modify_idx {
        let mut col_start = idx + 1;
        if tokens.get(col_start).map_or(false, |t| !t.is_quoted && t.upper == "COLUMN") {
            col_start += 1;
        }
        let col = tokens.get(col_start).map_or("column", |t| t.text.as_str());
        lines.push(format!("Action: Modify column {}", col));
        format!("Modifies the {} column in {}.", col, table)
    } else if let Some(idx) = change_idx {
        let mut col_start = idx + 1;
        if tokens.get(col_start).map_or(false, |t| !t.is_quoted && t.upper == "COLUMN") {
            col_start += 1;
        }
        let col = tokens.get(col_start).map_or("column", |t| t.text.as_str());
        lines.push(format!("Action: Change column {}", col));
        format!("Modifies the {} column in {}.", col, table)
    } else {
        lines.push("Action: Schema alteration".to_string());
        format!("Modifies the schema of table {}.", table)
    };

    Explanation {
        summary,
        lines,
        warnings,
    }
}

/// Handles TRUNCATE statement explanation.
fn handle_truncate(tokens: &[Token]) -> Explanation {
    let mut lines = Vec::new();
    let mut warnings = Vec::new();
    lines.push("Operation: TRUNCATE TABLE".to_string());

    let mut tbl_idx = 1;
    if tokens.get(tbl_idx).map_or(false, |t| !t.is_quoted && t.upper == "TABLE") {
        tbl_idx += 1;
    }
    let table = tokens.get(tbl_idx).map_or("table", |t| t.text.as_str());

    lines.push(format!("Target table: {}", table));
    warnings.push(format!(
        "TRUNCATE permanently deletes ALL rows from table '{}' and resets auto-increment keys. This cannot be rolled back.",
        table
    ));

    Explanation {
        summary: format!("Truncates table {}, removing all rows and resetting auto-increment counters.", table),
        lines,
        warnings,
    }
}

/// Explains a single parsed SQL statement.
fn explain_single_statement(stmt: &str) -> Explanation {
    let tokens = tokenize(stmt);
    if tokens.is_empty() {
        return Explanation {
            summary: "Empty query.".to_string(),
            lines: vec![],
            warnings: vec![],
        };
    }

    let first = &tokens[0];
    if first.is_quoted {
        return Explanation {
            summary: "Couldn't fully explain this query.".to_string(),
            lines: vec!["Query begins with a string literal rather than a SQL statement keyword.".to_string()],
            warnings: vec![],
        };
    }

    match first.upper.as_str() {
        "SELECT" => handle_select(&tokens),
        "INSERT" => handle_insert(&tokens),
        "UPDATE" => handle_update(&tokens),
        "DELETE" => handle_delete(&tokens),
        "CREATE" if tokens.get(1).map_or(false, |t| !t.is_quoted && t.upper == "TABLE") => {
            handle_create_table(&tokens)
        }
        "DROP" => handle_drop(&tokens),
        "ALTER" if tokens.get(1).map_or(false, |t| !t.is_quoted && t.upper == "TABLE") => {
            handle_alter_table(&tokens)
        }
        "TRUNCATE" => handle_truncate(&tokens),
        _ => Explanation {
            summary: "Couldn't fully explain this query.".to_string(),
            lines: vec![format!("Unrecognized or unsupported statement type: {}", first.text)],
            warnings: vec![],
        },
    }
}

/// Explains arbitrary SQL text offline in plain English.
///
/// Strips comments, splits multiple statements, analyzes clauses and operators,
/// and produces a high-level summary, clause breakdown lines, and risk warnings.
pub fn explain_query(sql: &str) -> Explanation {
    let clean = strip_comments(sql);
    let trimmed = clean.trim();
    if trimmed.is_empty() {
        return Explanation {
            summary: "Empty query.".to_string(),
            lines: vec![],
            warnings: vec![],
        };
    }

    let statements = split_statements(trimmed);
    if statements.is_empty() {
        return Explanation {
            summary: "Empty query.".to_string(),
            lines: vec![],
            warnings: vec![],
        };
    }

    if statements.len() == 1 {
        return explain_single_statement(&statements[0]);
    }

    // Multiple statements batch
    let mut summaries = Vec::new();
    let mut all_lines = Vec::new();
    let mut all_warnings = Vec::new();

    for (idx, stmt) in statements.iter().enumerate() {
        let exp = explain_single_statement(stmt);
        summaries.push(format!("({}) {}", idx + 1, exp.summary));
        all_lines.push(format!("Statement {}: {}", idx + 1, exp.summary));
        for line in exp.lines {
            all_lines.push(format!("  {}", line));
        }
        for warn in exp.warnings {
            all_warnings.push(format!("Statement {}: {}", idx + 1, warn));
        }
    }

    let summary = format!(
        "Executes {} statements: {}",
        statements.len(),
        summaries.join(" ")
    );

    Explanation {
        summary,
        lines: all_lines,
        warnings: all_warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_full_clauses() {
        let sql = "SELECT id, name, email FROM users WHERE active = 1 AND age >= 18 ORDER BY id DESC LIMIT 10;";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Fetches id, name, and email from the users table"));
        assert!(exp.summary.contains("active equals 1 and age at least 18"));
        assert!(exp.summary.contains("sorted by id descending"));
        assert!(exp.summary.contains("limited to 10 rows"));
        assert!(exp.warnings.is_empty());
        assert!(exp.lines.iter().any(|l| l.contains("Target table: users")));
    }

    #[test]
    fn test_select_star_without_limit_warning() {
        let sql = "SELECT * FROM users WHERE status = 'active'";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Fetches all columns from the users table"));
        assert_eq!(exp.warnings.len(), 1);
        assert!(exp.warnings[0].contains("SELECT * without LIMIT"));
    }

    #[test]
    fn test_select_with_joins() {
        let sql = "SELECT users.name, orders.total FROM users LEFT JOIN orders ON orders.user_id = users.id WHERE orders.total > 100 LIMIT 5";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("left joined with orders on orders.user_id equals users.id"));
        assert!(exp.summary.contains("orders.total is greater than 100"));
        assert!(exp.summary.contains("limited to 5 rows"));
        assert!(exp.warnings.is_empty());
    }

    #[test]
    fn test_insert_with_columns_and_values() {
        let sql = "INSERT INTO users (name, email) VALUES ('Alice', 'alice@test.com')";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Inserts a new row into users setting name to 'Alice', email to 'alice@test.com'."));
        assert!(exp.warnings.is_empty());
        assert!(exp.lines.iter().any(|l| l.contains("Operation: INSERT")));
    }

    #[test]
    fn test_update_with_where() {
        let sql = "UPDATE users SET status = 'active', role = 'admin' WHERE id = 42";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Updates rows in users, setting status to 'active', role to 'admin', where id equals 42."));
        assert!(exp.warnings.is_empty());
    }

    #[test]
    fn test_update_without_where_triggers_warning() {
        let sql = "UPDATE users SET status = 'inactive'";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Updates ALL rows in users"));
        assert_eq!(exp.warnings.len(), 1);
        assert!(exp.warnings[0].contains("UPDATE statement has no WHERE clause"));
    }

    #[test]
    fn test_delete_with_where() {
        let sql = "DELETE FROM sessions WHERE expires_at < '2026-01-01'";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Deletes rows from sessions where expires_at is less than '2026-01-01'."));
        assert!(exp.warnings.is_empty());
    }

    #[test]
    fn test_delete_without_where_triggers_warning() {
        let sql = "DELETE FROM logs";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Deletes ALL rows from logs."));
        assert_eq!(exp.warnings.len(), 1);
        assert!(exp.warnings[0].contains("DELETE statement has no WHERE clause"));
    }

    #[test]
    fn test_create_table() {
        let sql = "CREATE TABLE users (id INT PRIMARY KEY AUTO_INCREMENT, username VARCHAR(50) NOT NULL, created_at TIMESTAMP)";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Creates a new table named users with 3 columns"));
        assert!(exp.summary.contains("id (INT)"));
        assert!(exp.summary.contains("username (VARCHAR(50))"));
        assert!(exp.summary.contains("created_at (TIMESTAMP)"));
    }

    #[test]
    fn test_drop_table_triggers_warning() {
        let sql = "DROP TABLE IF EXISTS old_users";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Permanently drops the table named old_users. This cannot be undone."));
        assert_eq!(exp.warnings.len(), 1);
        assert!(exp.warnings[0].contains("DROP permanently destroys table 'old_users'"));
    }

    #[test]
    fn test_alter_table_add_and_drop_column() {
        let sql1 = "ALTER TABLE users ADD COLUMN age INT";
        let exp1 = explain_query(sql1);
        assert!(exp1.summary.contains("Adds a INT column named age to users."));

        let sql2 = "ALTER TABLE users DROP COLUMN legacy_field";
        let exp2 = explain_query(sql2);
        assert!(exp2.summary.contains("Removes the column legacy_field from users."));
        assert_eq!(exp2.warnings.len(), 1);
        assert!(exp2.warnings[0].contains("Dropping column 'legacy_field'"));
    }

    #[test]
    fn test_truncate_triggers_warning() {
        let sql = "TRUNCATE TABLE audit_log";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Truncates table audit_log"));
        assert_eq!(exp.warnings.len(), 1);
        assert!(exp.warnings[0].contains("TRUNCATE permanently deletes ALL rows"));
    }

    #[test]
    fn test_lowercase_keywords() {
        let sql = "select id, name from users where age >= 21 and status != 'banned' limit 25";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Fetches id and name from the users table"));
        assert!(exp.summary.contains("age at least 21 and status is not 'banned'"));
        assert!(exp.summary.contains("limited to 25 rows"));
        assert!(exp.warnings.is_empty());
    }

    #[test]
    fn test_string_literals_containing_keywords() {
        let sql = "SELECT * FROM notes WHERE content = 'WHERE is the DROP TABLE and LIMIT 5?' LIMIT 1";
        let exp = explain_query(sql);
        // The literal should not be matched as WHERE or DROP
        assert!(exp.summary.contains("Fetches all columns from the notes table"));
        assert!(exp.summary.contains("content equals 'WHERE is the DROP TABLE and LIMIT 5?'"));
        assert!(exp.summary.contains("limited to 1 row"));
    }

    #[test]
    fn test_comments_stripping() {
        let sql = "-- Line comment 1\n# Hash comment\n/* Block comment */ SELECT id FROM users /* inline */ WHERE id = 10;";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Fetches id from the users table"));
        assert!(exp.summary.contains("id equals 10"));
    }

    #[test]
    fn test_multiple_statements() {
        let sql = "SELECT * FROM users LIMIT 10; DROP TABLE logs;";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("Executes 2 statements:"));
        assert!(exp.summary.contains("(1) Fetches all columns from the users table"));
        assert!(exp.summary.contains("(2) Permanently drops the table named logs."));
        assert_eq!(exp.warnings.len(), 1);
        assert!(exp.warnings[0].contains("DROP permanently destroys"));
    }

    #[test]
    fn test_empty_and_whitespace_input() {
        let exp1 = explain_query("");
        assert_eq!(exp1.summary, "Empty query.");
        assert!(exp1.lines.is_empty());
        assert!(exp1.warnings.is_empty());

        let exp2 = explain_query("   \n\t  -- only comment\n  ");
        assert_eq!(exp2.summary, "Empty query.");
    }

    #[test]
    fn test_garbage_and_unsupported_input() {
        let exp = explain_query("XYZZY FLUMMOX 12345");
        assert_eq!(exp.summary, "Couldn't fully explain this query.");
        assert_eq!(exp.lines.len(), 1);
        assert!(exp.lines[0].contains("Unrecognized or unsupported statement type: XYZZY"));
        assert!(exp.warnings.is_empty());
    }

    #[test]
    fn test_operator_translations() {
        let sql = "SELECT id FROM t WHERE a = 1 AND b != 2 AND c <> 3 AND d > 4 AND e < 5 AND f >= 6 AND g <= 7 AND h LIKE '%test%' AND i IN (1, 2) AND j BETWEEN 10 AND 20 AND k IS NULL AND l IS NOT NULL LIMIT 10";
        let exp = explain_query(sql);
        assert!(exp.summary.contains("a equals 1"));
        assert!(exp.summary.contains("b is not 2"));
        assert!(exp.summary.contains("c is not 3"));
        assert!(exp.summary.contains("d is greater than 4"));
        assert!(exp.summary.contains("e is less than 5"));
        assert!(exp.summary.contains("f at least 6"));
        assert!(exp.summary.contains("g at most 7"));
        assert!(exp.summary.contains("h matches pattern '%test%'"));
        assert!(exp.summary.contains("i is one of (1, 2)"));
        assert!(exp.summary.contains("j is between 10 and 20"));
        assert!(exp.summary.contains("k is empty"));
        assert!(exp.summary.contains("l is not empty"));
    }
}
