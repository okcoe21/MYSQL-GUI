use std::rc::Rc;
use std::sync::{Arc, Mutex};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use serde_json::Value;

use crate::{
    AppWindow, ColumnInfoItem, CreateTableColumnItem, DatabaseItem, DbSummaryItem, DiagramTableItem,
    HistoryDisplayItem, InsertFieldItem, MockColumnBlueprint, ProcessItem, RelationItem, SlowLogItem,
    SqlResultRow, TableRowData, TableStatItem, UserItem,
};
use crate::state::SharedState;
use crate::db::{
    auth, database, table, data, query, server, objects, maintenance,
    history::HistoryManager,
    sanitize::{build_query_builder_select, sanitize_identifier, validate_column_length},
};

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
    current_db: Arc<Mutex<String>>,
    current_table: Arc<Mutex<String>>,
    table_limit: Arc<Mutex<i64>>,
    table_offset: Arc<Mutex<i64>>,
    table_sort_col: Arc<Mutex<Option<String>>>,
    table_sort_order: Arc<Mutex<String>>,
    create_columns: Arc<Mutex<Vec<CreateTableColumnItem>>>,
    insert_fields: Arc<Mutex<Vec<InsertFieldItem>>>,
    mock_blueprint: Arc<Mutex<Vec<MockColumnBlueprint>>>,
    export_format: Arc<Mutex<String>>,
    export_structure: Arc<Mutex<bool>>,
    export_data: Arc<Mutex<bool>>,
    pending_destructive_query: Arc<Mutex<String>>,
    pending_dialog_action: Arc<Mutex<(String, String)>>,
    raw_table_rows: Arc<Mutex<Vec<Value>>>,
}

impl AppController {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            history_mgr: Arc::new(HistoryManager::new()),
            current_db: Arc::new(Mutex::new(String::new())),
            current_table: Arc::new(Mutex::new(String::new())),
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
            mock_blueprint: Arc::new(Mutex::new(Vec::new())),
            export_format: Arc::new(Mutex::new("sql".to_string())),
            export_structure: Arc::new(Mutex::new(true)),
            export_data: Arc::new(Mutex::new(true)),
            pending_destructive_query: Arc::new(Mutex::new(String::new())),
            pending_dialog_action: Arc::new(Mutex::new((String::new(), String::new()))),
            raw_table_rows: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn setup_callbacks(self: Arc<Self>, app: &AppWindow) {
        let weak = app.as_weak();

        // 1. Authentication
        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_request_login(move |host, port, user, pwd| {
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
                    let res = auth::login(&ctrl.state, &host, &port, &user, pwd.as_deref()).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_login_loading(false);
                        match res {
                            Ok(_) => {
                                app.set_login_error_message("".into());
                                app.set_is_logged_in(true);
                                app.set_server_name(format!("{}:{}", host, port).into());
                                app.set_active_view("server_overview".into());
                                ctrl.refresh_databases(&app);
                                ctrl.refresh_server_overview(&app);
                            }
                            Err(e) => {
                                app.set_is_logged_in(false);
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
                        app.set_is_logged_in(false);
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
                                ctrl.refresh_databases(&app);
                                ctrl.refresh_server_overview(&app);
                            });
                        }
                        "drop_table" => {
                            let db = ctrl.current_db.lock().unwrap().clone();
                            let _ = table::drop_table(&ctrl.state, &db, &target).await;
                            let _ = weak.upgrade_in_event_loop(move |app| {
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
                        "destructive_query" => {
                            let sql = ctrl.pending_destructive_query.lock().unwrap().clone();
                            let db = ctrl.current_db.lock().unwrap().clone();
                            let db_opt = if db.is_empty() { None } else { Some(db.as_str()) };
                            let res = query::execute_query(&ctrl.state, db_opt, &sql, true).await;
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                ctrl.apply_query_result(&app, res);
                            });
                        }
                        _ => {}
                    }
                });
            });
        }

        {
            let weak = weak.clone();
            app.on_cancel_dialog_action(move || {
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

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let db_opt = if db.is_empty() { None } else { Some(db.as_str()) };

                    if crate::db::sanitize::is_destructive(&sql) {
                        *ctrl.pending_destructive_query.lock().unwrap() = sql.clone();
                        *ctrl.pending_dialog_action.lock().unwrap() = ("destructive_query".to_string(), String::new());
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            app.set_dialog_title("DESTRUCTIVE QUERY".into());
                            app.set_dialog_message("This query contains DROP, DELETE, TRUNCATE, or ALTER operations. Proceed?".into());
                            app.set_dialog_open(true);
                        });
                        return;
                    }

                    ctrl.history_mgr.add(&sql, db_opt);
                    let res = query::execute_query(&ctrl.state, db_opt, &sql, false).await;

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        ctrl.apply_query_result(&app, res);
                    });
                });
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
                let ctrl = ctrl.clone();
                let weak = weak.clone();
                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let tbl = ctrl.current_table.lock().unwrap().clone();
                    let target_row = {
                        let raw_rows = ctrl.raw_table_rows.lock().unwrap();
                        raw_rows.get(row_idx as usize).cloned()
                    };

                    if let Some(row_val) = target_row {
                        let _ = data::delete_row(&ctrl.state, &db, &tbl, &row_val).await;
                    }

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        ctrl.refresh_table_data(&app);
                    });
                });
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

                    let mut map = serde_json::Map::new();
                    for f in fields {
                        let val_str = f.value.to_string();
                        if val_str.is_empty() {
                            map.insert(f.name.to_string(), Value::Null);
                        } else {
                            map.insert(f.name.to_string(), Value::String(val_str));
                        }
                    }

                    let res = data::insert_row(&ctrl.state, &db, &tbl, &Value::Object(map)).await;
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
            app.on_set_export_format(move |fmt| {
                *ctrl.export_format.lock().unwrap() = fmt.to_string();
            });
        }

        {
            let ctrl = self.clone();
            app.on_toggle_export_structure(move |val| {
                *ctrl.export_structure.lock().unwrap() = val;
            });
        }

        {
            let ctrl = self.clone();
            app.on_toggle_export_data(move |val| {
                *ctrl.export_data.lock().unwrap() = val;
            });
        }

        {
            let ctrl = self.clone();
            app.on_submit_export(move || {
                let ctrl = ctrl.clone();

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let fmt = ctrl.export_format.lock().unwrap().clone();
                    let inc_struct = *ctrl.export_structure.lock().unwrap();
                    let inc_data = *ctrl.export_data.lock().unwrap();

                    let filename = format!("{}_dump.{}", db, fmt);
                    if let Some(path) = rfd::FileDialog::new().set_file_name(&filename).save_file() {
                        if let Ok(content) = maintenance::export_database(&ctrl.state, &db, &fmt, inc_struct, inc_data).await {
                            let _ = std::fs::write(path, content);
                        }
                    }
                });
            });
        }

        // 14. Import callbacks
        {
            let weak = weak.clone();
            app.on_choose_import_file(move || {
                if let Some(path) = rfd::FileDialog::new().add_filter("SQL", &["sql"]).pick_file() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if let Some(app) = weak.upgrade() {
                            app.set_import_sql_content(content.into());
                        }
                    }
                }
            });
        }

        {
            let weak = weak.clone();
            let ctrl = self.clone();
            app.on_submit_import(move |sql| {
                let sql_str = sql.to_string();
                let ctrl = ctrl.clone();
                let weak = weak.clone();

                if let Some(app) = weak.upgrade() {
                    app.set_import_loading(true);
                    app.set_import_error_message("".into());
                    app.set_import_message("".into());
                }

                tokio::spawn(async move {
                    let db = ctrl.current_db.lock().unwrap().clone();
                    let res = maintenance::import_sql(&ctrl.state, &db, &sql_str).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_import_loading(false);
                        match res {
                            Ok((count, errors)) => {
                                let msg = if errors.is_empty() {
                                    format!("Import completed: {} statements executed successfully.", count)
                                } else {
                                    format!("Import finished with {} statements and {} errors.", count, errors.len())
                                };
                                app.set_import_message(msg.into());
                                ctrl.refresh_databases(&app);
                                ctrl.refresh_db_overview(&app);
                            }
                            Err(e) => {
                                app.set_import_error_message(e.into());
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
        let weak = app.as_weak();

        app.set_table_loading(true);
        app.set_table_error_message("".into());

        tokio::spawn(async move {
            let res = data::get_data(
                &state,
                &db,
                &tbl,
                limit,
                offset,
                sort_col.as_deref(),
                Some(sort_order.as_str()),
            ).await;

            let _ = weak.upgrade_in_event_loop(move |app| {
                app.set_table_loading(false);
                match res {
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

    pub fn apply_query_result(&self, app: &AppWindow, res: Result<crate::db::models::QueryResult, String>) {
        app.set_sql_loading(false);
        match res {
            Ok(qr) => {
                app.set_sql_error_message("".into());
                app.set_sql_execution_time_ms(qr.execution_time_ms as i32);
                app.set_sql_affected_rows(qr.affected_rows.unwrap_or(0) as i32);
                let msg = format!("Execution finished in {}ms. Affected rows: {}", qr.execution_time_ms, qr.affected_rows.unwrap_or(0));
                app.set_sql_message(msg.into());

                let col_items: Vec<SharedString> = qr.columns.iter().map(|c| c.clone().into()).collect();
                app.set_sql_result_columns(ModelRc::from(Rc::new(VecModel::from(col_items))));

                let mut rows = Vec::new();
                if let Some(data_rows) = qr.data {
                    for r in data_rows {
                        let mut cells = Vec::new();
                        if let Some(obj) = r.as_object() {
                            for col in &qr.columns {
                                let val_str = match obj.get(col) {
                                    Some(serde_json::Value::Null) => "NULL".to_string(),
                                    Some(serde_json::Value::String(s)) => s.clone(),
                                    Some(other) => other.to_string(),
                                    None => "NULL".to_string(),
                                };
                                cells.push(SharedString::from(val_str));
                            }
                        }
                        rows.push(SqlResultRow {
                            cells: ModelRc::from(Rc::new(VecModel::from(cells))),
                        });
                    }
                }
                app.set_sql_result_rows(ModelRc::from(Rc::new(VecModel::from(rows))));
            }
            Err(e) => {
                app.set_sql_error_message(e.into());
                app.set_sql_message("".into());
                app.set_sql_result_columns(ModelRc::default());
                app.set_sql_result_rows(ModelRc::default());
            }
        }
    }
}
