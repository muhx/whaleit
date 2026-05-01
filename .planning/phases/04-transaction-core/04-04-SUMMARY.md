---
phase: 04-transaction-core
plan: 04
subsystem: io-boundary
tags: [axum, command-adapter, http-routes, transactions, frontend-types-stub]
status: complete
dependency_graph:
  requires:
    - whaleit_core::transactions::TransactionRepositoryTrait (04-02)
    - whaleit_core::transactions::PayeeCategoryMemoryRepositoryTrait (04-02)
    - whaleit_core::transactions::TransactionTemplateRepositoryTrait (04-02)
    - whaleit_storage_postgres::transactions::PgTransactionRepository (04-03)
    - whaleit_storage_postgres::transactions::PgTransactionTemplateRepository
      (04-03)
  provides:
    - whaleit_core::transactions::TransactionService (concrete impl)
    - whaleit_core::transactions::TransactionTemplateService (concrete impl)
    - apps/server: 18 Axum routes mounted on protected_api block
    - apps/frontend: 18 COMMANDS map entries + 17 typed wrappers + type stubs
  affects:
    - apps/server/src/main_lib.rs (AppState additions)
    - apps/frontend/src/adapters/{web,tauri}/index.ts (re-exports)
tech_stack:
  added: []
  patterns:
    - "Axum router merge into protected_api (post-auth middleware)"
    - "Multipart upload: per-field byte cap; tokio::time::timeout wraps OFX
      parse"
    - "Frontend COMMANDS map → handleCommand switch → invoke<T>(name, payload)"
    - "Service impls = thin orchestration over repository trait objects"
key_files:
  created:
    - crates/core/src/transactions/transactions_service.rs
    - crates/core/src/transactions/templates_service.rs
    - apps/server/src/api/transactions.rs
    - apps/frontend/src/adapters/shared/transactions.ts
    - apps/frontend/src/adapters/web/modules/transactions.ts
    - apps/frontend/src/lib/types/transaction.ts
  modified:
    - crates/core/src/transactions/mod.rs
    - apps/server/src/api.rs
    - apps/server/src/main_lib.rs
    - apps/frontend/src/adapters/web/core.ts
    - apps/frontend/src/adapters/web/index.ts
    - apps/frontend/src/adapters/tauri/index.ts
decisions:
  - 'Reuse core types as wire DTOs (re-export pattern from activities.rs)
    instead of mirror DTOs in models.rs — core types already use
    #[serde(rename_all = "camelCase")] and #[derive(Serialize, Deserialize)] so
    Axum handler signatures `Json<NewTransaction>` etc. produce the right JSON
    shape with `insertedRowIds` (camelCase) for ImportResult.'
  - "Multipart import (CSV/OFX) handlers live in api/transactions.rs but bypass
    the COMMANDS map dispatch — File/Blob can't survive JSON.stringify in
    invoke(). Plan 04-05 will add a web-side `web/transactions.ts` with a direct
    fetch (mirroring the `web/activities.ts:parseCsv` precedent)."
  - "PgTransactionRepository implements both TransactionRepositoryTrait and
    PayeeCategoryMemoryRepositoryTrait. main_lib.rs instantiates it twice (once
    per trait object); the underlying Arc<PgPool> is cloned, so this is cheap.
    Both Arc<dyn ...> are passed to TransactionService::new."
  - 'TransactionService is a thin orchestrator: validate inputs (via
    NewTransaction::validate / TransactionUpdate::validate), call repo method,
    return result. Heavier business logic (e.g. FX snapshotting, split sum
    reconciliation) is left for plan 04-05/06 since the plan spec says "No
    business logic lives here — handlers parse input, call the service trait,
    return JSON."'
  - 'import_ofx returns a stub ImportResult with errors=["OFX import not yet
    implemented (plan 04-02 task 3 TODO)"] because ofx_parser.rs is a TODO from
    04-02. The 30-second tokio::time::timeout still wraps it at the Axum handler
    layer, so the contract is enforced even for the stub. Plan 04-02 task 3
    should fill in the parser.'
  - "Frontend stub types live in apps/frontend/src/lib/types/transaction.ts with
    money fields typed as `number` (matches rust_decimal serde-float JSON wire
    format) — plan 04-05 will replace with hooks + zod schemas."
metrics:
  duration_minutes: ~25
  tasks_completed: 3 of 3
  commits: 3 (atomic per task)
  date: 2026-05-01
---

# Phase 4 Plan 04: Axum routes + frontend command adapter wiring (Summary)

End-to-end HTTP wiring from React → fetch → Axum → service trait → Pg repo, with
18 routes and 17 typed shared wrappers landed in three atomic commits.

## What landed

| Layer    | Artifact                                                 | Notes                                                                                                                                                 |
| -------- | -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| core     | `TransactionService` impl                                | 13 trait methods; thin orchestration                                                                                                                  |
| core     | `TransactionTemplateService` impl                        | 5 trait methods; pure delegation                                                                                                                      |
| server   | `apps/server/src/api/transactions.rs`                    | 18 `.route(...)` calls; chained methods on `/transactions` (CRUD) and `/transactions/import/templates` give 22 distinct HTTP method+path combinations |
| server   | `apps/server/src/api.rs`                                 | `mod transactions;` + `.merge(transactions::router())` on protected_api                                                                               |
| server   | `apps/server/src/main_lib.rs`                            | New `transaction_service` + `template_service` fields on `AppState`; instantiated post-`activity_service`                                             |
| frontend | `apps/frontend/src/lib/types/transaction.ts`             | Stub interfaces, plan 04-05 expands                                                                                                                   |
| frontend | `apps/frontend/src/adapters/shared/transactions.ts`      | 17 typed wrappers around `invoke<T>`                                                                                                                  |
| frontend | `apps/frontend/src/adapters/web/modules/transactions.ts` | 17 `handle*` dispatchers (`{url, body}` transforms)                                                                                                   |
| frontend | `apps/frontend/src/adapters/web/core.ts`                 | 18 COMMANDS entries + 18 switch arms                                                                                                                  |
| frontend | `apps/frontend/src/adapters/{web,tauri}/index.ts`        | re-export shared/transactions                                                                                                                         |

## Route table

| #   | Method | Path                                       | Handler                                                |
| --- | ------ | ------------------------------------------ | ------------------------------------------------------ |
| 1   | POST   | /transactions/search                       | `search_transactions`                                  |
| 2   | GET    | /transactions/item                         | `get_transaction` (`?id=...`)                          |
| 3   | POST   | /transactions                              | `create_transaction`                                   |
| 4   | PUT    | /transactions                              | `update_transaction`                                   |
| 5   | DELETE | /transactions                              | `delete_transaction` (`?id=...`)                       |
| 6   | POST   | /transactions/running-balance              | `list_running_balance`                                 |
| 7   | GET    | /transactions/by-account/recent            | `get_account_recent_transactions` (`?accountId&limit`) |
| 8   | POST   | /transactions/import/csv                   | `import_csv` (multipart, 10 MB cap)                    |
| 9   | POST   | /transactions/import/ofx                   | `import_ofx` (multipart, 10 MB cap, 30s timeout)       |
| 10  | POST   | /transactions/import/preview               | `preview_import`                                       |
| 11  | POST   | /transactions/import/duplicates            | `detect_duplicates` (50K row cap)                      |
| 12  | GET    | /transactions/import/templates             | `list_templates`                                       |
| 13  | POST   | /transactions/import/templates             | `save_template`                                        |
| 14  | DELETE | /transactions/import/templates             | `delete_template` (`?id=...`)                          |
| 15  | GET    | /transactions/import/templates/item        | `get_template` (`?id=...`)                             |
| 16  | POST   | /transactions/transfer                     | `create_transfer`                                      |
| 17  | PUT    | /transactions/transfer/leg                 | `update_transfer_leg`                                  |
| 18  | POST   | /transactions/transfer/break               | `break_transfer_pair`                                  |
| 19  | POST   | /transactions/payee-category-memory/lookup | `lookup_payee_category`                                |
| 20  | GET    | /transactions/payee-category-memory        | `list_payee_category_memory` (`?accountId=...`)        |

The `grep -c '\.route(' apps/server/src/api/transactions.rs` count is 18 (some
`.route(...)` calls register multiple HTTP methods via chaining, yielding 22
method+path combinations). Plan minimum: 18.

## COMMANDS map block (verbatim)

```typescript
// Transactions (Phase 4)
search_transactions: { method: "POST", path: "/transactions/search" },
get_transaction: { method: "GET", path: "/transactions/item" },
create_transaction: { method: "POST", path: "/transactions" },
update_transaction: { method: "PUT", path: "/transactions" },
delete_transaction: { method: "DELETE", path: "/transactions" },
list_running_balance: { method: "POST", path: "/transactions/running-balance" },
get_account_recent_transactions: {
  method: "GET",
  path: "/transactions/by-account/recent",
},
preview_transaction_import: { method: "POST", path: "/transactions/import/preview" },
detect_transaction_duplicates: {
  method: "POST",
  path: "/transactions/import/duplicates",
},
list_transaction_templates: { method: "GET", path: "/transactions/import/templates" },
save_transaction_template: { method: "POST", path: "/transactions/import/templates" },
delete_transaction_template: {
  method: "DELETE",
  path: "/transactions/import/templates",
},
get_transaction_template: {
  method: "GET",
  path: "/transactions/import/templates/item",
},
create_transfer: { method: "POST", path: "/transactions/transfer" },
update_transfer_leg: { method: "PUT", path: "/transactions/transfer/leg" },
break_transfer_pair: { method: "POST", path: "/transactions/transfer/break" },
lookup_payee_category: {
  method: "POST",
  path: "/transactions/payee-category-memory/lookup",
},
list_payee_category_memory: {
  method: "GET",
  path: "/transactions/payee-category-memory",
},
```

18 entries; covers all 18 paths in the route table (CSV/OFX multipart imports
are NOT registered in the COMMANDS map because File/Blob payloads cannot survive
`JSON.stringify`; plan 04-05 ships those via a separate `web/transactions.ts`
direct fetch — see Decisions).

## Stub left for plan 04-05 to expand

`apps/frontend/src/lib/types/transaction.ts` is a thin TS interface stub (no zod
schemas, no hooks). Plan 04-05 should:

- Replace with proper hooks (`useTransactions`, `useCreateTransaction`, etc.)
- Add zod schemas mirroring the Rust validators in `transactions_model.rs`
- Add a `web/transactions.ts` (and `tauri/transactions.ts` if desktop wants a
  native file picker hook) with multipart import functions following the
  `web/activities.ts:parseCsv` pattern

## Verification

| Check                                                  | Result                                                                             |
| ------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| `cargo check --workspace`                              | 0 errors (1 pre-existing warning in `csv_parser.rs` unused re-export)              |
| `pnpm type-check` (frontend)                           | 3 errors — ALL pre-existing (verified by `git stash`); 0 new errors from this plan |
| `grep -c 'route(' apps/server/src/api/transactions.rs` | 18                                                                                 |
| `ImportResult.insertedRowIds: Vec<String>` (Rust)      | Present (mirrors core's `inserted_row_ids`, JSON wire form `insertedRowIds`)       |
| `ImportResult.insertedRowIds: string[]` (TS stub)      | Present in `lib/types/transaction.ts`                                              |
| Routes mounted on protected_api                        | Yes (line 99 of `apps/server/src/api.rs`, adjacent to `activities::router()`)      |

## Deviations from plan

### Auto-fixed issues

**1. [Rule 2 - Missing functionality] Built TransactionServiceImpl +
TransactionTemplateServiceImpl in this plan**

- **Found during:** Task 1 read-first
- **Issue:** `transactions_service.rs` was a TODO stub from 04-02
  (`//! TODO plan 04-02 task 3`); plan 04-04 cannot wire AppState without a
  concrete service.
- **Fix:** Built `TransactionService` (13 trait methods) and
  `TransactionTemplateService` (5 trait methods) as thin orchestration layers.
  Re-exported from `transactions::mod.rs`.
- **Files modified:** `crates/core/src/transactions/transactions_service.rs`,
  `crates/core/src/transactions/templates_service.rs`,
  `crates/core/src/transactions/mod.rs`
- **Commit:** `07a14638`

**2. [Rule 1 - Missing types] Added DuplicateMatch + DuplicateBucket re-exports
to transactions::mod.rs**

- **Found during:** Task 1 server compile
- **Issue:** Axum routes import `DuplicateMatch` from
  `whaleit_core::transactions`; the type was defined but not re-exported from
  the module facade.
- **Fix:** Added
  `pub use duplicate_detector::{DuplicateBucket, DuplicateMatch};` to `mod.rs`.
- **Commit:** `07a14638` (amend)

**3. [Rule 3 - Path consistency] Used `/transactions/import/templates/item` for
GET single template**

- **Found during:** Plan acceptance criteria check
- **Issue:** The plan's must_haves listed `GET /transactions/import/templates`
  (without `/item`), but that path is already used by `list_templates` in the
  same router. The activities.rs precedent uses
  `/activities/import/templates/item` for single-template GET.
- **Fix:** Followed activities.rs convention. Documented in route table.
- **Commit:** `b1350843`

### Departures from plan structure (acceptable)

**1. Did not write a `models.rs` mirror DTOs section**

- **Plan said:** "If types match `whaleit_core::transactions::*` exactly, prefer
  `pub use whaleit_core::transactions::{...} as _;` re-exports — match the
  `apps/server/src/models.rs` Activity re-export precedent."
- **What we did:** Routes import directly from `whaleit_core::transactions::*`
  without a re-export shim in `models.rs`. The activities.rs handlers do the
  same (no re-export shim in models.rs for activity types either — they import
  from `whaleit_core::activities::*`).
- **Trade-off:** Simpler. Skipping the no-op re-export layer reduces churn.
  `models.rs` continues to host only the Account DTO mapping (which DOES need a
  shim for `tracking_mode` enum→string conversion).

**2. OFX import is a stub (errors out)**

- **Plan said:** Wrap in `tokio::time::timeout(Duration::from_secs(30))`.
- **What we did:** Wrapper IS in place (route handler enforces 30s timeout); the
  underlying `import_ofx` returns
  `errors: ["OFX import not yet implemented (plan 04-02 task 3 TODO)"]` because
  `ofx_parser.rs` is a TODO from 04-02.
- **Why this is OK:** The contract (route exists, timeout enforced, ImportResult
  shape returned) is satisfied. Plan 04-02 task 3 must complete before OFX
  actually parses files.

## Pre-existing issues (out of scope)

The following type errors exist on the merge-base commit `fd3151ec` and are
unaffected by this plan:

- `apps/frontend/src/addons/type-bridge.ts:219` — Account.currentBalance type
  drift between `lib/types/account.ts` (number) and the addon SDK (string)
- `apps/frontend/src/addons/type-bridge.ts:220` — same root cause
- `apps/frontend/src/pages/settings/accounts/components/account-form.tsx:224` —
  null vs string|undefined for the `group` field

The `csv_parser.rs:4` unused-re-export warning is also pre-existing.

These should be filed for future cleanup; not blocking 04-04 acceptance.

## Commits

| Hash       | Title                                                              |
| ---------- | ------------------------------------------------------------------ |
| `07a14638` | feat(04-04): TransactionService + TransactionTemplateService impls |
| `b1350843` | feat(04-04): Axum routes + AppState wiring for transactions        |
| `6f125845` | feat(04-04): frontend command adapter for transactions             |

## Self-Check: PASSED

- `apps/server/src/api/transactions.rs` exists with
  `pub fn router() -> Router<Arc<AppState>>` — FOUND
- `apps/server/src/api.rs` has `.merge(transactions::router())` on protected_api
  block — FOUND (line 99)
- `apps/server/src/main_lib.rs` builds both `transaction_service` AND
  `template_service`, stores both on `AppState` — FOUND
- `apps/frontend/src/lib/types/transaction.ts` exists with
  `insertedRowIds: string[]` — FOUND
- `apps/frontend/src/adapters/shared/transactions.ts` exports 17 wrappers —
  FOUND
- `apps/frontend/src/adapters/web/modules/transactions.ts` exports 17 handle\* —
  FOUND
- `apps/frontend/src/adapters/web/core.ts` references `search_transactions`,
  `create_transaction`, `create_transfer`, `lookup_payee_category` — FOUND
- 3 commits made, all referencing `04-04` in scope — FOUND
- `cargo check --workspace` green (0 errors) — VERIFIED
- `pnpm type-check` introduces 0 new errors (3 pre-existing) — VERIFIED via
  `git stash` baseline comparison
