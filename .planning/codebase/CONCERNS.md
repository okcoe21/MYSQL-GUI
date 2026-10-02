# Technical Concerns & Roadmap

**Analysis Date:** 2026-10-03 (v3.0.3 Native Rust + Slint Rewrite)

## Current Technical Debt & Risks

### 1. Test Coverage
- **Status:** Initial unit and security regression test suite active and passing (16/16 tests in `cargo test`). Covers identifier quoting, boundaries, SQL escaping, column length validation, destructive guards, and secret redaction.
- **Next Steps:** Expand integration testing with disposable MySQL containers (`testcontainers`) and Slint headless UI testing.

### 2. Slint Layout Clamping Behavior
- **Context:** Slint's Ahead-Of-Time (AOT) layout generator computes `max_height` as the sum of child max heights if all children are fixed.
- **Status:** Resolved in `sidebar.slint` and `server_overview.slint` via trailing flexible spacers (`Rectangle { }`), allowing reactive 100% height window scaling.
- **Rule:** Always keep an unconstrained spacer inside `VerticalLayout` containers that must expand to fill scrollable viewports.

### 3. File Dialog Pipeline
- **Context:** Native file dialogs (`rfd`) are imported in `Cargo.toml`.
- **Status:** File picker integration needs to be connected to `export_view.slint` and `import_view.slint` so users can choose arbitrary file destinations on disk.

### 4. Connection Pool & Reconnect Lifecycles
- **Context:** `sqlx::MySqlPool` manages async connections.
- **Status:** Configured with `max_connections(5)` and `idle_timeout(60s)`. Structured connection handling via `MySqlConnectOptions` prevents URL injection.
- **Next Steps:** Add `test_before_acquire` to automatically revalidate dropped connections during long idle intervals.

---

## Security Considerations

- **SQL Injection Prevention:** Fully audited and patched (see [`AUDIT.md`](file:///home/coes/Projects/MYSQL%20GUI/AUDIT.md)). Bound parameters enforced in introspection, strict 64-char identifier boundaries, type whitelists, and byte-safe export escaping.
- **Credential Storage (SEC-10):** Native OS Keyring (`keyring = "2"`) integration is scheduled as feature work for persistent credential saving and auto-fill in the login view.
- **Query History Privacy:** Automatic redaction of credentials in `IDENTIFIED BY` and `PASSWORD(...)` statements before persisting to disk (`~/.local/share/mysql-gui/history.json`).
