---
phase: 04-transaction-core
plan: 01
subsystem: database
tags: [postgres, diesel, migrations, transactions, sql]
status: complete
completed: 2026-05-01
notes: |
  Task 6 resolved by user (diesel migration run + diesel print-schema).
  Migration up.sql required surgical fix: dropped FK on category_id (taxonomy_categories
  has composite PK (id, taxonomy_id) — matches asset_taxonomy_assignments convention,
  service-layer integrity via TaxonomyService::delete_category). Seed ON CONFLICT
  changed from (id) to (id, taxonomy_id). All 4 new table! macros present in
  schema.rs; cargo check --workspace green (commit bf5fac4b).

# Dependency graph
requires:
  - phase: 03-bank-accounts-credit-cards
    provides: accounts table + NUMERIC(20,8) money column precedent (D-10)
  - phase: 01-initial-schema
    provides:
      taxonomies + taxonomy_categories tables (FK targets for category seeding)
provides:
  - PostgreSQL transactions table with full transfer/split/FX/idempotency schema
  - transaction_splits table for per-category split rows
  - payee_category_memory table for D-12 merchant→category learning
  - v_transactions_with_running_balance VIEW (window function, TXN-09)
  - transaction_csv_templates table for D-16/17/18 user-saved import templates
  - sys_taxonomy_transaction_categories taxonomy seeded with 10 default
    categories
  - crates/storage-postgres/src/transactions/ module stub wired into lib.rs
affects:
  - 04-02 (crates/core transactions model reads schema)
  - 04-03 (PgTransactionRepository uses Diesel table! macros from schema.rs)
  - 04-04 (import wizard uses templates table)
  - all downstream Phase 4 plans

# Tech tracking
tech-stack:
  added: [pg_trgm extension (GIN payee search index)]
  patterns:
    - Separate migration per logical feature group (transactions_initial +
      csv_templates)
    - ON CONFLICT (id) DO NOTHING for idempotent seed rows
    - transfer_leg_role column enables pure-aggregate running-balance VIEW
      without self-join
    - DATABASE_URL-gated async tests that skip gracefully when env var absent

key-files:
  created:
    - crates/storage-postgres/migrations/20260501000000_transactions_initial/up.sql
    - crates/storage-postgres/migrations/20260501000000_transactions_initial/down.sql
    - crates/storage-postgres/migrations/20260501010000_transaction_csv_templates/up.sql
    - crates/storage-postgres/migrations/20260501010000_transaction_csv_templates/down.sql
    - crates/storage-postgres/src/transactions/mod.rs
    - crates/storage-postgres/src/transactions/migration_tests.rs
  modified:
    - crates/storage-postgres/src/lib.rs (added pub mod transactions;)

key-decisions:
  - "transfer_leg_role TEXT (SOURCE/DESTINATION) added to transactions to enable
    pure-aggregate running-balance VIEW without self-join (mirrors RESEARCH
    §NOTE block at line 679)"
  - "down.sql tests use include_str! structural check instead of executing
    teardown against shared test DB — avoids breaking parallel tests"
  - "transaction_csv_templates is a separate migration (20260501010000) so
    diesel print-schema covers both tables in one pass"

patterns-established:
  - "Migration round-trip tests: DATABASE_URL-gated async tests with
    skip-on-missing-env pattern (mirrors accounts/migration_tests.rs)"
  - "Seed idempotency: stable string IDs + ON CONFLICT (id) DO NOTHING for
    taxonomy + category rows"

requirements-completed: [TXN-01, TXN-04, TXN-05, TXN-06, TXN-07, TXN-08, TXN-09]

# Metrics
duration: ~25min (tasks 1-5 complete; task 6 pending user action)
completed: 2026-04-30
---

# Phase 04 Plan 01: Database Migration + Schema Summary

**PostgreSQL schema for transactions ledger: 3 tables + 1 VIEW + 2 migrations +
seeded system taxonomy with 10 categories, blocked on `diesel print-schema`
regeneration by user**

## Performance

- **Duration:** ~25 min (Tasks 1-5 complete; Task 6 blocked on user action)
- **Started:** 2026-04-30
- **Completed:** In progress — halted at Task 6 checkpoint
- **Tasks:** 5/6 complete
- **Files modified:** 7

## Accomplishments

- Wrote `20260501000000_transactions_initial/up.sql`: `transactions` table (25
  columns, 5 CHECK constraints), `transaction_splits`, `payee_category_memory`,
  `v_transactions_with_running_balance` VIEW, 8 indexes, system taxonomy seed
  with 10 categories
- Wrote companion `down.sql` reversing all DDL in dependency order
- Wrote `20260501010000_transaction_csv_templates/up.sql` + `down.sql` for
  D-16/17/18 globally-scoped templates (JSONB mapping, TEXT[] header_signature,
  no account_id)
- Stubbed `crates/storage-postgres/src/transactions/mod.rs` + wired
  `pub mod transactions;` into `lib.rs`; `cargo check` green
- Wrote 7 async migration tests covering table existence, VIEW existence, seed
  counts, idempotency, structural down.sql verification, templates table +
  columns, templates round-trip

## Task Commits

1. **Task 1: Write up.sql — tables + view + seed** — `646790ed` (feat)
2. **Task 2: Write down.sql — reverse DDL** — `a732dfc0` (feat)
3. **Task 3: transaction_csv_templates migration** — `bdddec6f` (feat)
4. **Task 4: Stub transactions module + wire lib.rs** — `9f2ba396` (feat)
5. **Task 5: Write migration round-trip tests** — `7a45fbfd` (feat)

**Plan metadata (in-progress):** see this SUMMARY commit.

## Files Created/Modified

- `crates/storage-postgres/migrations/20260501000000_transactions_initial/up.sql`
  — Full transactions schema DDL + seed
- `crates/storage-postgres/migrations/20260501000000_transactions_initial/down.sql`
  — Reverse DDL in dependency order
- `crates/storage-postgres/migrations/20260501010000_transaction_csv_templates/up.sql`
  — CSV templates table (D-16/17/18)
- `crates/storage-postgres/migrations/20260501010000_transaction_csv_templates/down.sql`
  — Drop templates table
- `crates/storage-postgres/src/transactions/mod.rs` — Module root
  with #[cfg(test)] migration_tests gate
- `crates/storage-postgres/src/transactions/migration_tests.rs` — 7 async smoke
  tests
- `crates/storage-postgres/src/lib.rs` — Added `pub mod transactions;`
  (alphabetical between taxonomies and users)

## Decisions Made

- `transfer_leg_role TEXT CHECK (SOURCE/DESTINATION)` added to `transactions` to
  make `v_transactions_with_running_balance` a pure aggregate VIEW without a
  self-join (per RESEARCH §NOTE at lines 679-689).
- Down-migration tests use `include_str!` structural assertions rather than
  executing `down.sql` against the shared test DB — avoids destroying tables
  that other tests rely on in a shared DATABASE_URL environment.
- Separate migration timestamps (`20260501000000` vs `20260501010000`) so both
  migrations apply in a single `diesel migration run` pass before
  `print-schema`.

## Deviations from Plan

None — plan executed exactly as written for Tasks 1-5.

## Issues Encountered

None for Tasks 1-5.

## Blocked: Task 6

Task 6 requires the user to run `diesel print-schema` against a live Postgres
instance. This cannot be automated by the executor (requires DB connection +
local Diesel CLI).

**User action required:**

```
docker compose up -d postgres
export DATABASE_URL=postgres://whaleit:whaleit@localhost:5432/whaleit
diesel migration run --migration-dir crates/storage-postgres/migrations
diesel print-schema --database-url "$DATABASE_URL" > crates/storage-postgres/src/schema.rs
cargo check --workspace
cargo test -p whaleit-storage-postgres --features integration
```

Verification:
`grep "table! { transactions" crates/storage-postgres/src/schema.rs` should
return a match.

## Next Phase Readiness

- Once schema.rs is regenerated (Task 6), Plans 04-02 and 04-03 can proceed to
  add Rust types and Diesel repository.
- `pub mod transactions;` already wired in lib.rs — no further lib.rs edits
  needed from 04-03.

---

_Phase: 04-transaction-core_ _Plan status: in-progress (blocked on Task 6 user
action)_ _Completed tasks committed: 5/6_
