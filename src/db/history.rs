use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub query: String,
    pub timestamp: i64,
    pub database: Option<String>,
    pub is_favorite: bool,
}

pub struct HistoryManager {
    file_path: PathBuf,
}

/// Redacts passwords/secrets in statements matching `IDENTIFIED BY '...'` or `PASSWORD('...')`
/// to prevent storing plaintext credentials in history.json.
pub fn redact_secrets(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Check for IDENTIFIED BY '...'
        if i + 10 <= len {
            let slice: String = chars[i..i + 10].iter().collect();
            if slice.eq_ignore_ascii_case("IDENTIFIED") {
                let is_start = i == 0 || (!chars[i - 1].is_alphanumeric() && chars[i - 1] != '_');
                let after_id = i + 10;
                let is_end = after_id >= len || (!chars[after_id].is_alphanumeric() && chars[after_id] != '_');

                if is_start && is_end {
                    let mut j = after_id;
                    while j < len && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j + 2 <= len {
                        let by_slice: String = chars[j..j + 2].iter().collect();
                        let by_end = j + 2;
                        let by_is_end = by_end >= len || (!chars[by_end].is_alphanumeric() && chars[by_end] != '_');
                        if by_slice.eq_ignore_ascii_case("BY") && by_is_end {
                            let mut k = by_end;
                            while k < len && chars[k].is_whitespace() {
                                k += 1;
                            }
                            if k < len && (chars[k] == '\'' || chars[k] == '"') {
                                let quote = chars[k];
                                let mut m = k + 1;
                                while m < len {
                                    if chars[m] == quote {
                                        if m + 1 < len && chars[m + 1] == quote {
                                            m += 2;
                                            continue;
                                        }
                                        break;
                                    } else if chars[m] == '\\' && m + 1 < len {
                                        m += 2;
                                    } else {
                                        m += 1;
                                    }
                                }
                                if m < len && chars[m] == quote {
                                    for idx in i..k {
                                        result.push(chars[idx]);
                                    }
                                    result.push_str("'***'");
                                    i = m + 1;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for PASSWORD('...')
        if i + 8 <= len {
            let slice: String = chars[i..i + 8].iter().collect();
            if slice.eq_ignore_ascii_case("PASSWORD") {
                let is_start = i == 0 || (!chars[i - 1].is_alphanumeric() && chars[i - 1] != '_');
                let after_pwd = i + 8;
                let is_end = after_pwd >= len || (!chars[after_pwd].is_alphanumeric() && chars[after_pwd] != '_');

                if is_start && is_end {
                    let mut j = after_pwd;
                    while j < len && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < len && chars[j] == '(' {
                        let mut k = j + 1;
                        while k < len && chars[k].is_whitespace() {
                            k += 1;
                        }
                        if k < len && (chars[k] == '\'' || chars[k] == '"') {
                            let quote = chars[k];
                            let mut m = k + 1;
                            while m < len {
                                if chars[m] == quote {
                                    if m + 1 < len && chars[m + 1] == quote {
                                        m += 2;
                                        continue;
                                    }
                                    break;
                                } else if chars[m] == '\\' && m + 1 < len {
                                    m += 2;
                                } else {
                                    m += 1;
                                }
                            }
                            if m < len && chars[m] == quote {
                                for idx in i..k {
                                    result.push(chars[idx]);
                                }
                                result.push_str("'***'");
                                i = m + 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

impl HistoryManager {
    pub fn new() -> Self {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mysql-gui");
        let _ = fs::create_dir_all(&base_dir);
        let file_path = base_dir.join("history.json");
        Self { file_path }
    }

    pub fn load(&self) -> Vec<HistoryItem> {
        if let Ok(content) = fs::read_to_string(&self.file_path) {
            if let Ok(items) = serde_json::from_str::<Vec<HistoryItem>>(&content) {
                return items;
            }
        }
        Vec::new()
    }

    pub fn save(&self, items: &[HistoryItem]) {
        let sanitized: Vec<HistoryItem> = items.iter().map(|item| {
            let mut it = item.clone();
            it.query = redact_secrets(&it.query);
            it
        }).collect();
        if let Ok(json) = serde_json::to_string_pretty(&sanitized) {
            let _ = fs::write(&self.file_path, json);
        }
    }

    pub fn add(&self, query: &str, database: Option<&str>) {
        let clean_query = redact_secrets(query);
        let trimmed = clean_query.trim();
        if trimmed.is_empty() {
            return;
        }
        let mut items = self.load();
        // Remove existing duplicate if any, so latest query appears at top
        items.retain(|item| item.query != trimmed);
        items.insert(0, HistoryItem {
            query: trimmed.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            database: database.map(|s| s.to_string()),
            is_favorite: false,
        });
        // Limit to 200 history items
        if items.len() > 200 {
            items.truncate(200);
        }
        self.save(&items);
    }

    pub fn toggle_favorite(&self, index: usize) {
        let mut items = self.load();
        if index < items.len() {
            items[index].is_favorite = !items[index].is_favorite;
            self.save(&items);
        }
    }

    pub fn delete(&self, index: usize) {
        let mut items = self.load();
        if index < items.len() {
            items.remove(index);
            self.save(&items);
        }
    }

    pub fn clear(&self) {
        self.save(&[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secrets() {
        let q1 = "CREATE USER 'admin'@'localhost' IDENTIFIED BY 'Secret123!'";
        assert_eq!(redact_secrets(q1), "CREATE USER 'admin'@'localhost' IDENTIFIED BY '***'");

        let q2 = "SET PASSWORD FOR 'user'@'%' = PASSWORD('MySecretPassword');";
        assert_eq!(redact_secrets(q2), "SET PASSWORD FOR 'user'@'%' = PASSWORD('***');");

        let q3 = "ALTER USER 'u'@'h' IDENTIFIED BY \"pass1\";";
        assert_eq!(redact_secrets(q3), "ALTER USER 'u'@'h' IDENTIFIED BY '***';");
    }
}
