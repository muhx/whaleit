---
phase: 02-dual-database-engine
fixed_at: 2026-04-22T13:30:00Z
review_path: .planning/phases/02-dual-database-engine/02-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 02: Code Review Fix Report

**Fixed at:** 2026-04-22T13:30:00Z
**Source review:** `.planning/phases/02-dual-database-engine/02-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 8
- Fixed: 8
- Skipped: 0

## Fixed Issues

### WR-01: Alternative asset stubs silently succeed instead of returning errors

**Files modified:** `crates/storage-postgres/src/assets/alternative_asset.rs`
**Commit:** `b6db671b`
**Applied fix:** Changed all four trait methods (`delete_alternative_asset`, `update_asset_metadata`, `update_asset_details`, `find_liabilities_linked_to`) to return `Err(Error::Database(DatabaseError::Internal(...)))` matching the pattern used in `app_sync.rs`, instead of silently returning `Ok(())`/`Ok(Vec::new())`.

### WR-02: Fragile enum serialization with `trim_matches('"')` in import_run

**Files modified:** `crates/storage-postgres/src/sync/import_run.rs`
**Commit:** `865bb0b3`
**Applied fix:** Added `enum_to_db_string()` helper that safely round-trips through serde (`to_string` → `from_str::<String>`) instead of `trim_matches('"')`. Replaced all 6 occurrences of the fragile pattern across both core and connect trait implementations.

### WR-03: `provider_config::json` cast may fail at runtime

**Files modified:** `crates/storage-postgres/src/custom_provider/repository.rs`
**Commit:** `96c9f54e`
**Applied fix:** Added `provider_config IS NOT NULL AND provider_config != ''` guard before the cast, and changed `::json` to `::jsonb` for more robust handling in the asset count query.

### WR-04: Dead `run_sync` helper and trap `unimplemented!()` method

**Files modified:** `crates/storage-postgres/src/ai_chat/repository.rs`
**Commit:** `6d50f314`
**Applied fix:** Removed the unused `run_sync` function (22 lines of dead code) and the `get_conn` method that returned `unimplemented!()` — a runtime panic trap for future developers.

### WR-05: LIKE search pattern doesn't escape wildcards

**Files modified:** `crates/storage-postgres/src/ai_chat/repository.rs`
**Commit:** `794c6b5f`
**Applied fix:** Added wildcard escaping (`\`, `%`, `_`) for user input before interpolation into LIKE pattern, preventing semantic manipulation of search queries.

### WR-06: `EXCLUDED.*` raw SQL strings bypass compile-time column checking

**Files modified:** `crates/storage-postgres/src/portfolio/snapshot/repository.rs`, `crates/storage-postgres/src/portfolio/valuation/repository.rs`
**Commit:** `77aeb5c6`
**Applied fix:** Added documentation comments at all 4 EXCLUDED.* usage sites explaining the limitation and noting that column names must be updated manually if schema changes. Also documented intentional omission of `snapshot_date` from EXCLUDED sets.

### WR-07: `total_value::numeric` cast from TEXT may fail at runtime

**Files modified:** `crates/storage-postgres/src/portfolio/valuation/repository.rs`
**Commit:** `c03477d4`
**Applied fix:** Added PostgreSQL regex guard `total_value ~ '^-?\d'` before the `::numeric` cast to filter out non-numeric values, preventing runtime errors from invalid data.

### WR-08: NaiveDateTime `clone()` on Copy type

**Files modified:** `crates/storage-postgres/src/ai_chat/model.rs`
**Commit:** `3ab0baf1`
**Applied fix:** Removed redundant `.clone()` on `NaiveDateTime` which implements `Copy`.

---

_Fixed: 2026-04-22T13:30:00Z_
_Fixer: OpenCode (gsd-code-fixer)_
_Iteration: 1_
