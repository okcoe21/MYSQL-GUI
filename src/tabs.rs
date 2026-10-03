use crate::db::models::QueryResult;

pub const MAX_TABS: usize = 8;
pub const MAX_RESULT_ROWS: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlResultData {
    pub execution_time_ms: i32,
    pub affected_rows: i32,
    pub message: String,
    pub error_message: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExplainData {
    pub is_open: bool,
    pub summary: String,
    pub lines: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlTabState {
    pub id: u64,
    pub title: String,
    pub query: String,
    pub is_running: bool,
    pub result: Option<SqlResultData>,
    pub explain: ExplainData,
}

impl SqlTabState {
    pub fn has_unsaved_text(&self) -> bool {
        !self.query.trim().is_empty()
    }

    pub fn to_header(&self) -> SqlTabHeaderData {
        SqlTabHeaderData {
            id: self.id,
            title: self.title.clone(),
            is_running: self.is_running,
            has_unsaved_text: self.has_unsaved_text(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlTabHeaderData {
    pub id: u64,
    pub title: String,
    pub is_running: bool,
    pub has_unsaved_text: bool,
}

pub fn query_result_to_sql_result_data(
    res: Result<QueryResult, String>,
    max_rows: usize,
) -> SqlResultData {
    match res {
        Ok(qr) => {
            let mut message = qr.message.unwrap_or_else(|| {
                format!(
                    "Execution finished in {}ms. Affected rows: {}",
                    qr.execution_time_ms,
                    qr.affected_rows.unwrap_or(0)
                )
            });

            let mut rows = Vec::new();
            if let Some(data_rows) = qr.data {
                let total_rows = data_rows.len();
                let take_rows = total_rows.min(max_rows);

                for r in data_rows.into_iter().take(take_rows) {
                    let mut cells = Vec::new();
                    if let Some(obj) = r.as_object() {
                        for col in &qr.columns {
                            let val_str = match obj.get(col) {
                                Some(serde_json::Value::Null) => "NULL".to_string(),
                                Some(serde_json::Value::String(s)) => s.clone(),
                                Some(other) => other.to_string(),
                                None => "NULL".to_string(),
                            };
                            cells.push(val_str);
                        }
                    }
                    rows.push(cells);
                }

                if total_rows > max_rows {
                    message = format!("{} (Showing first {} rows of {})", message, max_rows, total_rows);
                }
            }

            SqlResultData {
                execution_time_ms: qr.execution_time_ms as i32,
                affected_rows: qr.affected_rows.unwrap_or(0) as i32,
                message,
                error_message: String::new(),
                columns: qr.columns,
                rows,
            }
        }
        Err(e) => SqlResultData {
            execution_time_ms: 0,
            affected_rows: -1,
            message: String::new(),
            error_message: e,
            columns: Vec::new(),
            rows: Vec::new(),
        },
    }
}

#[derive(Debug, Clone)]
pub struct TabManager {
    next_id: u64,
    epoch: u64,
    active_tab_id: u64,
    tabs: Vec<SqlTabState>,
}

impl TabManager {
    pub fn new() -> Self {
        let id = 1;
        let initial_tab = SqlTabState {
            id,
            title: format!("Query {}", id),
            query: String::new(),
            is_running: false,
            result: None,
            explain: ExplainData::default(),
        };

        Self {
            next_id: 2,
            epoch: 1,
            active_tab_id: id,
            tabs: vec![initial_tab],
        }
    }

    pub fn reset(&mut self) {
        self.epoch += 1;
        self.tabs.clear();
        let id = self.next_id;
        self.next_id += 1;
        let fresh_tab = SqlTabState {
            id,
            title: format!("Query {}", id),
            query: String::new(),
            is_running: false,
            result: None,
            explain: ExplainData::default(),
        };
        self.active_tab_id = id;
        self.tabs.push(fresh_tab);
    }

    pub fn can_add_tab(&self) -> bool {
        self.tabs.len() < MAX_TABS
    }

    pub fn add_tab(&mut self) -> Result<u64, String> {
        if !self.can_add_tab() {
            return Err(format!("Maximum limit of {} tabs reached", MAX_TABS));
        }

        let id = self.next_id;
        self.next_id += 1;

        let new_tab = SqlTabState {
            id,
            title: format!("Query {}", id),
            query: String::new(),
            is_running: false,
            result: None,
            explain: ExplainData::default(),
        };

        self.tabs.push(new_tab);
        self.active_tab_id = id;
        Ok(id)
    }

    pub fn active_tab_id(&self) -> u64 {
        self.active_tab_id
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    #[allow(dead_code)]
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn get_tab(&self, tab_id: u64) -> Option<&SqlTabState> {
        self.tabs.iter().find(|t| t.id == tab_id)
    }

    pub fn get_tab_mut(&mut self, tab_id: u64) -> Option<&mut SqlTabState> {
        self.tabs.iter_mut().find(|t| t.id == tab_id)
    }

    pub fn get_active_tab(&self) -> Option<&SqlTabState> {
        self.get_tab(self.active_tab_id)
    }

    #[allow(dead_code)]
    pub fn get_active_tab_mut(&mut self) -> Option<&mut SqlTabState> {
        let active_id = self.active_tab_id;
        self.get_tab_mut(active_id)
    }

    pub fn switch_tab(&mut self, target_id: u64, current_tab_query: Option<&str>) -> bool {
        if let Some(query) = current_tab_query {
            let active_id = self.active_tab_id;
            if let Some(active_tab) = self.get_tab_mut(active_id) {
                active_tab.query = query.to_string();
            }
        }

        if self.tabs.iter().any(|t| t.id == target_id) {
            self.active_tab_id = target_id;
            true
        } else {
            false
        }
    }

    pub fn update_active_query(&mut self, query: &str) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.get_tab_mut(active_id) {
            tab.query = query.to_string();
        }
    }

    pub fn update_active_explain_open(&mut self, is_open: bool) {
        let active_id = self.active_tab_id;
        if let Some(tab) = self.get_tab_mut(active_id) {
            tab.explain.is_open = is_open;
        }
    }

    pub fn close_tab(&mut self, tab_id: u64) -> Result<(), String> {
        let pos = self
            .tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or_else(|| "Tab not found".to_string())?;

        self.tabs.remove(pos);

        if self.tabs.is_empty() {
            // Closing last tab automatically opens a fresh empty tab
            let id = self.next_id;
            self.next_id += 1;
            let fresh_tab = SqlTabState {
                id,
                title: format!("Query {}", id),
                query: String::new(),
                is_running: false,
                result: None,
                explain: ExplainData::default(),
            };
            self.tabs.push(fresh_tab);
            self.active_tab_id = id;
        } else if self.active_tab_id == tab_id {
            // Select neighbor: tab at same index, or index - 1 if was at the end
            if pos < self.tabs.len() {
                self.active_tab_id = self.tabs[pos].id;
            } else {
                self.active_tab_id = self.tabs[pos - 1].id;
            }
        }

        Ok(())
    }

    pub fn set_running(&mut self, tab_id: u64, is_running: bool) -> Result<(), String> {
        let tab = self
            .get_tab_mut(tab_id)
            .ok_or_else(|| "Tab not found".to_string())?;

        if is_running && tab.is_running {
            return Err("Query is already running in this tab".to_string());
        }

        tab.is_running = is_running;
        Ok(())
    }

    pub fn set_result(
        &mut self,
        tab_id: u64,
        epoch: u64,
        result: Result<QueryResult, String>,
    ) -> bool {
        if self.epoch != epoch {
            return false;
        }

        let tab = match self.get_tab_mut(tab_id) {
            Some(t) => t,
            None => return false,
        };

        tab.is_running = false;
        tab.result = Some(query_result_to_sql_result_data(result, MAX_RESULT_ROWS));
        true
    }

    pub fn set_explain(&mut self, tab_id: u64, explain: ExplainData) -> bool {
        if let Some(tab) = self.get_tab_mut(tab_id) {
            tab.explain = explain;
            true
        } else {
            false
        }
    }

    pub fn clear_tab(&mut self, tab_id: u64) {
        if let Some(tab) = self.get_tab_mut(tab_id) {
            tab.query.clear();
            tab.result = None;
            tab.explain = ExplainData::default();
        }
    }

    pub fn get_headers(&self) -> Vec<SqlTabHeaderData> {
        self.tabs.iter().map(|t| t.to_header()).collect()
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let mgr = TabManager::new();
        assert_eq!(mgr.tab_count(), 1);
        assert_eq!(mgr.active_tab_id(), 1);
        assert_eq!(mgr.epoch(), 1);
        assert!(mgr.can_add_tab());

        let headers = mgr.get_headers();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].id, 1);
        assert_eq!(headers[0].title, "Query 1");
        assert!(!headers[0].is_running);
        assert!(!headers[0].has_unsaved_text);
    }

    #[test]
    fn test_limit_enforcement() {
        let mut mgr = TabManager::new();
        for _ in 1..MAX_TABS {
            assert!(mgr.add_tab().is_ok());
        }
        assert_eq!(mgr.tab_count(), MAX_TABS);
        assert!(!mgr.can_add_tab());

        let err = mgr.add_tab();
        assert!(err.is_err());
        assert_eq!(mgr.tab_count(), MAX_TABS);
    }

    #[test]
    fn test_switch_tab_saves_query() {
        let mut mgr = TabManager::new();
        let tab2_id = mgr.add_tab().unwrap();
        assert_eq!(mgr.active_tab_id(), tab2_id);

        // Switch to tab 1 while saving tab 2 text
        assert!(mgr.switch_tab(1, Some("SELECT 2;")));
        assert_eq!(mgr.active_tab_id(), 1);
        assert_eq!(mgr.get_tab(tab2_id).unwrap().query, "SELECT 2;");
        assert!(mgr.get_tab(tab2_id).unwrap().has_unsaved_text());

        // Switch back to tab 2 while saving tab 1 text
        assert!(mgr.switch_tab(tab2_id, Some("SELECT 1;")));
        assert_eq!(mgr.active_tab_id(), tab2_id);
        assert_eq!(mgr.get_tab(1).unwrap().query, "SELECT 1;");
    }

    #[test]
    fn test_close_middle_tab() {
        let mut mgr = TabManager::new();
        let t2 = mgr.add_tab().unwrap();
        let t3 = mgr.add_tab().unwrap();

        // Switch to middle tab (t2)
        mgr.switch_tab(t2, None);
        assert_eq!(mgr.active_tab_id(), t2);

        // Close t2, should select neighbor t3
        assert!(mgr.close_tab(t2).is_ok());
        assert_eq!(mgr.tab_count(), 2);
        assert_eq!(mgr.active_tab_id(), t3);
    }

    #[test]
    fn test_close_last_tab_opens_fresh() {
        let mut mgr = TabManager::new();
        assert_eq!(mgr.tab_count(), 1);
        let id1 = mgr.active_tab_id();

        // Close only remaining tab
        assert!(mgr.close_tab(id1).is_ok());
        assert_eq!(mgr.tab_count(), 1);
        let id2 = mgr.active_tab_id();
        assert_ne!(id1, id2);
        assert_eq!(mgr.get_tab(id2).unwrap().title, format!("Query {}", id2));
    }

    #[test]
    fn test_id_never_reused_after_close_and_reset() {
        let mut mgr = TabManager::new(); // tab id: 1, next: 2
        let t2 = mgr.add_tab().unwrap(); // id: 2, next: 3
        mgr.close_tab(t2).unwrap();

        let t3 = mgr.add_tab().unwrap(); // must be 3, NOT 2
        assert_eq!(t3, 3);

        mgr.reset(); // epoch: 2, new tab id: 4, next: 5
        assert_eq!(mgr.epoch(), 2);
        assert_eq!(mgr.active_tab_id(), 4);

        let t5 = mgr.add_tab().unwrap(); // must be 5
        assert_eq!(t5, 5);
    }

    #[test]
    fn test_result_routing_after_tab_switch() {
        let mut mgr = TabManager::new();
        let t1 = mgr.active_tab_id();
        let t2 = mgr.add_tab().unwrap();

        // Query starts on t1
        let epoch = mgr.epoch();
        mgr.set_running(t1, true).unwrap();

        // User is currently looking at t2
        assert_eq!(mgr.active_tab_id(), t2);

        // Result arrives for t1
        let qr = QueryResult {
            success: true,
            data: Some(vec![]),
            columns: vec!["id".into()],
            affected_rows: None,
            error: None,
            message: Some("t1 completed".into()),
            execution_time_ms: 15,
        };

        let applied = mgr.set_result(t1, epoch, Ok(qr));
        assert!(applied);

        // Verify t1 has result and is not running
        let tab1 = mgr.get_tab(t1).unwrap();
        assert!(!tab1.is_running);
        assert_eq!(tab1.result.as_ref().unwrap().message, "t1 completed");

        // Verify t2 is unaffected
        let tab2 = mgr.get_tab(t2).unwrap();
        assert!(tab2.result.is_none());
    }

    #[test]
    fn test_stale_epoch_result_discarded() {
        let mut mgr = TabManager::new();
        let t1 = mgr.active_tab_id();
        let old_epoch = mgr.epoch();
        mgr.set_running(t1, true).unwrap();

        // Reset occurs (e.g. database switched or logged out)
        mgr.reset();
        assert_eq!(mgr.epoch(), old_epoch + 1);

        // Late result from old epoch arrives
        let qr = QueryResult {
            success: true,
            data: None,
            columns: vec![],
            affected_rows: Some(1),
            error: None,
            message: Some("stale".into()),
            execution_time_ms: 10,
        };

        let applied = mgr.set_result(t1, old_epoch, Ok(qr));
        assert!(!applied);

        // Active tab in new epoch should not have this result
        let active_tab = mgr.get_active_tab().unwrap();
        assert!(active_tab.result.is_none());
    }

    #[test]
    fn test_result_discarded_if_tab_closed() {
        let mut mgr = TabManager::new();
        let t1 = mgr.active_tab_id();
        let t2 = mgr.add_tab().unwrap();
        let epoch = mgr.epoch();

        mgr.set_running(t1, true).unwrap();

        // Close t1 while query is in-flight
        mgr.close_tab(t1).unwrap();

        let qr = QueryResult {
            success: true,
            data: None,
            columns: vec![],
            affected_rows: None,
            error: None,
            message: None,
            execution_time_ms: 5,
        };

        let applied = mgr.set_result(t1, epoch, Ok(qr));
        assert!(!applied);

        let tab2 = mgr.get_tab(t2).unwrap();
        assert!(tab2.result.is_none());
    }

    #[test]
    fn test_run_query_rejected_while_running() {
        let mut mgr = TabManager::new();
        let t1 = mgr.active_tab_id();

        assert!(mgr.set_running(t1, true).is_ok());
        // Second attempt to set running on same tab fails
        let second_run = mgr.set_running(t1, true);
        assert!(second_run.is_err());
    }

    #[test]
    fn test_row_capping_and_note() {
        let mut data = Vec::new();
        for i in 0..550 {
            let mut obj = serde_json::Map::new();
            obj.insert("num".into(), serde_json::Value::Number(i.into()));
            data.push(serde_json::Value::Object(obj));
        }

        let qr = QueryResult {
            success: true,
            data: Some(data),
            columns: vec!["num".into()],
            affected_rows: None,
            error: None,
            message: Some("Query OK".into()),
            execution_time_ms: 42,
        };

        let res = query_result_to_sql_result_data(Ok(qr), MAX_RESULT_ROWS);
        assert_eq!(res.rows.len(), MAX_RESULT_ROWS);
        assert!(res.message.contains("Showing first 500 rows of 550"));
    }
}
