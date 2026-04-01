# Technical Concerns

**Analysis Date:** 2026-04-01

## Critical Technical Debt

**Absolute Absence of Tests:**
- The project has no unit, integration, or E2E tests (`0% coverage`).
- **Risk:** New features could break core MySQL functionality (e.g., table deletion, row editing) without warning.

**Version Mismatch:**
- **README** mentions Next.js 15, but `package.json` specifies Next.js 14.2.15.
- **Risk:** Potential confusion during deployment or upgrades; reliance on Next.js 14-specific behaviors.

## Security Concerns

**Basic Destructive Query Detection:**
- `lib/sanitize.ts` tracks strings (`DROP`, `DELETE`, etc.) but doesn't parse the SQL AST.
- **Risk:** Complex queries with these strings in comments or values might trigger false alarms, and obfuscated commands might bypass it.

**Session Secret Handling:**
- `process.env.SESSION_SECRET` has a hardcoded default: `"default-secret-key-change-it-in-production"`.
- **Risk:** If not configured correctly in the `.env.local` file, sessions are vulnerable to tampering.

## Performance & Scalability

**Connection on Every Request:**
- `lib/db.ts` creates a *new connection* for every query and closes it immediately.
- **Risk:** Heavy overhead for multiple small queries (e.g., browsing a table with 50+ rows if pagination is high); no connection pooling across requests.
- **Benefit:** Simplifies stateless management in a multi-user context.

**Pagination Limitations:**
- Standard `mysql2` is used for queries; large tables might suffer from basic `OFFSET` pagination without cursor-based optimizations.

## Fragility & Bugs

**Empty Identifier Scope:**
- `sanitizeIdentifier` throws an error if `name` is empty.
- **Risk:** Some MySQL commands like `USE` might not need backticks on all identifiers; needs careful integration.

**Mock Data Generation:**
- Custom generator `MockDataView.tsx` might be fragile relative to complex table constraints (e.g., unique indices, foreign keys).

---

*Concern analysis: 2026-04-01*
*Update after addressing major debt*
