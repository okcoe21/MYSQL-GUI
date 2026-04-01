# External Integrations

**Analysis Date:** 2026-04-01

## Primary Databases

**MySQL:**
- **Protocol:** TCP (default port 3306).
- **Client:** `mysql2/promise`
- **Purpose:** All database management tasks (queries, schema changes, etc.).
- **Configuration:** Host, user, password, and port are stored in the user's session (JWT cookie).

## Authentication & Security

**JWT Sessions:**
- **Provider:** Internal (`jose` library).
- **Mechanism:** HS256 signed tokens stored in `httpOnly` cookies.
- **Secret Management:** `SESSION_SECRET` from `.env.local`.

## External APIs

**None Detected:**
- The application appears to be a standalone tool for direct MySQL interaction.
- No third-party SaaS integrations found.

## Webhooks

**None Detected:**
- No incoming or outgoing webhooks found.

---

*Integration analysis: 2026-04-01*
*Update after adding external services*
