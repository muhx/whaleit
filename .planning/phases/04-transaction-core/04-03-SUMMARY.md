---
phase: 04-transaction-core
plan: 03
subsystem: storage
tags: [postgres, diesel, transactions, repository, integration-tests]
status: complete
completed: 2026-04-30

requires:
  - phase: 04-transaction-core
    plan: 01
    provides:
      transactions/transaction_splits/payee_category_memory/transaction_csv_templates
      tables + v_transactions_with_running_balance VIEW

provides:
  - PgTransactionRepository implementing TransactionRepositoryTrait +
    PayeeCategoryMemoryRepositoryTrait
  - PgTransactionTemplateRepository implementing
    TransactionTemplateRepositoryTrait
  - TransactionDB, TransactionSplitDB, PayeeCategoryMemoryDB,
    TransactionTemplateDB Diesel models
  - whaleit_core::transactions domain types (Transaction, NewTransaction,
    TransactionUpdate, TransactionSplit, NewSplit, PayeeCategoryMemory,
    TransactionFilters, TransactionSearchResult, TransactionWithRunningBalance,
    TransactionTemplate, NewTransactionTemplate, TransactionTemplateUpdate)
  - 22 integration tests covering TXN-03/04/06/08/09 + D-01 cascade + D-16/17/18
    templates

affects:
  - 04-02 (owns same crates/core/src/transactions/ types — merge will pick one
    copy)
  - 04-04 (web/IPC handlers bind to PgTransactionRepository::new(pool))

tech-stack:
  added:
    - diesel serde_json feature (added to workspace Cargo.toml for JSONB column
      support)
  patterns:
    - IntoCore trait for unambiguous DieselError → whaleit_core::Error
      conversion
    - hydrate_one / hydrate_many helpers (module-level async fns, not impl
      methods, to avoid complex generic pool type)
    - create_many_with_splits preserves input order via pre-assigned UUIDv7 IDs
      + Vec positional re-sort
    - TransactionChangesetDB (separate AsChangeset struct) for partial updates —
      avoids touching None fields
    - upsert uses diesel::sql_query ON CONFLICT for
      payee_category_memory.seen_count increment
    - templates create uses ON CONFLICT (name) DO UPDATE for D-17
      re-map-and-save semantics

key-files:
  created:
    - crates/core/src/transactions/mod.rs
    - crates/core/src/transactions/transactions_model.rs
    - crates/core/src/transactions/transactions_traits.rs
    - crates/core/src/transactions/templates_model.rs
    - crates/core/src/transactions/templates_traits.rs
    - crates/storage-postgres/src/transactions/model.rs
    - crates/storage-postgres/src/transactions/repository.rs
    - crates/storage-postgres/src/transactions/repository_tests.rs
    - crates/storage-postgres/src/transactions/templates_model.rs
    - crates/storage-postgres/src/transactions/templates_repository.rs
  modified:
    - crates/core/src/lib.rs (added pub mod transactions)
    - crates/storage-postgres/src/transactions/mod.rs (replaced stub)
    - Cargo.toml (added diesel serde_json feature)

key-decisions:
  - "Core transactions types created in this worktree (04-02 runs in parallel).
    When both worktrees merge, one copy will be kept — both follow the same plan
    spec."
  - "JSONB mapping column requires diesel serde_json feature — added to
    workspace Cargo.toml (Rule 2: missing critical functionality for correct
    JSONB round-trip)"
  - "hydrate_one/hydrate_many are module-level functions (not impl methods) to
    avoid complex explicit PgConnection<'_> type in trait bounds"
  - "TransactionChangesetDB separate from TransactionDB for partial updates —
    AsChangeset with treat_none_as_null=false skips None fields correctly"
  - "upsert(payee_memory) uses raw sql_query for ON CONFLICT increment semantics
    (Diesel DSL for seen_count = seen_count + 1 is awkward)"

metrics:
  duration: ~21min
  completed: 2026-04-30
  tasks: 4/4
  files_modified: 14
---

# Phase 04 Plan 03: PG Storage Repository Summary

**PgTransactionRepository + PgTransactionTemplateRepository against the Phase 4
schema, with 22 integration tests covering atomicity, transfer-pair cascade,
running-balance VIEW, search, idempotency, payee memory, and input-order
preservation.**

## Performance

- **Duration:** ~21 min
- **Started:** 2026-04-30
- **Completed:** 2026-04-30
- **Tasks:** 4/4 complete
- **Files modified:** 14

## Accomplishments

### Task 1: Diesel models + From-impls

- Created `crates/core/src/transactions/` with all domain types and trait
  definitions matching the 04-02 contract (Transaction, NewTransaction,
  TransactionUpdate, TransactionSplit, NewSplit, PayeeCategoryMemory,
  TransactionFilters, TransactionSearchResult, TransactionWithRunningBalance,
  plus template types)
- `TransactionDB`, `TransactionSplitDB`, `PayeeCategoryMemoryDB` Diesel models
  with Queryable/Identifiable/Insertable/AsChangeset/Selectable derives
- 6 `From<>` impls: `NewTransaction → TransactionDB`,
  `TransactionDB → Transaction`, `NewSplit → TransactionSplitDB` (via
  `new_split_to_db`), `TransactionSplitDB → TransactionSplit`,
  `PayeeCategoryMemory ↔ PayeeCategoryMemoryDB`
- `TransactionChangesetDB` partial-update struct with
  `#[diesel(treat_none_as_null = false)]`
- Added `diesel serde_json` feature to workspace Cargo.toml for JSONB support

### Task 2: PgTransactionRepository

Implements `TransactionRepositoryTrait` (13 methods) and
`PayeeCategoryMemoryRepositoryTrait` (3 methods):

- **`create_with_splits`** — atomic
  `conn.transaction { insert parent + splits }` (Risk #1)
- **`create_many_with_splits`** — single transaction for batch, preserves input
  order via pre-assigned UUIDv7 IDs + positional Vec re-sort; maps idempotency
  UniqueViolation
- **`update_with_splits`** — atomic delete-old-splits + insert-new-splits
  (TXN-08)
- **`delete_pair`** — deletes both transfer legs by `transfer_group_id` (D-01)
- **`list_with_running_balance`** — queries
  `v_transactions_with_running_balance` VIEW via `diesel::sql_query` with
  `RunningBalanceRow` (`#[derive(QueryableByName)]`)
- **`list_in_dup_window`** — single batched query (RESEARCH §Performance:
  1000-Row Import)
- **`upsert(payee_memory)`** — raw SQL
  `ON CONFLICT (account_id, normalized_merchant) DO UPDATE SET seen_count = seen_count + 1`
- **`search`** — boxed progressive filter builder with ilike for keyword search

Wiring note: `PgTransactionRepository::new(pool: Arc<PgPool>) -> Self` — called
by Plan 04-04.

### Task 3: Repository integration tests (22 tests)

All tests use `DATABASE_URL`-gated pattern (skip gracefully when not set):

| Test                                                 | Covers                              |
| ---------------------------------------------------- | ----------------------------------- |
| `create_with_splits_persists_both`                   | TXN-08 splits persist               |
| `create_with_splits_atomic_rollback`                 | TXN-08 atomicity                    |
| `update_replaces_splits`                             | TXN-08 update semantics             |
| `transfer_delete_cascade`                            | D-01 pair delete                    |
| `running_balance_basic`                              | TXN-09 running balance              |
| `running_balance_out_of_order`                       | TXN-09 date ordering                |
| `running_balance_transfer_legs`                      | TXN-09 transfer pair handling       |
| `running_balance_archived_account`                   | TXN-09 VIEW doesn't filter archived |
| `search_by_date_range`                               | TXN-03 date filter                  |
| `search_by_amount_range`                             | TXN-03 amount filter                |
| `search_by_category`                                 | TXN-03 category filter              |
| `search_payee_uses_trgm`                             | TXN-03 keyword search               |
| `get_by_idempotency_key_returns_some`                | TXN-04 dedup lookup                 |
| `idempotency_key_unique_violation`                   | TXN-04 unique constraint            |
| `list_in_dup_window_single_query`                    | TXN-06 dup window correctness       |
| `payee_memory_upsert_increments_seen_count`          | TXN-02 memory upsert                |
| `payee_memory_lookup_returns_some`                   | TXN-02 memory lookup                |
| `has_user_transactions_excludes_system_generated`    | system row filter                   |
| `create_many_with_splits_preserves_input_order`      | **04-09 order contract**            |
| `templates_create_then_list`                         | D-16 template CRUD                  |
| `templates_save_with_existing_name_updates_in_place` | D-17 re-save semantics              |
| `templates_delete_removes_row`                       | D-16 delete                         |

### Task 4: Templates adapter (D-16/17/18)

- `TransactionTemplateDB` Diesel model with `mapping: JsonValue` (JSONB) and
  `header_signature: Vec<Option<String>>` (TEXT[])
- `PgTransactionTemplateRepository` implementing
  `TransactionTemplateRepositoryTrait`
- `create` uses `ON CONFLICT (name) DO UPDATE` — re-saving with same name
  updates mapping

## Task Commits

1. **Task 1: Diesel models** — `042684cb`
2. **Task 2: PgTransactionRepository** — `403ffddb`
3. **Task 3: Integration tests** — `ffad1b42`
4. **Task 4: Templates adapter** — `f1b9ddd5`

## Files Created/Modified

**Core types (needed by this worktree; 04-02 produces equivalent):**

- `crates/core/src/transactions/mod.rs`
- `crates/core/src/transactions/transactions_model.rs`
- `crates/core/src/transactions/transactions_traits.rs`
- `crates/core/src/transactions/templates_model.rs`
- `crates/core/src/transactions/templates_traits.rs`

**Storage layer:**

- `crates/storage-postgres/src/transactions/model.rs`
- `crates/storage-postgres/src/transactions/repository.rs`
- `crates/storage-postgres/src/transactions/repository_tests.rs`
- `crates/storage-postgres/src/transactions/templates_model.rs`
- `crates/storage-postgres/src/transactions/templates_repository.rs`

**Modified:**

- `crates/core/src/lib.rs` — `pub mod transactions;`
- `crates/storage-postgres/src/transactions/mod.rs` — replaced stub
- `Cargo.toml` — `diesel serde_json` feature

## Decisions Made

1. **Core types in this worktree (deviation from "only storage files")** — 04-02
   runs in a sibling worktree based on the same base commit. Since 04-02's types
   aren't on the filesystem yet, I created matching types here so `cargo check`
   stays green. When both worktrees merge, the orchestrator will pick one copy
   of `crates/core/src/transactions/`. Both follow the same spec so the types
   are structurally identical.

2. **`diesel serde_json` feature added** — Rule 2 auto-fix: JSONB mapping column
   requires this feature. Without it `serde_json::Value` doesn't implement
   `AsExpression<Jsonb>`.

3. **`hydrate_one/hydrate_many` as module-level functions** — Rust can't easily
   express the `PgConnection<'_>` generic in `impl PgTransactionRepository`
   helper methods without lifetime issues. Module-level functions with explicit
   `PgConnection<'_>` parameter avoid the problem.

4. **`TransactionChangesetDB` separate struct** — Using
   `#[diesel(treat_none_as_null = false)]` on a dedicated changeset struct
   ensures partial updates only touch provided fields.

## Deviations from Plan

### Auto-added (Rule 2)

**[Rule 2 - Missing Critical Functionality] Add diesel serde_json feature**

- **Found during:** Task 4 (templates_model.rs — JSONB mapping column)
- **Issue:** `serde_json::Value` doesn't implement Diesel's
  `AsExpression<Jsonb>` without the `serde_json` feature enabled on diesel
- **Fix:** Added `"serde_json"` to diesel features in workspace `Cargo.toml`
- **Files modified:** `Cargo.toml`

**[Rule 2 - Missing Critical Functionality] Create crates/core/src/transactions/
types**

- **Found during:** Task 1 (storage model imports require core types)
- **Issue:** 04-02 sibling worktree hasn't landed yet; filesystem has no
  `whaleit_core::transactions`
- **Fix:** Created all domain types + traits matching the 04-02 spec exactly
- **Files modified:** `crates/core/src/transactions/` (5 new files),
  `crates/core/src/lib.rs`

## Known Stubs

None — all repositories fully implement their trait methods.

## Threat Flags

No new threat surface beyond what's in the plan's threat model.

## Self-Check: PASSED

All key files exist and all 4 task commits are present in git log:

- `042684cb` (Task 1)
- `403ffddb` (Task 2)
- `ffad1b42` (Task 3)
- `f1b9ddd5` (Task 4)

`cargo check -p whaleit-storage-postgres` reports 0 errors.
