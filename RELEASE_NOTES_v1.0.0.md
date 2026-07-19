# MySQL GUI v1.0.0 — Initial Release

The first public release of **MySQL GUI** — a beginner-focused, open-source MySQL client that runs entirely in the browser. No installation, no Electron, no fluff.

---

## What's in v1.0.0

### Core Database Management
- Connect to any MySQL server (local or remote) with host/port/user/password
- Browse, create, and drop databases and tables
- Inline row insert, edit, and delete with paginated, sortable table view
- Full schema browser — columns, types, keys, indexes

### SQL Editor
- Write and execute any SQL query
- **Keyword autocomplete** with live table and column hints from your active schema
- **Rule-based query explainer** — get a plain-English breakdown of any query, no AI or API key required
- Query history panel with one-click re-run
- Destructive operation confirmation guard (DROP, DELETE, raw queries)

### AI — Natural Language → SQL
- Type what you want in plain English → get runnable SQL
- **Zero vendor lock-in** — bring your own key for Anthropic, OpenAI, Gemini, or run fully locally with Ollama
- Provider priority is automatic — set whichever key you have

### Schema & Export Tools
- Foreign key relationship diagram
- Views, stored procedures, and function listing
- Export as **SQL dump**, **JSON**, or **CSV**
- Import `.sql` script files
- Mock data generator — instantly fill tables with test data

### Server Monitoring
- Live process list (`SHOW FULL PROCESSLIST`)
- Server metrics dashboard — traffic, buffer pools, uptime graphs
- Slow query log viewer
- User and host account listing

---

## Getting Started

```bash
git clone https://github.com/okcoe21/MYSQL-GUI.git
cd MYSQL-GUI
npm install
cp .env.example .env.local  # add SESSION_SECRET + optional LLM keys
npm run dev
```

Open `http://localhost:3000`, enter your MySQL credentials, done.

Full setup guide in [README.md](./README.md).

---

## What's Next

**v2.0.0** is already in development — a full **desktop app** (Linux AppImage/.deb/.pacman, macOS .dmg, Windows .msi/.exe) built with Tauri + Rust. Download and run, no Node.js required.

After that: multi-query tabs, saved queries, EXPLAIN plan viewer, schema diff tool, and live connection monitor.

---

## Tech

Next.js 16 · TypeScript · mysql2 · jose · CSS Modules · lucide-react

No Prisma. No ORMs. No AI SDKs — just raw fetch to provider APIs.

---

*If this is useful to you, drop a ⭐ — it helps more people find it.*