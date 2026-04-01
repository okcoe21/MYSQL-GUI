# Technology Stack

**Analysis Date:** 2026-04-01

## Languages

**Primary:**
- TypeScript 5.x - All application code and API routes.
- React 18.x - UI components and frontend logic.

**Secondary:**
- JavaScript (ESM) - Configuration files (`next.config.mjs`, `eslint.config.mjs`).
- CSS - Vanilla CSS Modules for styling.

## Runtime

**Environment:**
- Node.js 20.x+ - Server-side execution and build environment.
- Browser - Client-side React execution.

**Package Manager:**
- npm 10.x
- Lockfile: `package-lock.json` present.

## Frameworks

**Core:**
- Next.js 14.2.15 - Full-stack React framework (App Router).

**Testing:**
- None detected (no `tests/` directory or testing dependencies in `package.json`).

**Build/Dev:**
- TypeScript Compiler - Type checking and transpilation.
- ESLint 9.x - Linting and code quality.

## Key Dependencies

**Critical:**
- `mysql2` 3.16.x - MySQL client for Node.js (Promise-based).
- `jose` 6.1.x - JWT signing and verification for session management.
- `lucide-react` 0.562.x - Icon library for the UI.

**Infrastructure:**
- `next` 14.2.xx - Routing, SSR, and API handling.

## Configuration

**Environment:**
- `.env.local` - Local environment variables.
- Required keys: `SESSION_SECRET` (for JWT encryption).

**Build:**
- `next.config.mjs` - Next.js framework configuration.
- `tsconfig.json` - TypeScript compiler options.
- `eslint.config.mjs` - ESLint rules.

## Platform Requirements

**Development:**
- Node.js installed locally.
- MySQL server (local or remote) for database operations.

**Production:**
- Vercel or any Node.js compatible hosting (e.g., Docker, VPS).
- Environment variables must be set on the host.

---

*Stack analysis: 2026-04-01*
*Update after major dependency changes*
