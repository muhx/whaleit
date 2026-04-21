# Phase 2: Dual Database Engine - Research

**Researched:** 2026-04-21
**Status:** Complete

## Research Question

What do I need to know to PLAN this phase well?

---

## Standard Stack

### diesel-async + deadpool (PostgreSQL Async)

**Version:** diesel-async 0.5.x (latest stable, compatible with diesel 2.2+)
**Pool:** deadpool (re-exported via `diesel_async::pooled_connection::deadpool::Pool`)

**Setup pattern:**
```rust
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool as DeadpoolPool;

let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
let pool = DeadpoolPool::builder(config).max_size(8).build()?;
let mut conn = pool.get().await?;
```

**Async transactions:**
```rust
conn.transaction::<_, diesel::result::Error, _>(|conn| {
    Box::pin(async move {
        diesel::insert_into(users::table)
            .values(users::name.eq("Ruby"))
            .execute(conn)
            .await?;
        Ok(())
    })
}).await?;
```

**Key API differences from sync Diesel (SQLite):**
- `load()` → `load().await`
- `execute()` → `execute().await`
- `first()` → `first().await`
- `get_result()` → `get_result().await`
- Transaction closure must return `Box::pin(async move { ... })`
- No `immediate_transaction` — PG uses standard `transaction()`

**Dependency:**
```toml
diesel-async = { version = "0.5", features = ["postgres", "deadpool"] }
```

### Diesel 2.2 → 2.3.x Upgrade

**Current:** diesel 2.2, diesel_migrations 2.2
**Target:** diesel 2.3.7 (latest stable)

**Breaking changes in 2.3.x (from changelog):**
- `returning_clauses_for_sqlite_3_35` feature still needed
- No breaking changes to query DSL or model derives
- Minor: Some deprecated methods removed (check `InferConnection` changes)
- diesel-async 0.5.x is compatible with diesel 2.2+ (no forced upgrade)
- **Recommendation:** Upgrade diesel first in a separate task, verify all tests pass, then proceed with PG crate

**Critical:** The workspace `Cargo.toml` diesel feature flags include `"sqlite"`. For PG, a separate crate with `"postgres"` feature is needed. Cannot mix `sqlite` and `postgres` features in same crate (Diesel limitation — feature flags are additive).

### UUID v7

**Already in workspace:** `uuid = { version = "1", features = ["v4", "v7", "serde"] }`
**Generation:** `uuid::Uuid::now_v7()` for time-sortable UUIDs
**Current usage:** `uuid::Uuid::new_v4().to_string()` — already generates UUIDs as strings

**Migration impact for 34 tables:**
- SQLite: `id TEXT PRIMARY KEY` (stores UUID as text — already the case!)
- PostgreSQL: `id UUID PRIMARY KEY DEFAULT gen_random_uuid()` (native PG uuid type)
- Foreign keys: `account_id TEXT` → `account_id UUID` (PG) — stays TEXT in SQLite
- **Key insight:** SQLite already stores IDs as TEXT, so SQLite migrations don't need to change data types. Only PG migrations use native UUID type.

---

## Architecture Patterns

### Current SQLite Repository Pattern

```rust
pub struct AccountRepository {
    pool: Arc<Pool<ConnectionManager<SqliteConnection>>>,  // sync r2d2 pool
    writer: WriteHandle,                                     // tokio mpsc → sync write actor
}
```

- **Reads:** sync `pool.get()` → `diesel::RunQueryDsl` (blocking, but fine for SQLite)
- **Writes:** `writer.exec_tx(|tx| { /* sync closure */ })` → single-writer actor
- **Traits:** `#[async_trait]` on trait, but individual read methods are `fn` (sync), write methods are `async fn`
- **Outbox:** `DbWriteTx` captures model mutations → flushes to `sync_outbox` table

### PostgreSQL Repository Pattern (Planned)

```rust
pub struct PgAccountRepository {
    pool: Arc<deadpool::Pool<AsyncPgConnection>>,  // async deadpool
}
```

- **Reads:** `pool.get().await` → `diesel_async::RunQueryDsl` (truly async)
- **Writes:** Direct `conn.transaction(|conn| Box::pin(async { ... })).await`
- **No write actor needed** — PG handles concurrent writes natively
- **Outbox:** Direct insert in transaction + optional `LISTEN/NOTIFY`

### Critical Issue: Sync Trait Methods

**Problem:** Current repository traits have SYNC read methods:
```rust
fn get_by_id(&self, account_id: &str) -> Result<Account>;  // sync!
fn list(&self, ...) -> Result<Vec<Account>>;                // sync!
```

PostgreSQL reads MUST be async (`pool.get().await`). These sync trait methods **cannot** be implemented for PG without blocking the tokio runtime.

**Solutions (evaluated):**

| Option | Pros | Cons |
|--------|------|------|
| A: Make all trait methods async | Clean, future-proof | Touch every caller in core + services |
| B: `tokio::task::block_in_place` for PG sync methods | No trait changes | Unsafe, degrades async runtime, fragile |
| C: Separate PG traits | No SQLite changes | Duplicate trait definitions, confusing |
| D: Make reads async, keep sync SQLite impl via `spawn_blocking` | Minimal caller changes | Complex dual impl |

**Recommendation:** Option A — Make all repository trait methods async. This is the correct long-term approach:
- `fn get_by_id(&self, ...)` → `async fn get_by_id(&self, ...)`
- SQLite impl: use `tokio::task::spawn_blocking` to wrap sync Diesel calls
- PG impl: native async diesel-async calls
- Touches ~16 repository traits in `crates/core/` but changes are mechanical (add `async` keyword)

### 16+ Repository Implementations Needed

Counted from codebase — these all need PG implementations:

| Module | Repository | Key Operations |
|--------|-----------|----------------|
| accounts | AccountRepository | CRUD, list, filter |
| activities | ActivityRepository | CRUD, complex queries, bulk insert |
| ai_chat | AiChatRepository | Thread/message CRUD |
| assets | AssetRepository | CRUD, bulk queries |
| assets | AlternativeAssetRepository | CRUD |
| custom_provider | CustomProviderSqliteRepository | CRUD |
| fx | FxRepository | Exchange rate CRUD |
| goals | GoalRepository | CRUD |
| health | HealthDismissalRepository | Dismissal tracking |
| limits | ContributionLimitRepository | CRUD |
| market_data | MarketDataRepository | Quotes, sync state |
| market_data | QuoteSyncStateRepository | Sync state tracking |
| portfolio/snapshot | SnapshotRepository | Snapshots, valuations |
| portfolio/valuation | ValuationRepository | Daily valuations |
| settings | SettingsRepository | Key-value settings |
| sync | AppSyncRepository | Outbox, cursor, entity metadata |
| sync | BrokerSyncStateRepository | Broker sync tracking |
| sync | ImportRunRepository | Import run tracking |
| sync | PlatformRepository | Platform CRUD |
| taxonomies | TaxonomyRepository | Taxonomy CRUD |

### Schema: 34 Tables

From `crates/storage-sqlite/src/schema.rs` — 34 tables total:
accounts, activities, import_account_templates, import_templates, ai_messages, ai_thread_tags, ai_threads, app_settings, asset_taxonomy_assignments, assets, brokers_sync_state, contribution_limits, market_data_custom_providers, daily_account_valuation, goals, goals_allocation, health_issue_dismissals, holdings_snapshots, import_runs, market_data_providers, platforms, quote_sync_state, quotes, sync_applied_events, sync_cursor, sync_device_config, sync_engine_state, sync_entity_metadata, sync_outbox, sync_table_state, taxonomies, taxonomy_categories

Plus 13 joinable relationships and foreign key references.

---

## Don't Hand-Roll

1. **diesel-async** — Standard async adapter for Diesel. Don't create custom async wrappers around sync Diesel.
2. **deadpool** — Use the built-in deadpool integration from diesel-async, not a custom pool.
3. **diesel_migrations** — Use `embed_migrations!()` macro for PG just like SQLite. Don't write custom migration runners.
4. **UUID generation** — Use `uuid::Uuid::now_v7()`, don't implement custom timestamp-based ID generation.

---

## Common Pitfalls

1. **Mixing diesel feature flags:** Cannot enable both `sqlite` and `postgres` features in the same crate. Each storage crate must specify its own backend feature.

2. **Sync trait methods:** Current traits have sync `fn` for reads. PG requires `async fn`. Must convert traits — this is the single biggest design decision.

3. **Write actor removal:** SQLite write actor serializes writes to avoid `SQLITE_BUSY`. PG doesn't need this. Removing write actor from PG is correct, but the outbox pattern (sync_outbox table writes) must be preserved in PG transactions.

4. **Diesel model derives:** `#[diesel(check_for_backend(diesel::sqlite::Sqlite))]` must become `#[diesel(check_for_backend(diesel::pg::Pg))]`. Can't share model files between crates.

5. **Connection URL format:** SQLite uses file path (`./db/app.db`), PG uses connection string (`postgres://user:pass@host:5432/dbname`). Server config must handle both.

6. **DateTime types:** SQLite stores timestamps as `Text` or `Timestamp`, PG uses `Timestamptz` or `Timestamp`. Schema.rs types differ.

7. **Boolean storage:** SQLite uses `Integer` (0/1), PG uses native `Bool`. Diesel schema reflects this — `Integer` vs `Bool` in `diesel::table!` macro.

8. **Transaction isolation:** SQLite WAL mode ≈ `READ COMMITTED`. Must set PG to `READ COMMITTED` explicitly (default for PG, but worth confirming).

9. **diesel-async version compatibility:** diesel-async 0.5.x requires diesel 2.2+. Verify exact version pin before starting.

10. **Schema regeneration:** Each crate must run `diesel print-schema` independently against its own database. Cannot share `schema.rs` between SQLite and PG crates.

---

## Integration Points

### ServiceContext (Tauri — Desktop)
- File: `apps/tauri/src/context/providers.rs`
- Currently: links `storage-sqlite` at compile time
- No changes needed — desktop always uses SQLite

### AppState (Axum — Web)
- File: `apps/server/src/main_lib.rs` (~490 lines of wiring)
- Currently: imports all from `wealthfolio_storage_sqlite::*`
- Needs: conditional compilation to import from `wealthfolio_storage_postgres::*` when `postgres` feature enabled
- `Config::from_env()` needs `DATABASE_URL` support alongside `WF_DB_PATH`

### Cargo Workspace
- File: `Cargo.toml` (root)
- Needs: `wealthfolio-storage-postgres = { path = "crates/storage-postgres" }`
- Needs: `diesel-async`, `deadpool` workspace deps
- Needs: `postgres` feature flag for apps/server

### Compose (Docker)
- File: `compose.yml`
- Needs: `postgres` service alongside `whaleit` service
- Needs: `DATABASE_URL` environment variable
- Needs: volume for PG data persistence

---

## Validation Architecture

### Parity Tests

**Approach:** For each repository, create a test that:
1. Inserts identical data into both SQLite and PG
2. Calls every repository method
3. Asserts identical results

**Test structure:**
```rust
#[cfg(test)]
mod parity_tests {
    async fn setup_both_engines() -> (SqliteRepo, PgRepo) { ... }

    #[tokio::test]
    async fn account_create_parity() { ... }

    #[tokio::test]
    async fn account_list_parity() { ... }
}
```

**CI integration:** GitHub Actions with `postgres` service container for parity tests.

### Migration Parity

Verify PG migrations produce identical schema structure:
- Same table names
- Same column names  
- Same constraints (unique, foreign keys, not-null)
- Compatible data types (TEXT↔UUID, INTEGER↔BOOL, etc.)

---

## Key Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Sync→Async trait conversion breaks callers | High | Mechanical change, but touches many files. Do first. |
| Diesel 2.2→2.3 upgrade breaks existing SQLite | Medium | Upgrade first, run full test suite before PG work |
| 34 PG migrations have subtle SQL dialect issues | Medium | Systematic translation + parity tests |
| Outbox pattern works differently in PG | Low | LISTEN/NOTIFY is well-understood |
| diesel-async version incompatibility | Low | Pin exact compatible versions |

---

## Summary

This phase has three major workstreams:

1. **Foundation (Wave 1):** Diesel upgrade, async trait conversion, storage-common DTOs
2. **PostgreSQL Crate (Wave 2):** New storage-postgres crate with all 16+ repositories and 34 PG migrations
3. **Integration & Verification (Wave 3):** Server wiring, Docker compose, parity tests, CI matrix

The sync→async trait conversion is the critical path — it unblocks everything else and touches the most files.

---

*Phase: 02-dual-database-engine*
*Research completed: 2026-04-21*
