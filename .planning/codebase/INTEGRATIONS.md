# External Integrations

**Analysis Date:** 2026-10-03 (v3.0.0 Native Rust + Slint Rewrite)

## Primary Databases

**MySQL & MariaDB:**
- **Protocol:** Native MySQL Wire Protocol over TCP (default port 3306) with TLS support.
- **Client Driver:** `sqlx::MySqlPool` (v0.8).
- **Capabilities:**
  - Connection pooling with automatic thread-safe connection distribution.
  - Multi-statement execution for custom scripts and DDL.
  - Performance introspection via `information_schema` and `performance_schema`.

---

## Operating System & Native Services

**Credential Storage:**
- **Provider:** `keyring-rs` (v2.x).
- **Backends:**
  - Linux: Secret Service API (`org.freedesktop.secrets` / `gnome-keyring` / `KWallet`).
  - macOS: Apple Keychain Services.
  - Windows: Windows Credential Manager.

**File System Pickers:**
- **Provider:** `rfd` (Rust File Dialogs v0.15).
- **Target Operations:** Native open/save file modals for `.sql` script import and CSV/JSON export.

---

## Tooling Integrations

**Offline UI Preview:**
- **Tool:** `slint-viewer` (v1.18.1).
- **Configuration:** [`ui/preview_dashboard.json`](file:///home/coes/Projects/MYSQL%20GUI/ui/preview_dashboard.json).
- **Workflow:** Allows rapid front-end tweaking without compiling Rust code.

**Developer Project Memory:**
- **Tool:** `pmem` (Workstation project catalog & FTS5 database).
- **Card:** `/home/coes/Projects/project-memory/mysql-gui.md`.
