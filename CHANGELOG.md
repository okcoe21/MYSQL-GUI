# Changelog

All notable changes to MySQL GUI are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [1.0.0] — 2026-07-19

Initial public release of MySQL GUI — a beginner-focused, open-source MySQL client
that runs entirely in the browser with no installation required.

### Added

**Core Database Management**
- Connect to any MySQL server with host, port, user, and password credentials
- Browse, create, and drop databases
- Browse, create, and drop tables with full column/type configuration
- View, insert, update, and delete rows with paginated table view
- Sortable columns and row-level edit/delete actions inline

**SQL Editor**
- Full SQL editor with syntax highlighting
- Execute arbitrary queries with destructive-operation confirmation guard
- Query history panel — browse and re-run previous queries
- SQL keyword autocomplete with table and column hints from the active schema (`lib/sqlAutocomplete.ts`)
- Rule-based query explainer — plain-English breakdown of any SQL statement, no AI required (`lib/sqlExplainer.ts`)

**AI / Natural Language**
- Natural language → SQL via multi-provider LLM abstraction (`lib/llm.ts`)
- Supports Anthropic (claude-haiku-3-5), OpenAI (gpt-4o-mini), Gemini (gemini-1.5-flash), and local Ollama
- Provider selection via environment variables — zero vendor lock-in
- LLM provider status endpoint (`/api/nlq/status`)

**Schema & Structure**
- Schema browser — tables, columns, types, keys, indexes via DESCRIBE
- Foreign key relationship diagram (DiagramView)
- Views, stored procedures, and function listing
- Foreign key dependency resolution from `information_schema`
- Autocomplete schema map generation from live DB

**Export & Import**
- Export databases as SQL dump, JSON, or CSV
- Import raw `.sql` script files
- CSV / JSON export per table result set

**Mock Data**
- Generate and insert mock rows based on column type inference
- Batch insertion with configurable row count

**Server Monitoring**
- Live process list (`SHOW FULL PROCESSLIST`)
- Server metrics dashboard — traffic, buffer pools, uptime graphs
- Slow query log viewer
- Database size and engine stats from `information_schema`
- User and host account listing

**Auth & Security**
- Session-based auth with encrypted JWT cookie (HS256 via `jose`)
- Route guards via Next.js middleware — all `/api/*` and `/dashboard` protected
- SQL injection safeguards via `lib/sanitize.ts` across all identifier inputs
- Destructive operation confirmation layer on DROP, DELETE, and raw query execution

**Design**
- Neobrutalist design system — OLED dark background, #7C3AED purple accent, 2px borders, hard offset box-shadows, zero border-radius
- Typography: JetBrains Mono + Space Grotesk
- CSS Modules throughout — no utility framework dependency

---

## [Upcoming] — v2.0.0

- Tauri-based desktop app for Linux (AppImage, .deb, .pacman), macOS (.dmg), and Windows (.msi, .exe)
- OS keychain storage for connection credentials and LLM API keys
- All 26 API routes ported to native Rust commands via sqlx
- Cross-platform GitHub Actions release pipeline

## [Upcoming] — v2.1.0

- Query history: fuzzy search, timestamps, persist to disk
- Saved queries library with tags and search
- Multi-query tabs in SQL editor

## [Upcoming] — v2.2.0

- EXPLAIN plan viewer — visual query execution tree
- Live connection monitor with auto-refresh and kill-query
- Schema diff / migration preview — compare two DBs, generate ALTER script