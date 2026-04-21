# Phase 2: Dual Database Engine - Context

**Gathered:** 2026-04-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Both SQLite and PostgreSQL work as storage backends through shared repository traits. Desktop (Tauri) uses SQLite, web (Axum) uses PostgreSQL. All existing investment domain queries return identical results on both engines. Includes UUID v7 migration for all 34 tables across both engines.

</domain>

<decisions>
## Implementation Decisions

### Async Strategy
- **D-01:** PostgreSQL uses `diesel-async` + `deadpool` for truly async database operations. Read and write methods are natively async — no tokio channel bridging.
- **D-02:** SQLite retains existing sync Diesel + r2d2 + write actor pattern. No changes to the proven SQLite async bridge.
- **D-03:** PostgreSQL connection configured via `DATABASE_URL` env var (standard connection string format). Replaces `WF_DB_PATH` when `postgres` feature is enabled.
- **D-04:** `deadpool` as the async connection pool library for diesel-async. Simpler than bb8, standard pairing with diesel-async.

### Crate & Schema Organization
- **D-05:** New `crates/storage-postgres/` crate implementing all 16+ repository traits using diesel-async + PostgreSQL. Mirrors `storage-sqlite` structure (one module per domain).
- **D-06:** Separate `schema.rs` per storage crate. Each crate runs Diesel CLI against its own database to generate engine-specific schema. No shared schema abstraction.
- **D-07:** New `crates/storage-common/` crate for shared DTO types (NewAccount, ActivityFilters, etc.) used by both storage crates. Avoids duplicating data transfer objects.

### Migration Management
- **D-08:** Separate hand-written migration directories per engine — SQLite migrations in `crates/storage-sqlite/migrations/`, PostgreSQL migrations in `crates/storage-postgres/migrations/`.
- **D-09:** Diesel CLI manages migrations for both engines. Same workflow as current SQLite setup.
- **D-10:** Full 31-migration historical parity for PostgreSQL — all existing SQLite migrations have corresponding PG migrations. Ensures full schema traceability.
- **D-11:** Exact structural parity between engines — same table names, same column names, same relationships. Only SQL dialect differs (SERIAL vs AUTOINCREMENT, BOOLEAN vs INTEGER, ON CONFLICT vs REPLACE INTO, UUID vs TEXT).
- **D-12:** Automated parity tests that run all repository methods against both engines with identical data and assert matching results. Runs in CI on every PR.

### Write Pattern & Pool Design
- **D-13:** PostgreSQL uses native async writes — no write actor. Each write operation is a truly async diesel-async call with its own pooled connection. PostgreSQL handles concurrency natively.
- **D-14:** PostgreSQL pool size configurable via environment, default 8 connections (matching SQLite pool size).
- **D-15:** ServiceContext (Tauri) and AppState (Axum) remain concrete structs with compile-time engine selection. Desktop builds link `storage-sqlite`, web builds link `storage-postgres` (or `storage-sqlite` if `postgres` feature disabled).
- **D-16:** Each storage crate has its own error type (StorageSqliteError, StoragePgError) that converts to `core::Error::DatabaseError`. Consistent with existing error bridge pattern.

### Diesel Version
- **D-17:** Direct upgrade to latest stable Diesel version (2.3.x or latest). Review full changelog for breaking changes before upgrading. diesel-async compatibility verified.

### Device Sync Outbox
- **D-18:** PostgreSQL outbox uses `LISTEN/NOTIFY` for real-time event propagation. Replaces the channel-based write actor outbox capture. Natural fit for PG's async architecture.

### Data Migration
- **D-19:** Phase 2 delivers PostgreSQL engine support only. No SQLite-to-PostgreSQL data migration tool. Existing web users on SQLite can continue with SQLite or start fresh with PG. Migration tool can be added in a future phase.

### Feature Flags
- **D-20:** Single `postgres` feature flag at workspace level. Tauri binary always uses SQLite (no feature). Axum binary uses `storage-postgres` when `postgres` feature enabled, falls back to `storage-sqlite` otherwise.

### Docker & Deployment
- **D-21:** External PostgreSQL instance for Docker deployments. `compose.yml` includes a `postgres` service alongside the app service. App connects via `DATABASE_URL`. No bundled PostgreSQL.

### CI/Test Matrix
- **D-22:** Both engines tested on every PR. GitHub Actions runs SQLite tests (fast) and PostgreSQL parity tests (using `postgres` service container) as separate CI jobs.

### Transaction Isolation
- **D-23:** Explicitly set `READ COMMITTED` isolation level on PostgreSQL connections to match SQLite WAL behavior. Ensures consistent semantics across engines.

### ID Strategy
- **D-24:** UUID v7 as primary key for all 34 tables in both engines. Replaces all INTEGER PRIMARY KEY AUTOINCREMENT with UUID v7. Domain models use `uuid::Uuid` type throughout.
- **D-25:** Both engines migrated to UUID v7 in Phase 2. SQLite stores UUIDs as TEXT, PostgreSQL uses native `uuid` type. All foreign keys updated to reference UUID columns.
  > **Implementation note:** IDs are stored as TEXT in both engines (not native UUID in PG).
  > This was an intentional implementation decision documented in 02-02-SUMMARY.md:
  > "Core domain models use String for all IDs. Native PG UUID columns would require
  > conversion at every repository boundary." This deviation from D-25 is accepted.

### OpenCode's Discretion
- Exact Diesel model struct organization per crate
- Connection pool configuration details (timeout, max_lifetime)
- Diesel CLI configuration (diesel.toml) for PG crate
- PG-specific connection customizers (equivalent to SQLite PRAGMAs)
- Migration SQL translation strategy (systematic conversion of SQLite→PG SQL)
- Error message formatting in engine-specific error types

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` §DB-01 through DB-05 — Dual database engine acceptance criteria
- `.planning/ROADMAP.md` §Phase 2 — Success criteria (4 items)
- `.planning/STATE.md` §Blockers/Concerns — Diesel 2.2→2.3.7 upgrade flag

### Architecture Context
- `.planning/codebase/ARCHITECTURE.md` — Full architecture: repository trait pattern, write actor, dual-runtime, ServiceContext/AppState wiring
- `.planning/codebase/CONVENTIONS.md` — Rust naming, module organization, error handling patterns
- `.planning/codebase/CONCERNS.md` — SQLite single-writer scaling limit, Diesel+SQLite 3.35 feature dependency

### Key Source Files (existing patterns to follow)
- `crates/storage-sqlite/src/lib.rs` — Module structure, re-exports, ASCII diagram
- `crates/storage-sqlite/src/db/mod.rs` — Pool creation, init, migrations, PRAGMA setup
- `crates/storage-sqlite/src/db/write_actor.rs` — Write actor pattern (to understand what PG does NOT need)
- `crates/storage-sqlite/src/errors.rs` — StorageError → core::Error bridge pattern
- `crates/storage-sqlite/src/schema.rs` — Current Diesel schema (34 tables, SQLite types)
- `crates/storage-sqlite/migrations/` — 31 existing migrations to replicate for PG
- `crates/core/src/accounts/accounts_traits.rs` — Example repository + service trait pattern
- `crates/storage-sqlite/src/accounts/repository.rs` — Example repository implementation
- `crates/storage-sqlite/src/accounts/model.rs` — Example Diesel model (Queryable/Insertable)

### Service Context Wiring
- `apps/tauri/src/context/registry.rs` — ServiceContext struct definition
- `apps/tauri/src/context/providers.rs` — ServiceContext construction (SQLite)
- `apps/server/src/main_lib.rs` — AppState struct and construction (SQLite)
- `apps/server/src/config.rs` — Server configuration (env vars)

### Device Sync (outbox pattern)
- `crates/device-sync/src/engine/ports.rs` — OutboxStore + ReplayStore trait definitions
- `crates/storage-sqlite/src/sync/app_sync/engine_ports.rs` — SQLite outbox implementation

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/core/src/*/` — All 16+ repository trait definitions are clean, async_trait-based, database-agnostic. These are the contracts both engines implement.
- `crates/storage-sqlite/src/` — Complete reference implementation of every repository. Structure, patterns, and error handling provide the blueprint for `storage-postgres`.
- `crates/storage-sqlite/src/errors.rs` — StorageError → core::Error bridge pattern. PG crate follows identical pattern.
- `crates/core/src/errors.rs` — Core Error enum with DatabaseError variant. No changes needed.

### Established Patterns
- Repository struct holds `Arc<Pool>` + `WriteHandle`. PG version holds `Arc<deadpool::Pool<AsyncPgConnection>>` instead.
- Read methods: `fn` (sync) on SQLite via `pool.get()`. PG reads become `async fn` via `pool.get().await`.
- Write methods: `async fn` dispatching to write actor on SQLite. PG writes become direct `async fn` diesel-async calls.
- `DbWriteTx` with outbox capture on SQLite. PG uses `LISTEN/NOTIFY` + direct transaction commits.
- Model files: Diesel `Queryable`, `Insertable`, `AsChangeset` derives. PG versions use PG-compatible derives.

### Integration Points
- `apps/tauri/src/context/providers.rs` — Desktop ServiceContext construction. Links `storage-sqlite` at compile time. No changes needed if we keep concrete types.
- `apps/server/src/main_lib.rs` — Web AppState construction. Currently links `storage-sqlite`. Needs conditional compilation to link `storage-postgres` when `postgres` feature enabled.
- `Cargo.toml` workspace — Diesel version, features, and workspace dependencies. Needs `postgres` feature addition, diesel upgrade, diesel-async + deadpool additions.
- `apps/server/src/config.rs` — Server config. Needs `DATABASE_URL` support alongside/instead of `WF_DB_PATH`.
- `compose.yml` — Needs `postgres` service for Docker deployments.

</code_context>

<specifics>
## Specific Ideas

- PostgreSQL schema should be structurally identical to SQLite — same table/column names, only SQL dialect differs
- UUID v7 gives time-sortable IDs that work well for both local-first (SQLite) and cloud-connected (PG) scenarios
- diesel-async is the standard async Diesel adapter — well-maintained, designed for this exact use case
- deadpool is simpler than bb8 and the standard pairing with diesel-async
- PG LISTEN/NOTIFY is the idiomatic way to handle the outbox pattern in PostgreSQL — replaces the channel-based approach

</specifics>

<deferred>
## Deferred Ideas

- Cloud API storage via API Key — User scenario mentioned during discussion: "user can connect to API via API Key to store the data in cloud via API and store the record in cloud." This is a new capability beyond dual engine support — belongs in a future phase.
- SQLite-to-PostgreSQL data migration tool — Could be added as a future utility for existing web users who want to migrate from SQLite to PG.

</deferred>

---

*Phase: 02-dual-database-engine*
*Context gathered: 2026-04-21*
