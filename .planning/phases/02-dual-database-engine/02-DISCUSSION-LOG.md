# Phase 2: Dual Database Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-21
**Phase:** 02-dual-database-engine
**Areas discussed:** Async strategy, Crate & schema organization, Migration management, Write pattern & pool design, Diesel upgrade, Device sync outbox, Data migration, Feature flags, Docker & deployment, CI/test matrix, Transaction isolation, ID generation strategy

---

## Async Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| diesel-async + deadpool | Truly async PG with native async connection pooling | ✓ |
| Sync Diesel + r2d2 for both | Same sync pattern, tokio bridge for PG | |
| Migrate both to diesel-async | Refactor SQLite repos to also use diesel-async | |

**User's choice:** diesel-async + deadpool
**Notes:** SQLite keeps existing sync+actor pattern. Each engine uses its natural async model.

**Follow-up — Connection pool library:**
| Option | Description | Selected |
|--------|-------------|----------|
| deadpool | Simpler, standard pairing with diesel-async | ✓ |
| bb8 | More features (health checks, reaper thread) | |

**Follow-up — PG connection config:**
| Option | Description | Selected |
|--------|-------------|----------|
| DATABASE_URL only | Standard 12-factor, used by Diesel CLI | ✓ |
| Separate PG_* env vars | More granular for Docker | |

---

## Crate & Schema Organization

| Option | Description | Selected |
|--------|-------------|----------|
| New crates/storage-postgres/ | Clean separation, mirrors storage-sqlite | ✓ |
| Unify into crates/storage/ | Shared code at top level | |

**Follow-up — Schema management:**
| Option | Description | Selected |
|--------|-------------|----------|
| Separate schemas per crate | Each crate generates its own Diesel schema | ✓ |
| Shared schema with conditional types | Single schema, multi-backend macros | |

**Follow-up — Shared code location:**
| Option | Description | Selected |
|--------|-------------|----------|
| New storage-common crate | Shared DTOs without odd dependency direction | ✓ |
| PG depends on storage-sqlite | Simpler but odd dependency | |
| Core holds shared models | Domain models in core, DB models in each crate | |

---

## Migration Management

| Option | Description | Selected |
|--------|-------------|----------|
| Separate hand-written migrations | Idiomatic SQL per engine, easier to review | ✓ |
| Generate from single source | Less duplication but adds abstraction | |

**Follow-up — Migration tooling:**
| Option | Description | Selected |
|--------|-------------|----------|
| Diesel CLI for both | Existing workflow extends naturally | ✓ |
| Alternative migration library | refinery, sea-query | |

**Follow-up — Migration scope:**
| Option | Description | Selected |
|--------|-------------|----------|
| Full 31-migration historical parity | Complete traceability | ✓ |
| Single consolidated migration | Simpler, loses history | |

**Follow-up — Schema parity:**
| Option | Description | Selected |
|--------|-------------|----------|
| Exact structural parity | Same tables/columns, different SQL types | ✓ |
| PG-native optimizations | BOOLEAN, JSONB, ENUM types | |

**Follow-up — Parity testing:**
| Option | Description | Selected |
|--------|-------------|----------|
| Automated parity tests in CI | Runs on every PR, catches drift | ✓ |
| Manual verification only | Simpler, risks drift | |

---

## Write Pattern & Pool Design

| Option | Description | Selected |
|--------|-------------|----------|
| Native async writes for PG | No write actor, PG handles concurrency | ✓ |
| Write actor for both engines | Consistent but artificially limits PG | |

**Follow-up — ServiceContext wiring:**
| Option | Description | Selected |
|--------|-------------|----------|
| Keep concrete, compile-time selected | Minimal change to wiring code | ✓ |
| Fully trait-objectified | Runtime selection possible | |

**Follow-up — Error handling:**
| Option | Description | Selected |
|--------|-------------|----------|
| Engine-specific errors → core Error | Follows existing pattern | ✓ |
| Shared StorageError enum | Single type, tighter coupling | |

---

## Diesel Version

| Option | Description | Selected |
|--------|-------------|----------|
| Patch upgrade to 2.2.x first | Incremental, less risk | |
| Direct upgrade to latest Diesel | All improvements at once | ✓ |
| Stay on Diesel 2.2, add diesel-async | Skip major bumps | |

**User's choice:** Direct upgrade to latest Diesel
**Notes:** Review full changelog for breaking changes before upgrading.

---

## Device Sync Outbox

| Option | Description | Selected |
|--------|-------------|----------|
| PG LISTEN/NOTIFY for outbox | Idiomatic PG, replaces write actor channel | ✓ |
| Replicate SQLite outbox pattern | Consistent but misses PG optimization | |

---

## Data Migration

| Option | Description | Selected |
|--------|-------------|----------|
| Fresh start, no migration tool | Phase 2 delivers engine support only | ✓ |
| Include SQLite-to-PG migration tool | Removes upgrade blocker | |

---

## Feature Flags

| Option | Description | Selected |
|--------|-------------|----------|
| Single `postgres` feature flag | Clean, workspace-level toggle | ✓ |
| Mutually exclusive dual features | More explicit, more complex | |

---

## Docker & Deployment

| Option | Description | Selected |
|--------|-------------|----------|
| External PG, compose includes it | 12-factor, clean separation | ✓ |
| Bundled PG in Docker | Complex, defeats PG purpose | |

---

## CI/Test Matrix

| Option | Description | Selected |
|--------|-------------|----------|
| Both engines on every PR | Full parity verification | ✓ |
| PG tests on main/scheduled only | Faster CI, catches PG regressions later | |

---

## Transaction Isolation

| Option | Description | Selected |
|--------|-------------|----------|
| Match SQLite: READ COMMITTED | Explicit SET, consistent behavior | ✓ |
| Use PG default (already READ COMMITTED) | No config needed | |

---

## ID Generation Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| SERIAL/BIGSERIAL to match SQLite | Integer IDs, compatible | |
| UUID v7 for both engines | Time-sortable, modern, cloud-ready | ✓ |

**User's choice:** UUID v7 for both engines
**Notes:** User's rationale includes future cloud API storage scenario. Both engines migrate in Phase 2.

**Follow-up — UUID scope:**
| Option | Description | Selected |
|--------|-------------|----------|
| Both engines in Phase 2 | Migrate all 34 tables, all foreign keys | ✓ |
| PG gets UUID, SQLite stays INTEGER | Defer SQLite migration | |

**Follow-up — UUID as PK:**
| Option | Description | Selected |
|--------|-------------|----------|
| UUID v7 as PK everywhere | Replace all INTEGER PKs with UUID v7 | ✓ |
| Integer PK + UUID column for external ref | Less migration impact, two ID systems | |

---

## OpenCode's Discretion

- Exact Diesel model struct organization per crate
- Connection pool configuration details (timeout, max_lifetime)
- Diesel CLI configuration (diesel.toml) for PG crate
- PG-specific connection customizers (equivalent to SQLite PRAGMAs)
- Migration SQL translation strategy
- Error message formatting in engine-specific error types

## Deferred Ideas

- Cloud API storage via API Key — future phase for cloud-connected storage
- SQLite-to-PostgreSQL data migration tool — future utility
