# MySQL GUI

A beginner-focused, open-source MySQL client that runs in the browser.
No installation. No Electron. No bloat — just connect and go.

![License](https://img.shields.io/github/license/okcoe21/MYSQL-GUI)
![Version](https://img.shields.io/badge/version-1.0.0-7C3AED)
![Next.js](https://img.shields.io/badge/Next.js-16-black)
![Stars](https://img.shields.io/github/stars/okcoe21/MYSQL-GUI)

---

## What it does

MySQL GUI lets you connect to any MySQL server and manage it through a clean, fast web interface — no desktop app required. Built for learners and developers who want something lighter than TablePlus or DBeaver, and more powerful than phpMyAdmin.

---

## Features

**Database & Table Management**
- Browse, create, and drop databases and tables
- Inline row insert, edit, and delete with paginated table view
- Full schema browser — columns, types, keys, indexes

**SQL Editor**
- Write and run any SQL with syntax highlighting
- Keyword autocomplete with live table/column hints
- Rule-based query explainer — understand any query in plain English, no AI needed
- Query history panel with one-click re-run
- Destructive operation guard (DROP, DELETE prompt for confirmation)

**AI — Natural Language → SQL**
- Type "show me all users who signed up last month" → get runnable SQL
- Supports Anthropic, OpenAI, Gemini, and local Ollama
- Bring your own key — zero vendor lock-in

**Schema Tools**
- Foreign key relationship diagram
- Views, procedures, and function listing
- Export as SQL dump, JSON, or CSV
- Import `.sql` script files
- Mock data generator — fill tables instantly for testing

**Server Monitoring**
- Live process list
- Server metrics dashboard (traffic, buffer pools, uptime)
- Slow query log viewer
- User and host account listing

---

## Getting Started

### Prerequisites

- Node.js 18+
- A running MySQL server (local or remote)

### Run locally

```bash
git clone https://github.com/okcoe21/MYSQL-GUI.git
cd MYSQL-GUI
npm install
cp .env.example .env.local
npm run dev
```

Open [http://localhost:3000](http://localhost:3000), enter your MySQL credentials, and connect.

### Environment variables

```env
# Required
SESSION_SECRET=your-secret-key-change-this

# Optional — enable AI natural language → SQL
# Only one is needed. Priority: Anthropic > OpenAI > Gemini > Ollama
ANTHROPIC_API_KEY=
OPENAI_API_KEY=
GEMINI_API_KEY=
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=llama3
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Framework | Next.js 16 (App Router) |
| Language | TypeScript |
| Database driver | mysql2/promise |
| Auth | JWT via jose |
| AI | Anthropic / OpenAI / Gemini / Ollama (fetch, no SDK) |
| Styling | CSS Modules, neobrutalist design system |
| Icons | lucide-react |

---

## Project Structure

```
app/
├── api/          # 26 API route handlers
├── dashboard/    # All UI modules (SQL editor, diagrams, monitoring, etc.)
└── login/        # Credentials form

lib/
├── db.ts             # mysql2 connection pool management
├── llm.ts            # Multi-provider LLM abstraction
├── sanitize.ts       # SQL injection safeguards
├── session.ts        # JWT cookie utilities
├── sqlAutocomplete.ts # Client-side autocomplete engine
└── sqlExplainer.ts   # Rule-based query explainer
```

---

## Roadmap

| Version | What's coming |
|---|---|
| **v2.0.0** | Desktop app — Linux, macOS, Windows (Tauri + Rust) |
| **v2.1.0** | Multi-query tabs, saved queries library, improved history |
| **v2.2.0** | EXPLAIN plan viewer, live monitor, schema diff tool |

See [CHANGELOG.md](./CHANGELOG.md) for full history.

---

## Contributing

PRs are welcome. Branch off `dev`, not `main`.

```
main      ← stable releases only
dev       ← active development
feature/* ← branch off dev for new features
```

Open an issue before starting large features so we can align on approach.

---

## License

MIT — use it, fork it, build on it.

---

<p align="center">Built by <a href="https://github.com/okcoe21">okcoe21</a></p>