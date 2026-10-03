use std::rc::Rc;
use std::sync::{Arc, Mutex};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use serde_json::Value;

use crate::{
    AppWindow, ColumnInfoItem, CreateTableColumnItem, DatabaseItem, DbSummaryItem, DiagramTableItem,
    HistoryDisplayItem, InsertFieldItem, MockColumnBlueprint, ProcessItem, RelationItem, SlowLogItem,
    SqlResultRow, SqlTabItem, TableRowData, TableStatItem, UserItem, TableInsertField,
};
use crate::state::SharedState;
use crate::tabs::{TabManager, SqlResultData, ExplainData};
use crate::db::{
    auth, database, table, data::{self, BindValue, DeleteRowSnapshot}, query, server, objects, maintenance,
    history::HistoryManager,
    models::TableColumnInfo,
    sanitize::{build_query_builder_select, sanitize_identifier, validate_column_length, is_destructive},
};

#[derive(Clone, Debug)]
pub struct PendingDestructiveQuery {
    pub sql: String,
    pub tab_id: u64,
    pub epoch: u64,
}

#[derive(Clone)]
pub struct PendingCellEdit {
    pub table: String,
    pub column: String,
    pub pk_values: Vec<(String, BindValue)>,
}

fn format_size(bytes: i64) -> String {
    if bytes <= 0 {
        return "0 B".to_string();
    }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(k).floor() as usize;
    let i = i.min(sizes.len() - 1);
    let val = (bytes as f64) / k.powi(i as i32);
    format!("{:.2} {}", val, sizes[i])
}

fn format_uptime(seconds: i64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    format!("{}d {}h {}m", days, hours, mins)
}

pub struct AppController {
    state: SharedState,
    history_mgr: Arc<HistoryManager>,
    pub tab_mgr: Arc<Mutex<TabManager>>,
    current_db: Arc<Mutex<String>>,
    current_table: Arc<Mutex<String>>,
    current_table_columns: Arc<Mutex<Vec<TableColumnInfo>>>,
    table_limit: Arc<Mutex<i64>>,
    table_offset: Arc<Mutex<i64>>,
    table_sort_col: Arc<Mutex<Option<String>>>,
    table_sort_order: Arc<Mutex<String>>,
    create_columns: Arc<Mutex<Vec<CreateTableColumnItem>>>,
    insert_fields: Arc<Mutex<Vec<InsertFieldItem>>>,
    table_insert_fields: Arc<Mutex<Vec<TableInsertField>>>,
    table_insert_target_table: Arc<Mutex<String>>,
    pending_cell_edit: Arc<Mutex<Option<PendingCellEdit>>>,
    pending_delete_row: Arc<Mutex<Option<DeleteRowSnapshot>>>,
    mock_blueprint: Arc<Mutex<Vec<MockColumnBlueprint>>>,
    export_format: Arc<Mutex<String>>,
    export_structure: Arc<Mutex<bool>>,
    export_data: Arc<Mutex<bool>>,
    pending_destructive_query: Arc<Mutex<Option<PendingDestructiveQuery>>>,
    pending_import_sql: Arc<Mutex<String>>,
    pending_dialog_action: Arc<Mutex<(String, String)>>,
    raw_table_rows: Arc<Mutex<Vec<Value>>>,
}

impl AppController {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            history_mgr: Arc::new(HistoryManager::new()),
            tab_mgr: Arc::new(Mutex::new(TabManager::new())),
            current_db: Arc::new(Mutex::new(String::new())),
            current_table: Arc::new(Mutex::new(String::new())),
            current_table_columns: Arc::new(Mutex::new(Vec::new())),
            table_limit: Arc::new(Mutex::new(50)),
            table_offset: Arc::new(Mutex::new(0)),
            table_sort_col: Arc::new(Mutex::new(None)),
            table_sort_order: Arc::new(Mutex::new("ASC".to_string())),
            create_columns: Arc::new(Mutex::new(vec![CreateTableColumnItem {
                name: "id".into(),
                col_type: "INT".into(),
                length: "".into(),
                is_null: false,
                is_primary: true,
                is_auto_increment: true,
            }])),
            insert_fields: Arc::new(Mutex::new(Vec::new())),
            table_insert_fields: Arc::new(Mutex::new(Vec::new())),
            table_insert_target_table: Arc::new(Mutex::new(String::new())),
            pending_cell_edit: Arc::new(Mutex::new(None)),
            pending_delete_row: Arc::new(Mutex::new(None)),
            mock_blueprint: Arc::new(Mutex::new(Vec::new())),
            export_format: Arc::new(Mutex::new("sql".to_string())),
            export_structure: Arc::new(Mutex::new(true)),
            export_data: Arc::new(Mutex::new(true)),
            pending_destructive_query: Arc::new(Mutex::new(None)),
            pending_import_sql: Arc::new(Mutex::new(String::new())),
            pending_dialog_action: Arc::new(Mutex::new((String::new(), String::new()))),
            raw_table_rows: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn clear_pending_mutation_state(&self, app: Option<&AppWindow>) {
        *self.pending_delete_row.lock().unwrap() = None;
        *self.pending_cell_edit.lock().unwrap() = None;
        *self.table_insert_target_table.lock().unwrap() = String::new();
        if let Some(app) = app {
            app.set_table_cell_edit_open(false);
            app.set_table_insert_modal_open(false);
            let action = self.pending_dialog_action.lock().unwrap().0.clone();
            if action == "delete_row" {
                *self.pending_dialog_action.lock().unwrap() = (String::new(), String::new());
                app.set_dialog_open(false);
            }
        }
    }

    pub fn setup_callbacks(self: Arc<Self>, app: &AppWindow) {
        let weak = app.as_weak();

        // 1. Authentication
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_request_login(move |host, port, user, pwd, req_ssl| {
                let weak = weak.clone();
                let ctrl = ctrl.clone();
                let host = host.to_string();
                let port = port.to_string();
                let user = user.to_string();
                let raw_pwd = pwd.to_string();
                let pwd = if raw_pwd.is_empty() { None } else { Some(raw_pwd.clone()) };

                if let Some(app) = weak.upgrade() {
                    app.set_login_error_message("".into());
                    app.set_login_loading(true);
                }

                tokio::spawn(async move {
                    let res = auth::login(&ctrl.state, &host, &port, &user, pwd.as_deref(), req_ssl).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_login_loading(false);
                        match res {
                            Ok(conn_res) => {
                                app.set_login_error_message("".into());
                                app.set_is_logged_in(true);
                                app.set_is_encrypted(conn_res.is_encrypted);
                                app.set_server_name(format!("{}:{}", host, port).into());
                                app.set_active_view("server_overview".into());
                                ctrl.refresh_databases(&app);
                                ctrl.refresh_server_overview(&app);
                            }
                            Err(e) => {
                                app.set_is_logged_in(false);
                                app.set_is_encrypted(false);
                                let scrubbed = if !raw_pwd.is_empty() {
                                    e.replace(&raw_pwd, "******")
                                } else {
                                    e
                                };
                                app.set_login_error_message(scrubbed.into());
                            }
                        }
                    });
                });
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_request_logout(move || {
                let weak = weak.clone();
                let ctrl = ctrl.clone();
                tokio::spawn(async move {
                    let _ = auth::logout(&ctrl.state).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        ctrl.clear_pending_mutation_state(Some(&app));
                        ctrl.tab_mgr.lock().unwrap().reset();
                        ctrl.sync_active_tab_to_ui(&app);
                        app.set_is_logged_in(false);
                        app.set_is_encrypted(false);
                        app.set_login_error_message("".into());
                        app.set_login_loading(false);
                        app.set_selected_db("".into());
                        app.set_selected_table("".into());
                        app.set_active_view("server_overview".into());
                    });
                });
            });
        }

        // 2. Navigation
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_navigate_view(move |view_name| {
                let v = view_name.as_str();
                if let Some(app) = weak.upgrade() {
                    match v {
                        "server_overview" => ctrl.refresh_server_overview(&app),
                        "db_overview" => ctrl.refresh_db_overview(&app),
                        "browse" => ctrl.refresh_table_data(&app),
                        "structure" => ctrl.refresh_structure(&app),
                        "users" => ctrl.refresh_users(&app),
                        "performance" => ctrl.refresh_performance(&app),
                        "slow_log" => ctrl.refresh_slow_log(&app),
                        "history" => ctrl.load_history(&app),
                        "insert" => ctrl.prepare_insert_view(&app),
                        "mock_data" => ctrl.prepare_mock_data_view(&app),
                        "query_builder" => ctrl.refresh_query_builder(&app),
                        "import" => {
                            ctrl.refresh_query_builder(&app);
                            let cur_tbl = ctrl.current_table.lock().unwrap().clone();
                            if !cur_tbl.is_empty() && app.get_import_target_table().is_empty() {
                                app.set_import_target_table(cur_tbl.into());
                            }
                        }
                        _ => {}
                    }
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_select_database(move |db_name| {
                *ctrl.current_db.lock().unwrap() = db_name.to_string();
                *ctrl.current_table.lock().unwrap() = String::new();
                ctrl.state.set_current_db(Some(db_name.to_string()));
                if let Some(app) = weak.upgrade() {
                    ctrl.clear_pending_mutation_state(Some(&app));
                    ctrl.tab_mgr.lock().unwrap().reset();
                    ctrl.sync_active_tab_to_ui(&app);
                    app.set_selected_db(db_name);
                    app.set_selected_table("".into());
                    app.set_active_view("db_overview".into());
                    ctrl.refresh_db_overview(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_select_table(move |db_name, table_name| {
                *ctrl.current_db.lock().unwrap() = db_name.to_string();
                *ctrl.current_table.lock().unwrap() = table_name.to_string();
                *ctrl.table_offset.lock().unwrap() = 0;
                *ctrl.table_sort_col.lock().unwrap() = None;
                ctrl.state.set_current_db(Some(db_name.to_string()));
                if let Some(app) = weak.upgrade() {
                    ctrl.clear_pending_mutation_state(Some(&app));
                    app.set_selected_db(db_name);
                    app.set_selected_table(table_name);
                    app.set_active_view("browse".into());
                    ctrl.refresh_table_data(&app);
                    ctrl.refresh_structure(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_toggle_db_expanded(move |idx| {
                if let Some(app) = weak.upgrade() {
                    let dbs = app.get_databases();
                    if let Some(mut db_item) = dbs.row_data(idx as usize) {
                        let will_expand = !db_item.expanded;
                        db_item.expanded = will_expand;
                        let db_name = db_item.name.to_string();

                        if will_expand && db_item.tables.row_count() == 0 {
                            let ctrl = ctrl.clone();
                            let weak = weak.clone();
                            tokio::spawn(async move {
                                let tbl_res = table::list_tables(&ctrl.state, &db_name).await.unwrap_or_default();
                                let obj_res = objects::get_objects(&ctrl.state, &db_name).await.ok();

                                let _ = weak.upgrade_in_event_loop(move |app| {
                                    let dbs = app.get_databases();
                                    if let Some(mut item) = dbs.row_data(idx as usize) {
                                        let tbl_model = Rc::new(VecModel::from(
                                            tbl_res.into_iter().map(SharedString::from).collect::<Vec<_>>()
                                        ));
                                        item.tables = ModelRc::from(tbl_model);

                                        if let Some(objs) = obj_res {
                                            let vw_model = Rc::new(VecModel::from(
                                                objs.views.into_iter().map(SharedString::from).collect::<Vec<_>>()
                                            ));
                                            item.views = ModelRc::from(vw_model);

                                            let proc_model = Rc::new(VecModel::from(
                                                objs.procedures.into_iter().map(SharedString::from).collect::<Vec<_>>()
                                            ));
                                            item.procedures = ModelRc::from(proc_model);

                                            let func_model = Rc::new(VecModel::from(
                                                objs.functions.into_iter().map(SharedString::from).collect::<Vec<_>>()
                                            ));
                                            item.functions = ModelRc::from(func_model);
                                        }
                                        dbs.set_row_data(idx as usize, item);
                                    }
                                });
                            });
                        } else {
                            dbs.set_row_data(idx as usize, db_item);
                        }
                    }
                }
            });
        }

        // 3. Database operations (Create / Drop)
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_create_db(move |db_name| {
                let weak = weak.clone();
                let ctrl = ctrl.clone();
                let name = db_name.to_string();
                tokio::spawn(async move {
                    let res = database::create_database(&ctrl.state, &name).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        if res.is_ok() {
                            ctrl.refresh_databases(&app);
                            ctrl.refresh_server_overview(&app);
                        }
                    });
                });
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_request_drop_db(move |db_name| {
                *ctrl.pending_dialog_action.lock().unwrap() = ("drop_db".to_string(), db_name.to_string());
                if let Some(app) = weak.upgrade() {
                    app.set_dialog_title("DROP DATABASE".into());
                    app.set_dialog_message(format!(
                        "Are you sure you want to permanently DROP database [{}]? All tables and data will be erased.",
                        db_name
                    ).into());
                    app.set_dialog_open(true);
                }
            });
        }

        // 4. Table Operations (Drop / Truncate)
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_request_drop_table(move |tbl_name| {
                let db = ctrl.current_db.lock().unwrap().clone();
                *ctrl.pending_dialog_action.lock().unwrap() = ("drop_table".to_string(), tbl_name.to_string());
                if let Some(app) = weak.upgrade() {
                    app.set_dialog_title("DROP TABLE".into());
                    app.set_dialog_message(format!(
                        "Are you sure you want to permanently DROP table [{}.{}]? This action cannot be undone.",
                        db, tbl_name
                    ).into());
                    app.set_dialog_open(true);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_request_truncate_table(move || {
                let db = ctrl.current_db.lock().unwrap().clone();
                let tbl = ctrl.current_table.lock().unwrap().clone();
                *ctrl.pending_dialog_action.lock().unwrap() = ("truncate_table".to_string(), tbl.clone());
                if let Some(app) = weak.upgrade() {
                    app.set_dialog_title("TRUNCATE TABLE".into());
                    app.set_dialog_message(format!(
                        "Are you sure you want to TRUNCATE table [{}.{}]? All rows will be permanently deleted.",
                        db, tbl
                    ).into());
                    app.set_dialog_open(true);
                }
            });
        }

        // 5. Dialog Confirm / Cancel
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_confirm_dialog_action(move || {
                let (action, target) = ctrl.pending_dialog_action.lock().unwrap().clone();
                let weak = weak.clone();
                let ctrl = ctrl.clone();

                if let Some(app) = weak.upgrade() {
                    app.set_dialog_open(false);
                }

                tokio::spawn(async move {
                    match action.as_str() {
                        "drop_db" => {
                            let _ = database::drop_database(&ctrl.state, &target).await;
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                ctrl.clear_pending_mutation_state(Some(&app));
                                ctrl.refresh_databases(&app);
                                ctrl.refresh_server_overview(&app);
                            });
                        }
                        "drop_table" => {
                            let db = ctrl.current_db.lock().unwrap().clone();
                            let _ = table::drop_table(&ctrl.state, &db, &target).await;
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                ctrl.clear_pending_mutation_state(Some(&app));
                                ctrl.refresh_databases(&app);
                                app.set_selected_table("".into());
                                app.set_active_view("db_overview".into());
                                ctrl.refresh_db_overview(&app);
                            });
                        }
                        "truncate_table" => {
                            let db = ctrl.current_db.lock().unwrap().clone();
                            let _ = table::truncate_table(&ctrl.state, &db, &target).await;
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                ctrl.refresh_table_data(&app);
                            });
                        }
                        "delete_row" => {
                            let snapshot_opt = ctrl.pending_delete_row.lock().unwrap().take();
                            let active_table = ctrl.current_table.lock().unwrap().clone();
                            let snapshot = match data::validate_delete_snapshot(snapshot_opt.as_ref(), &active_table) {
                                Ok(()) => snapshot_opt.unwrap(),
                                Err(err) => {
                                    let _ = weak.upgrade_in_event_loop(move |app| {
                                        app.set_table_error_message(err.into());
                                    });
                                    return;
                                }
                            };

                            let db = ctrl.current_db.lock().unwrap().clone();
                            let cols = ctrl.current_table_columns.lock().unwrap().clone();
                            let pk_refs: Vec<(&str, BindValue)> = snapshot.pk_values.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                            let del_res = data::delete_row(&ctrl.state, &db, &snapshot.table, &pk_refs, &cols).await;

                            let query_str = {
                                let sanitized_table = sanitize_identifier(&snapshot.table).unwrap_or_else(|_| snapshot.table.clone());
                                let where_parts: Vec<String> = snapshot.pk_values.iter().map(|(k, _)| format!("{} = ?", sanitize_identifier(k).unwrap_or_else(|_| k.to_string()))).collect();
                                format!("DELETE FROM {} WHERE {} LIMIT 1", sanitized_table, where_parts.join(" AND "))
                            };

                            let _ = weak.upgrade_in_event_loop(move |app| {
                                match del_res {
                                    Ok(_) => {
                                        ctrl.history_mgr.add(&query_str, Some(&db));
                                        ctrl.refresh_table_data(&app);
                                    }
                                    Err(err) => {
                                        app.set_table_error_message(err.into());
                                    }
                                }
                            });
                        }
                        "destructive_query" => {
                            let pending = ctrl.pending_destructive_query.lock().unwrap().take();
                            if let Some(pending) = pending {
                                let (tab_valid, is_active) = {
                                    let mgr = ctrl.tab_mgr.lock().unwrap();
                                    let valid = mgr.epoch() == pending.epoch && mgr.get_tab(pending.tab_id).is_some();
                                    let active = mgr.active_tab_id() == pending.tab_id;
                                    (valid, active)
                                };

                                if !tab_valid {
                                    let _ = weak.upgrade_in_event_loop(move |app| {
                                        app.set_sql_error_message("Query aborted: target tab was closed or database was reset.".into());
                                    });
                                    return;
                                }

                                {
                                    let mut mgr = ctrl.tab_mgr.lock().unwrap();
                                    if mgr.set_running(pending.tab_id, true).is_err() {
                                        return;
                                    }
                                }

                                {
                                    let ctrl = ctrl.clone();
                                    let _ = weak.upgrade_in_event_loop(move |app| {
                                        if is_active {
                                            app.set_sql_loading(true);
                                        }
                                        ctrl.sync_tab_headers(&app);
                                    });
                                }

                                let db = ctrl.current_db.lock().unwrap().clone();
                                let db_opt = if db.is_empty() { None } else { Some(db.as_str()) };
                                ctrl.history_mgr.add(&pending.sql, db_opt);

                                let res = query::execute_query(&ctrl.state, db_opt, &pending.sql, true).await;

                                let _ = weak.upgrade_in_event_loop(move |app| {
                                    let should_apply_to_ui = {
                                        let mut mgr = ctrl.tab_mgr.lock().unwrap();
                                        mgr.set_result(pending.tab_id, pending.epoch, res);
                                        mgr.active_tab_id() == pending.tab_id
                                    };

                                    if should_apply_to_ui {
                                        let mgr = ctrl.tab_mgr.lock().unwrap();
                                        if let Some(tab) = mgr.get_tab(pending.tab_id) {
                                            ctrl.apply_tab_result_to_ui(&app, tab.result.as_ref());
                                        }
                                    }
                                    ctrl.sync_tab_headers(&app);
                                });
                            }
                        }
                        "import_sql" => {
                            let sql = ctrl.pending_import_sql.lock().unwrap().clone();
                            let db = target.clone();
                            ctrl.perform_sql_import(weak.clone(), db, sql).await;
                        }
                        _ => {}
                    }
                });
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_cancel_dialog_action(move || {
                *ctrl.pending_delete_row.lock().unwrap() = None;
                *ctrl.pending_destructive_query.lock().unwrap() = None;
                *ctrl.pending_import_sql.lock().unwrap() = String::new();
                if let Some(app) = weak.upgrade() {
                    app.set_dialog_open(false);
                }
            });
        }

        // 6. SQL Editor
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_run_sql_query(move |sql_query| {
                let sql = sql_query.to_string();
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                if query::split_sql_statements(&sql).is_empty() {
                    return;
                }

                let (tab_id, epoch) = {
                    let mut mgr = ctrl.tab_mgr.lock().unwrap();
                    let tab = match mgr.get_active_tab() {
                        Some(t) if !t.is_running => t,
                        _ => return, // reject if running
                    };
                    let tab_id = tab.id;
                    let epoch = mgr.epoch();
                    mgr.update_active_query(&sql);
                    if crate::db::sanitize::is_destructive(&sql) {
                        *ctrl.pending_destructive_query.lock().unwrap() = Some(PendingDestructiveQuery {
                            sql: sql.clone(),
                            tab_id,
                            epoch,
                        });
                        *ctrl.pending_dialog_action.lock().unwrap() = ("destructive_query".to_string(), String::new());
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            app.set_dialog_title("DESTRUCTIVE QUERY".into());
                            app.set_dialog_message("This query contains DROP, DELETE, TRUNCATE, or ALTER operations. Proceed?".into());
                            app.set_dialog_open(true);
                        });
                        return;
                    }

                    if mgr.set_running(tab_id, true).is_err() {
                        return;
                    }
                    (tab_id, epoch)
                };

                if let Some(app) = weak.upgrade() {
                    app.set_sql_loading(true);
                    ctrl.sync_tab_headers(&app);
                }

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let db_opt = if db.is_empty() { None } else { Some(db.as_str()) };

                    ctrl.history_mgr.add(&sql, db_opt);
                    let res = query::execute_query(&ctrl.state, db_opt, &sql, false).await;

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let should_apply_to_ui = {
                            let mut mgr = ctrl.tab_mgr.lock().unwrap();
                            mgr.set_result(tab_id, epoch, res);
                            mgr.active_tab_id() == tab_id
                        };

                        if should_apply_to_ui {
                            let mgr = ctrl.tab_mgr.lock().unwrap();
                            if let Some(tab) = mgr.get_tab(tab_id) {
                                ctrl.apply_tab_result_to_ui(&app, tab.result.as_ref());
                            }
                        }
                        ctrl.sync_tab_headers(&app);
                    });
                });
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_explain_sql_query(move |sql_query| {
                let sql = sql_query.to_string();
                let exp = crate::explain::explain_query(&sql);
                let explain_data = ExplainData {
                    is_open: true,
                    summary: exp.summary.clone(),
                    lines: exp.lines.clone(),
                    warnings: exp.warnings.clone(),
                };

                {
                    let mut mgr = ctrl.tab_mgr.lock().unwrap();
                    let active_id = mgr.active_tab_id();
                    mgr.update_active_query(&sql);
                    mgr.set_explain(active_id, explain_data);
                }

                if let Some(app) = weak.upgrade() {
                    app.set_sql_explain_summary(exp.summary.into());
                    let lines: Vec<SharedString> = exp.lines.into_iter().map(Into::into).collect();
                    app.set_sql_explain_lines(Rc::new(VecModel::from(lines)).into());
                    let warnings: Vec<SharedString> = exp.warnings.into_iter().map(Into::into).collect();
                    app.set_sql_explain_warnings(Rc::new(VecModel::from(warnings)).into());
                    app.set_sql_explain_open(true);
                    ctrl.sync_tab_headers(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_clear_sql_query(move || {
                {
                    let mut mgr = ctrl.tab_mgr.lock().unwrap();
                    let active_id = mgr.active_tab_id();
                    mgr.clear_tab(active_id);
                }

                if let Some(app) = weak.upgrade() {
                    app.set_sql_query_text("".into());
                    app.set_sql_message("".into());
                    app.set_sql_error_message("".into());
                    app.set_sql_execution_time_ms(0);
                    app.set_sql_affected_rows(-1);
                    app.set_sql_result_columns(ModelRc::default());
                    app.set_sql_result_rows(ModelRc::default());
                    app.set_sql_explain_open(false);
                    app.set_sql_explain_summary("".into());
                    app.set_sql_explain_lines(Rc::new(VecModel::from(vec![])).into());
                    app.set_sql_explain_warnings(Rc::new(VecModel::from(vec![])).into());
                    ctrl.sync_tab_headers(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_sql_switch_tab(move |target_id| {
                if let Some(app) = weak.upgrade() {
                    let cur_query = app.get_sql_query_text().to_string();
                    let cur_explain_open = app.get_sql_explain_open();
                    {
                        let mut mgr = ctrl.tab_mgr.lock().unwrap();
                        mgr.update_active_query(&cur_query);
                        mgr.update_active_explain_open(cur_explain_open);
                        let _ = mgr.switch_tab(target_id as u64, None);
                    }
                    ctrl.sync_active_tab_to_ui(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_sql_close_tab(move |tab_id| {
                if let Some(app) = weak.upgrade() {
                    let cur_query = app.get_sql_query_text().to_string();
                    let cur_explain_open = app.get_sql_explain_open();
                    {
                        let mut mgr = ctrl.tab_mgr.lock().unwrap();
                        mgr.update_active_query(&cur_query);
                        mgr.update_active_explain_open(cur_explain_open);
                        let _ = mgr.close_tab(tab_id as u64);
                    }
                    ctrl.sync_active_tab_to_ui(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_sql_add_tab(move || {
                if let Some(app) = weak.upgrade() {
                    let cur_query = app.get_sql_query_text().to_string();
                    let cur_explain_open = app.get_sql_explain_open();
                    {
                        let mut mgr = ctrl.tab_mgr.lock().unwrap();
                        mgr.update_active_query(&cur_query);
                        mgr.update_active_explain_open(cur_explain_open);
                        let _ = mgr.add_tab();
                    }
                    ctrl.sync_active_tab_to_ui(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_history_run_query(move |query_text| {
                let sql = query_text.to_string();
                if let Some(app) = weak.upgrade() {
                    {
                        let mut mgr = ctrl.tab_mgr.lock().unwrap();
                        mgr.update_active_query(&sql);
                    }
                    app.set_sql_query_text(sql.into());
                    ctrl.sync_tab_headers(&app);
                }
            });
        }

        // 7. Table Browser controls
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_prev_table_page(move || {
                let mut offset = ctrl.table_offset.lock().unwrap();
                let limit = *ctrl.table_limit.lock().unwrap();
                if *offset >= limit {
                    *offset -= limit;
                }
                drop(offset);
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_table_data(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_next_table_page(move || {
                let mut offset = ctrl.table_offset.lock().unwrap();
                let limit = *ctrl.table_limit.lock().unwrap();
                *offset += limit;
                drop(offset);
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_table_data(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_set_table_limit(move |l| {
                *ctrl.table_limit.lock().unwrap() = l as i64;
                *ctrl.table_offset.lock().unwrap() = 0;
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_table_data(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_sort_table_column(move |col| {
                let col_str = col.to_string();
                let mut cur_col = ctrl.table_sort_col.lock().unwrap();
                let mut cur_order = ctrl.table_sort_order.lock().unwrap();

                if cur_col.as_deref() == Some(&col_str) {
                    *cur_order = if *cur_order == "ASC" { "DESC".to_string() } else { "ASC".to_string() };
                } else {
                    *cur_col = Some(col_str);
                    *cur_order = "ASC".to_string();
                }
                *ctrl.table_offset.lock().unwrap() = 0;
                drop(cur_col);
                drop(cur_order);

                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_table_data(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_delete_table_row(move |row_idx| {
                let tbl = ctrl.current_table.lock().unwrap().clone();
                let cols = ctrl.current_table_columns.lock().unwrap().clone();
                let pri_cols: Vec<_> = cols.iter().filter(|c| c.is_primary()).collect();
                if pri_cols.is_empty() || pri_cols.iter().any(|c| c.has_unsupported_pk_type()) {
                    return;
                }

                let offset = *ctrl.table_offset.lock().unwrap();
                let local_idx = if (row_idx as i64) >= offset {
                    (row_idx as i64 - offset) as usize
                } else {
                    row_idx as usize
                };

                let target_row = {
                    let raw_rows = ctrl.raw_table_rows.lock().unwrap();
                    match raw_rows.get(local_idx) {
                        Some(r) => r.clone(),
                        None => return,
                    }
                };

                let mut captured_pks = Vec::new();
                let mut pk_strs = Vec::new();
                for pri in &pri_cols {
                    let val = target_row.as_object().and_then(|obj| obj.get(&pri.field)).unwrap_or(&Value::Null);
                    let bind_val = BindValue::from(val);
                    let val_str = match &bind_val {
                        BindValue::Null => "NULL".to_string(),
                        BindValue::String(s) => format!("'{}'", s),
                        BindValue::Int(i) => i.to_string(),
                        BindValue::Float(f) => f.to_string(),
                        BindValue::Bool(b) => b.to_string(),
                    };
                    pk_strs.push(format!("{}={}", pri.field, val_str));
                    captured_pks.push((pri.field.clone(), bind_val));
                }

                let snapshot = DeleteRowSnapshot::new(&tbl, captured_pks);
                *ctrl.pending_delete_row.lock().unwrap() = Some(snapshot);
                *ctrl.pending_dialog_action.lock().unwrap() = ("delete_row".to_string(), tbl.clone());

                let pk_summary = pk_strs.join(", ");
                let message = format!("Delete row {} from {}? This action cannot be undone.", pk_summary, tbl);

                if let Some(app) = weak.upgrade() {
                    app.set_dialog_title("DELETE ROW".into());
                    app.set_dialog_message(message.into());
                    app.set_dialog_open(true);
                }
            });
        }

        // Table Data: Inline Cell Edit Callbacks
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_request_cell_edit(move |row_idx, col_idx| {
                let cols = ctrl.current_table_columns.lock().unwrap().clone();
                let pri_cols: Vec<_> = cols.iter().filter(|c| c.is_primary()).collect();
                if pri_cols.is_empty() || pri_cols.iter().any(|c| c.has_unsupported_pk_type()) {
                    return;
                }

                let offset = *ctrl.table_offset.lock().unwrap();
                let local_idx = if (row_idx as i64) >= offset {
                    (row_idx as i64 - offset) as usize
                } else {
                    row_idx as usize
                };

                let raw_rows = ctrl.raw_table_rows.lock().unwrap();
                let target_row = match raw_rows.get(local_idx) {
                    Some(r) => r.clone(),
                    None => return,
                };

                let target_col = match cols.get(col_idx as usize) {
                    Some(c) => c.clone(),
                    None => return,
                };

                if target_col.is_read_only() {
                    return;
                }

                let mut captured_pks = Vec::new();
                for pri in &pri_cols {
                    let val = target_row.as_object().and_then(|obj| obj.get(&pri.field)).unwrap_or(&Value::Null);
                    captured_pks.push((pri.field.clone(), BindValue::from(val)));
                }

                let tbl = ctrl.current_table.lock().unwrap().clone();
                *ctrl.pending_cell_edit.lock().unwrap() = Some(PendingCellEdit {
                    table: tbl,
                    column: target_col.field.clone(),
                    pk_values: captured_pks,
                });

                let cell_val = target_row.as_object().and_then(|obj| obj.get(&target_col.field)).unwrap_or(&Value::Null);
                let is_null = cell_val.is_null();
                let val_str = match cell_val {
                    Value::Null => String::new(),
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };

                let is_pk = target_col.is_primary();

                if let Some(app) = weak.upgrade() {
                    app.set_table_cell_edit_col_name(target_col.field.clone().into());
                    app.set_table_cell_edit_col_type(target_col.r#type.clone().into());
                    app.set_table_cell_edit_is_pk(is_pk);
                    app.set_table_cell_edit_nullable(target_col.is_nullable());
                    app.set_table_cell_edit_is_null(is_null);
                    app.set_table_cell_edit_value(val_str.into());
                    app.set_table_cell_edit_error("".into());
                    app.set_table_cell_edit_loading(false);
                    app.set_table_cell_edit_open(true);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_cell_edit(move |new_val, is_null| {
                let pending = match ctrl.pending_cell_edit.lock().unwrap().clone() {
                    Some(p) => p,
                    None => return,
                };

                let cols = ctrl.current_table_columns.lock().unwrap().clone();
                if let Some(target_col) = cols.iter().find(|c| c.field.eq_ignore_ascii_case(&pending.column)) {
                    if target_col.is_primary() {
                        if let Some(app) = weak.upgrade() {
                            app.set_table_cell_edit_loading(false);
                            app.set_table_cell_edit_error("Primary key: delete and re-insert instead".into());
                        }
                        return;
                    }
                }

                let weak = weak.clone();
                let ctrl = ctrl.clone();

                if let Some(app) = weak.upgrade() {
                    app.set_table_cell_edit_loading(true);
                    app.set_table_cell_edit_error("".into());
                }

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let cols = ctrl.current_table_columns.lock().unwrap().clone();
                    let bind_val = if is_null {
                        BindValue::Null
                    } else {
                        BindValue::String(new_val.to_string())
                    };

                    let pk_refs: Vec<(&str, BindValue)> = pending.pk_values.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();

                    let res = data::update_row(
                        &ctrl.state,
                        &db,
                        &pending.table,
                        &pending.column,
                        bind_val,
                        &pk_refs,
                        &cols,
                    ).await;

                    let query_str = {
                        let sanitized_table = sanitize_identifier(&pending.table).unwrap_or_else(|_| pending.table.clone());
                        let sanitized_col = sanitize_identifier(&pending.column).unwrap_or_else(|_| pending.column.clone());
                        let where_parts: Vec<String> = pending.pk_values.iter().map(|(k, _)| format!("{} = ?", sanitize_identifier(k).unwrap_or_else(|_| k.to_string()))).collect();
                        format!("UPDATE {} SET {} = ? WHERE {} LIMIT 1", sanitized_table, sanitized_col, where_parts.join(" AND "))
                    };

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_table_cell_edit_loading(false);
                        match res {
                            Ok(_) => {
                                app.set_table_cell_edit_open(false);
                                *ctrl.pending_cell_edit.lock().unwrap() = None;
                                ctrl.history_mgr.add(&query_str, Some(&db));
                                ctrl.refresh_table_data(&app);
                            }
                            Err(err) => {
                                app.set_table_cell_edit_error(err.into());
                            }
                        }
                    });
                });
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_cancel_cell_edit(move || {
                *ctrl.pending_cell_edit.lock().unwrap() = None;
                if let Some(app) = weak.upgrade() {
                    app.set_table_cell_edit_open(false);
                    app.set_table_cell_edit_error("".into());
                }
            });
        }

        // Table Data: Insert Modal Callbacks
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_open_insert_modal(move || {
                let tbl = ctrl.current_table.lock().unwrap().clone();
                let cols = ctrl.current_table_columns.lock().unwrap().clone();
                let mut insert_fields = Vec::new();

                for col in cols {
                    if col.is_generated() {
                        continue;
                    }
                    let is_auto = col.is_auto_increment();
                    let has_default = col.default.is_some();
                    let is_nullable = col.is_nullable();
                    let default_checked = is_auto || has_default;
                    let is_null_checked = !default_checked && is_nullable;

                    insert_fields.push(TableInsertField {
                        name: col.field.into(),
                        col_type: col.r#type.into(),
                        nullable: is_nullable,
                        is_default: default_checked,
                        is_null: is_null_checked,
                        value: "".into(),
                    });
                }

                *ctrl.table_insert_fields.lock().unwrap() = insert_fields.clone();
                *ctrl.table_insert_target_table.lock().unwrap() = tbl.clone();

                if let Some(app) = weak.upgrade() {
                    app.set_table_insert_fields(ModelRc::from(Rc::new(VecModel::from(insert_fields))));
                    app.set_table_insert_modal_error("".into());
                    app.set_table_insert_modal_loading(false);
                    app.set_table_insert_modal_open(true);
                }
            });
        }

        {
            let ctrl = self.clone();
            let weak = weak.clone();
            app.on_update_insert_field_val(move |idx, val| {
                let mut fields = ctrl.table_insert_fields.lock().unwrap();
                if let Some(fld) = fields.get_mut(idx as usize) {
                    fld.value = val;
                    fld.is_default = false;
                    fld.is_null = false;
                }
                if let Some(app) = weak.upgrade() {
                    app.set_table_insert_fields(ModelRc::from(Rc::new(VecModel::from(fields.clone()))));
                }
            });
        }

        {
            let ctrl = self.clone();
            let weak = weak.clone();
            app.on_update_insert_field_null(move |idx, is_null| {
                let mut fields = ctrl.table_insert_fields.lock().unwrap();
                if let Some(fld) = fields.get_mut(idx as usize) {
                    fld.is_null = is_null;
                    if is_null {
                        fld.is_default = false;
                    }
                }
                if let Some(app) = weak.upgrade() {
                    app.set_table_insert_fields(ModelRc::from(Rc::new(VecModel::from(fields.clone()))));
                }
            });
        }

        {
            let ctrl = self.clone();
            let weak = weak.clone();
            app.on_update_insert_field_default(move |idx, is_def| {
                let mut fields = ctrl.table_insert_fields.lock().unwrap();
                if let Some(fld) = fields.get_mut(idx as usize) {
                    fld.is_default = is_def;
                    if is_def {
                        fld.is_null = false;
                    }
                }
                if let Some(app) = weak.upgrade() {
                    app.set_table_insert_fields(ModelRc::from(Rc::new(VecModel::from(fields.clone()))));
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_insert_modal(move || {
                let weak = weak.clone();
                let ctrl = ctrl.clone();

                if let Some(app) = weak.upgrade() {
                    app.set_table_insert_modal_loading(true);
                    app.set_table_insert_modal_error("".into());
                }

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let tbl = ctrl.current_table.lock().unwrap().clone();
                    let target_tbl = ctrl.table_insert_target_table.lock().unwrap().clone();

                    if let Err(err) = data::validate_insert_target_table(&target_tbl, &tbl) {
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            app.set_table_insert_modal_loading(false);
                            app.set_table_insert_modal_error(err.into());
                        });
                        return;
                    }

                    let fields = ctrl.table_insert_fields.lock().unwrap().clone();
                    let cols = ctrl.current_table_columns.lock().unwrap().clone();

                    let mut insert_tuples = Vec::new();
                    for f in &fields {
                        let opt_val = if f.is_default {
                            None
                        } else if f.is_null {
                            Some(BindValue::Null)
                        } else {
                            Some(BindValue::String(f.value.to_string()))
                        };
                        insert_tuples.push((f.name.as_str(), opt_val));
                    }

                    let res = data::insert_row(&ctrl.state, &db, &tbl, &insert_tuples, &cols).await;

                    let query_str = {
                        let non_defaults: Vec<String> = fields
                            .iter()
                            .filter(|f| !f.is_default)
                            .map(|f| sanitize_identifier(&f.name).unwrap_or_else(|_| f.name.to_string()))
                            .collect();
                        let sanitized_table = sanitize_identifier(&tbl).unwrap_or_else(|_| tbl.clone());
                        let col_str = if non_defaults.is_empty() {
                            "() VALUES ()".to_string()
                        } else {
                            let placeholders: Vec<&str> = non_defaults.iter().map(|_| "?").collect();
                            format!("({}) VALUES ({})", non_defaults.join(", "), placeholders.join(", "))
                        };
                        format!("INSERT INTO {} {}", sanitized_table, col_str)
                    };

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_table_insert_modal_loading(false);
                        match res {
                            Ok(_) => {
                                app.set_table_insert_modal_open(false);
                                *ctrl.table_insert_target_table.lock().unwrap() = String::new();
                                ctrl.history_mgr.add(&query_str, Some(&db));
                                ctrl.refresh_table_data(&app);
                            }
                            Err(err) => {
                                app.set_table_insert_modal_error(err.into());
                            }
                        }
                    });
                });
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_cancel_insert_modal(move || {
                *ctrl.table_insert_target_table.lock().unwrap() = String::new();
                if let Some(app) = weak.upgrade() {
                    app.set_table_insert_modal_open(false);
                    app.set_table_insert_modal_error("".into());
                }
            });
        }

        // 8. Refresh callbacks
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_refresh_server_overview(move || {
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_server_overview(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_refresh_db_overview(move || {
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_db_overview(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_refresh_table_data(move || {
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_table_data(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_refresh_structure(move || {
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_structure(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_refresh_users(move || {
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_users(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_refresh_performance(move || {
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_performance(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_refresh_slow_log(move || {
                if let Some(app) = weak.upgrade() {
                    ctrl.refresh_slow_log(&app);
                }
            });
        }

        // 9. History callbacks
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_history_toggle_fav(move |idx| {
                ctrl.history_mgr.toggle_favorite(idx as usize);
                if let Some(app) = weak.upgrade() {
                    ctrl.load_history(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_history_delete(move |idx| {
                ctrl.history_mgr.delete(idx as usize);
                if let Some(app) = weak.upgrade() {
                    ctrl.load_history(&app);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_history_clear(move || {
                ctrl.history_mgr.clear();
                if let Some(app) = weak.upgrade() {
                    ctrl.load_history(&app);
                }
            });
        }

        // 10. Visual Query Builder callbacks
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_builder_select_table(move |tbl| {
                let db = ctrl.current_db.lock().unwrap().clone();
                let tbl_str = tbl.to_string();
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                if let Some(app) = weak.upgrade() {
                    app.set_selected_table(tbl_str.clone().into());
                }

                tokio::spawn(async move {
                    let desc = table::get_structure(&ctrl.state, &db, &tbl_str).await.unwrap_or_default();
                    let cols: Vec<SharedString> = desc.into_iter().map(|c| c.field.into()).collect();
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_selected_table(tbl_str.clone().into());
                        app.set_builder_columns(ModelRc::from(Rc::new(VecModel::from(cols))));

                        let cond_col = app.get_builder_cond_col().to_string();
                        let cond_op = app.get_builder_cond_op().to_string();
                        let cond_val = app.get_builder_cond_val().to_string();
                        let limit_str = app.get_builder_limit_str().to_string();
                        let limit = limit_str.trim().parse::<i64>().unwrap_or(100);

                        match build_query_builder_select(
                            &tbl_str,
                            &[],
                            if cond_col.trim().is_empty() { None } else { Some(&cond_col) },
                            Some(&cond_op),
                            if cond_val.is_empty() { None } else { Some(&cond_val) },
                            None,
                            Some(limit),
                        ) {
                            Ok(sql) => app.set_builder_generated_sql(sql.into()),
                            Err(e) => app.set_builder_generated_sql(format!("-- Error: {}", e).into()),
                        }
                    });
                });
            });
        }

        // 11. Insert row callbacks
        {
            let ctrl = self.clone();
            app.on_update_insert_field(move |idx, val| {
                let mut fields = ctrl.insert_fields.lock().unwrap();
                if let Some(fld) = fields.get_mut(idx as usize) {
                    fld.value = val;
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_insert_row(move || {
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let tbl = ctrl.current_table.lock().unwrap().clone();
                    let fields = ctrl.insert_fields.lock().unwrap().clone();

                    let mut insert_tuples = Vec::new();
                    for f in &fields {
                        let val_str = f.value.to_string();
                        let opt_val = if val_str.is_empty() {
                            None
                        } else {
                            Some(BindValue::String(val_str))
                        };
                        insert_tuples.push((f.name.as_str(), opt_val));
                    }

                    let cols = ctrl.current_table_columns.lock().unwrap().clone();
                    let res = data::insert_row(&ctrl.state, &db, &tbl, &insert_tuples, &cols).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        if res.is_ok() {
                            app.set_active_view("browse".into());
                            ctrl.refresh_table_data(&app);
                        }
                    });
                });
            });
        }

        // 12. Mock Data callbacks
        {
            let ctrl = self.clone();
            app.on_update_mock_type(move |idx, gen_type| {
                let mut bp = ctrl.mock_blueprint.lock().unwrap();
                if let Some(col) = bp.get_mut(idx as usize) {
                    col.generator_type = gen_type;
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_mock_data(move || {
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let tbl = ctrl.current_table.lock().unwrap().clone();
                    let bp = ctrl.mock_blueprint.lock().unwrap().clone();

                    let mut bp_map = serde_json::Map::new();
                    for c in bp {
                        bp_map.insert(c.column_name.to_string(), Value::String(c.generator_type.to_string()));
                    }

                    let _ = maintenance::generate_mock_data(&ctrl.state, &db, &tbl, 50, &Value::Object(bp_map)).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_active_view("browse".into());
                        ctrl.refresh_table_data(&app);
                    });
                });
            });
        }

        // 13. Export callbacks
        {
            let ctrl = self.clone();
            let weak = weak.clone();
            app.on_set_export_format(move |fmt| {
                *ctrl.export_format.lock().unwrap() = fmt.to_string();
                if let Some(app) = weak.upgrade() {
                    app.set_export_format(fmt);
                }
            });
        }

        {
            let ctrl = self.clone();
            let weak = weak.clone();
            app.on_toggle_export_structure(move |val| {
                *ctrl.export_structure.lock().unwrap() = val;
                if let Some(app) = weak.upgrade() {
                    app.set_export_include_structure(val);
                }
            });
        }

        {
            let ctrl = self.clone();
            let weak = weak.clone();
            app.on_toggle_export_data(move |val| {
                *ctrl.export_data.lock().unwrap() = val;
                if let Some(app) = weak.upgrade() {
                    app.set_export_include_data(val);
                }
            });
        }

        {
            let ctrl = self.clone();
            let weak = weak.clone();
            app.on_submit_export(move || {
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                let db = ctrl.current_db.lock().unwrap().clone();
                if db.is_empty() {
                    if let Some(app) = weak.upgrade() {
                        app.set_export_error_message("Please select a database first.".into());
                    }
                    return;
                }

                let fmt = if let Some(app) = weak.upgrade() {
                    app.get_export_format().to_string()
                } else {
                    ctrl.export_format.lock().unwrap().clone()
                };
                let inc_struct = if let Some(app) = weak.upgrade() {
                    app.get_export_include_structure()
                } else {
                    *ctrl.export_structure.lock().unwrap()
                };
                let inc_data = if let Some(app) = weak.upgrade() {
                    app.get_export_include_data()
                } else {
                    *ctrl.export_data.lock().unwrap()
                };

                let filename = format!("{}_dump.{}", db, fmt);

                tokio::spawn(async move {
                    let mut dialog = rfd::AsyncFileDialog::new().set_file_name(&filename);
                    if fmt == "sql" {
                        dialog = dialog.add_filter("SQL Dump (*.sql)", &["sql"]);
                    } else if fmt == "csv" {
                        dialog = dialog.add_filter("CSV (*.csv)", &["csv"]);
                    } else if fmt == "json" {
                        dialog = dialog.add_filter("JSON (*.json)", &["json"]);
                    }

                    let file = dialog.save_file().await;
                    let file_handle = match file {
                        Some(f) => f,
                        None => {
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_export_loading(false);
                            });
                            return;
                        }
                    };

                    let path = file_handle.path().to_path_buf();

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_export_loading(true);
                        app.set_export_error_message("".into());
                        app.set_export_message("Streaming export to file...".into());
                    });

                    let res = maintenance::export_database_stream(
                        &ctrl.state,
                        &db,
                        &fmt,
                        inc_struct,
                        inc_data,
                        &path,
                    ).await;

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_export_loading(false);
                        match res {
                            Ok((tables_exported, file_size)) => {
                                let size_str = if file_size < 1024 {
                                    format!("{} B", file_size)
                                } else if file_size < 1024 * 1024 {
                                    format!("{:.2} KB", file_size as f64 / 1024.0)
                                } else {
                                    format!("{:.2} MB", file_size as f64 / (1024.0 * 1024.0))
                                };
                                app.set_export_message(format!(
                                    "Export completed successfully: {} tables exported ({}) to {}",
                                    tables_exported,
                                    size_str,
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                ).into());
                                app.set_export_error_message("".into());
                            }
                            Err(e) => {
                                app.set_export_error_message(e.into());
                                app.set_export_message("".into());
                            }
                        }
                    });
                });
            });
        }

        // 14. Import callbacks
        {
            let weak = weak.clone();
            app.on_choose_import_file(move || {
                let weak = weak.clone();
                tokio::spawn(async move {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("SQL Files (*.sql)", &["sql"])
                        .pick_file()
                        .await;

                    if let Some(file_handle) = file {
                        let path = file_handle.path().to_path_buf();
                        let metadata = match tokio::fs::metadata(&path).await {
                            Ok(m) => m,
                            Err(e) => {
                                let _ = weak.upgrade_in_event_loop(move |app| {
                                    app.set_import_error_message(format!("Failed to read file metadata: {}", e).into());
                                });
                                return;
                            }
                        };

                        const MAX_SQL_SIZE: u64 = 50 * 1024 * 1024;
                        if metadata.len() > MAX_SQL_SIZE {
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_import_error_message(format!(
                                    "SQL file size ({:.2} MB) exceeds maximum permitted limit of 50 MB",
                                    metadata.len() as f64 / (1024.0 * 1024.0)
                                ).into());
                            });
                            return;
                        }

                        match tokio::fs::read_to_string(&path).await {
                            Ok(content) => {
                                let _ = weak.upgrade_in_event_loop(move |app| {
                                    app.set_import_sql_content(content.into());
                                    app.set_import_error_message("".into());
                                    app.set_import_message(format!(
                                        "Loaded SQL file: {} ({:.1} KB)",
                                        path.file_name().unwrap_or_default().to_string_lossy(),
                                        metadata.len() as f64 / 1024.0
                                    ).into());
                                });
                            }
                            Err(e) => {
                                let _ = weak.upgrade_in_event_loop(move |app| {
                                    app.set_import_error_message(format!("Failed to read SQL file: {}", e).into());
                                });
                            }
                        }
                    }
                });
            });
        }

        {
            let weak = weak.clone();
            app.on_choose_import_csv_file(move || {
                let weak = weak.clone();
                tokio::spawn(async move {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("CSV Files (*.csv)", &["csv"])
                        .pick_file()
                        .await;

                    if let Some(file_handle) = file {
                        let path_str = file_handle.path().to_string_lossy().to_string();
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            app.set_import_csv_path(path_str.into());
                            app.set_import_error_message("".into());
                        });
                    }
                });
            });
        }

        {
            let weak = weak.clone();
            app.on_set_import_format(move |fmt| {
                if let Some(app) = weak.upgrade() {
                    app.set_import_format(fmt);
                    app.set_import_error_message("".into());
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_import(move |sql| {
                let sql_str = sql.to_string();
                if sql_str.trim().is_empty() {
                    return;
                }
                let db = ctrl.current_db.lock().unwrap().clone();
                if db.is_empty() {
                    if let Some(app) = weak.upgrade() {
                        app.set_import_error_message("Please select a database first.".into());
                    }
                    return;
                }

                let stmts = query::split_sql_statements(&sql_str);
                let is_dest = stmts.iter().any(|s| is_destructive(s));

                if is_dest {
                    *ctrl.pending_dialog_action.lock().unwrap() = ("import_sql".to_string(), db.clone());
                    *ctrl.pending_import_sql.lock().unwrap() = sql_str.clone();
                    if let Some(app) = weak.upgrade() {
                        app.set_dialog_title("DESTRUCTIVE SQL IMPORT".into());
                        app.set_dialog_message(format!(
                            "The SQL script contains destructive operations (DROP, TRUNCATE, or DELETE) for database [{}]. Are you sure you want to execute it?",
                            db
                        ).into());
                        app.set_dialog_open(true);
                    }
                } else {
                    ctrl.clone().execute_sql_import(weak.clone(), db, sql_str);
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_csv_import(move |tbl, path| {
                let tbl_str = tbl.to_string();
                let path_str = path.to_string();
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                if tbl_str.trim().is_empty() {
                    if let Some(app) = weak.upgrade() {
                        app.set_import_error_message("Target table name cannot be empty.".into());
                    }
                    return;
                }

                if path_str.trim().is_empty() {
                    if let Some(app) = weak.upgrade() {
                        app.set_import_error_message("Please select a CSV file first.".into());
                    }
                    return;
                }

                let db = ctrl.current_db.lock().unwrap().clone();
                if db.is_empty() {
                    if let Some(app) = weak.upgrade() {
                        app.set_import_error_message("Please select a database first.".into());
                    }
                    return;
                }

                if let Some(app) = weak.upgrade() {
                    app.set_import_loading(true);
                    app.set_import_error_message("".into());
                    app.set_import_message("Importing CSV data into table...".into());
                }

                tokio::spawn(async move {
                    let file_path = std::path::PathBuf::from(path_str);
                    let res = maintenance::import_csv_file(&ctrl.state, &db, &tbl_str, &file_path).await;

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_import_loading(false);
                        match res {
                            Ok(inserted) => {
                                app.set_import_message(format!(
                                    "CSV import completed successfully: {} rows inserted into table '{}'.",
                                    inserted, tbl_str
                                ).into());
                                app.set_import_error_message("".into());
                                ctrl.refresh_databases(&app);
                                ctrl.refresh_db_overview(&app);
                            }
                            Err(e) => {
                                app.set_import_error_message(e.into());
                                app.set_import_message("".into());
                            }
                        }
                    });
                });
            });
        }

        // 15. Diagram callbacks
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_select_diagram(move |db_name| {
                let db = db_name.to_string();
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                tokio::spawn(async move {
                    let rels_res = objects::get_relations(&ctrl.state, &db).await.unwrap_or_default();
                    let tbls_res = table::list_tables(&ctrl.state, &db).await.unwrap_or_default();

                    let mut raw_tables = Vec::new();
                    for t in &tbls_res {
                        let desc = table::get_structure(&ctrl.state, &db, t).await.unwrap_or_default();
                        let col_names: Vec<String> = desc.into_iter().map(|c| c.field).collect();
                        raw_tables.push((t.clone(), col_names));
                    }

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let diagram_tables: Vec<DiagramTableItem> = raw_tables.into_iter().map(|(tname, cols)| {
                            let col_items: Vec<SharedString> = cols.into_iter().map(Into::into).collect();
                            DiagramTableItem {
                                name: tname.into(),
                                columns: ModelRc::from(Rc::new(VecModel::from(col_items))),
                            }
                        }).collect();

                        let rel_items: Vec<RelationItem> = rels_res.into_iter().map(|v| {
                            RelationItem {
                                table_name: v.get("TABLE_NAME").and_then(|s| s.as_str()).unwrap_or_default().into(),
                                column_name: v.get("COLUMN_NAME").and_then(|s| s.as_str()).unwrap_or_default().into(),
                                ref_table: v.get("REFERENCED_TABLE_NAME").and_then(|s| s.as_str()).unwrap_or_default().into(),
                                ref_column: v.get("REFERENCED_COLUMN_NAME").and_then(|s| s.as_str()).unwrap_or_default().into(),
                            }
                        }).collect();

                        app.set_diagram_tables(ModelRc::from(Rc::new(VecModel::from(diagram_tables))));
                        app.set_diagram_relations(ModelRc::from(Rc::new(VecModel::from(rel_items))));
                    });
                });
            });
        }

        // 16. Create Table callbacks
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_open_create_table(move || {
                if let Some(app) = weak.upgrade() {
                    app.set_active_view("create_table".into());
                    let mut cols = ctrl.create_columns.lock().unwrap();
                    *cols = vec![CreateTableColumnItem {
                        name: "id".into(),
                        col_type: "INT".into(),
                        length: "".into(),
                        is_null: false,
                        is_primary: true,
                        is_auto_increment: true,
                    }];
                    app.set_create_table_columns(ModelRc::from(Rc::new(VecModel::from(cols.clone()))));
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_add_create_column(move || {
                let mut cols = ctrl.create_columns.lock().unwrap();
                cols.push(CreateTableColumnItem {
                    name: "".into(),
                    col_type: "VARCHAR".into(),
                    length: "255".into(),
                    is_null: true,
                    is_primary: false,
                    is_auto_increment: false,
                });
                if let Some(app) = weak.upgrade() {
                    app.set_create_table_columns(ModelRc::from(Rc::new(VecModel::from(cols.clone()))));
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_remove_create_column(move |idx| {
                let mut cols = ctrl.create_columns.lock().unwrap();
                if cols.len() > 1 && (idx as usize) < cols.len() {
                    cols.remove(idx as usize);
                    if let Some(app) = weak.upgrade() {
                        app.set_create_table_columns(ModelRc::from(Rc::new(VecModel::from(cols.clone()))));
                    }
                }
            });
        }

        {
            let ctrl = self.clone();
            app.on_update_create_col_name(move |idx, val| {
                let mut cols = ctrl.create_columns.lock().unwrap();
                if let Some(col) = cols.get_mut(idx as usize) {
                    col.name = val;
                }
            });
        }

        {
            let ctrl = self.clone();
            app.on_update_create_col_type(move |idx, val| {
                let mut cols = ctrl.create_columns.lock().unwrap();
                if let Some(col) = cols.get_mut(idx as usize) {
                    col.col_type = val;
                }
            });
        }

        {
            let ctrl = self.clone();
            app.on_update_create_col_length(move |idx, val| {
                let mut cols = ctrl.create_columns.lock().unwrap();
                if let Some(col) = cols.get_mut(idx as usize) {
                    col.length = val;
                }
            });
        }

        {
            let ctrl = self.clone();
            app.on_update_create_col_null(move |idx, val| {
                let mut cols = ctrl.create_columns.lock().unwrap();
                if let Some(col) = cols.get_mut(idx as usize) {
                    col.is_null = val;
                }
            });
        }

        {
            let ctrl = self.clone();
            app.on_update_create_col_primary(move |idx, val| {
                let mut cols = ctrl.create_columns.lock().unwrap();
                if let Some(col) = cols.get_mut(idx as usize) {
                    col.is_primary = val;
                }
            });
        }

        {
            let ctrl = self.clone();
            app.on_update_create_col_auto(move |idx, val| {
                let mut cols = ctrl.create_columns.lock().unwrap();
                if let Some(col) = cols.get_mut(idx as usize) {
                    col.is_auto_increment = val;
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_create_table(move || {
                let tbl_name = if let Some(app) = weak.upgrade() {
                    app.get_create_table_name().to_string()
                } else {
                    return;
                };
                let weak = weak.clone();
                let ctrl = ctrl.clone();
                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let cols_def = ctrl.create_columns.lock().unwrap().clone();

                    if tbl_name.is_empty() || cols_def.is_empty() {
                        return;
                    }

                    let sanitized_db = match sanitize_identifier(&db) {
                        Ok(s) => s,
                        Err(e) => {
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_create_table_error_message(e.into());
                            });
                            return;
                        }
                    };

                    let sanitized_tbl = match sanitize_identifier(&tbl_name) {
                        Ok(s) => s,
                        Err(e) => {
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_create_table_error_message(e.into());
                            });
                            return;
                        }
                    };

                    let mut col_defs_sql = Vec::new();
                    let mut pk_cols = Vec::new();

                    for c in cols_def {
                        let cname = c.name.to_string();
                        if cname.trim().is_empty() { continue; }
                        let sanitized_cname = match sanitize_identifier(&cname) {
                            Ok(s) => s,
                            Err(e) => {
                                let _ = weak.upgrade_in_event_loop(move |app| {
                                    app.set_create_table_error_message(e.into());
                                });
                                return;
                            }
                        };
                        let ctype = c.col_type.to_string();
                        let clen = c.length.to_string();
                        let type_str = if !clen.trim().is_empty() {
                            match validate_column_length(&clen) {
                                Ok(val) => format!("{}({})", ctype, val),
                                Err(e) => {
                                    let _ = weak.upgrade_in_event_loop(move |app| {
                                        app.set_create_table_error_message(e.into());
                                    });
                                    return;
                                }
                            }
                        } else {
                            ctype
                        };
                        let null_str = if c.is_null { "NULL" } else { "NOT NULL" };
                        let auto_str = if c.is_auto_increment { "AUTO_INCREMENT" } else { "" };
                        col_defs_sql.push(format!("{} {} {} {}", sanitized_cname, type_str, null_str, auto_str).trim().to_string());
                        if c.is_primary {
                            pk_cols.push(sanitized_cname);
                        }
                    }

                    if col_defs_sql.is_empty() {
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            app.set_create_table_error_message("At least one valid column definition is required.".into());
                        });
                        return;
                    }

                    if !pk_cols.is_empty() {
                        col_defs_sql.push(format!("PRIMARY KEY ({})", pk_cols.join(", ")));
                    }

                    let create_sql = format!(
                        "CREATE TABLE {}.{} (\n  {}\n) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
                        sanitized_db, sanitized_tbl, col_defs_sql.join(",\n  ")
                    );

                    let res = query::execute_query(&ctrl.state, Some(&db), &create_sql, false).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        match res {
                            Ok(_) => {
                                app.set_active_view("db_overview".into());
                                ctrl.refresh_databases(&app);
                                ctrl.refresh_db_overview(&app);
                            }
                            Err(e) => {
                                app.set_create_table_error_message(e.into());
                            }
                        }
                    });
                });
            });
        }

        {
            let weak = weak.clone();
            app.on_builder_generate(move || {
                if let Some(app) = weak.upgrade() {
                    let tbl = app.get_selected_table().to_string();
                    if tbl.is_empty() {
                        app.set_builder_generated_sql("".into());
                        return;
                    }
                    let cond_col = app.get_builder_cond_col().to_string();
                    let cond_op = app.get_builder_cond_op().to_string();
                    let cond_val = app.get_builder_cond_val().to_string();
                    let limit_str = app.get_builder_limit_str().to_string();
                    let limit = limit_str.trim().parse::<i64>().unwrap_or(100);

                    match build_query_builder_select(
                        &tbl,
                        &[],
                        if cond_col.trim().is_empty() { None } else { Some(&cond_col) },
                        Some(&cond_op),
                        if cond_val.is_empty() { None } else { Some(&cond_val) },
                        None,
                        Some(limit),
                    ) {
                        Ok(sql) => app.set_builder_generated_sql(sql.into()),
                        Err(e) => app.set_builder_generated_sql(format!("-- Error: {}", e).into()),
                    }
                }
            });
        }

        self.sync_active_tab_to_ui(app);
    }

    // Helper refresh functions
    pub fn refresh_databases(&self, app: &AppWindow) {
        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let dbs = database::list_databases(&state).await.unwrap_or_default();
            let _ = weak.upgrade_in_event_loop(move |app| {
                let db_items: Vec<DatabaseItem> = dbs.into_iter().map(|name| {
                    DatabaseItem {
                        name: name.into(),
                        expanded: false,
                        tables: ModelRc::default(),
                        views: ModelRc::default(),
                        procedures: ModelRc::default(),
                        functions: ModelRc::default(),
                    }
                }).collect();

                app.set_databases(ModelRc::from(Rc::new(VecModel::from(db_items))));
            });
        });
    }

    pub fn refresh_server_overview(&self, app: &AppWindow) {
        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let (status, proc_data) = server::server_status(&state).await.unwrap_or_default();
            let dbs = database::list_databases(&state).await.unwrap_or_default();

            let _ = weak.upgrade_in_event_loop(move |app| {
                app.set_server_uptime(format_uptime(status.uptime).into());
                app.set_server_threads_connected(status.threads_connected.to_string().into());
                app.set_server_threads_running(status.threads_running.to_string().into());
                app.set_server_total_queries(status.queries.to_string().into());
                app.set_server_slow_queries(status.slow_queries.to_string().into());

                let db_summaries: Vec<DbSummaryItem> = dbs.into_iter().map(|db| {
                    DbSummaryItem {
                        name: db.into(),
                        table_count: 0,
                    }
                }).collect();
                app.set_server_databases(ModelRc::from(Rc::new(VecModel::from(db_summaries))));

                let proc_items: Vec<ProcessItem> = proc_data.into_iter().map(|p| {
                    ProcessItem {
                        id: p.get("Id").map(|v| v.to_string()).unwrap_or_default().into(),
                        user: p.get("User").and_then(|v| v.as_str()).unwrap_or_default().into(),
                        host: p.get("Host").and_then(|v| v.as_str()).unwrap_or_default().into(),
                        db: p.get("db").and_then(|v| v.as_str()).unwrap_or_default().into(),
                        command: p.get("Command").and_then(|v| v.as_str()).unwrap_or_default().into(),
                        time: p.get("Time").map(|v| v.to_string()).unwrap_or_default().into(),
                        state: p.get("State").and_then(|v| v.as_str()).unwrap_or_default().into(),
                        info: p.get("Info").and_then(|v| v.as_str()).unwrap_or_default().into(),
                    }
                }).collect();
                app.set_server_processes(ModelRc::from(Rc::new(VecModel::from(proc_items))));
            });
        });
    }

    pub fn refresh_db_overview(&self, app: &AppWindow) {
        let db = self.current_db.lock().unwrap().clone();
        if db.is_empty() {
            return;
        }
        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let stats = database::get_database_stats(&state, &db).await.unwrap_or_default();
            let _ = weak.upgrade_in_event_loop(move |app| {
                let total_tables = stats.len();
                let total_rows: i64 = stats.iter().map(|s| s.row_count).sum();
                let total_size: i64 = stats.iter().map(|s| s.data_size).sum();

                app.set_db_total_tables(total_tables.to_string().into());
                app.set_db_total_rows(total_rows.to_string().into());
                app.set_db_total_size(format_size(total_size).into());

                let tbl_items: Vec<TableStatItem> = stats.into_iter().map(|t| {
                    TableStatItem {
                        name: t.table_name.into(),
                        row_count: t.row_count.to_string().into(),
                        data_size: format_size(t.data_size).into(),
                        engine: t.engine.into(),
                        collation: t.collation.into(),
                    }
                }).collect();
                app.set_db_tables(ModelRc::from(Rc::new(VecModel::from(tbl_items))));
            });
        });
    }

    pub fn refresh_table_data(&self, app: &AppWindow) {
        let db = self.current_db.lock().unwrap().clone();
        let tbl = self.current_table.lock().unwrap().clone();
        if db.is_empty() || tbl.is_empty() {
            return;
        }

        let limit = *self.table_limit.lock().unwrap();
        let offset = *self.table_offset.lock().unwrap();
        let sort_col = self.table_sort_col.lock().unwrap().clone();
        let sort_order = self.table_sort_order.lock().unwrap().clone();

        let state = self.state.clone();
        let raw_cache = self.raw_table_rows.clone();
        let current_cols = self.current_table_columns.clone();
        let weak = app.as_weak();

        app.set_table_loading(true);
        app.set_table_error_message("".into());

        tokio::spawn(async move {
            let (data_res, structure_res) = tokio::join!(
                data::get_data(
                    &state,
                    &db,
                    &tbl,
                    limit,
                    offset,
                    sort_col.as_deref(),
                    Some(sort_order.as_str()),
                ),
                table::get_structure(&state, &db, &tbl)
            );

            let _ = weak.upgrade_in_event_loop(move |app| {
                app.set_table_loading(false);

                let cols = structure_res.unwrap_or_default();
                let pri_cols: Vec<_> = cols.iter().filter(|c| c.is_primary()).collect();
                let has_usable_pk = !pri_cols.is_empty() && !pri_cols.iter().any(|c| c.has_unsupported_pk_type());
                *current_cols.lock().unwrap() = cols;
                app.set_table_has_primary_key(has_usable_pk);

                match data_res {
                    Ok(data_resp) => {
                        *raw_cache.lock().unwrap() = data_resp.data.clone();
                        app.set_table_total_rows(data_resp.pagination.total as i32);
                        let total_pages = if data_resp.pagination.total == 0 {
                            1
                        } else {
                            ((data_resp.pagination.total as f64) / (limit as f64)).ceil() as i32
                        };
                        let curr_page = (offset / limit) + 1;
                        app.set_table_current_page(curr_page as i32);
                        app.set_table_total_pages(total_pages);
                        app.set_table_limit(limit as i32);

                        let col_items: Vec<SharedString> = data_resp.columns.iter().map(|c| c.clone().into()).collect();
                        app.set_table_columns(ModelRc::from(Rc::new(VecModel::from(col_items))));

                        let mut row_items = Vec::new();
                        for (idx, row_val) in data_resp.data.iter().enumerate() {
                            let mut cells = Vec::new();
                            if let Some(obj) = row_val.as_object() {
                                for col in &data_resp.columns {
                                    let v_str = match obj.get(col) {
                                        Some(serde_json::Value::Null) => "NULL".to_string(),
                                        Some(serde_json::Value::String(s)) => s.clone(),
                                        Some(other) => other.to_string(),
                                        None => "NULL".to_string(),
                                    };
                                    cells.push(SharedString::from(v_str));
                                }
                            }
                            row_items.push(TableRowData {
                                index: (offset + idx as i64) as i32,
                                cells: ModelRc::from(Rc::new(VecModel::from(cells))),
                            });
                        }
                        app.set_table_rows(ModelRc::from(Rc::new(VecModel::from(row_items))));
                    }
                    Err(e) => {
                        app.set_table_error_message(e.into());
                    }
                }
            });
        });
    }

    pub fn refresh_structure(&self, app: &AppWindow) {
        let db = self.current_db.lock().unwrap().clone();
        let tbl = self.current_table.lock().unwrap().clone();
        if db.is_empty() || tbl.is_empty() {
            return;
        }

        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let cols = table::get_structure(&state, &db, &tbl).await.unwrap_or_default();
            let _ = weak.upgrade_in_event_loop(move |app| {
                let items: Vec<ColumnInfoItem> = cols.into_iter().map(|c| {
                    ColumnInfoItem {
                        field: c.field.into(),
                        col_type: c.r#type.into(),
                        is_null: c.null.into(),
                        key_type: c.key.into(),
                        default_val: c.default.unwrap_or_default().into(),
                        extra: c.extra.into(),
                    }
                }).collect();
                app.set_structure_columns(ModelRc::from(Rc::new(VecModel::from(items))));
            });
        });
    }

    pub fn refresh_users(&self, app: &AppWindow) {
        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let res = server::list_users(&state).await;
            let _ = weak.upgrade_in_event_loop(move |app| {
                match res {
                    Ok((users_list, note)) => {
                        app.set_user_note(note.unwrap_or_default().into());
                        app.set_user_error_message("".into());
                        let items: Vec<UserItem> = users_list.into_iter().map(|u| {
                            UserItem {
                                user: u.user.into(),
                                host: u.host.into(),
                                account_locked: u.account_locked.into(),
                            }
                        }).collect();
                        app.set_user_items(ModelRc::from(Rc::new(VecModel::from(items))));
                    }
                    Err(e) => {
                        app.set_user_error_message(e.into());
                    }
                }
            });
        });
    }

    pub fn refresh_performance(&self, app: &AppWindow) {
        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let metrics_res = server::server_metrics(&state).await;
            let _ = weak.upgrade_in_event_loop(move |app| {
                if let Ok(m) = metrics_res {
                    app.set_perf_questions(m.metrics.questions.to_string().into());
                    app.set_perf_threads_connected(m.metrics.threads_connected.to_string().into());
                    app.set_perf_threads_running(m.metrics.threads_running.to_string().into());
                    app.set_perf_bytes_received(format!("{:.1} KB", m.metrics.bytes_received as f64 / 1024.0).into());
                    app.set_perf_bytes_sent(format!("{:.1} KB", m.metrics.bytes_sent as f64 / 1024.0).into());
                    app.set_perf_innodb_buffer(format!("{} / {} pages", m.metrics.innodb_total - m.metrics.innodb_free, m.metrics.innodb_total).into());
                    app.set_perf_slow_queries(m.metrics.slow_queries.to_string().into());
                }
            });
        });
    }

    pub fn refresh_slow_log(&self, app: &AppWindow) {
        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let logs_res = server::server_slow_log(&state).await;
            let _ = weak.upgrade_in_event_loop(move |app| {
                match logs_res {
                    Ok(logs) => {
                        app.set_slow_log_error_message("".into());
                        let get_str = |obj: &serde_json::Value, key: &str| -> String {
                            obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
                                .or_else(|| obj.get(key).map(|v| v.to_string()))
                                .unwrap_or_default()
                        };

                        let items: Vec<SlowLogItem> = logs.into_iter().map(|l| {
                            SlowLogItem {
                                start_time: get_str(&l, "start_time").into(),
                                user_host: get_str(&l, "user_host").into(),
                                query_time: get_str(&l, "query_time").into(),
                                lock_time: get_str(&l, "lock_time").into(),
                                rows_sent: get_str(&l, "rows_sent").into(),
                                rows_examined: get_str(&l, "rows_examined").into(),
                                sql_text: get_str(&l, "sql_text").into(),
                            }
                        }).collect();
                        app.set_slow_log_items(ModelRc::from(Rc::new(VecModel::from(items))));
                    }
                    Err(e) => {
                        app.set_slow_log_error_message(e.into());
                    }
                }
            });
        });
    }

    pub fn load_history(&self, app: &AppWindow) {
        let items = self.history_mgr.load();
        let history_items: Vec<HistoryDisplayItem> = items.into_iter().map(|item| {
            let dt = chrono::DateTime::from_timestamp_millis(item.timestamp)
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();

            HistoryDisplayItem {
                query: item.query.into(),
                time_str: dt.into(),
                database: item.database.unwrap_or_default().into(),
                is_favorite: item.is_favorite,
            }
        }).collect();

        app.set_history_items(ModelRc::from(Rc::new(VecModel::from(history_items))));
    }

    pub fn prepare_insert_view(&self, app: &AppWindow) {
        let db = self.current_db.lock().unwrap().clone();
        let tbl = self.current_table.lock().unwrap().clone();
        if db.is_empty() || tbl.is_empty() {
            return;
        }

        let state = self.state.clone();
        let insert_cache = self.insert_fields.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let desc = table::get_structure(&state, &db, &tbl).await.unwrap_or_default();
            let mut fields = Vec::new();

            for col in desc {
                if !col.extra.contains("auto_increment") {
                    fields.push(InsertFieldItem {
                        name: col.field.into(),
                        col_type: col.r#type.into(),
                        value: col.default.unwrap_or_default().into(),
                        is_null: col.null == "YES",
                    });
                }
            }

            *insert_cache.lock().unwrap() = fields.clone();
            let _ = weak.upgrade_in_event_loop(move |app| {
                app.set_insert_fields(ModelRc::from(Rc::new(VecModel::from(fields))));
            });
        });
    }

    pub fn prepare_mock_data_view(&self, app: &AppWindow) {
        let db = self.current_db.lock().unwrap().clone();
        let tbl = self.current_table.lock().unwrap().clone();
        if db.is_empty() || tbl.is_empty() {
            return;
        }

        let state = self.state.clone();
        let mock_cache = self.mock_blueprint.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let desc = table::get_structure(&state, &db, &tbl).await.unwrap_or_default();
            let mut bp = Vec::new();

            for col in desc {
                if !col.extra.contains("auto_increment") {
                    let gen_type = if col.r#type.to_lowercase().contains("int") {
                        "integer"
                    } else if col.r#type.to_lowercase().contains("date") {
                        "date"
                    } else if col.field.to_lowercase().contains("email") {
                        "email"
                    } else if col.field.to_lowercase().contains("name") {
                        "name"
                    } else if col.field.to_lowercase().contains("phone") {
                        "phone"
                    } else {
                        "text"
                    };

                    bp.push(MockColumnBlueprint {
                        column_name: col.field.into(),
                        generator_type: gen_type.into(),
                    });
                }
            }

            *mock_cache.lock().unwrap() = bp.clone();
            let _ = weak.upgrade_in_event_loop(move |app| {
                app.set_mock_blueprint(ModelRc::from(Rc::new(VecModel::from(bp))));
            });
        });
    }

    pub fn refresh_query_builder(&self, app: &AppWindow) {
        let db = self.current_db.lock().unwrap().clone();
        if db.is_empty() {
            return;
        }
        let state = self.state.clone();
        let weak = app.as_weak();

        tokio::spawn(async move {
            let tbls = table::list_tables(&state, &db).await.unwrap_or_default();
            let _ = weak.upgrade_in_event_loop(move |app| {
                let items: Vec<SharedString> = tbls.into_iter().map(Into::into).collect();
                app.set_builder_tables(ModelRc::from(Rc::new(VecModel::from(items))));
            });
        });
    }

    pub fn apply_tab_result_to_ui(&self, app: &AppWindow, res: Option<&SqlResultData>) {
        app.set_sql_loading(false);
        if let Some(r) = res {
            app.set_sql_execution_time_ms(r.execution_time_ms);
            app.set_sql_affected_rows(r.affected_rows);
            app.set_sql_message(r.message.clone().into());
            app.set_sql_error_message(r.error_message.clone().into());

            let col_items: Vec<SharedString> = r.columns.iter().map(|c| c.clone().into()).collect();
            app.set_sql_result_columns(ModelRc::from(Rc::new(VecModel::from(col_items))));

            let row_items: Vec<SqlResultRow> = r
                .rows
                .iter()
                .map(|row_cells| {
                    let cells: Vec<SharedString> =
                        row_cells.iter().map(|cell| cell.clone().into()).collect();
                    SqlResultRow {
                        cells: ModelRc::from(Rc::new(VecModel::from(cells))),
                    }
                })
                .collect();
            app.set_sql_result_rows(ModelRc::from(Rc::new(VecModel::from(row_items))));
        } else {
            app.set_sql_execution_time_ms(0);
            app.set_sql_affected_rows(-1);
            app.set_sql_message("".into());
            app.set_sql_error_message("".into());
            app.set_sql_result_columns(ModelRc::default());
            app.set_sql_result_rows(ModelRc::default());
        }
    }

    pub fn sync_tab_headers(&self, app: &AppWindow) {
        let (headers, active_id, can_add) = {
            let mgr = self.tab_mgr.lock().unwrap();
            (mgr.get_headers(), mgr.active_tab_id(), mgr.can_add_tab())
        };

        let tab_items: Vec<SqlTabItem> = headers
            .into_iter()
            .map(|h| SqlTabItem {
                id: h.id as i32,
                title: h.title.into(),
                is_running: h.is_running,
                has_unsaved_text: h.has_unsaved_text,
            })
            .collect();

        app.set_sql_tabs(ModelRc::from(Rc::new(VecModel::from(tab_items))));
        app.set_sql_active_tab_id(active_id as i32);
        app.set_sql_can_add_tab(can_add);
    }

    pub fn sync_active_tab_to_ui(&self, app: &AppWindow) {
        let tab = {
            let mgr = self.tab_mgr.lock().unwrap();
            mgr.get_active_tab().cloned()
        };

        if let Some(tab) = tab {
            app.set_sql_query_text(tab.query.into());
            app.set_sql_loading(tab.is_running);
            self.apply_tab_result_to_ui(app, tab.result.as_ref());
            app.set_sql_explain_open(tab.explain.is_open);
            app.set_sql_explain_summary(tab.explain.summary.into());
            let lines: Vec<SharedString> = tab.explain.lines.into_iter().map(Into::into).collect();
            app.set_sql_explain_lines(Rc::new(VecModel::from(lines)).into());
            let warnings: Vec<SharedString> =
                tab.explain.warnings.into_iter().map(Into::into).collect();
            app.set_sql_explain_warnings(Rc::new(VecModel::from(warnings)).into());
        }
        self.sync_tab_headers(app);
    }

    pub fn execute_sql_import(self: Arc<Self>, weak: slint::Weak<AppWindow>, db: String, sql: String) {
        tokio::spawn(async move {
            self.perform_sql_import(weak, db, sql).await;
        });
    }

    pub async fn perform_sql_import(self: Arc<Self>, weak: slint::Weak<AppWindow>, db: String, sql: String) {
        let _ = weak.upgrade_in_event_loop(|app| {
            app.set_import_loading(true);
            app.set_import_error_message("".into());
            app.set_import_message("Executing SQL import...".into());
            app.set_import_errors(ModelRc::from(Rc::new(VecModel::from(vec![]))));
        });

        let res = maintenance::import_sql(&self.state, &db, &sql).await;
        let _ = weak.upgrade_in_event_loop(move |app| {
            app.set_import_loading(false);
            match res {
                Ok((count, errors)) => {
                    let err_items: Vec<SharedString> = errors.iter().map(Into::into).collect();
                    app.set_import_errors(ModelRc::from(Rc::new(VecModel::from(err_items))));
                    if errors.is_empty() {
                        app.set_import_message(format!("Import completed: {} statements executed successfully.", count).into());
                        app.set_import_error_message("".into());
                    } else {
                        app.set_import_message(format!("Import finished: {} succeeded, {} failed.", count, errors.len()).into());
                        app.set_import_error_message(format!("{} statement(s) failed during execution.", errors.len()).into());
                    }
                    self.refresh_databases(&app);
                    self.refresh_db_overview(&app);
                }
                Err(e) => {
                    app.set_import_error_message(e.into());
                    app.set_import_message("".into());
                }
            }
        });
    }
}
