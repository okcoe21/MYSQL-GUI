# MySQL GUI (v3.0.7)

A high-performance, native MySQL desktop client built with **Rust** and **Slint UI**.  
Zero Electron. Zero Node.js. Zero web runtime overhead. Just pure native speed and direct SQL execution.

![Version](https://img.shields.io/badge/version-3.0.7-blue)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![UI](https://img.shields.io/badge/UI-Slint%201.18-green)
![Database](https://img.shields.io/badge/Database-MySQL%20%2F%20MariaDB-4479A1)
![Tests](https://img.shields.io/badge/Tests-69%20Passing-brightgreen)

---

## Highlights

* **Instant Startup & Minimal Footprint:** Compiles directly into a single native binary. No Chromium, No WebViews, sub-50ms cold startup.
* **Fully Native Declarative UI:** Built with [Slint](https://slint.dev), utilizing hardware-accelerated rendering and responsive layouts.
* **Async MySQL Driver:** Directly driven by [`sqlx`](https://github.com/launchbadge/sqlx) and `tokio` for pooled, concurrent database operations.
* **Defensive Security & Input Validation:** Built-in SQL injection defenses, DDL validation, identifier boundaries, and automated credential redaction.

---

## Features

### Database & Table Management
- **Database Explorer:** Browse, create, and drop databases with live table counts.
- **Table Data Viewer:** Paginated data grid with customizable limit (25, 50, 100), column sorting, and row deletion.
- **Table Structure Inspector:** Deep inspection of column names, data types, nullability, keys (PRI, UNI, MUL), and extra attributes.
- **Create Table Designer:** Visual column definition builder with type selection, constraints, and instant DDL generation.

### SQL Query Console & Security
- **Custom SQL Execution:** Execute custom queries, DDL, batch updates, and transactions.
- **Offline Rule-Based Query Explainer:** Translate arbitrary SQL queries into plain English summaries with clause-by-clause breakdowns (SELECT, INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, TRUNCATE) and risk warnings (unbounded updates/deletes, unconstrained SELECT *, drops) without requiring any network or database connection.
- **Execution Timeout Guard:** Background 60-second execution timeouts to prevent hung threads on long-running queries.
- **Destructive Operation Prompts:** UX confirmation guard identifying `DROP`, `TRUNCATE`, `DELETE`, `ALTER`, and unbounded `UPDATE` operations before execution.
- **Query Results Table:** Paginated column and row visualization with execution duration telemetry.
- **Sanitized Query History:** Local query history with automatic credential redaction (`IDENTIFIED BY`, `PASSWORD(...)`), one-click re-run, favorites, and timestamping.
- **Presets Toolbar:** Fast shortcuts for `SELECT *`, count checks, query explanation, and query formatting.

### Developer Tools & Monitoring
- **Visual Query Builder:** Construct complex queries visually by selecting tables and conditions with live safe SQL generation.
- **Transport Security Controls:** Optional "Require SSL" switch on login enforcing TLS connections; live encryption status badge (`SSL` / `NOT ENCRYPTED`) in the top navigation bar.
- **Server Health & Monitoring:** Live connection stats, thread count, queries executed, slow query counter, and live process list.
- **Slow Query Log Viewer:** Read and inspect slow query performance logs.
- **Mock Data Generator:** Generate synthetic rows based on table column schemas for testing.
- **Native File Dialogs & Streaming Export/Import:**
  - **Streaming Export:** Native `rfd::AsyncFileDialog` saving SQL dumps, CSV, or JSON directly to disk streamed in 500-row chunks to prevent memory bloat.
  - **CSV Formula Injection Defense:** RFC 4180 CSV export neutralizes formula injection by prefixing `=, +, -, @` with `'`.
  - **Safe SQL Script Import:** Load and execute batch `.sql` scripts with safe statement splitting, pure DML transaction wrapping, and destructive operation confirmation guards (`DROP`, `TRUNCATE`, `DELETE`).
  - **Transactional CSV Import:** Native RFC 4180 CSV file loader into target tables with column count introspection, parameter binding (`.bind()`), 50 MB size protection, and atomic transaction rollback on failure.
- **User & Privileges:** Inspect MySQL users, hosts, and account permissions.

---

## Security Architecture

| Security Domain | Mitigation / Implementation |
|---|---|
| **Identifier Quoting** | Strict alphanumeric/`_`/`$` whitelist, 64-char MySQL length boundary, backtick escaping. |
| **DDL Validation** | Numeric, precision pair (`10,2`), and quoted ENUM/SET value validation on column lengths. |
| **Credential Safety** | URL parsing avoided; sensitive credentials redacted from UI error dialogs and history logs. |
| **SQL Export Escaping** | Byte-safe string escaping (`\\`, `''`, `\0`, `\n`, `\r`, `\x1a`) preventing dump restore breakouts. |
| **Query Safeguards** | 60-second `tokio::time::timeout` and strict prevention of empty-`WHERE` updates/deletions. |
| **Transport Security** | SSL/TLS session detection (`is_encrypted`), user-controlled Require SSL enforcement (`MySqlSslMode::Required`), and live topbar status badge. |

---

## Architecture

| Component | Technology | Description |
|---|---|---|
| **GUI Framework** | [Slint 1.18](https://slint.dev) | Hardware-accelerated native UI declarative engine |
| **Language** | Rust (2021 edition) | High safety, low memory overhead, zero runtime GC |
| **Async Runtime** | Tokio | Multi-threaded async runtime |
| **Database Engine** | SQLx (MySQL) | Native non-blocking async SQL connection pool |
| **Credential Store** | keyring-rs | Secret storage via system keychain (SecretService / Keychain) |
| **File Dialogs** | rfd | Native OS file picker dialogs |

### Directory Layout
```text
├── build.rs             # Slint AOT compiler build hook
├── Cargo.toml           # Project dependencies and crate metadata
├── src/
│   ├── main.rs          # Application entry point and window lifecycle
│   ├── app_controller.rs# State coordinator and UI-to-Database callbacks
│   ├── state.rs         # Global application state and connection metadata
│   └── db/              # Modular SQLx database handlers
│       ├── auth.rs      # Connection authentication and keyring storage
│       ├── database.rs  # Database level inspection and creation
│       ├── table.rs     # Table schemas and metadata
│       ├── data.rs      # Paginated row retrieval, mutation, and counts
│       ├── query.rs     # Arbitrary query executor
│       ├── server.rs    # Server stats and process list
│       ├── history.rs   # Persistent query history
│       └── ...
└── ui/
    ├── app.slint        # Root AppWindow layout and view router
    ├── theme.slint      # Design tokens (colors, typography, radii)
    ├── components/      # Reusable Slint widgets (buttons, cards, inputs, dialogs)
    └── views/           # Dedicated application screens
        ├── login.slint
        ├── sidebar.slint
        ├── topbar.slint
        ├── server_overview.slint
        ├── db_overview.slint
        ├── table_data.slint
        ├── sql_editor.slint
        └── ...
```

---

## Getting Started

### Prerequisites
- **Rust Toolchain:** `rustc` and `cargo` 1.75+ ([rustup.rs](https://rustup.rs/))
- **MySQL / MariaDB:** A running MySQL server instance (local or remote)
- **Linux Packages (if building on Linux):** Standard X11 / Wayland development headers and `libfontconfig` (`libfontconfig1-dev` on Debian/Ubuntu).

### Running in Development
```bash
cargo run
```

### Running Unit & Security Tests
```bash
cargo test
```

### Building for Release
```bash
cargo build --release
```
The optimized native binary will be generated at `target/release/mysql-gui`.

### Live UI Preview with `slint-viewer`
You can preview and test UI components live without compiling Rust:
```bash
slint-viewer ui/app.slint
```

---

## License
MIT