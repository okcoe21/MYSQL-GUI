# Coding Conventions

**Analysis Date:** 2026-10-03 (v3.0.0 Native Rust + Slint Rewrite)

## Rust Coding Standards

**General Standards:**
- **Edition:** Rust 2021 edition.
- **Formatting:** Standard `rustfmt` formatting (4 spaces indentation).
- **Error Handling:** Use `Result<T, sqlx::Error>` or idiomatic domain errors. Avoid unwraps on user-facing database I/O.
- **Concurrency & State:**
  - Wrap shared application state in `Arc<Mutex<AppState>>`.
  - Perform all database queries and file I/O inside asynchronous tasks spawned via `tokio::spawn`.
  - Never block the GUI main thread. Update Slint properties from async tasks using `slint::invoke_from_event_loop()`.

**Model Mapping:**
- Keep domain models in `src/db/models.rs`.
- Implement clean conversion helpers (e.g., converting SQL row data into `slint::ModelRc<TableRowData>`).

---

## Slint UI Standards

**File Organization:**
- Reusable UI widgets belong in `ui/components/`.
- Screen-level views belong in `ui/views/`.
- Application-wide tokens (colors, font, radii) belong in `ui/theme.slint`.

**Naming Conventions:**
- Properties, callbacks, and element identifiers use **kebab-case** in `.slint` files:
  ```slint
  in-out property <string> db-name: "";
  callback run-query(string);
  ```
- Component names use **PascalCase**:
  ```slint
  export component TableDataView inherits Rectangle { ... }
  ```

**Layout Constraints & Sizing:**
- Layouts (`VerticalLayout`, `HorizontalLayout`) should keep explicit `padding: 0px; spacing: 0px;` when flush edges are intended, as Slint defaults to style-dependent margins otherwise.
- To enable infinite vertical scrolling inside a `Flickable`, always place an unconstrained flexible spacer `Rectangle { }` **inside** the child `VerticalLayout` before its closing brace. This prevents Slint's AOT layout solver from computing a finite `max_height` and clamping the window layout.

---

## Styling & Design Tokens

- **Aesthetic:** Terminal-core dark theme with high-contrast text and electric green highlights.
- **Palette (`Theme`):**
  - Base background: `#0d0d0d`
  - Panel / Card background: `#161616`
  - Header background: `#1a1a1a`
  - Borders: `#262626`
  - Accent / Highlights: `#00ff9d`
  - Text primary: `#f0f0f0`
  - Text muted: `#888888`
- **Typography:** JetBrains Mono / monospace is loaded and enforced across all tables, headers, and code editors.
- **Border Radii:** Minimalist borders with a default radius of `3px`. Excessive rounded corners, shadows, and blur effects are avoided.

---

## SQL & Data Safety

- **Prepared Statements:** Parameterize all values where possible via SQLx query bindings (`sqlx::query().bind(...)`).
- **Identifier Escaping:** Dynamically constructed table or column names must always pass through `sanitize_identifier()` to wrap names in backticks and escape internal backticks.
- **Destructive Operation Guard:** Queries matching `DROP`, `TRUNCATE`, or `DELETE` trigger a confirmation modal before execution.
