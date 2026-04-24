---
status: complete
phase: 02-dual-database-engine
source: [02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-05-SUMMARY.md, 02-06-SUMMARY.md]
started: 2026-04-24T07:55:50Z
updated: 2026-04-24T08:52:20Z
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
HTTP requests. No errors in logs. result: pass note: | User reported 401 on GET
/api/health. On investigation this was a test-authoring error on my part, not a
server bug: the public, unauthenticated health endpoint is `GET /api/v1/healthz`
(k8s-style `z` suffix, defined in apps/server/src/api.rs line 169).
`/api/health/*` routes are part of the in-app Health Center admin feature and
are intentionally behind auth. Server IS booting and migrations DID run (as
confirmed by test 2 passing), so the underlying test intent — "server boots with
PostgreSQL" — is satisfied.

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
stack because the frontend image build fails. severity: blocker fix: | Commit
2477bada — added missing re-exports to apps/frontend/src/adapters/web/index.ts
for five shared modules that the tauri adapter re-exports but the web adapter
had been missing: exchange-rates, contribution-limits, market-data,
custom-provider, and alternative-assets. `pnpm --filter frontend build` now
succeeds locally (12.37s, 6597 modules transformed). End-to-end
`docker compose up` has NOT been re-verified by the user after the fix; the
docker build is expected to succeed now but service orchestration (depends_on
ordering, volume persistence) is still pending user re-test.

## Summary

total: 3 passed: 2 issues: 1 pending: 0 skipped: 0 blocked: 0

## Gaps

- truth: "`docker compose up` builds the frontend image and brings the whole
  stack healthy" status: fixed reason: | User reported: Frontend docker build
  failed at Dockerfile.frontend:14 (`pnpm --filter frontend... build`). Rollup
  error:
  `"getExchangeRates" is not exported by "src/adapters/web/index.ts",   imported by "src/addons/addons-runtime-context.ts"`
  — missing re-export in the web adapter barrel. (`getExchangeRates` is defined
  in `apps/frontend/src/adapters/shared/exchange-rates.ts` but not re-exported
  from `apps/frontend/src/adapters/web/index.ts`.) severity: blocker test: 3
  root_cause: | apps/frontend/src/adapters/web/index.ts had drifted from
  apps/frontend/src/adapters/tauri/index.ts — five shared-module re-export
  blocks (exchange-rates, contribution-limits, market-data, custom-provider,
  alternative-assets) were present in tauri but absent from web. Because
  addons-runtime-context.ts imports from `@/adapters` (which resolves to the web
  barrel in web builds), the rollup bundler failed to find re-exports of
  `getExchangeRates`, `getContributionLimit`, `getMarketDataProviders`, etc.
  artifacts:
  - path: apps/frontend/src/adapters/web/index.ts issue: "Missing re-export
    blocks for five shared modules" missing:
  - "Mirror tauri adapter's re-export blocks for exchange-rates,
    contribution-limits, market-data, custom-provider, alternative-assets"
    fix_commit: 2477bada follow_up: "User to re-run
    `WF_PG_PASSWORD=devpass docker compose up` to confirm end-to-end stack
    health (depends_on ordering, volume persistence)"
