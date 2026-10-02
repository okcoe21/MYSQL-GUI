# Technical Concerns & Roadmap

**Analysis Date:** 2026-10-03 (v3.0.0 Native Rust + Slint Rewrite)

## Current Technical Debt & Risks

### 1. Test Coverage
- **Status:** Automated test suite is currently minimal (`cargo check` and `cargo test` pass, but specific unit tests for all `src/db/` modules need expansion).
- **Risk:** Regressions in complex SQL parsing, column mapping, or identifier sanitization could go unnoticed.
- **Remediation:** Add automated unit tests for `src/db/sanitize.rs`, `src/db/models.rs`, and query builders.

### 2. Slint Layout Clamping Behavior
- **Context:** Slint's Ahead-Of-Time (AOT) layout generator computes `max_height` as the sum of child max heights if all children are fixed.
- **Risk:** Adding new fixed-height children to a view inside a `Flickable` without an unconstrained spacer can accidentally re-clamp the parent `HorizontalLayout` and freeze the window height.
- **Remediation:** Always keep a trailing `Rectangle { }` inside `VerticalLayout` containers that are meant to fill scrollable viewports.

### 3. File Dialog Pipeline
- **Context:** Native file dialogs (`rfd`) are imported in `Cargo.toml`.
- **Status:** File picker integration needs to be wired directly into `export_view.slint` and `import_view.slint` so users can choose exact file locations on disk.

### 4. Connection Pool & Reconnect Lifecycles
- **Context:** `sqlx::MySqlPool` manages async connections.
- **Risk:** Server timeouts (e.g., MySQL `wait_timeout`) or network drops when idle might cause a query to fail on stale connections.
- **Remediation:** Ensure pool options configure `idle_timeout` and `test_before_acquire` in `src/db/auth.rs`.

---

## Security Considerations

- **Keyring Reliability Across Linux Environments:** While Windows Credential Manager and macOS Keychain are universal, headless Linux servers or lightweight window managers without DBus/SecretService daemons (like `gnome-keyring` or `kwallet`) might require a fallback.
- **SQL Injection Prevention:** Continue enforcing `sanitize_identifier()` for all identifier names and using parameterized bindings for values.
