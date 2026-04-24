---
phase: 02-dual-database-engine
reviewed: 2026-04-22T12:00:00Z
depth: standard
files_reviewed: 27
files_reviewed_list:
  - crates/storage-postgres/src/sync/engine_ports.rs
  - crates/storage-postgres/src/sync/app_sync.rs
  - crates/storage-postgres/src/sync/import_run.rs
  - crates/storage-postgres/src/sync/platform.rs
  - crates/storage-postgres/src/sync/state.rs
  - crates/storage-postgres/src/sync/mod.rs
  - crates/storage-postgres/src/ai_chat/repository.rs
  - crates/storage-postgres/src/ai_chat/model.rs
  - crates/storage-postgres/src/custom_provider/repository.rs
  - crates/storage-postgres/src/custom_provider/model.rs
  - crates/storage-postgres/src/assets/alternative_asset.rs
  - apps/server/src/main_lib.rs
  - apps/server/src/api/device_sync_engine.rs
  - apps/server/src/api/holdings/handlers.rs
  - crates/storage-postgres/src/fx/model.rs
  - crates/storage-postgres/src/fx/repository.rs
  - crates/storage-postgres/src/fx/mod.rs
  - crates/storage-postgres/src/market_data/repository.rs
  - crates/storage-postgres/src/market_data/model.rs
  - crates/storage-postgres/src/portfolio/snapshot/model.rs
  - crates/storage-postgres/src/portfolio/snapshot/repository.rs
  - crates/storage-postgres/src/portfolio/snapshot/mod.rs
  - crates/storage-postgres/src/portfolio/valuation/model.rs
  - crates/storage-postgres/src/portfolio/valuation/repository.rs
  - crates/storage-postgres/src/portfolio/valuation/mod.rs
findings:
  critical: 0
  warning: 8
  info: 5
  total: 13
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-04-22T12:00:00Z **Depth:** standard **Files Reviewed:** 27
**Status:** issues_found

## Summary

Reviewed all 27 source files from Phase 02 plans 02-05 and 02-06
(dual-database-engine gap closure). The PostgreSQL repository implementations
are well-structured and follow consistent diesel-async patterns. All raw SQL
queries use parameterized bindings (`$1`, `$2`) — no SQL injection risks found.
The code compiles cleanly and follows the established project conventions.

Key concerns: silent-success stubs in `PgAlternativeAssetRepository` that could
mislead callers, fragile
`serde_json::to_string().unwrap_or_default().trim_matches('"')` pattern in
import_run repository, and a potential runtime failure from
`provider_config::json` cast if the column type doesn't match. Several dead code
items and a trap `unimplemented!()` method should be cleaned up.

No critical (security/data-loss) issues found.

## Warnings

### WR-01: Alternative asset stubs silently succeed instead of returning errors

**File:** `crates/storage-postgres/src/assets/alternative_asset.rs:21-50`
**Issue:** All four trait methods (`delete_alternative_asset`,
`update_asset_metadata`, `update_asset_details`, `find_liabilities_linked_to`)
return `Ok(())` or `Ok(Vec::new())` without performing any operation. Callers
receive success responses while the operations are silently discarded. This
contrasts with `app_sync.rs` which correctly returns
`Err("not supported in PostgreSQL mode")` for unsupported snapshot operations.
Users deleting or updating alternative assets will receive HTTP 200 responses
while nothing actually happens in the database. **Fix:**

```rust
async fn delete_alternative_asset(&self, _asset_id: &str) -> Result<()> {
    Err(Error::Database(DatabaseError::Internal(
        "Alternative assets are not yet supported in PostgreSQL mode".to_string(),
    )))
}

async fn update_asset_metadata(
    &self,
    _asset_id: &str,
    _metadata: Option<serde_json::Value>,
) -> Result<()> {
    Err(Error::Database(DatabaseError::Internal(
        "Alternative assets are not yet supported in PostgreSQL mode".to_string(),
    )))
}
// Apply same pattern to update_asset_details
```

### WR-02: Fragile enum serialization with `trim_matches('"')` in import_run

**File:** `crates/storage-postgres/src/sync/import_run.rs:52-57,79,162-168,189`
**Issue:** The pattern
`serde_json::to_string(&import_run.run_type).unwrap_or_default().trim_matches('"')`
is used to serialize enum values to strings for DB storage. Two problems: (1)
`unwrap_or_default()` silently produces empty string `""` if serialization
fails, storing corrupt data in the database; (2) `trim_matches('"')` will
incorrectly strip embedded quote characters if any variant's serialization
contains them. If a future enum variant serializes to something other than a
simple string (e.g., a number or object), this will silently produce wrong
values. **Fix:** Use the enum's string representation directly or add a helper:

```rust
// Option A: If enums impl Display or AsRef<str>
import_runs::run_type.eq(import_run.run_type.to_string()),

// Option B: Safer serde extraction helper
fn enum_to_string<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string(val)
        .ok()
        .and_then(|s| serde_json::from_str::<String>(&s).ok())
        .unwrap_or_default()
}
```

### WR-03: `provider_config::json` cast may fail at runtime

**File:** `crates/storage-postgres/src/custom_provider/repository.rs:282-283`
**Issue:** The raw SQL query uses
`provider_config::json->>'custom_provider_code'` to query assets. The `::json`
cast requires the `provider_config` column to be either a PostgreSQL JSON/JSONB
type, or a TEXT column containing valid JSON. If the PG schema defines
`provider_config` as TEXT and any row contains invalid JSON (or NULL without the
cast handling), this query will fail at runtime with a PostgreSQL error.
**Fix:** Verify the column type in the PG schema. If TEXT, use
`provider_config::jsonb->>'custom_provider_code'` with a NULL guard, or add
`WHERE provider_config IS NOT NULL AND provider_config != ''` to the query.
Alternatively, if the column is already JSONB, remove the `::json` cast for
clarity.

### WR-04: Dead `run_sync` helper and trap `unimplemented!()` method

**File:** `crates/storage-postgres/src/ai_chat/repository.rs:138-159,170-174`
**Issue:** The `run_sync` helper function (lines 138-159) is defined but never
used — all sync-over-async bridging is done inline with
`tokio::task::block_in_place`. The `get_conn` method (lines 170-174) returns
`unimplemented!()` which will panic at runtime if called. While currently
unreachable, this is a trap for future developers who may try to use it.
**Fix:** Remove both the `run_sync` function and the `get_conn` method entirely.
They add dead code and the `unimplemented!()` is a hidden landmine.

### WR-05: LIKE search pattern doesn't escape wildcards

**File:** `crates/storage-postgres/src/ai_chat/repository.rs:316-317` **Issue:**
User input for thread title search is interpolated directly into a LIKE pattern:
`format!("%{}%", search_str)`. Special SQL LIKE characters `%` and `_` in the
search string are not escaped, allowing users to manipulate search semantics.
For example, searching for `%` produces `%%%` which matches any title, and
searching for `_` matches any single character. While Diesel's parameterized
queries prevent SQL injection, the LIKE semantics can be manipulated. **Fix:**
Escape LIKE wildcards before interpolation:

```rust
let escaped = search_str.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
let search_pattern = format!("%{}%", escaped);
query = query.filter(ai_threads::title.like(search_pattern));
```

### WR-06: `EXCLUDED.*` raw SQL strings bypass compile-time column checking

**File:**
`crates/storage-postgres/src/portfolio/snapshot/repository.rs:55-64,447-456,489-498`
and `crates/storage-postgres/src/portfolio/valuation/repository.rs:51-59`
**Issue:** The batch upsert pattern uses
`diesel::dsl::sql("EXCLUDED.column_name")` as a workaround for Diesel's batch
upsert limitations. Column names are hardcoded strings that won't be caught by
the compiler if renamed or removed from the schema. This affects 4 distinct
locations across the snapshot and valuation repositories. **Fix:** Document this
as a known limitation with a comment referencing the schema. Consider extracting
column name constants to centralize the risk:

```rust
// Batch upsert uses raw EXCLUDED strings — update if schema columns change
const EXCLUDED_CURRENCY: &str = "EXCLUDED.currency";
const EXCLUDED_POSITIONS: &str = "EXCLUDED.positions";
// ...
```

### WR-07: `total_value::numeric` cast from TEXT may fail at runtime

**File:** `crates/storage-postgres/src/portfolio/valuation/repository.rs:221`
**Issue:** The negative balance query casts `total_value` from TEXT to numeric:
`total_value::numeric < 0`. Since the PG schema stores decimals as TEXT strings
(matching the SQLite convention), this cast will fail at runtime with a
PostgreSQL error if any row contains non-numeric text (empty strings, "N/A",
etc.). **Fix:** Add a NULL/empty guard or use PostgreSQL's `~` regex to validate
before casting:

```sql
WHERE account_id = ANY($1) AND total_value ~ '^-?\d' AND total_value::numeric < 0
```

Or more robustly:
`AND CASE WHEN total_value ~ '^-?\d+\.?\d*$' THEN total_value::numeric < 0 ELSE FALSE END`

### WR-08: NaiveDateTime `clone()` on Copy type

**File:** `crates/storage-postgres/src/ai_chat/model.rs:47-48` **Issue:**
`chrono::NaiveDateTime` implements `Copy`, so `.clone()` on line 47 is
redundant. While harmless, it suggests the developer may have incorrectly
assumed `NaiveDateTime` is move-only, which could indicate similar
misunderstandings elsewhere. **Fix:**

```rust
created_at: now,  // NaiveDateTime is Copy
updated_at: now,
```

## Info

### IN-01: Magic numbers for pagination defaults

**File:** `crates/storage-postgres/src/ai_chat/repository.rs:299` **Issue:**
`request.limit.unwrap_or(20).min(100)` uses magic numbers 20 (default page size)
and 100 (max page size) without named constants. **Fix:** Extract to constants:
`const DEFAULT_PAGE_SIZE: i64 = 20; const MAX_PAGE_SIZE: i64 = 100;`

### IN-02: `block_in_place` + `block_on` pattern used extensively for sync trait bridge

**Files:**
`crates/storage-postgres/src/sync/platform.rs:101-117,121-138,143-163`,
`crates/storage-postgres/src/sync/state.rs:84-98,250-264`,
`crates/storage-postgres/src/custom_provider/repository.rs:106-121,131-154,266-292`,
`crates/storage-postgres/src/ai_chat/repository.rs:230-252,257-291,303-386,515-539,546-571,668-687`
**Issue:** The
`tokio::task::block_in_place(|| { Handle::current().block_on(async { ... }) })`
pattern is used to bridge sync trait methods to async diesel-async queries. This
is correct for multi-threaded runtimes but blocks a worker thread per call. When
real implementations replace stubs, this could become a bottleneck under high
concurrency. **Fix:** Acceptable for now. Document the trade-off. Long-term,
advocate for upstream trait methods to be async.

### IN-03: Unused `log` import in snapshot repository

**File:** `crates/storage-postgres/src/portfolio/snapshot/repository.rs:7`
**Issue:** `use log::{debug, warn};` is imported but `debug` and `warn` macros
are not used in the file. **Fix:** Remove unused import: `use log::warn;` or
remove the entire `use log` line if no logging is done.

### IN-04: Missing `snapshot_date` in EXCLUDED set for snapshot upserts

**File:** `crates/storage-postgres/src/portfolio/snapshot/repository.rs:55-64`
**Issue:** The `EXCLUDED.*` set in `save_snapshots` and
`overwrite_all_snapshots_for_account` includes most columns but not
`snapshot_date`. This means if an existing snapshot is updated via upsert, its
`snapshot_date` won't be changed. This is likely intentional (snapshot date
should be immutable for a given ID), but should be documented explicitly.
**Fix:** Add a comment explaining the intentional omission:
`// Note: snapshot_date intentionally excluded from EXCLUDED — dates are immutable per ID`

### IN-05: `find_liabilities_linked_to` returns empty Vec in alternative asset stub

**File:** `crates/storage-postgres/src/assets/alternative_asset.rs:35-38`
**Issue:** `find_liabilities_linked_to` returns `Ok(Vec::new())` suggesting no
liabilities exist, when the real answer is unknown. If callers use this to
decide whether to allow deletion of an asset, they might allow deleting an asset
that actually has linked liabilities (once PG mode is fully implemented).
**Fix:** Return an error matching the other methods, or document that this stub
always returns empty and callers should treat it as "unknown" rather than
"none".

---

_Reviewed: 2026-04-22T12:00:00Z_ _Reviewer: OpenCode (gsd-code-reviewer)_
_Depth: standard_
