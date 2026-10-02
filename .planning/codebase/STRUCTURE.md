# Directory Structure

**Analysis Date:** 2026-10-03 (v3.0.0 Native Rust + Slint Rewrite)

## Folder Layout

```text
.
├── Cargo.toml               # Rust package manifest, dependencies, and metadata
├── Cargo.lock               # Deterministic dependency resolution lockfile
├── build.rs                 # Slint Ahead-Of-Time (AOT) compiler build script
├── README.md                # Project documentation and quickstart
├── CLAUDE_UPDATE_SUMMARY.md # Handoff report for AI agents and developers
│
├── src/                     # Core Rust backend and native application logic
│   ├── main.rs              # Application entry point, window instantiation, event loop
│   ├── app_controller.rs    # Controller binding Slint UI callbacks to async SQLx ops
│   ├── state.rs             # Thread-safe global state (AppState with MySqlPool)
│   ├── explain.rs           # Offline rule-based SQL query explainer & risk detector
│   └── db/                  # Modular SQLx database services
│       ├── mod.rs           # DB module exports
│       ├── auth.rs          # MySQL connection authentication and keyring handling
│       ├── database.rs      # Database cataloging, creation, and deletion
│       ├── table.rs         # Table schemas and column metadata introspection
│       ├── data.rs          # Paginated data querying, sorting, and row deletion
│       ├── query.rs         # Raw SQL query runner and tabular result mapping
│       ├── server.rs        # Live server metrics and process list inspection
│       ├── history.rs       # Persistent query execution history
│       ├── objects.rs       # Database views, stored procedures, and triggers
│       ├── maintenance.rs   # Table optimization, repair, and analysis
│       ├── models.rs        # Typed Rust domain models and Slint conversions
│       └── sanitize.rs      # SQL identifier quoting and query sanitization
│
├── ui/                      # Slint Declarative UI source files
│   ├── app.slint            # Root AppWindow layout, view router, and dialogs
│   ├── theme.slint          # Design tokens (colors, typography, radii, dark/light)
│   ├── preview_dashboard.json # Mock preview configuration for slint-viewer
│   ├── components/          # Reusable Slint UI widgets
│   │   ├── button.slint     # Primary, Secondary, Danger, and Tab buttons
│   │   ├── card.slint       # PanelCard and StatCard containers
│   │   ├── dialog.slint     # ConfirmDialog modal
│   │   └── input.slint      # CustomInput and MultilineInputBox
│   └── views/               # 18 specialized application views
│       ├── login.slint      # Connection credentials portal
│       ├── topbar.slint     # Navigation breadcrumbs, server status, theme toggle
│       ├── sidebar.slint    # Collapsible database, table, and view tree
│       ├── server_overview.slint # Server telemetry and live processes
│       ├── db_overview.slint     # Database summaries and quick actions
│       ├── table_data.slint      # Paginated data grid with sorting and deletion
│       ├── structure_view.slint  # Column schema, keys, and indexes
│       ├── sql_editor.slint      # Query console with tabular results
│       ├── history_view.slint    # Query history log with favorite toggles
│       ├── query_builder.slint   # Visual query builder
│       ├── create_table_view.slint # Visual table schema designer
│       ├── diagram_view.slint    # ER schema relationship visualizer
│       ├── export_view.slint     # SQL/CSV/JSON export configuration
│       ├── import_view.slint     # SQL script import processor
│       ├── user_management.slint # MySQL user accounts and hosts
│       ├── performance_view.slint # Buffer pools and traffic metrics
│       ├── slow_log_view.slint   # Slow query log viewer
│       └── mock_data_view.slint  # Synthetic test data generator
│
├── .github/                 # GitHub CI/CD automation
│   └── workflows/
│       ├── build-check.yml  # Native Rust check & test on push/PR
│       └── release.yml      # Cross-platform matrix binary releases (Linux/Win/Mac)
│
└── .planning/               # Project documentation and specifications
    └── codebase/            # Codebase architecture, conventions, and stack cards
```

## Key Files & Entry Points

- **[`Cargo.toml`](file:///home/coes/Projects/MYSQL%20GUI/Cargo.toml):** Master crate configuration.
- **[`build.rs`](file:///home/coes/Projects/MYSQL%20GUI/build.rs):** Compiles `ui/app.slint` into generated Rust code via `slint_build::compile()`.
- **[`src/main.rs`](file:///home/coes/Projects/MYSQL%20GUI/src/main.rs):** Instantiates the Slint GUI window and hands off control to `AppController`.
- **[`src/app_controller.rs`](file:///home/coes/Projects/MYSQL%20GUI/src/app_controller.rs):** Connects user actions from Slint to async SQLx database workers.
- **[`ui/app.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/app.slint):** Root declarative component managing window geometry, theme tokens, and view routing.
