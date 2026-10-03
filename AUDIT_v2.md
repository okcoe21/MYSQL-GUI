# Security & Integrity Audit Report (v2) — Data Mutation Paths

**Date:** October 3, 2026  
**Target:** `mysql-gui` v3.0.9 (`src/db/data.rs`, `src/app_controller.rs`, `ui/views/table_data.slint`)  
**Scope:** Inline Cell Editing, Row Insertion, and Row Deletion paths, including SQL builders, parameter binding, identifier quoting, transaction boundaries, and state concurrency.

---

## 1. Executive Summary

This targeted security audit analyzed the data mutation pipeline:
- Double-click inline cell editing (`on_request_cell_edit`, `on_submit_cell_edit`, `data::update_row`)
- In-grid row insertion modal (`on_open_insert_modal`, `on_submit_insert_modal`, `data::insert_row`)
- In-grid row deletion (`on_delete_table_row`, `on_confirm_dialog_action`, `data::delete_row`)
- Pure query builder and validation functions in [`src/db/data.rs`](file:///home/coes/Projects/MYSQL%20GUI/src/db/data.rs) (`build_update_query`, `build_insert_query`, `build_delete_query`, `validate_insert_target_table`, `validate_delete_snapshot`)
- UI interaction and state bindings in [`ui/views/table_data.slint`](file:///home/coes/Projects/MYSQL%20GUI/ui/views/table_data.slint) and [`src/app_controller.rs`](file:///home/coes/Projects/MYSQL%20GUI/src/app_controller.rs)

The implementation enforces comprehensive security and integrity guarantees: all values are bound as query parameters, all identifiers are validated and quoted with backticks via `sanitize_identifier()`, `WHERE` clauses are constructed strictly from primary-key metadata, read-only columns (generated, BLOB, binary, spatial) and tables without usable primary keys are enforced in Rust, updates use dedicated connection transactions with row-level locking (`SELECT ... FOR UPDATE`), deletions run inside atomic transactions verifying `rows_affected == 1`, and query history logging uses query templates with `?` placeholders rather than raw cell values.

All findings identified in the audit pass have been fully resolved and verified with pure unit tests and UI integration.

---

## 2. Summary of Findings & Remediation Status

| ID | Location | Category | Severity | Summary | Status / Resolution |
|---|---|---|---|---|---|
| **SEC2-01** | `src/db/data.rs:239` | Schema Integrity | **LOW** | `build_insert_query` does not explicitly reject columns missing from metadata | **FIXED:** Unknown columns rejected via `.ok_or_else()`. Covered by unit test `test_insert_rejects_unknown_columns`. |
| **SEC2-02** | `src/app_controller.rs:633` | Concurrency / Race | **LOW** | `on_delete_table_row` resolved row via transient `row_idx` without PK snapshot | **FIXED:** Captures immutable `DeleteRowSnapshot`, gates behind `ConfirmDialog` ("Delete row ... from ...?"), and handler executes solely against snapshot without touching `row_idx`. |
| **SEC2-03** | `src/db/data.rs:150` | Data Integrity | **INFORMATIONAL** | Primary key columns were not excluded from inline cell editing | **BLOCKED BY DESIGN:** Primary key cells marked read-only in UI with warning hint ("Primary key: delete and re-insert instead") and blocked with `Err` in `build_update_query`. |
| **SEC2-04** | `src/app_controller.rs:873` | State Concurrency | **INFORMATIONAL** | `table_insert_fields` did not lock target table name | **FIXED:** `table_insert_target_table` tracks open table and validates against active table on submit via `validate_insert_target_table`. Mismatch aborts immediately. |
| **SEC2-05** | `src/db/data.rs:461` | Transaction Boundary | **INFORMATIONAL** | `delete_row` ran directly on connection rather than in transaction | **FIXED:** Wrapped in dedicated pooled connection transaction (`conn.begin()`), checks `rows_affected == 1`, and rolls back otherwise. |

---

## 3. SQL Construction & Parameter Classification Matrix

| Statement Type | Location | SQL Template | Identifier Status | Value Status |
|---|---|---|---|---|
| **Row Lock & Existence** | `src/db/data.rs:165` | `SELECT 1 FROM \`table\` WHERE \`pk1\` = ? AND \`pk2\` = ? FOR UPDATE` | **Quoted** (`sanitize_identifier`) | **Bound** (`?` via `sqlx::query().bind()`) |
| **Row Update** | `src/db/data.rs:166` | `UPDATE \`table\` SET \`col\` = ? WHERE \`pk1\` = ? AND \`pk2\` = ? LIMIT 1` | **Quoted** (`sanitize_identifier`) | **Bound** (`?` for SET and WHERE values) |
| **Row Insert (Values)** | `src/db/data.rs:260` | `INSERT INTO \`table\` (\`col1\`, \`col2\`) VALUES (?, ?)` | **Quoted** (`sanitize_identifier`) | **Bound** (`?` for all column values) |
| **Row Insert (Defaults)** | `src/db/data.rs:257` | `INSERT INTO \`table\` () VALUES ()` | **Quoted** (`sanitize_identifier`) | **N/A** (All columns omitted for DB defaults) |
| **Row Deletion** | `src/db/data.rs:327` | `DELETE FROM \`table\` WHERE \`pk1\` = ? AND \`pk2\` = ? LIMIT 1` | **Quoted** (`sanitize_identifier`) | **Bound** (`?` for PK values) |
| **History Log (UPDATE)** | `src/app_controller.rs:812` | `UPDATE \`table\` SET \`col\` = ? WHERE \`pk1\` = ? LIMIT 1` | **Quoted** (`sanitize_identifier`) | **Placeholders only** (`?`, zero raw values) |
| **History Log (INSERT)** | `src/app_controller.rs:979` | `INSERT INTO \`table\` (\`col1\`) VALUES (?)` | **Quoted** (`sanitize_identifier`) | **Placeholders only** (`?`, zero raw values) |
| **History Log (DELETE)** | `src/app_controller.rs:446` | `DELETE FROM \`table\` WHERE \`pk1\` = ? LIMIT 1` | **Quoted** (`sanitize_identifier`) | **Placeholders only** (`?`, zero raw values) |

---

## 4. Detailed Audit Step Analysis & Remediations

### 4.1. Metadata Provenance of Column Names (SEC2-01)
- **Cell Edit (`SET` clause):**
  - In `on_request_cell_edit`, the target column is resolved via `cols.get(col_idx as usize)`. The UI supplies only `col_idx: int`.
  - In `build_update_query`, the column name is validated against `columns_info` via `find(|c| c.field.eq_ignore_ascii_case(column)).ok_or_else(...)`.
  - Both table and column names are passed through `sanitize_identifier()`.
- **Row Insert (`INSERT` clause):**
  - In `build_insert_query`, every column supplied in `fields` is checked against `columns_info` with `ok_or_else(|| format!("Column '{}' does not exist in table '{}'", col_name, table))`. Unrecognized columns immediately return an error.
  - Generated columns are rejected with `"Cannot insert into generated column"`.
  - Verified by unit test: `test_insert_rejects_unknown_columns`.

### 4.2. Primary Key WHERE Clause & Validation (Step 3)
- In `build_update_query` and `build_delete_query`:
  - Primary key columns are extracted from table metadata: `let pri_cols: Vec<&TableColumnInfo> = columns_info.iter().filter(|c| c.is_primary()).collect();`.
  - If `pri_cols.is_empty()`, returns `"Table has no primary key; cannot safely identify row"`.
  - If any primary key column has an unsupported type (`has_unsupported_pk_type()`), returns `"Primary key column has unsupported type (FLOAT, DOUBLE, REAL, BLOB, BINARY); table is read-only"`.
  - If `pk_values.is_empty()`, returns `"Refusing to update/delete row without primary key condition"`.
  - If `pk_values.len() != pri_cols.len()`, returns `"Primary key count mismatch: expected N PK column(s), got M"`.
  - Verifies presence of every required primary key in `pk_values`.
  - Preserves metadata column ordering in parameter binding.
  - Verified by unit test: `test_delete_builder_requires_complete_pk`.

### 4.3. Transaction Lifecycle & Row Verification (SEC2-05)
- **Update Transactions:**
  - `update_row` acquires a dedicated pooled connection: `let mut tx = conn.begin().await?;`.
  - Executes `SELECT 1 FROM <table> WHERE <pk> = ? FOR UPDATE`.
  - If not found, rolls back and returns `"Row not found (may have been deleted or modified concurrently)"`.
  - Executes `UPDATE ... LIMIT 1` and commits.
- **Delete Transactions:**
  - `delete_row` acquires a dedicated pooled connection: `let mut tx = conn.begin().await?;`.
  - Executes `DELETE FROM <table> WHERE <pk> = ? LIMIT 1`.
  - Verifies `rows_affected == 1`. If `rows_affected != 1`, calls `tx.rollback().await` and returns an error: `"Delete affected N rows (expected exactly 1); transaction rolled back"`.
  - On single-row success, calls `tx.commit().await`.

### 4.4. Enforcement of Read-Only Columns & Primary Keys (SEC2-03)
- **Read-Only Columns:**
  - Generated, BLOB, binary, and spatial column types cannot be edited (`is_read_only()`).
- **Primary Key Protection:**
  - Primary key columns are blocked from in-place updates by design.
  - In `build_update_query`: returns `Err("Column '<col>' is a primary key and cannot be edited directly (delete and re-insert instead)")`.
  - In UI: `cell-edit-is-pk` is set to `true`.
  - Displays a visible warning banner: `🔒 Primary key: delete and re-insert instead`.
  - `TextInputBox`, `Checkbox`, and `PrimaryButton` ("Save Change") are disabled.
  - Verified by unit test: `test_primary_key_rejected_on_update`.

### 4.5. Concurrency & Stale State Isolation (SEC2-02 & SEC2-04)
- **Delete Snapshot Isolation:**
  - When the user clicks delete on a row, `on_delete_table_row` captures an immutable `DeleteRowSnapshot` with the table name and PK bindings.
  - Opens `ConfirmDialog` with the table and PK values ("Delete row id=42 from users?").
  - `on_confirm_dialog_action` resolves the action strictly using `pending_delete_row.take()`. It never refers back to `row_idx`, making deletion immune to re-sorting, filtering, or pagination while the dialog is open.
  - If the snapshot is missing (`None`), it reports an error via `validate_delete_snapshot` and halts without side effects.
  - Cancel explicitly clears `pending_delete_row`.
  - Verified by unit tests: `test_delete_snapshot_carries_table_name` and `test_validate_delete_snapshot_missing_and_mismatch`.
- **Insert Target Table Guard:**
  - `table_insert_target_table` locks the table name when the modal opens.
  - `on_submit_insert_modal` validates `target_tbl == active_tbl` via `validate_insert_target_table`. If mismatched, aborts immediately without touching the database.
  - Verified by unit test: `test_insert_target_table_mismatch_rejected`.
- **Global Invalidation on Context Switch:**
  - Switching databases (`on_select_database`), switching tables (`on_select_table`), logging out (`on_request_logout`), or dropping objects (`drop_db`, `drop_table`) calls `clear_pending_mutation_state()`, resetting all pending snapshots and closing modals.

### 4.6. Query History Logging
- History queries use parameterized query templates with `?` placeholders:
  - `DELETE FROM \`users\` WHERE \`id\` = ? LIMIT 1`
  - `UPDATE \`users\` SET \`name\` = ? WHERE \`id\` = ? LIMIT 1`
  - `INSERT INTO \`users\` (\`name\`) VALUES (?)`
- Zero cell values or primary key values are interpolated into `history.json`.

---

## 5. Verified Safe List

1. **Parameter Binding:** 100% of data values across `UPDATE`, `INSERT`, and `DELETE` paths are passed via `.bind()` parameters.
2. **Identifier Quoting:** 100% of table and column identifiers are validated and escaped in backticks through `sanitize_identifier()`.
3. **Primary Key Exclusivity:** `WHERE` clauses are built exclusively from introspected primary key columns.
4. **Primary Key Completeness:** Mismatched, missing, or empty primary key value sets are strictly rejected.
5. **Rust-Enforced Read-Only Rules:** Read-only column types (generated, BLOB, binary, spatial) and read-only tables (no primary key, float/double/blob PK) are blocked in backend query builders.
6. **Primary Key Immobility:** Primary keys are protected against in-place mutation in the UI and backend builder.
7. **Isolated Transaction Updates:** `update_row` acquires a dedicated connection, starts a transaction, performs row-level locking via `SELECT ... FOR UPDATE`, updates, and commits.
8. **Isolated Transaction Deletes:** `delete_row` acquires a dedicated connection, starts a transaction, executes deletion, verifies `rows_affected == 1`, and rolls back otherwise.
9. **Safe Found-Rows Handling:** Identical cell values commit cleanly without false-positive error alerts.
10. **Clean History Logging:** Query history records parameterized query templates (`?`) with zero data value leakage.
11. **Stale Index Immunity:** Cell edits and row deletions capture immutable snapshots, decoupling execution from subsequent grid sorting or pagination.
12. **Context-Switch Hygiene:** All pending mutation snapshots and modal states are automatically invalidated on navigation, table switch, or logout.
