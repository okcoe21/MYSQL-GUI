# Testing Strategy

**Analysis Date:** 2026-10-03 (v3.0.0 Native Rust + Slint Rewrite)

## Current Status

- **Build Check:** `cargo check` passes with 0 warnings/errors.
- **Test Suite:** `cargo test` executes the Rust standard test harness.
- **Coverage:** Unit testing coverage for `src/db/` modules is in progress.

---

## Recommended Testing Architecture

### 1. Unit Tests (`cargo test`)
* **Targets:**
  * `src/db/sanitize.rs`: Verify backtick escaping, quote handling, and detection of destructive commands (`DROP`, `DELETE`, `TRUNCATE`).
  * `src/db/models.rs`: Verify typed row mapping, data type formatting, and null cell handling.
  * SQL builders: Test DDL generation logic in `create_table_view` and `query_builder`.

### 2. Integration Tests
* **Targets:** Connection pooling, query execution, and database schema introspection.
* **Approach:** Use `testcontainers` or a local disposable MySQL test container (e.g., `mysql:8.0` / `mariadb:latest`).
* **Validation:** Test that stored procedures, views, and complex foreign key relations introspect accurately without panics.

### 3. Slint UI Headless Testing
* **Targets:** Callback dispatching and property updates.
* **Tooling:** Slint provides `slint::testing` helpers to simulate clicks, touch areas, and keyboard inputs programmatically in CI without an active X11 display.

---

## Running Verification Commands

```bash
# Verify compiler validity
cargo check --verbose

# Run all automated tests
cargo test --verbose

# Run live UI preview
slint-viewer ui/app.slint
```
