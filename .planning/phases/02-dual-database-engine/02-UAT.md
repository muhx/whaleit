---
status: complete
phase: 02-dual-database-engine
source: [02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-05-SUMMARY.md, 02-06-SUMMARY.md]
started: 2026-04-24T07:55:50Z
updated: 2026-04-24T15:43:10Z
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
/api/health. On investigation this was a test-authoring error, not a server bug:
the public, unauthenticated health endpoint is `GET /api/v1/healthz` (k8s-style
`z` suffix, defined in apps/server/src/api.rs line 169). `/api/health/*` routes
are part of the in-app Health Center admin feature and are intentionally behind
auth.

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
Stopping and restarting preserves data in the PG volume. result: pass note: |
Verified end-to-end after three fixes uncovered during this UAT session:

- 2477bada: missing web-adapter re-exports (exchange-rates, contribution-limits,
  market-data, custom-provider, alternative-assets) unblocked the frontend
  rollup bundle.
- 017e36f5: Dockerfile.backend was missing `COPY --from=xx / /`, so xx-apk /
  xx-cargo were not on PATH during cross-compile.
- 5b7de8e7: compose.yml backend ran with `read_only: true` but the server writes
  secrets.json to `./data` (→ `/data`). Mounted a named `whaleit-data` volume at
  `/data` and pointed `WF_ADDONS_DIR=/data/addons` so the addons live in the
  same persistent tree. After all three fixes, `docker compose up` brings
  postgres + backend + frontend healthy end-to-end.

## Summary

total: 3 passed: 3 issues: 0 pending: 0 skipped: 0 blocked: 0

## Gaps

- truth: "`docker compose up` builds the frontend image and brings the whole
  stack healthy" status: resolved reason: | Three distinct blockers surfaced in
  the deploy path: 1. Missing re-exports in
  apps/frontend/src/adapters/web/index.ts caused the frontend rollup build to
  fail on `getExchangeRates`, `getContributionLimit`, `getMarketDataProviders`,
  etc. 2. Dockerfile.backend declared `FROM tonistiigi/xx AS xx` but never
  copied its binaries into the build stage, so xx-apk / xx-cargo were not on
  PATH and the backend image build failed with exit 127. 3. compose.yml ran the
  backend with `read_only: true` but the server hardcodes `./data` for secrets
  and addons and crashed with "Failed to create data root directory: Read-only
  file system". severity: blocker test: 3 root_cause: | Items 1 and 2 were
  latent drift: the web adapter barrel and backend Dockerfile had fallen out of
  sync with the tauri adapter and the standard xx-cross-compile pattern
  respectively. Item 3 was a compose config oversight — hardening the container
  without providing the writable path the server expects. artifacts:
  - path: apps/frontend/src/adapters/web/index.ts issue: "Missing re-export
    blocks for five shared modules"
  - path: Dockerfile.backend issue: "Missing `COPY --from=xx / /` after WORKDIR"
  - path: compose.yml issue: "read_only backend with no writable /data volume"
    fix_commits:
  - 2477bada: "fix(frontend): add missing shared-module re-exports to web
    adapter"
  - 017e36f5: "fix(docker): copy xx tools into backend build stage"
  - 5b7de8e7: "fix(compose): mount writable /data volume for backend persistent
    state"
