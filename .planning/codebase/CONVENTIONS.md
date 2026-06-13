# Coding Conventions

**Analysis Date:** 2026-04-01

## Language Standards

**TypeScript:**
- **Strict Mode:** Enabled in `tsconfig.json`. Every component and utility must use proper type annotations.
- **Interfaces over Types:** Interfaces are prioritized for defining object structures (e.g., `SessionPayload`).

**React:**
- **Client Components:** Use `"use client"` directive for interactive dashboard elements.
- **Hooks:** Functional components with React Hooks are the standard. Use callbacks (`onSelectTable`) for cross-component communication.
- **Modularity:** UI components are colocated with their corresponding routes in `app/dashboard/`.

## SQL & Data Access

**Safety First:**
- **Prepared Statements:** Use parameterized queries via `mysql2/promise`’s `connection.query(query, params)`.
- **Identifier Sanitization:** *Always* use `lib/sanitize.ts` (`sanitizeIdentifier`) for any dynamic table or column name. Never interpolate strings directly into SQL headers.
- **Destructive Detection:** API routes must check for destructive keywords and require explicit `confirmed: true` before execution.

**Connections:**
- **Short-Lived:** Open connections on-demand and close them immediately after the query completes (`lib/db.ts`).
- **Stateless:** Connection details are pulled from the `getSession()` JWT; no long-lived connection pools are shared across requests.

## API Design

- **Next.js Route Handlers:** Grouped by feature set (e.g., `api/data`, `api/schema`).
- **Response Format:** Standard JSON object:
    ```json
    {
      "success": true,
      "data": [...],
      "columns": ["id", "name"],
      "affectedRows": 1
    }
    ```
- **Error Handling:** Always wrap API logic in `try/catch` and return `{ success: false, error: "message" }` with a 500 status code.

## Styling & Design Tokens

- **CSS Modules:** Styles are modularized using `.module.css` files, referencing global design tokens defined in `globals.css`.
- **Theme Constraints:** The styling adheres to a terminal-core dark aesthetic (base color `#0d0d0d`, panels `#1a1a1a`, borders `#2a2a2a`, and text `#e8e8e8`).
- **Typography:** The `JetBrains Mono` font family is loaded and enforced globally for all monospaced elements, headers, and inputs.
- **Decoration Constraints:** All rounded borders are restricted to a maximum of `2px` (using `var(--radius-max)`). Shadows, gradients, blur effects, and glassmorphism are completely avoided.
- **Interactive State Indication:** Electric green (`#00ff9d`) is used strictly as a highlight color for active state indicators, focus borders, and hover text/icons, with no background changes on hover.

---

*Convention analysis: 2026-04-01*
*Update after evolving coding patterns*
