---
phase: 02-dual-database-engine
plan: 05
subsystem: database
tags:
  [
    postgresql,
    diesel,
    diesel-async,
    diesel-async,
    repository-pattern,
    trait-impl,
    cfg-feature,
  ]

# Dependency graph
requires:
  - phase: 02-02
    provides: "PG repository stubs and schema that needed return-type fixes"
  - phase: 02-03
    provides: "PG repository stubs for missing trait implementations"
provides:
  - "Zero-error postgres server build (cargo check -p whaleit-server --features
    postgres)"
  - "ChatRepositoryTrait impl on PgAiChatRepository"
  - "CustomProviderRepository impl on PgCustomProviderRepository"
  - "ImportRunRepositoryTrait impl (both core + connect) on
    PgImportRunRepository"
  - "PlatformRepositoryTrait impl on PgPlatformRepository"
  - "BrokerSyncStateRepositoryTrait impl on PgBrokerSyncStateRepository"
  - "PgAppSyncRepository with correct domain return types (SyncEngineStatus,
    SyncLocalDataSummary)"
  - "PgSyncEngineDbPorts for device sync engine"
affects: [02-verification, server-build, postgres-mode]

# Tech tracking
tech-stack:
  added: [diesel-async, tokio::task::block_in_place for sync-over-async bridge]
  patterns:
    [
      conditional-compilation-via-cfg-feature,
      stub-returns-err-for-unsupported-ops,
      dual-trait-impl-for-core-and-connect,
    ]

key-files:
  created:
    - crates/storage-postgres/src/sync/engine_ports.rs
  modified:
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
    - .planning/phases/02-dual-database-engine/02-CONTEXT.md

key-decisions:
  - "PG sync stubs return Err for unsupported operations (export/restore
    snapshot, LWW batch apply) — sync is not functional in PG mode"
  - "IDs stored as TEXT in PG (not native UUID) — matches core domain String
    types, avoids conversion at repo boundaries"
  - "Dual ImportRunRepositoryTrait implementations (core + connect) on single
    PgImportRunRepository struct"
  - "Conditional cfg(feature = postgres) imports in server for type aliases and
    engine ports"
  - "SnapshotRepositoryTrait import gated behind cfg(feature = postgres) in
    holdings handler"

patterns-established:
  - 'Conditional compilation: #[cfg(feature = "postgres")] guards on imports and
    type aliases in server'
  - "PG engine ports: PgSyncEngineDbPorts wraps PgAppSyncRepository implementing
    OutboxStore + ReplayStore"
  - "Dual trait impl pattern: one PG struct implements same trait from both
    whaleit-core and whaleit-connect"

requirements-completed: [DB-01, DB-02, DB-04]

# Metrics
duration: 6min
completed: 2026-04-21
---

# Phase 02 Plan 05: Fix PG Server Compilation Errors Summary

**Eliminated all 60 compilation errors for
`cargo check -p whaleit-server --features postgres` by fixing PG repository
return types, implementing 6 missing trait impls, and wiring conditional
compilation in the server.**

## Performance

- **Duration:** 6 min (continuation session)
- **Started:** 2026-04-21T20:14:44Z
- **Completed:** 2026-04-21T20:21:00Z
- **Tasks:** 2
- **Files modified:** 16

## Accomplishments

- PgAppSyncRepository rewritten with correct return types (SyncEngineStatus,
  SyncLocalDataSummary, SyncTableRowCount)
- ChatRepositoryTrait, CustomProviderRepository, ImportRunRepositoryTrait (x2),
  PlatformRepositoryTrait, BrokerSyncStateRepositoryTrait all implemented
- PgSyncEngineDbPorts created for device sync engine (OutboxStore + ReplayStore
  traits)
- Server compiles with 0 errors in both modes (--features postgres and default
  sqlite)
- TEXT ID deviation from D-25 documented in 02-CONTEXT.md

## Task Commits

Each task was committed atomically:

1. **task 1: Fix PG repository return types and implement missing trait
   impls** - `164cf4ed` (feat)
2. **task 2: Wire PG repos into server build, fix all remaining compilation
   errors** - `86b16b14` (fix)

## Files Created/Modified

- `crates/storage-postgres/src/sync/app_sync.rs` - Full PgAppSyncRepository with
  ~30 methods matching SQLite interface
- `crates/storage-postgres/src/sync/engine_ports.rs` - NEW: PgSyncEngineDbPorts
  implementing OutboxStore + ReplayStore
- `crates/storage-postgres/src/sync/import_run.rs` - Dual
  ImportRunRepositoryTrait impls (core + connect)
- `crates/storage-postgres/src/sync/platform.rs` - PlatformRepositoryTrait impl
- `crates/storage-postgres/src/sync/state.rs` - BrokerSyncStateRepositoryTrait
  impl
- `crates/storage-postgres/src/sync/mod.rs` - Updated exports for all new types
- `crates/storage-postgres/src/ai_chat/repository.rs` - ChatRepositoryTrait impl
  with diesel-async
- `crates/storage-postgres/src/ai_chat/model.rs` - Diesel models with
  Insertable/AsChangeset
- `crates/storage-postgres/src/custom_provider/repository.rs` -
  CustomProviderRepository impl
- `crates/storage-postgres/src/custom_provider/model.rs` - Diesel model with
  Insertable/AsChangeset
- `crates/storage-postgres/src/assets/alternative_asset.rs` - Fixed constructor
  to accept Arc<PgPool>
- `apps/server/src/main_lib.rs` - Conditional imports, fixed data_root
  use-after-move, removed unused PgPool import
- `apps/server/src/api/device_sync_engine.rs` - Conditional type aliases for
  SyncTableRowCount and EngineDbPorts
- `apps/server/src/api/holdings/handlers.rs` - Conditional
  SnapshotRepositoryTrait import for PG mode
- `.planning/phases/02-dual-database-engine/02-CONTEXT.md` - Documented TEXT ID
  deviation from D-25

## Decisions Made

- **PG sync operations are stubs:** export_snapshot_sqlite_image,
  restore_snapshot_tables_from_file, LWW batch apply return errors — sync is not
  functional in PG mode
- **Dual trait impl pattern:** PgImportRunRepository implements both
  `whaleit_core::activities::ImportRunRepositoryTrait` and
  `whaleit_connect::ImportRunRepositoryTrait` because core and connect define
  different method signatures
- **Conditional compilation over generics:** Used `#[cfg(feature = "postgres")]`
  type aliases rather than generics to keep the server code simple
- **brokers_sync_state composite PK:** Used `.filter()` instead of `.find()`
  because the table has composite primary key (account_id, provider)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed duplicate PgAppSyncRepository definitions**

- **Found during:** task 1
- **Issue:** Two definitions existed — one in `sync/app_sync.rs` (module entry)
  and one in `sync/app_sync/repository.rs` (orphan submodule). The submodule had
  different pool types.
- **Fix:** Rewrote `app_sync.rs` as canonical source, removed orphan submodule
  reference
- **Files modified:** crates/storage-postgres/src/sync/app_sync.rs
- **Committed in:** 164cf4ed

**2. [Rule 3 - Blocking] Created PgSyncEngineDbPorts for device sync**

- **Found during:** task 2
- **Issue:** Server's device_sync_engine.rs imported SqliteSyncEngineDbPorts
  which doesn't exist in PG crate
- **Fix:** Created PgSyncEngineDbPorts in engine_ports.rs implementing
  OutboxStore + ReplayStore traits
- **Files modified:** crates/storage-postgres/src/sync/engine_ports.rs (new),
  crates/storage-postgres/src/sync/mod.rs
- **Committed in:** 86b16b14

**3. [Rule 1 - Bug] Fixed export_snapshot_sqlite_image sync/async mismatch**

- **Found during:** task 2
- **Issue:** PG version was sync (`fn`) but caller used `.await`; SQLite version
  is `async fn`
- **Fix:** Changed PG method to `async fn`
- **Files modified:** crates/storage-postgres/src/sync/app_sync.rs
- **Committed in:** 86b16b14

**4. [Rule 1 - Bug] Fixed restore_snapshot_tables_from_file key_version type**

- **Found during:** task 2
- **Issue:** PG version had `key_version: String` but SQLite uses
  `key_version: Option<i32>`
- **Fix:** Changed to `Option<i32>` to match SQLite interface
- **Files modified:** crates/storage-postgres/src/sync/app_sync.rs
- **Committed in:** 86b16b14

**5. [Rule 2 - Missing Critical] Added SnapshotRepositoryTrait import for PG
mode**

- **Found during:** task 2
- **Issue:** `delete_snapshots_for_account_and_dates` is a trait method on
  `SnapshotRepositoryTrait`, not on the concrete PgSnapshotRepository. In PG
  mode the concrete type is PgSnapshotRepository which needs the trait in scope.
- **Fix:** Added
  `#[cfg(feature = "postgres")] use whaleit_core::portfolio::snapshot::SnapshotRepositoryTrait;`
- **Files modified:** apps/server/src/api/holdings/handlers.rs
- **Committed in:** 86b16b14

**6. [Rule 1 - Bug] Fixed data_root use-after-move in main_lib.rs**

- **Found during:** task 2
- **Issue:** `data_root` was moved into a struct field and then used again for
  `db_path`
- **Fix:** Added `.clone()` before the move
- **Files modified:** apps/server/src/main_lib.rs
- **Committed in:** 86b16b14

**7. [Rule 1 - Bug] Fixed PgAlternativeAssetRepository constructor type**

- **Found during:** task 2
- **Issue:** Constructor took `Pool` directly instead of `Arc<PgPool>`,
  inconsistent with all other PG repos
- **Fix:** Changed to accept `Arc<PgPool>`
- **Files modified:** crates/storage-postgres/src/assets/alternative_asset.rs
- **Committed in:** 86b16b14

---

**Total deviations:** 7 auto-fixed (4 bugs, 1 missing critical, 2 blocking)
**Impact on plan:** All auto-fixes necessary for correct compilation. No scope
creep — all related to making the postgres feature flag work.

## Issues Encountered

- Previous session (task 1) had already resolved 52 of 60 errors; this
  continuation session resolved the remaining 8 errors down to 0
- The storage-postgres crate has 29 warnings (all in stub code with
  intentionally unused parameters/imports) — these are expected for stub
  repositories

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Server compiles with postgres feature — ready for integration testing
- PG repository stubs return errors for unsupported sync operations — a future
  phase will need to implement real PG sync
- TEXT ID deviation from D-25 documented — future phases should maintain TEXT
  IDs for consistency

---

_Phase: 02-dual-database-engine_ _Completed: 2026-04-21_

## Self-Check: PASSED

- All 6 key files verified present
- Both task commits (164cf4ed, 86b16b14) verified in git history
- cargo check -p whaleit-server --features postgres: 0 errors ✅
- cargo check -p whaleit-server: 0 errors ✅
- cargo check -p whaleit-storage-postgres: 0 errors ✅
