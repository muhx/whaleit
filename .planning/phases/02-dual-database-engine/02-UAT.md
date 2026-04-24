---
status: complete
phase: 02-dual-database-engine
source: [02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-05-SUMMARY.md, 02-06-SUMMARY.md]
started: 2026-04-24T07:55:50Z
updated: 2026-04-24T08:15:36Z
note: |
  Phase originally shipped dual SQLite+PostgreSQL engines. On 2026-04-24 the
  project pivoted to PostgreSQL-only (commit a5f0515e removed SQLite). UAT
  rewritten as PG-only — SQLite backward-compat and parity tests are obsolete.
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start — Server Boots With PostgreSQL

expected: Fresh start of postgres (docker compose) + whaleit-server against a
clean DATABASE_URL. Server boots, migrations apply on first run, server serves
HTTP requests. No errors in logs. result: issue reported: "When accessing
/api/health got 401" severity: major

### 2. Migrations Create All Tables On Fresh PG Database

expected: Drop/recreate a fresh PG database, start the server, let migrations
run. Connect via `psql` and run `\dt`. All expected tables from the
`20260101000000_initial_schema` migration are listed (accounts, activities,
assets, quotes, portfolio_snapshots, etc.) with FKs and unique constraints
intact. No migration errors in server logs. result: pass

### 3. Docker Compose Brings Up Whaleit + Postgres

expected: From a clean state: `WF_PG_PASSWORD=devpass docker compose up`. Both
`postgres` and `whaleit` services reach healthy. Postgres passes `pg_isready`,
whaleit waits via `depends_on`, then boots and serves requests on its HTTP port.
Stopping and restarting preserves data in the PG volume. result: issue reported:
| Frontend docker build stage failed (exit code 1). Rollup error in
apps/frontend/src/addons/addons-runtime-context.ts:28: "getExchangeRates" is not
exported by "src/adapters/web/index.ts". Docker compose cannot bring up the
stack because the frontend image build fails. severity: blocker

## Summary

total: 3 passed: 1 issues: 2 pending: 0 skipped: 0 blocked: 0

## Gaps

- truth: "/api/health responds 200 without authentication (public endpoint for
  orchestrator health checks)" status: failed reason: "User reported: When
  accessing /api/health got 401" severity: major test: 1 artifacts: [] # Filled
  by diagnosis missing: [] # Filled by diagnosis

- truth: "`docker compose up` builds the frontend image and brings the whole
  stack healthy" status: failed reason: | User reported: Frontend docker build
  failed at Dockerfile.frontend:14 (`pnpm --filter frontend... build`). Rollup
  error:
  `"getExchangeRates" is not exported by "src/adapters/web/index.ts",   imported by "src/addons/addons-runtime-context.ts"`
  — missing re-export in the web adapter barrel. (`getExchangeRates` is defined
  in `apps/frontend/src/adapters/shared/exchange-rates.ts` but not re-exported
  from `apps/frontend/src/adapters/web/index.ts`.) severity: blocker test: 3
  artifacts: [] # Filled by diagnosis missing: [] # Filled by diagnosis
