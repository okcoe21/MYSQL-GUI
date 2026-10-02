# MySQL GUI — Project Migration & Status Report (v3.0.0)

**Date:** October 3, 2026  
**Target:** Claude Context / Development Handoff  
**Project:** `mysql-gui` (`/home/coes/Projects/MYSQL GUI`)  
**Status:** Successfully Migrated & Native Build Verified  

---

## 1. Executive Summary

The application underwent a complete architectural rewrite from a dual Next.js 15 / Tauri v2 hybrid into a **100% native Rust + Slint desktop application**. All legacy JavaScript, TypeScript, React, Next.js, and Tauri v2 code has been removed. The repository is now a clean, single-crate Rust project with instant compilation, zero Node.js/Chromium overhead, and hardware-accelerated rendering.

---

## 2. Tech Stack Migration Matrix

| Layer | Previous Stack (v1 / v2) | Current Stack (v3.0.0) |
|---|---|---|
| **Frontend Framework** | Next.js 15, React 18 | **Slint UI 1.18** |
| **Styling & Design System** | Tailwind CSS 3 | **Slint Declarative Styles (`ui/theme.slint`)** |
| **Desktop Shell** | Tauri v2 (WebKitGTK / WebView2) | **Native Slint Hardware Engine (OpenGL / Skia / Software)** |
| **Language** | TypeScript / JavaScript (Node.js) | **Pure Rust (2021 edition)** |
| **Database Driver** | MySQL2 / SQLx (Tauri backend) | **SQLx 0.8 (async MySQL with tokio-native-tls)** |
| **Async Runtime** | Node.js Event Loop + Tokio | **Tokio 1.x (multi-threaded)** |
| **Secret Storage** | OS Keyring (Tauri IPC) | **Native `keyring-rs` (SecretService / Keychain)** |
| **File Dialogs** | Tauri Dialog Plugin | **Native `rfd` (Rust File Dialogs)** |

---

## 3. Architecture & Codebase Map

### Core Rust Backend (`src/`)
* **[`src/main.rs`](file:///home/coes/Projects/MYSQL%20GUI/src/main.rs):** Entry point. Initializes Slint window (`AppWindow`), sets up Tokio runtime, and attaches controller.
* **[`src/app_controller.rs`](file:///home/coes/Projects/MYSQL%20GUI/src/app_controller.rs):** Central event coordinator. Binds all Slint callbacks (`connect`, `select_db`, `select_table`, `run_query`, etc.) to async SQLx database operations.
* **[`src/state.rs`](file:///home/coes/Projects/MYSQL%20GUI/src/state.rs):** Thread-safe application state (`Arc<Mutex<AppState>>`) holding the active SQLx connection pool and session metadata.
* **[`src/db/`](file:///home/coes/Projects/MYSQL%20GUI/src/db/):** Modular SQL operations:
  * `auth.rs`: Connection authentication, SSL modes, and keyring storage.
  * `database.rs`: Database listing, creation, and dropping.
  * `table.rs`: Table catalog, schema extraction, and column definitions.
  * `data.rs`: Paginated row queries, limit/offset handling, inline deletion.
  * `query.rs`: Arbitrary SQL execution, execution time measurement, tabular result conversion.
  * `server.rs`: Server metrics (uptime, threads, queries) and live process list (`SHOW FULL PROCESSLIST`).
  * `history.rs`: Persistent query execution log with favorite toggles.
  * `objects.rs`: Views, stored procedures, and triggers.
  * `sanitize.rs`: SQL identifier quoting and query sanitization.
  * `maintenance.rs`: Optimize, repair, and analyze table routines.

### Native Declarative UI (`ui/`)
* **[`ui/app.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/app.slint):** Main window component (`AppWindow`). Manages login screen vs main dashboard switching, dynamic navigation breadcrumbs/tabs, and confirmation dialogs.
* **[`ui/theme.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/theme.slint):** Design tokens (palettes, typography, border radii, dark/light mode toggle).
* **[`ui/components/`](file:///home/coes/Projects/MYSQL%20GUI/ui/components/):** Reusable widgets: `PrimaryButton`, `SecondaryButton`, `DangerButton`, `TabButton`, `PanelCard`, `StatCard`, `AlertBanner`, `CustomInput`, `MultilineInputBox`, `ConfirmDialog`.
* **[`ui/views/`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/):** 18 specialized view screens:
  * `login.slint`: Connection credentials portal with port/host/SSL options.
  * `sidebar.slint`: Collapsible tree of databases, tables, and views with live count badges.
  * `topbar.slint`: Breadcrumb navigation, server status badge, theme switcher, logout.
  * `server_overview.slint`: Server vitals, memory/buffer stats, active process list.
  * `db_overview.slint`: Database stats, table summaries, quick actions.
  * `table_data.slint`: High-performance paginated data grid, limit selector (25/50/100), column sorting.
  * `structure_view.slint`: Column type, collation, nullability, keys, and default values.
  * `sql_editor.slint`: Interactive query console, execution time, tabular output.
  * `history_view.slint`: Query execution log with favorites and search.
  * `query_builder.slint`: Visual SQL query generator.
  * `create_table_view.slint`: Column schema builder and DDL generator.
  * `diagram_view.slint`: ER foreign key schema relationship visualizer.
  * `export_view.slint` & `import_view.slint`: SQL/CSV export and import pipeline.
  * `user_management.slint`: Database user accounts and hosts.
  * `performance_view.slint` & `slow_log_view.slint`: Engine metrics and slow query logs.
  * `mock_data_view.slint`: Synthetic test data generator.

---

## 4. Key Bug Resolution: Slint Height & Clamping

* **Symptom:** In compiled Rust builds, the window content stopped stretching at ~520px height, leaving a large black void at the bottom when resized or maximized, even though `slint-viewer` rendered 100% height.
* **Root Cause Discovered in Compiler Output (`out/app.rs`):**
  Slint's Ahead-Of-Time (AOT) compiler (`slint-build`) calculates layout maximum height as `sum(children.max_height)` when all children have fixed heights. In `ui/views/sidebar.slint`, all database tree items had fixed heights (`28px`, `32px`, etc.), causing `Sidebar` to report a hard `max_height: 424px`. In `HorizontalLayout`, Slint calculates orthogonal maximum as $\min(\text{child}_1.\text{max}, \text{child}_2.\text{max})$, which clamped the entire dashboard row to 424px.
* **Resolution:**
  Inserted an unconstrained flexible spacer `Rectangle { }` **inside** `db_layout := VerticalLayout` in `sidebar.slint` (and inside `content` in `server_overview.slint`). This set `max_height: f32::MAX`, allowing the layout solver (`solve_box_layout`) to expand the window smoothly to 100% height without any artificial ceiling.

---

## 5. Current Project Status

- [x] **Compilation:** `cargo check` and `cargo build` pass with 0 errors.
- [x] **Artifact Cleanup:** Legacy Next.js / React / Tauri files completely removed.
- [x] **Versioning:** Bumped to `3.0.0` in `Cargo.toml`.
- [x] **Git Tracking:** Modern Rust `.gitignore` configured; untracked bloat removed.
- [x] **Documentation:** `README.md` updated with full native documentation.
- [x] **Offline Tooling:** `slint-viewer` (v1.18.1) installed and configured with `ui/preview_dashboard.json`.

---

## 6. Recommended Next Steps for Future Audit

1. **Security & Input Validation:** Review all SQL formatting in `src/db/` to ensure parameterized queries are consistently used and identifier escaping is strict.
2. **File Dialog Pipeline:** Connect `rfd` (Rust File Dialogs) to `export_view.slint` and `import_view.slint` for direct `.sql` and `.csv` disk operations.
3. **Table Data Mutations:** Implement inline cell update and new row insert modals in `table_data.slint`.
4. **Binary Packaging:** Set up GitHub Actions CI workflow to build release binaries for Linux (`.tar.gz`, `.deb`, AppImage), Windows (`.exe`), and macOS.
