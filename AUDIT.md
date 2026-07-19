# Codebase Architecture Audit & Tauri Desktop Port Readiness Report

This report documents the current architecture of the MySQL GUI Next.js application and assesses its readiness for porting to a Tauri-based cross-platform desktop application.

---

## Project Structure Map

```
├── .agents/                      # Custom AI instructions & helper skills
├── app/                          # Next.js App Router root
│   ├── api/                      # Backend API route handlers (26 routes total)
│   ├── dashboard/                # Main workspace views & components
│   │   ├── dashboard.module.css  # Layout & layout-theme styling for dashboard
│   │   ├── page.tsx              # Dashboard layout coordinator & orchestrator
│   │   └── *.tsx                 # UI modules (SQL Editor, diagrams, mock data, etc.)
│   ├── login/                    # Login portal pages
│   │   ├── login.module.css      # Login page styling
│   │   └── page.tsx              # Credentials form & handshake handler
│   ├── globals.css               # Core styling tokens & neobrutalist system
│   ├── layout.tsx                # Page shell & metadata
│   └── page.tsx                  # Root redirection trigger to login/dashboard
│   
├── lib/                          # Non-component core logic (highly portable)
│   ├── ThemeProvider.tsx         # Next.js context hook for layout styling
│   ├── db.ts                     # mysql2/promise connection management
│   ├── llm.ts                    # Multi-provider chat interface API
│   ├── sanitize.ts               # SQL injection safeguards
│   ├── session.ts                # encrypted JWT cookie utilities (jose)
│   ├── sqlAutocomplete.ts        # Client-side SQL keyword & schema index engine
│   └── sqlExplainer.ts           # Rule-based query translation engine
│   
├── public/                       # Static SVGs & branding assets
├── middleware.ts                 # Next.js route auth guards
├── package.json                  # Dependencies & scripts
└── tsconfig.json                 # TypeScript compiler configuration
```

---

## API Routes Inventory

All API routes run in the standard Node.js runtime (`export const runtime = "nodejs"` / `export const dynamic = "force-dynamic"`).

| Method | Route | Purpose | File Dependency |
|--------|-------|---------|-----------------|
| `POST` | `/api/auth` | Validates MySQL server credentials and initializes session cookie. | `lib/session.ts` |
| `DELETE` | `/api/auth` | Destroys current session cookie (logout). | `lib/session.ts` |
| `GET` | `/api/databases` | Fetches a list of available databases (`SHOW DATABASES`). | `lib/db.ts` |
| `POST` | `/api/databases/create` | Spawns a new database (`CREATE DATABASE`). | `lib/db.ts`, `lib/sanitize.ts` |
| `DELETE` | `/api/databases/drop` | Drops a database (`DROP DATABASE`). Requires confirmation. | `lib/db.ts`, `lib/sanitize.ts` |
| `GET` | `/api/tables` | Lists all tables in the specified database (`SHOW TABLES`). | `lib/db.ts`, `lib/sanitize.ts` |
| `POST` | `/api/tables/create` | Compiles options and runs `CREATE TABLE` query. | `lib/db.ts`, `lib/sanitize.ts` |
| `DELETE` | `/api/tables/drop` | Runs `DROP TABLE` query. Requires confirmation. | `lib/db.ts`, `lib/sanitize.ts` |
| `GET` | `/api/data` | Fetches table rows with pagination and sorting. | `lib/db.ts`, `lib/sanitize.ts` |
| `POST` | `/api/data/insert` | Sanitizes inputs and runs single-row `INSERT`. | `lib/db.ts`, `lib/sanitize.ts` |
| `PATCH` | `/api/data/update` | Runs a single-row `UPDATE` targeting original primary keys/values. | `lib/db.ts`, `lib/sanitize.ts` |
| `DELETE` | `/api/data/delete` | Runs a single-row `DELETE` targeting original keys/values. | `lib/db.ts`, `lib/sanitize.ts` |
| `POST` | `/api/data/generate` | Generates and batches insertion of mock tables rows based on blueprint schema types. | `lib/db.ts`, `lib/sanitize.ts` |
| `GET` | `/api/database-stats` | Aggregates size, engine, and counts from `information_schema.tables`. | `lib/db.ts` |
| `GET` | `/api/structure` | Inspects schema column columns, keys, types (`DESCRIBE`). | `lib/db.ts`, `lib/sanitize.ts` |
| `GET` | `/api/objects` | Inspects views, procedures, and function lists. Fallbacks to `SHOW STATUS` on mysql.proc permission issues. | `lib/db.ts`, `lib/sanitize.ts` |
| `GET` | `/api/schema/relations` | Resolves foreign key dependencies from `information_schema.key_column_usage`. | `lib/db.ts` |
| `GET` | `/api/schema/suggestions` | Generates autocomplete map (tables, columns, type definitions) for client-side matching. | `lib/db.ts` |
| `GET` | `/api/export` | Dumps database structural schemas and row contents into SQL, JSON, or CSV. | `lib/db.ts`, `lib/sanitize.ts` |
| `POST` | `/api/import` | Parses and sequences execution of raw uploaded SQL script. | `lib/db.ts`, `lib/sanitize.ts` |
| `GET` | `/api/server/status` | Reports active running threads and processes (`SHOW FULL PROCESSLIST`). | `lib/db.ts` |
| `GET` | `/api/server/metrics` | Queries metrics (traffic, buffer pools, uptime) for live graphing. | `lib/db.ts` |
| `GET` | `/api/server/slow-log` | Inspects slow-running query logs from `mysql.slow_log`. | `lib/db.ts` |
| `GET` | `/api/users` | Lists users, hosts, and accounts (`mysql.user` with fallback to `USER()`). | `lib/db.ts` |
| `POST` | `/api/nlq` | Transforms natural language prompts to executable SQL statements. | `lib/llm.ts` |
| `GET` | `/api/nlq/status` | Confirms if any compatible LLM endpoint context is configured. | `lib/llm.ts` |
| `POST` | `/api/query` | Executes arbitrary SQL query. Intercepts potentially destructive operations unless confirmed. | `lib/db.ts`, `lib/sanitize.ts` |

---

## DB Connection Lifecycle

Database interactions are processed via `lib/db.ts` utilizing `mysql2/promise` pools:
1. **Pool Caching**: Connections are pooled and indexed in a global `pools` Map (`globalForDb.pools`) to survive hot-reload events in non-production environments.
2. **Context Resolution**: The connection configurations (`host`, `user`, `password`, `port`) are retrieved dynamically from the active user session cookie. The map lookup key compiles these credentials:
   `key = ${host}:${port}:${user}:${password || ""}:${database || ""}`
3. **Pool Settings**:
   * `connectionLimit`: 10
   * `idleTimeout`: 60000ms (Automatic teardown of unused, lingering connections)
   * `enableKeepAlive`: `true` (Sends TCP keep-alive probes every 10000ms delay)
4. **Execution Cycle**: `executeQuery` requests a worker from the resolved pool via `pool.getConnection()`, runs the statement, and executes `connection.release()` inside a `finally` block to return the process cleanly to the pool.

---

## Auth/Session Flow

Security enforces a JWT session flow:
1. **Credentials Validation**: The user provides credentials on `/login`. The server attempts to establish a single instance connection via `mysql.createConnection` to verify host routing and passwords.
2. **JWT Issuance**: Upon verification, `login` (`lib/session.ts`) signs a payload using `jose` with algorithm `HS256`, signed with `SESSION_SECRET` (falling back to `"default-secret-key-change-it-in-production"`).
3. **Cookie Handshake**: The signed token is saved in a cookie named `session` with flags: `httpOnly: true`, and `expires` set to `Date.now() + 2 hours`.
4. **Route Guarding**: `middleware.ts` intercepts requests, parsing the cookie to decrypt the payload. Unauthenticated calls to protected routes (`/dashboard` and `/api/*` excluding `/api/auth`) are redirected back to `/login`.

---

## LLM Abstraction Layer

The system features an LLM interface in `lib/llm.ts` that relies on native environment variable checks instead of bloated library packages:
1. **Provider Precedence**:
   * `ANTHROPIC_API_KEY` → uses Anthropic (`claude-haiku-3-5` via `api.anthropic.com`)
   * `OPENAI_API_KEY` → uses OpenAI (`gpt-4o-mini` via `api.openai.com`)
   * `GEMINI_API_KEY` → uses Gemini (`gemini-1.5-flash` via `generativelanguage.googleapis.com`)
   * `OLLAMA_HOST` → uses Ollama local server (custom `OLLAMA_MODEL` or fallback to `llama3` via `/api/chat`)
2. **Interface Implementation**: `llmComplete` issues standard `fetch` POST requests with the corresponding JSON request schema. No wrapper dependencies (like `openai` or `@google/generative-ai`) are installed.

---

## Dependency Categorization

```json
  "dependencies": {
    "jose": "^6.1.3",
    "lucide-react": "^0.562.0",
    "mysql2": "^3.16.1",
    "next": "^16.2.9",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  }
```

* **Frontend-only**: `lucide-react` (icon components).
* **Server-only**: `mysql2` (requires Node's native TCP `net` socket binding; cannot compile/run inside client browsers).
* **Shared / Client-safe**: `jose` (compiled for Web Cryptography API compliance, works in browser/Edge), `react`, `react-dom`.
* **Dev / Tooling**: `typescript`, `@types/node`, `@types/react`, `@types/react-dom`, `eslint`, `eslint-config-next`.

---

## Next.js-specific vs Portable Code

* **Next.js-specific Elements**:
  * **API Endpoints**: `app/api/**/route.ts` rely completely on the App Router routing structure, middleware runtime support, `NextRequest`, and `NextResponse` helpers.
  * **Session Management**: Cookie setters (`cookies()` from `next/headers`) bind explicitly to Next.js middleware and Server Component execution contexts.
* **Portable Elements (Re-usable in Vite + React)**:
  * **UI / Components**: Almost all workspace modules (e.g., `SqlEditor.tsx`, `PerformanceDashboard.tsx`, `DiagramView.tsx`) declare `"use client"` and communicate with endpoints entirely using the browser's native `fetch` API.
  * **State & Layout**: Modules use traditional React state hooks (`useState`, `useEffect`, `useRef`).
  * **Helper Libraries**: `lib/sqlAutocomplete.ts` (autocomplete scoring logic) and `lib/sqlExplainer.ts` (custom AST-like tokenizer and rule explainer) are written in pure TypeScript and have zero Node.js/Next.js environment coupling.

---

## Env Vars & Config

* `SESSION_SECRET`: The encryption key for signing session tokens.
* `ANTHROPIC_API_KEY`: API access token for Claude models.
* `OPENAI_API_KEY`: API access token for GPT models.
* `GEMINI_API_KEY`: API access token for Gemini models.
* `OLLAMA_HOST`: The endpoint URL hosting the local Ollama daemon.
* `OLLAMA_MODEL`: Target model configuration for local Ollama completion queries.

---

## Issues / TODOs / Dead Code

1. **Double Sanitization Backtick Bug**:
   * Found in [export/route.ts:47](file:///home/coes/Projects/MYSQL%20GUI/app/api/export/route.ts#L47) and [data/generate/route.ts:20](file:///home/coes/Projects/MYSQL%20GUI/app/api/data/generate/route.ts#L20):
     `const colNames = columns.map(c => \`\\\`${sanitizeIdentifier(c)}\\\`\`).join(", ");`
     `sanitizeIdentifier` already encapsulates the output in backticks. Wrapping it in backticks again causes double-backticking (e.g. `` `\`column_name\`` ``), raising syntax exceptions on execution.
2. **Brittle SQL Script Import Splitter**:
   * Found in [import/route.ts:25](file:///home/coes/Projects/MYSQL%20GUI/app/api/import/route.ts#L25):
     Uses a simplified regex split: `.split(/;(?=(?:[^']*'[^']*')*[^']*$)/)`. This easily crashes or splits incorrectly when statements include double quotes (`"`), backslashes, escape sequences, or multi-line comment patterns containing semicolons.
3. **Dead Session Refresh Logic**:
   * `lib/session.ts` exposes `updateSession` to extend session cookies, but this function is **never invoked** inside `middleware.ts`. Users are forcibly logged out after 2 hours of continuous active use.
4. **Database-specific Connection Pools**:
   * `lib/db.ts` keys pools by database name. Switching databases spawns an entirely new `mysql.Pool` setup instead of switching contexts on a single pool connection (`USE database;` or explicit queries), which consumes excessive memory and limits pool efficiency.

---

## Tauri Port Readiness Summary

Converting this application into a cross-platform desktop Tauri application is highly feasible because the entire UI layer is composed of portable React client components communicating via REST patterns.

```mermaid
flowchart TD
    subgraph Current Architecture (Next.js)
        UI[React Frontend] -->|HTTP Fetch| API[Next.js API Routes]
        API -->|mysql2/promise| DB[(MySQL Server)]
    end
    subgraph Proposed Tauri Architecture
        TUI[React Frontend in Vite] -->|Tauri IPC invoke| IPC[Tauri Rust Commands]
        IPC -->|sqlx / mysql backend| TDB[(MySQL Server)]
    end
```

### What Maps Cleanly
* **Frontend Components**: All files in `app/dashboard` and `app/login` (excluding Next.js routing boundaries and `.module.css` imports depending on preference, though CSS modules are supported in Vite).
* **Core Helpers**: `lib/sqlExplainer.ts` and `lib/sqlAutocomplete.ts` can be imported directly into the Vite React client since they contain no server bindings.

### What Needs Rewriting
* **Next.js Routing**: Replace App Router directory conventions with standard client-side routing (e.g., `react-router-dom` or simple conditional state tabs) within a single-page application (SPA) wrapper.
* **Authentication**: Desktop apps should maintain connection credentials locally (stored securely in state or operating system keychain APIs) rather than writing encrypted JWTs to HTTP-only cookies.
* **API Endpoints**: Eliminate HTTP endpoints (`app/api/*`). Frontend operations should map directly to Tauri commands via IPC (`invoke()`).

### Rust Backend Replacements
* **Database Connections**: Replace `mysql2/promise` with a Rust database driver like `sqlx` or `mysql` crate running within the Tauri Core thread.
* **LLM Requests**: Port the logic of `lib/llm.ts` to Rust using native HTTP client crates (`reqwest`) or handle the API calls directly inside the React client context (since desktop apps do not need to hide keys behind intermediate server endpoints if user configures them locally).
