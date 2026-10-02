# MySQL GUI v3.0.3 — Native Desktop Release

**Release Date:** October 3, 2026  
**Target:** Native Linux, Windows & macOS Desktop  
**Binary Name:** `mysql-gui`  

---

## Executive Summary

**MySQL GUI v3.0.3** marks the official production release of our **100% native Rust desktop rewrite**, powered by **Slint UI 1.18** and **SQLx 0.8**. 

This major milestone completely replaces the former Web / Next.js / Tauri hybrid with a compiled native application offering sub-50ms cold startups, zero Chromium / Node.js overhead, hardware-accelerated rendering, an exhaustive SQL injection security audit, and a comprehensive automated unit test suite.

---

## What's New in v3.0.3

### 1. Pure Native Rust & Slint UI Engine
* **Single Native Binary:** Zero Chromium, zero WebKitGTK, zero Node.js runtime. Compiles into an ultra-lean executable with immediate response times.
* **Hardware-Accelerated UI:** Built with [Slint](https://slint.dev), utilizing modern GPU backends (OpenGL / Skia) with responsive layouts and full dark/light theme tokens.
* **Direct Async Database Pool:** Native, non-blocking MySQL connection pooling powered by `sqlx 0.8` on `tokio`.

### 2. Comprehensive Security Hardening & Injection Defense
* **Bound Introspection Queries (SEC-01):** Replaced raw string interpolation in stored procedure and function fallbacks with parameterized statements (`WHERE Db = ?`).
* **DDL Validation & Length Boundaries (SEC-02, SEC-03):** Strict verification of column length specifiers (numeric, precision pairs `10,2`, and quoted `ENUM`/`SET` lists) plus data type whitelisting. Table designer inputs are completely sanitized.
* **64-Character Identifier Limit:** Enforced MySQL's specification limit of 64 characters across all database, table, and column names.
* **Byte-Safe SQL Export Escaping (SEC-04):** Export dump generation now handles backslashes (`\\`), single quotes (`''`), and control characters (`\0`, `\n`, `\r`, `\x1a`) preventing restore breakouts.
* **Defensive Connection Handling (SEC-05):** Structured connection initialization via `MySqlConnectOptions` (.host, .port, .username, .password, .database) preventing URL delimiter injection; passwords are automatically redacted from error banners.
* **Transport Encryption Detection (SEC-06):** Automatic introspection of SSL/TLS connection status (`is_encrypted`) via session status checks.
* **Destructive Operation UX Guard (SEC-07):** Hardened client-side safety guard stripping leading comments (`--`, `#`, `/* ... */`) with word-boundary matching on `DROP`, `TRUNCATE`, `DELETE`, `ALTER`, `GRANT`, `REVOKE`, and unbounded `UPDATE`.
* **Query Execution Timeouts (SEC-09):** Background 60-second timeouts (`DEFAULT_QUERY_TIMEOUT`) using `tokio::time::timeout` prevent hung threads on unbounded queries.
* **Scrubbed Query History (SEC-11):** Plaintext passwords in `IDENTIFIED BY` and `PASSWORD(...)` statements are automatically replaced with `'***'` before persisting to disk.
* **Empty WHERE Guard (SEC-12):** Strict prevention of empty `WHERE` clauses on row updates and deletions.

### 3. Automated Unit & Regression Test Suite
* 16 automated tests passing in `cargo test`:
  * Identifier quoting, space rejection, backtick rejection, NUL byte rejection, 64-char accept / 65-char reject boundary, and SQL injection attempt rejection.
  * Sort direction whitelisting and limit/offset boundary clamping.
  * SQL string multi-character escaping.
  * Column length numeric, precision, and enum/set parsing.
  * Destructive keyword detection with comment stripping.
  * Password redaction in query history.

### 4. 18 Native Slint Views & Tools
* **Database & Table Explorers:** Real-time database catalog, table summaries, and row counts.
* **Paginated Data Grid:** Table data viewer with configurable limits (25, 50, 100), sortable columns, and inline deletion.
* **Schema Inspector & DDL Designer:** Deep column type inspection and visual column schema creation.
* **Interactive SQL Editor:** Syntax-ready console, execution duration telemetry, and tabular results view.
* **Developer Utilities:** Visual Query Builder, ER foreign key relationship visualizer, live process list (`SHOW FULL PROCESSLIST`), server vitals, slow query log viewer, and synthetic mock data generator.

---

## Installation & Getting Started

### Prerequisites
- **Rust Toolchain:** 1.75+ ([rustup.rs](https://rustup.rs/))
- **MySQL / MariaDB:** Local or remote MySQL instance
- **Linux Packages:** Standard development headers and `libfontconfig`

### Build from Source
```bash
git clone https://github.com/okcoe21/MYSQL-GUI.git
cd MYSQL-GUI
cargo build --release
```
The optimized native binary will be generated at `target/release/mysql-gui`.

### Run Tests
```bash
cargo test
```

---

## Verification Metrics
- **Compilation:** Clean (`cargo check` & `cargo build` pass with 0 errors, 0 warnings).
- **Unit Tests:** 16 passed; 0 failed.
- **Security Audit:** 100% of High, Medium, and Low-severity findings resolved or tracked.
- **License:** MIT