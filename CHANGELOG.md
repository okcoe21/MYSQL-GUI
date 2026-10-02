# Changelog

All notable changes to MySQL GUI are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

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