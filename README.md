# MySQL GUI (v3.0)

A high-performance, native MySQL desktop client built with **Rust** and **Slint UI**.  
Zero Electron. Zero Node.js. Zero web runtime overhead. Just pure native speed and direct SQL execution.

![Version](https://img.shields.io/badge/version-3.0.0-blue)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![UI](https://img.shields.io/badge/UI-Slint%201.18-green)
![Database](https://img.shields.io/badge/Database-MySQL%20%2F%20MariaDB-4479A1)

---

## Highlights

* **Instant Startup & Minimal Footprint:** Compiles directly into a single native binary. No Chromium, No WebViews, sub-50ms cold startup.
* **Fully Native Declarative UI:** Built with [Slint](https://slint.dev), utilizing hardware-accelerated rendering and responsive layouts.
* **Async MySQL Driver:** Directly driven by [`sqlx`](https://github.com/launchbadge/sqlx) and `tokio` for pooled, concurrent database operations.
* **Secure Credential Storage:** Zero plaintext passwords on disk; utilizes OS Keyring via `keyring-rs`.

---

## Features

### Database & Table Management
- **Database Explorer:** Browse, create, and drop databases with live table counts.
- **Table Data Viewer:** Paginated data grid with customizable limit (25, 50, 100), column sorting, and row deletion.
- **Table Structure Inspector:** Deep inspection of column names, data types, nullability, keys (PRI, UNI, MUL), and extra attributes.
- **Create Table Designer:** Visual column definition builder with type selection, constraints, and instant DDL generation.

### SQL Query Console
- **Custom SQL Execution:** Execute custom queries, DDL, batch updates, and transactions.
- **Query Results Table:** Paginated column and row visualization with execution duration telemetry.
- **Query History:** Log of executed queries with one-click re-run, favorites, and timestamping.
- **Presets Toolbar:** Fast shortcuts for `SELECT *`, count checks, and query formatting.

### Developer Tools & Monitoring
- **Visual Query Builder:** Construct complex queries visually by selecting tables and conditions.
- **Server Health & Monitoring:** Live connection stats, thread count, queries executed, slow query counter, and live process list.
- **Slow Query Log Viewer:** Read and inspect slow query performance logs.
- **Mock Data Generator:** Generate synthetic rows based on table column schemas for testing.
- **Export & Import:** Export data as SQL dump or JSON/CSV, import external `.sql` scripts.
- **User & Privileges:** Inspect MySQL users, hosts, and account permissions.

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