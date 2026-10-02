# Testing Strategy

**Analysis Date:** 2026-10-03 (v3.0.3 Native Rust + Slint Rewrite)

## Current Status

- **Build Check:** `cargo check` passes with 0 warnings/errors.
- **Test Suite:** `cargo test` executes the Rust standard test harness with 16 automated tests passing.
- **Coverage:** Unit testing covers `src/db/sanitize.rs` (identifier quoting, boundaries, escaping, column length parsing, destructive guard) and `src/db/history.rs` (secret redaction).

---

## Testing Architecture

### 1. Active Unit Tests (`cargo test`)
* **`src/db/sanitize.rs`:**
  * `test_sanitize_identifier_valid_plain`: Plain, underscore, and dollar-prefixed identifiers.
  * `test_sanitize_identifier_rejects_empty`: Rejection of empty names.
  * `test_sanitize_identifier_rejects_spaces`: Rejection of leading/trailing/embedded whitespace.
  * `test_sanitize_identifier_rejects_embedded_backtick`: Prevention of backtick escape injections.
  * `test_sanitize_identifier_rejects_nul_byte`: Rejection of poison NUL bytes.
  * `test_sanitize_identifier_length_boundary`: Acceptance of 64-char names and rejection of 65-char names (MySQL limit).
  * `test_sanitize_identifier_rejects_injection_attempt`: Defense against payload injection (`users`; DROP TABLE x;--).
  * `test_sanitize_sort_direction`: Strict case-insensitive whitelisting of `ASC` and `DESC`.
  * `test_clamp_limit_offset`: Clamping negative and over-large limits/offsets.
  * `test_escape_sql_string_special_chars`: Multi-character escaping (`\\`, `''`, `\0`, `\n`, `\r`, `\x1a`).
  * `test_validate_column_length_numeric_and_precision`: Validating `"255"`, `"10,2"`, `"10, 2"`.
  * `test_validate_column_length_enum_set`: Quoted ENUM/SET values and unclosed quote detection.
  * `test_validate_column_length_rejects_injections`: Rejection of DDL breakout syntax.
  * `test_is_destructive_comments_and_whitespace`: Comment-stripping (`--`, `#`, `/* */`) destructive command detection.
  * `test_is_destructive_update_where`: Destructive detection for `UPDATE` without `WHERE` clause.
* **`src/db/history.rs`:**
  * `test_redact_secrets`: Scrubbing passwords in `IDENTIFIED BY` and `PASSWORD(...)` queries.

### 2. Integration Testing (Roadmap)
* **Targets:** Multi-step connection pooling, live query execution, and database schema introspection.
* **Approach:** Use `testcontainers` or a local disposable MySQL test container (e.g., `mysql:8.0` / `mariadb:latest`).
* **Validation:** Test that stored procedures, views, and complex foreign key relations introspect accurately without panics.

### 3. Slint UI Headless Testing (Roadmap)
* **Targets:** Callback dispatching and property updates.
* **Tooling:** Slint provides `slint::testing` helpers to simulate clicks, touch areas, and keyboard inputs programmatically in CI without an active X11 display.

---

## Running Verification Commands

```bash
# Verify compiler validity
cargo check --verbose

# Run all automated unit and security tests
cargo test --verbose

# Run live UI preview
slint-viewer ui/app.slint
```
