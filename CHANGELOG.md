# Changelog

All notable changes to MySQL GUI are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

## [3.0.7] — 2026-10-03

### Added
- **Native Async File Dialogs (`rfd::AsyncFileDialog`):** Integrated native file dialogs for export save paths and SQL/CSV import file selection running asynchronously on Tokio without blocking the Slint UI thread.
- **Chunked Streaming Database Export (`export_database_stream`):** Implemented streaming export to `tokio::fs::File` with `BufWriter` in batches of 500 rows to prevent high memory consumption during large table dumps.
- **RFC 4180 CSV Export & Formula Injection Protection:** Formats CSV data strictly per RFC 4180 with escaped quotes (`""`), and neutralizes spreadsheet formula injection by prefixing cells starting with `=`, `+`, `-`, or `@` with `'`.
- **Transactional CSV Table Data Import (`import_csv_file`):** Hand-crafted RFC 4180 CSV parser with 50 MB size protection, introspects table metadata in `INFORMATION_SCHEMA.COLUMNS` to validate column count, binds values safely with parameterized queries (`.bind()`), and rolls back atomically on failure.
- **Safe Batch SQL Script Import:** Safe statement splitter recognizing quotes and comments, wrapping pure DML statements inside an explicit transaction, and triggering the native `ConfirmDialog` for destructive statements (`DROP`, `TRUNCATE`, `DELETE`).
- **Interactive Dual-Format Import View (`ui/views/import_view.slint`):** Added a tabbed format selector (`SQL Script` vs `CSV Table Data`), target table selector with quick-select chips for current database tables, loading states, and error reporting.
- **Maintenance Unit Test Suite:** Added 19 unit tests in `src/db/maintenance.rs` testing formula neutralization, CSV round-tripping, RFC 4180 parser edge cases, and safe SQL statement splitting (total 69 passing unit tests).

---

## [3.0.6] — 2026-10-03

### Added
- **Offline Rule-Based Query Explainer Engine (`src/explain.rs`):** Added a pure, zero-dependency Rust module translating SQL into plain English summaries, clause breakdowns, and risk warnings. Supports `SELECT` (columns, `FROM`, `JOIN`s, `WHERE`, `GROUP BY`, `HAVING`, `ORDER BY`, `LIMIT`), `INSERT`, `UPDATE`, `DELETE`, `CREATE TABLE`, `DROP`, `ALTER TABLE`, and `TRUNCATE`.
- **Operator Translation & Clause Parsing:** Translates SQL operators (`=`, `!=`/`<>`, `>`, `<`, `>=`, `<=`, `LIKE`, `IN`, `BETWEEN`, `IS NULL`, `IS NOT NULL`, `AND`/`OR`) into natural English. Handles parenthesized subqueries, string literals containing keywords, and multiple statements separated by semicolons.
- **Risk & Safeguard Detection:** Automatically flags unbounded `DELETE` or `UPDATE` queries without `WHERE`, `DROP` statements, `TRUNCATE` operations, and unconstrained `SELECT *` without `LIMIT`.
- **Collapsible Explanation UI Panel:** Added an interactive "💡 Explain" button in `ui/views/sql_editor.slint` toolbar with a collapsible details card featuring summary text, bulleted clause breakdowns, and styled `Theme.warning` alert cards.
- **Controller & View Wiring:** Connected `on_explain_sql_query` and `on_clear_sql_query` in `src/app_controller.rs` and bound properties in `ui/app.slint`.
- **Comprehensive Explainer Test Suite:** Added 19 new unit tests in `src/explain.rs` verifying clause extraction, risk warnings, lowercase inputs, string literals with keywords, comments (`--`, `#`, `/* ... */`), multiple statements, empty input, garbage resilience, and operator translation (50 total passing tests).

---

## [3.0.5] — 2026-10-03

### Added
- **Require SSL Switch:** Added an interactive "Require SSL" checkbox to `ui/views/login.slint` (off by default) that switches the connection mode from `Preferred` to `MySqlSslMode::Required`.
- **TopBar Active Encryption Indicator:** Added a live transport security badge to `ui/views/topbar.slint` displaying "SSL" in the accent color when encrypted or "NOT ENCRYPTED" in warning yellow when cleartext, with hover tooltip explanations.
- **SSL Error Mapping & Safeguards:** Implemented `map_login_error()` in `src/db/auth.rs` mapping SSL negotiation failures on non-TLS servers to clear guidance ("Server does not support SSL. Turn off Require SSL to connect unencrypted.") with zero credential leakage.
- **Transport Security Unit Tests:** Added unit tests in `src/db/auth.rs` verifying `resolve_ssl_mode()` and `map_login_error()` across unsupported TLS, preferred fallbacks, unrelated auth errors, and password scrubbing (31 total passing unit tests).

---

## [3.0.4] — 2026-10-03

### Fixed
- **Visual Query Builder Live Generation:** Implemented pure function `build_query_builder_select()` in `src/db/sanitize.rs` with identifier sanitization, operator whitelisting (`=`, `!=`, `<`, `>`, `<=`, `>=`, `LIKE`, `IS NULL`, `IS NOT NULL`), string escaping via `escape_sql_string()`, and limit clamping (1–1000). Wired `on_builder_generate` in `src/app_controller.rs` and bound condition properties (`builder-cond-col`, `builder-cond-op`, `builder-cond-val`, `builder-limit-str`) in `ui/app.slint` so edits update SQL live.
- **Login Error Propagation:** Wired error messages and loading state from `auth::login` to `AlertBanner` in `ui/views/login.slint` through `ui/app.slint`. Automatically clears error banners on each new connect attempt and scrubs raw credentials.

### Added
- **Query Builder Regression Tests:** Added 10 unit test cases in `src/db/sanitize.rs` covering normal select, value SQL injection attempt, column name SQL injection attempt, table name SQL injection attempt, empty filter list, limit boundary clamping, sort ordering, and unsupported operator rejection (total 26 passing tests).

---

## [3.0.3] — 2026-10-03

### Added
- **Unit Test Suite:** Added 16 automated unit and security regression tests in `src/db/sanitize.rs` and `src/db/history.rs` covering edge cases, injection attempts, and boundaries.
- **Identifier Boundary Enforcement:** Added 64-character MySQL identifier length limit validation to `sanitize_identifier()`.
- **Query Helpers:** Added `sanitize_sort_direction()` with fallback whitelisting and `clamp_limit_offset()` to prevent unbounded allocations.
- **Badge & Documentation:** Added passing test suite status badge and test instructions to `README.md`.

---

## [3.0.2] — 2026-10-03

### Security & Hardening
- **SEC-05 (Structured Connection Handling):** Replaced URL string parsing in `src/db/auth.rs` with `MySqlConnectOptions` (`.host()`, `.port()`, `.username()`, `.password()`, `.database()`), preventing connection string injection.
- **Credential Redaction:** Redacted raw passwords from connection failure error strings before propagating to UI banners.
- **SEC-06 (Transport Encryption Introspection):** Returned `is_encrypted: bool` on `ConnectionResult` derived via MySQL's `SHOW STATUS LIKE 'Ssl_cipher'`.
- **SEC-07 (Destructive Query Guard Hardening):** Rewrote `is_destructive` UX safeguard to strip leading comments (`--`, `#`, `/* ... */`) and whitespace, applying word-boundary checks for `DROP`, `TRUNCATE`, `DELETE`, `ALTER`, `GRANT`, `REVOKE`, and `UPDATE` without `WHERE`.
- **SEC-09 (Query Timeout Protection):** Wrapped all SQL execution paths in `tokio::time::timeout` with a 60-second default (`DEFAULT_QUERY_TIMEOUT`), preventing hanging threads on unbounded queries.
- **SEC-11 (Query History Credential Scrubbing):** Added `redact_secrets()` replacing passwords in `IDENTIFIED BY '...'` and `PASSWORD('...')` with `'***'` before persisting to disk.
- **SEC-12 (Empty WHERE Guard):** Enforced non-empty `where_clause` check in `update_row` and `delete_row`, returning an `Err` to prevent syntax errors and unintended whole-table modifications.

---

## [3.0.1] — 2026-10-03

### Security & Hardening
- **SEC-01 (Stored Routine Fallback Parameterization):** Replaced string interpolation in fallback `SHOW PROCEDURE STATUS` and `SHOW FUNCTION STATUS` queries with bound parameters (`WHERE Db = ?` + `.bind(db)`).
- **SEC-02 (DDL Column Length & Type Validation):** Added `validate_column_length()` in `sanitize.rs` supporting numeric lengths, precision pairs (`10,2`), and quoted `ENUM`/`SET` lists while rejecting DDL breakouts. Whitelisted column types before DDL creation.
- **SEC-03 (Table Designer DDL Hardening):** Routed database, table, and column names through `sanitize_identifier()` and lengths through validation in `app_controller.rs`.
- **SEC-04 (Multi-Character SQL Dump Escaping):** Implemented `escape_sql_string()` in `sanitize.rs` escaping backslashes (`\\`), single quotes (`''`), and control characters (`\0`, `\n`, `\r`, `\x1a`), standardizing export dumps in `maintenance.rs`.
- **SEC-08 (Visual Query Builder Table Quoting):** Sanitized table name interpolation in the query builder using `sanitize_identifier()`.

---

## [3.0.0] — 2026-10-03

### Added
- **Native Rust & Slint Rewrite:** Complete architectural replacement of the legacy Next.js 15 / React / Tauri v2 frontend with a 100% native Rust desktop app using [Slint 1.18](https://slint.dev).
- **Instant Cold Startup:** Sub-50ms native application startup; removed all Node.js runtime, Chromium, and WebKitGTK dependencies.
- **Hardware-Accelerated UI:** Declarative Slint views with responsive layouts and dark/light design token system (`ui/theme.slint`).
- **Async MySQL Driver:** Directly integrated `sqlx 0.8` on `tokio` for pooled, concurrent database operations.
- **Comprehensive View Suite:** 18 native Slint view screens including Database Explorer, Table Data Grid, Table Designer, SQL Editor with duration telemetry, Process List, Server Overview, Query Builder, ER Diagram Visualizer, and Mock Data Generator.
- **Native File Dialogs:** Integrated `rfd` (Rust File Dialogs) for cross-platform OS file picking.

### Removed
- Removed legacy Next.js 15, TypeScript, React 18, Tailwind CSS, and Tauri v2 configurations and dependencies.

---

## [1.0.0] — 2026-07-19

### Added
- Initial public release of MySQL GUI (browser-based Next.js / Tauri hybrid).