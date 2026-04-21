---
phase: 02-dual-database-engine
plan: 06
subsystem: storage-postgres
tags: [postgres, diesel-async, repository, fx, market-data, portfolio, snapshot, valuation]
dependency_graph:
  requires: ["02-05"]
  provides: [fx-pg-repo, market-data-pg-repo, snapshot-pg-repo, valuation-pg-repo]
  affects: [whaleit-storage-postgres, whaleit-server]
tech_stack:
  added: [diesel-async, postgres-DISTINCT-ON, ON-CONFLICT-DO-UPDATE]
  patterns: [pg-repo-pattern, model-with-from-impls]
key_files:
  created:
    - crates/storage-postgres/src/fx/model.rs
    - crates/storage-postgres/src/portfolio/snapshot/model.rs
    - crates/storage-postgres/src/portfolio/valuation/model.rs
  modified:
    - crates/storage-postgres/src/fx/repository.rs
    - crates/storage-postgres/src/fx/mod.rs
    - crates/storage-postgres/src/market_data/repository.rs
    - crates/storage-postgres/src/market_data/model.rs
    - crates/storage-postgres/src/portfolio/snapshot/repository.rs
    - crates/storage-postgres/src/portfolio/snapshot/mod.rs
    - crates/storage-postgres/src/portfolio/valuation/repository.rs
    - crates/storage-postgres/src/portfolio/valuation/mod.rs
decisions:
  - "Used PG DISTINCT ON instead of ROW_NUMBER() window function for latest-per-group queries — more idiomatic and efficient"
  - "Used diesel::dsl::sql('EXCLUDED.column') in ON CONFLICT DO UPDATE SET to work around Diesel's batch upsert limitations"
  - "Shared QuoteDB model between fx and market_data modules via pub(crate) re-export from fx::model"
  - "Chunked batch upserts at 100 rows (snapshots) and 1000 rows (valuations) to avoid oversized queries"
  - "Used native NaiveDate for PG Date columns (no string conversion needed unlike SQLite TEXT dates)"
metrics:
  duration: 45m
  completed: 2026-04-22
---

# Phase 02 Plan 06: Replace Remaining PG Repository Stubs Summary

Replaced all stub implementations in 4 PostgreSQL repository modules (fx, market_data, portfolio/snapshot, portfolio/valuation) with real diesel-async queries, using PG-native DISTINCT ON for latest-per-group queries and ON CONFLICT DO UPDATE for upserts.

## What Was Done

### Task 1: FX and Market Data PG Repositories
- Created `fx/model.rs` with `QuoteDB` (Queryable, QueryableByName, Selectable), `NewQuoteDB` (Insertable, AsChangeset), and `From` impls for bidirectional conversion
- Rewrote `fx/repository.rs` — all 11 `FxRepositoryTrait` methods with real diesel-async queries
- Updated `fx/mod.rs` to export model as `pub(crate)` for market_data sharing
- Rewrote `market_data/repository.rs` — all `QuoteStore` (20+ methods) and `ProviderSettingsStore` (3 methods) implementations
- Updated `market_data/model.rs` with `From<MarketDataProviderSettingDB>` and `AsChangeset` on `UpdateMarketDataProviderSettingDB`

### Task 2: Portfolio Snapshot and Valuation PG Repositories
- Created `portfolio/snapshot/model.rs` with `AccountStateSnapshotDB` and bidirectional `From` impls
- Rewrote `portfolio/snapshot/repository.rs` — all 17 `SnapshotRepositoryTrait` methods with anchor date preservation logic
- Created `portfolio/valuation/model.rs` with `DailyAccountValuationDB` and bidirectional `From` impls
- Rewrote `portfolio/valuation/repository.rs` — all 7 `ValuationRepositoryTrait` methods

## Commits

| Commit | Description |
|--------|-------------|
| `5ffc4a35` | feat(02-06): implement FX and market_data PG repositories with real diesel-async queries |
| `fb7798bc` | feat(02-06): implement portfolio snapshot and valuation PG repositories with real diesel-async queries |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed move-after-borrow errors in upsert_quotes**
- **Found during:** Task 1
- **Issue:** Strings created via `.to_string()` were moved into `.values()` then reused in `.on_conflict().do_update().set()`
- **Fix:** Clone strings before the `.values()` call, use clones in `.set()`
- **Files:** `crates/storage-postgres/src/market_data/repository.rs`
- **Commit:** `5ffc4a35`

**2. [Rule 1 - Bug] Fixed heterogeneous Vec type in update_provider**
- **Found during:** Task 1
- **Issue:** Building `Vec` of different Diesel column expressions (`Eq<priority, i32>` vs `Eq<enabled, bool>`) is not possible in Rust's type system
- **Fix:** Used `AsChangeset` derive on `UpdateMarketDataProviderSettingDB` and called `.set(&changes_db)` directly instead of building Vec
- **Files:** `crates/storage-postgres/src/market_data/model.rs`, `repository.rs`
- **Commit:** `5ffc4a35`

**3. [Rule 1 - Bug] Fixed dynamic SQL bind type mismatch in latest_batch**
- **Found during:** Task 1
- **Issue:** Diesel's `UncheckedBind` returns different types for different bind counts, so conditional `.bind()` chaining produces type mismatches
- **Fix:** Split into two separate `if/else` branches, each with their own complete query chain
- **Files:** `crates/storage-postgres/src/market_data/repository.rs`
- **Commit:** `5ffc4a35`

**4. [Rule 3 - Blocking] Missing imports in valuation model and repository**
- **Found during:** Task 2
- **Issue:** Missing `TimeZone` import for `Utc.from_utc_datetime`, missing `FromStr` and `Decimal` imports for parsing
- **Fix:** Added missing imports
- **Files:** `crates/storage-postgres/src/portfolio/valuation/model.rs`, `repository.rs`
- **Commit:** `fb7798bc`

## Verification

- `cargo check -p whaleit-storage-postgres` — **0 errors**, 27 warnings (all pre-existing)
- `cargo check -p whaleit-server` — **0 errors**
- No "not yet implemented" strings remain in fx, market_data, snapshot, or valuation modules
- All 4 repository modules now have real diesel-async query implementations

## Key Technical Decisions

1. **PG DISTINCT ON** — Used instead of `ROW_NUMBER() OVER (PARTITION BY ...)` for latest-per-group queries. More idiomatic PostgreSQL, often more efficient.
2. **`diesel::dsl::sql("EXCLUDED.column")`** — Workaround for Diesel's batch upsert limitations. Diesel's typed `ON CONFLICT` API doesn't support batch operations cleanly, so raw SQL strings for the EXCLUDED virtual table.
3. **Shared QuoteDB** — Both `fx` and `market_data` modules use the same `quotes` table. Created `QuoteDB` in `fx/model.rs` and re-exported from `market_data/model.rs` via `pub use crate::fx::{QuoteDB, NewQuoteDB}`.
4. **Native NaiveDate** — PG `Date` columns map directly to `chrono::NaiveDate` in diesel, unlike SQLite's TEXT storage which requires string conversion.

## Self-Check: PASSED

All 12 files verified present. Both commits (`5ffc4a35`, `fb7798bc`) found in git log.
