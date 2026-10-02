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
        if let Ok(json) = serde_json::to_string_pretty(items) {
            let _ = fs::write(&self.file_path, json);
        }
    }

    pub fn add(&self, query: &str, database: Option<&str>) {
        let trimmed = query.trim();
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
