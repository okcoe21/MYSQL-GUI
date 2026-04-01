# Directory Structure

**Analysis Date:** 2026-04-01

## Folder Layout

```bash
.
├── app/                 # Next.js App Router root
│   ├── api/             # Backend route handlers (thin MySQL adapter)
│   │   ├── auth/        # Login/Logout sessions
│   │   ├── data/        # Table data browsing / search
│   │   ├── database-stats/ # Performance metrics
│   │   ├── databases/   # CRUD for MySQL databases
│   │   ├── export/      # SQL/JSON/CSV export logic
│   │   ├── import/      # Large script importer
│   │   ├── objects/     # Views, Procedures, Functions
│   │   ├── query/       # Raw SQL editor runner
│   │   ├── schema/      # Table schema modification (ALTER)
│   │   ├── server/      # Global server status, slow logs, users
│   │   ├── structure/   # Table column/index introspection
│   │   └── tables/      # CRUD for database tables
│   ├── dashboard/       # Main User Interface components
│   │   └── Sidebar.tsx  # Global navigation sidebar
│   ├── login/           # Login screen (entry point)
│   ├── globals.css      # Base CSS and theme tokens
│   ├── layout.tsx       # Root layout with ThemeProvider
│   └── page.tsx         # Redirect to login logic
├── lib/                 # Core business and utility logic
│   ├── ThemeProvider.tsx # Dark mode management
│   ├── db.ts           # MySQL connection pool and query executor
│   ├── sanitize.ts     # SQL safety and identifier escaping
│   └── session.ts      # JWT session encryption/decryption
├── public/              # Static assets (favicons, manifest)
└── .planning/           # GSD project documentation (this folder)
```

## Key Files

**Core Configuration:**
- `next.config.mjs`: Next.js framework settings.
- `package.json`: Project manifest and dependencies.
- `tsconfig.json`: TypeScript compiler rules.
- `.env.local`: Environment secrets (not in Git).

**Orchestration:**
- `app/dashboard/page.tsx`: The primary UI state container and router.
- `middleware.ts`: Handles session validation and login redirects.

## Organization Patterns

- **Component Colocation:** UI components (`.tsx`) are located within their corresponding route folder (e.g., `app/dashboard/`).
- **Feature Grouping:** API routes are grouped logically by the domain they manage (e.g., `api/tables`, `api/queries`).
- **Shared Logic:** Purely functional or cross-cutting logic (DB, Auth, Sanitization) is isolated in `lib/`.

---

*Structure analysis: 2026-04-01*
*Update after major file reorganization*
