# Feature Status Audit Report

**Date:** October 3, 2026  
**Target:** `mysql-gui` v3.0.3 (Native Rust + Slint 1.18, SQLx 0.8, Tokio)  
**Auditor:** Feature Status Scanner  
**Scope:** Verification of roadmap checklist items V1–V3 against actual Rust backend (`src/`) and Slint declarative UI (`ui/`) implementations.

---

## 1. Executive Summary

This scan audited the 13 feature checklist items across the native Rust and Slint codebase. The audit reveals that core database exploration, query execution, mock data generation, routine fallback introspection, and terminal design tokens are fully implemented. However, several advanced capabilities from previous web/Tauri iterations (natural language to SQL, offline query explainer, multi-tab editor, saved query libraries, schema diff) are currently missing, and certain UI features (visual query builder condition edits, process auto-refresh, history copy/search) are only partially implemented or stubbed.

### Summary Metrics
* **Implemented:** 3 / 13 (23%)
* **Partial:** 3 / 13 (23%)
* **Stub:** 0 / 13 (0%)
* **Missing:** 7 / 13 (54%)

---

## 2. Feature Status Matrix

| # | Feature Item | Version Category | Status | Primary Evidence (File & Lines) | Summary Note |
|---|---|---|---|---|---|
| **1** | **MockDataView Table Auto-Refresh** | V1 | **Implemented** | [`src/app_controller.rs:720–724`](file:///home/coes/Projects/MYSQL%20GUI/src/app_controller.rs#L720-L724) | On generation completion, switches view to `"browse"` and executes `ctrl.refresh_table_data(&app)`. |
| **2** | **Routines Fallback Introspection** | V1 | **Implemented** | [`src/db/objects.rs:23–48`](file:///home/coes/Projects/MYSQL%20GUI/src/db/objects.rs#L23-L48) | If `INFORMATION_SCHEMA.ROUTINES` returns empty, executes fallback `SHOW PROCEDURE STATUS WHERE Db = ?` and `SHOW FUNCTION STATUS WHERE Db = ?`. |
| **3** | **Terminal-Style Design System** | V1 | **Implemented** | [`ui/theme.slint:4, 16, 38`](file:///home/coes/Projects/MYSQL%20GUI/ui/theme.slint#L4-L38), [`ui/components/*.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/components/) | Uses `#0d0d0d` near-black base, `#00ff9d` terminal accent, `JetBrains Mono` font, and consistent 1px borders. |
| **4** | **SQL Autocomplete Popup** | V2 | **Partial** | [`src/db/objects.rs:82–119`](file:///home/coes/Projects/MYSQL%20GUI/src/db/objects.rs#L82-L119), [`ui/views/sql_editor.slint:106–111`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/sql_editor.slint#L106-L111) | Backend helper `get_schema_suggestions()` exists with `#[allow(dead_code)]`, but no popup/overlay or callback is wired in `sql_editor.slint`. |
| **5** | **Rule-Based Query Explainer** | V2 | **Missing** | Searched `src/` and `ui/` for `explainer`, `explain`, `rule` | Legacy TypeScript explainer (`lib/sqlExplainer.ts`) was removed in v3.0 rewrite; no Rust/Slint offline explainer exists. |
| **6** | **Natural Language to SQL (LLM)** | V2 | **Missing** | Searched `src/`, `ui/`, `Cargo.toml` for `llm`, `nlq`, `openai`, `anthropic`, `gemini`, `ollama` | Zero LLM dependencies or endpoints exist. Crate has no HTTP client (`reqwest`, `ureq`). Legacy `lib/llm.ts` was dropped. |
| **7** | **Multiple SQL Editor Tabs (Max 8)** | V3 | **Missing** | [`ui/views/sql_editor.slint:10–25`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/sql_editor.slint#L10-L25), [`ui/app.slint:517–529`](file:///home/coes/Projects/MYSQL%20GUI/ui/app.slint#L517-L529) | UI and controller manage only a single query string (`sql-query-text`). No tab model, vector, or multi-tab UI exists. |
| **8** | **Saved Queries Library** | V3 | **Missing** | [`src/db/history.rs:5–11`](file:///home/coes/Projects/MYSQL%20GUI/src/db/history.rs#L5-L11), [`ui/views/history_view.slint:102–120`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/history_view.slint#L102-L120) | Only a binary favorite toggle (`is_favorite`) exists on history records. Named queries, folders, tags, and snippets are missing. |
| **9** | **Per-Table CSV/JSON Export of Result Set** | V3 | **Missing** | [`ui/views/table_data.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/table_data.slint), [`ui/views/export_view.slint:6–20`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/export_view.slint#L6-L20) | Full database dump exists in `export_view.slint`, but `table_data.slint` and `sql_editor.slint` have no buttons to export current result sets. |
| **10** | **EXPLAIN Plan Viewer** | V3 | **Missing** | [`src/db/query.rs:156`](file:///home/coes/Projects/MYSQL%20GUI/src/db/query.rs#L156) | `EXPLAIN` can be executed as raw text in the console, but no visual tree viewer, execution graph, or dedicated view exists. |
| **11** | **Live Process Monitor & Kill Query** | V3 | **Partial** | [`src/db/server.rs:9–14`](file:///home/coes/Projects/MYSQL%20GUI/src/db/server.rs#L9-L14), [`ui/views/server_overview.slint:255–282`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/server_overview.slint#L255-L282) | Active process table is rendered from `SHOW FULL PROCESSLIST`. However, auto-refresh intervals and kill-query buttons are missing. |
| **12** | **Schema Diff / ALTER Migration** | V3 | **Missing** | Searched `src/` and `ui/` for `diff`, `migration`, `compare` | Zero schema comparison or ALTER script generation code exists. |
| **13** | **History Persistence, Search & Re-Run** | V3 | **Partial** | [`src/db/history.rs:23, 38`](file:///home/coes/Projects/MYSQL%20GUI/src/db/history.rs#L23-L38), [`ui/views/history_view.slint:143–147`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/history_view.slint#L143-L147), [`src/app_controller.rs:589–593`](file:///home/coes/Projects/MYSQL%20GUI/src/app_controller.rs#L589-L593) | Persistence to `history.json` and one-click re-run are implemented. Search text filtering and row copy-to-clipboard are missing. |

---

## 3. Detailed Feature Analysis

### V1 Features

#### 1. MockDataView Table Auto-Refresh
* **Status:** **Implemented**
* **Evidence:** [`src/app_controller.rs:706–726`](file:///home/coes/Projects/MYSQL%20GUI/src/app_controller.rs#L706-L726)
* **Details:**
  When the user triggers mock data generation, `app.on_submit_mock_data()` invokes `maintenance::generate_mock_data()`. Once the async generation succeeds, the event loop callback immediately executes:
  ```rust
  let _ = weak.upgrade_in_event_loop(move |app| {
      app.set_active_view("browse".into());
      ctrl.refresh_table_data(&app);
  });
  ```
  This transitions the UI to the table data view (`browse`) and triggers an automatic reload of table records from the database.

#### 2. Routines Listing MariaDB Fallback
* **Status:** **Implemented**
* **Evidence:** [`src/db/objects.rs:16–48`](file:///home/coes/Projects/MYSQL%20GUI/src/db/objects.rs#L16-L48)
* **Details:**
  In `objects::get_objects()`, queries to `INFORMATION_SCHEMA.ROUTINES` for both `PROCEDURE` and `FUNCTION` types include explicit fallback checks. When `procedures.is_empty()`, lines 24–30 query `SHOW PROCEDURE STATUS WHERE Db = ?`. When `functions.is_empty()`, lines 41–47 query `SHOW FUNCTION STATUS WHERE Db = ?`. The results are bound into `ObjectsSummary` and rendered in the sidebar routine tree via `app_controller.rs:217`.

#### 3. Terminal-Style Design System
* **Status:** **Implemented**
* **Evidence:** [`ui/theme.slint:4, 16, 38`](file:///home/coes/Projects/MYSQL%20GUI/ui/theme.slint#L4-L38)
* **Details:**
  `ui/theme.slint` enforces the exact specification:
  - Base background: `bg-base: is-dark ? #0d0d0d : #f8fafc;` (near-black `#0d0d0d`).
  - Terminal accent: `accent: is-dark ? #00ff9d : #0d9488;` (`#00ff9d` neon green).
  - Typography: `font-mono: "JetBrains Mono";` configured as primary monospace font.
  - Borders: 1px border width applied across all components (`ui/components/button.slint:12`, `card.slint:6`, `input.slint:13`, `dialog.slint:23`).

---

### V2 Features

#### 4. SQL Autocomplete
* **Status:** **Partial**
* **Evidence:** [`src/db/objects.rs:82–119`](file:///home/coes/Projects/MYSQL%20GUI/src/db/objects.rs#L82-L119), [`ui/views/sql_editor.slint:106–111`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/sql_editor.slint#L106-L111)
* **Details:**
  A backend introspection helper `get_schema_suggestions(&state, db)` exists in `src/db/objects.rs` that extracts table and column metadata into JSON schema maps. However, it is marked `#[allow(dead_code)]` and is not called anywhere in `src/app_controller.rs`. `ui/views/sql_editor.slint` contains only a standard `MultilineInputBox` with no suggestion list, popup trigger, or completion logic.

#### 5. Rule-Based Query Explainer
* **Status:** **Missing**
* **Evidence:** Searched codebase for `explainer`, `explain_query`, `sqlExplainer`
* **Details:**
  The legacy TypeScript offline explainer (`lib/sqlExplainer.ts`) was removed during the v3.0.0 rewrite. No rule-based AST inspection or English explanation engine has been created in Rust or Slint.

#### 6. Natural-Language to SQL (LLM Integration)
* **Status:** **Missing**
* **Evidence:** Searched `Cargo.toml`, `src/`, and `ui/` for `llm`, `nlq`, `openai`, `anthropic`, `gemini`, `ollama`
* **Details:**
  There are no LLM provider integrations, API client crates (`reqwest`, `ureq`), or natural language query input boxes anywhere in the repository.

---

### V3 Features

#### 7. Multiple SQL Editor Tabs
* **Status:** **Missing**
* **Evidence:** [`ui/views/sql_editor.slint:10–25`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/sql_editor.slint#L10-L25), [`ui/app.slint:517–529`](file:///home/coes/Projects/MYSQL%20GUI/ui/app.slint#L517-L529)
* **Details:**
  `ui/views/sql_editor.slint` only contains a single text input area (`sql_input := MultilineInputBox`). The parent `AppWindow` manages a single `sql-query-text` string. There is no tab header bar, no tab state vector, and no tab switching mechanism.

#### 8. Saved Queries Library
* **Status:** **Missing**
* **Evidence:** [`src/db/history.rs:5–11`](file:///home/coes/Projects/MYSQL%20GUI/src/db/history.rs#L5-L11), [`ui/views/history_view.slint:102–120`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/history_view.slint#L102-L120)
* **Details:**
  While queries in `history_view.slint` can be starred via `is_favorite: bool`, there is no dedicated saved query library supporting query naming, descriptions, folders, tags, or categorized storage.

#### 9. Per-Table CSV/JSON Export of Current Result Set
* **Status:** **Missing**
* **Evidence:** [`ui/views/table_data.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/table_data.slint), [`ui/views/sql_editor.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/sql_editor.slint)
* **Details:**
  Whole-database dump export is supported via `ui/views/export_view.slint` and `src/db/maintenance.rs:114–220`. However, neither the paginated table viewer (`table_data.slint`) nor the SQL console results grid (`sql_editor.slint`) provides an export button to save the active tabular rows or filtered pagination view to CSV or JSON.

#### 10. EXPLAIN Plan Viewer
* **Status:** **Missing**
* **Evidence:** [`src/db/query.rs:156`](file:///home/coes/Projects/MYSQL%20GUI/src/db/query.rs#L156)
* **Details:**
  `EXPLAIN` queries can be executed manually in the SQL editor, returning standard tabular rows. However, there is no dedicated visual EXPLAIN plan visualizer, cost analyzer, or node graph view.

#### 11. Live Process Monitor
* **Status:** **Partial**
* **Evidence:** [`src/db/server.rs:9–14`](file:///home/coes/Projects/MYSQL%20GUI/src/db/server.rs#L9-L14), [`ui/views/server_overview.slint:215–282`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/server_overview.slint#L215-L282)
* **Details:**
  The active process list is queried via `SHOW FULL PROCESSLIST` and rendered in `server_overview.slint` with columns for ID, User, Host, DB, Command, Time, State, and Info. However:
  - There is no auto-refresh timer; the process list only reloads on manual navigation.
  - There is no "Kill Query" button or `KILL <id>` callback.

#### 12. Schema Diff / ALTER Migration Generator
* **Status:** **Missing**
* **Evidence:** Searched codebase for `diff`, `compare`, `migration`, `alter_script`
* **Details:**
  No schema comparison algorithms, database diffing tools, or ALTER migration generators exist in the codebase.

#### 13. History Search, Persistence & Re-Run
* **Status:** **Partial**
* **Evidence:** [`src/db/history.rs:23, 38`](file:///home/coes/Projects/MYSQL%20GUI/src/db/history.rs#L23-L38), [`ui/views/history_view.slint:143–147`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/history_view.slint#L143-L147), [`src/app_controller.rs:589–593`](file:///home/coes/Projects/MYSQL%20GUI/src/app_controller.rs#L589-L593)
* **Details:**
  - **Persistence:** **Implemented.** Persists queries to `~/.local/share/mysql-gui/history.json` on disk.
  - **Re-run:** **Implemented.** The `▶ Re-run` button on each item immediately sets the SQL console text and runs the query.
  - **Search:** **Missing.** There is no search input bar or query filtering in `history_view.slint`.
  - **Copy:** **Missing.** There is no per-row copy-to-clipboard button.

---

## 4. Dead UI & Stubbed Callback Catalog

The following UI components and backend helpers were audited for dead, stubbed, or unwired paths:

1. **Visual Query Builder Live Generation:**
   * **Status:** **FIXED (v3.0.4)**
   * **Implementation:** Pure function `build_query_builder_select()` in `src/db/sanitize.rs` with identifier quoting, operator whitelisting, string escaping, and limit clamping. Wired live to `on_builder_generate` and property bindings (`builder-cond-col`, `builder-cond-op`, `builder-cond-val`, `builder-limit-str`) in `src/app_controller.rs` and `ui/app.slint`.

2. **Login Error Message Banner Suppression:**
   * **Status:** **FIXED (v3.0.4)**
   * **Implementation:** In `src/app_controller.rs`, connection errors are sanitized and passed to `app.set_login_error_message()`. Connected to `AlertBanner` in `ui/views/login.slint` via `ui/app.slint`, clearing automatically on each new connect attempt.

3. **Dead Backend Autocomplete Helper:**
   * **Status:** **Unwired**
   * **Location:** [`src/db/objects.rs:82–119`](file:///home/coes/Projects/MYSQL%20GUI/src/db/objects.rs#L82-L119)
   * **Code:** `#[allow(dead_code)] pub async fn get_schema_suggestions(...)`
   * **Impact:** Schema column introspection logic was written but never connected to a Slint UI callback or autocomplete popup widget.

4. **Dead Inline Row Update Function:**
   * **Location:** [`src/db/data.rs:117–180`](file:///home/coes/Projects/MYSQL%20GUI/src/db/data.rs#L117-L180)
   * **Code:** `#[allow(dead_code)] pub async fn update_row(...)`
   * **Impact:** Backend logic for updating cells exists, but `ui/views/table_data.slint` contains no inline cell editing or update modals (only row deletion `✕`).

5. **Dead Standalone Table Creation Helper:**
   * **Location:** [`src/db/table.rs:20–72`](file:///home/coes/Projects/MYSQL%20GUI/src/db/table.rs#L20-L72)
   * **Code:** `#[allow(dead_code)] pub async fn create_table(...)`
   * **Impact:** Standalone helper is bypassed because `src/app_controller.rs:987–1090` formats and executes the `CREATE TABLE` query inline.
