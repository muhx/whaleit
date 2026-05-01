# Phase 4: Transaction Core - Pattern Map

**Mapped:** 2026-04-30 **Files analyzed:** ~70 new + ~6 modified across 8 layers
**Analogs found:** 65 / 76 (high coverage; 11 fresh files have no direct analog
— notably `ofx_parser.rs`, `duplicate_detector.rs`, `merchant_normalizer.rs`,
`reconciliation.rs`, `duplicate-review-sheet.tsx`, `duplicate-review-list.tsx`,
`split-editor.tsx`, `duplicate-banner.tsx`, plus their tests; documented in §No
Analog Found).

> **Read first:** Project layout in `<pattern_mapping_context>` overrides any
> stale path references in CONTEXT/RESEARCH. Server routes live at
> `apps/server/src/api/`, NOT `apps/server/src/routes/`. Frontend hooks live at
> `apps/frontend/src/hooks/`. Bottom-nav primary slot is at
> `apps/frontend/src/pages/layouts/navigation/app-navigation.tsx` (not
> `components/header.tsx`).

> **Universal pattern reminders (apply to ALL Phase 4 files):**
>
> 1. **Money** = `rust_decimal::Decimal` in Rust, `NUMERIC(20,8)` in PG, JSON
>    number on the wire (per Phase 3 fix `7e9eb697` — `serde-float` is on),
>    `z.number()` in frontend Zod. NEVER `String`.
> 2. **IDs** = UUIDv7 string. Always `Uuid::now_v7().to_string()` at the
>    repository boundary, NEVER `Uuid::new_v4()` for transactions/splits/etc.
>    The single exception in the codebase is
>    `generate_manual_idempotency_key()`, which uses v4 — Phase 4 reuses that
>    helper unchanged.
> 3. **Repository pattern** = trait in
>    `crates/core/src/<domain>/<domain>_traits.rs`, impl in
>    `crates/storage-postgres/src/<domain>/repository.rs`, Diesel AsChangeset
>    model in `crates/storage-postgres/src/<domain>/model.rs`.
> 4. **Time** = `chrono::NaiveDateTime` in Rust, `TIMESTAMP` (no tz) in PG, ISO
>    string in TS, server-side stamping with `chrono::Utc::now().naive_utc()`.
>    Mirrors Phase 3 D-12 `balance_updated_at` rule: `accounts_service.rs:118`
>    is canonical.
> 5. **Domain events** — every mutation that affects derived state emits via
>    `event_sink.emit(DomainEvent::*)`; transactions emit a new
>    `TransactionsChanged` (planner adds the variant in
>    `crates/core/src/events/domain_event.rs`).
> 6. **`#[serde(rename_all = "camelCase")]`** on every public domain struct that
>    crosses the FFI/HTTP boundary — frontend types use camelCase fields.

## File Classification

### Layer 1 — Migration SQL (NEW)

| New file                                                                          | Role               | Data flow  | Closest analog                                                                                                                                                                                                    | Match quality                                                                          |
| --------------------------------------------------------------------------------- | ------------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `crates/storage-postgres/migrations/20260501000000_transactions_initial/up.sql`   | migration          | DDL + seed | `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql` (extends pattern: file-per-DDL-step + nullable cols) and `20260101000000_initial_schema/up.sql` (full table create) | exact-pattern (combine both: NEW table create + system-taxonomy seed in one migration) |
| `crates/storage-postgres/migrations/20260501000000_transactions_initial/down.sql` | migration-rollback | DROP       | `20260425000000_accounts_extend_types_and_balances/down.sql`                                                                                                                                                      | exact-pattern                                                                          |

### Layer 2 — Rust core domain (NEW under `crates/core/src/transactions/`)

| New file                        | Role         | Data flow                       | Closest analog                                                                                                                                                                               | Match quality                                                                                         |
| ------------------------------- | ------------ | ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `mod.rs`                        | module index | re-exports                      | `crates/core/src/accounts/mod.rs` (12 lines, re-exports + cfg-test) and `crates/core/src/activities/mod.rs` (45 lines — multi-export for compiler/parser/idempotency)                        | exact (use activities/mod.rs as the template — Phase 4 also has compiler + parser + idempotency)      |
| `transactions_model.rs`         | model        | domain struct + validation      | `crates/core/src/accounts/accounts_model.rs` (270 lines: `Account`, `NewAccount`, `AccountUpdate` + `validate()` impls, all `#[serde(rename_all = "camelCase")]`)                            | exact-pattern                                                                                         |
| `transactions_constants.rs`     | constants    | CRUD-touched                    | `crates/core/src/accounts/accounts_constants.rs` (`account_types::CHECKING` etc., short flat const strings)                                                                                  | exact-pattern                                                                                         |
| `transactions_traits.rs`        | traits       | repository + service contract   | `crates/core/src/accounts/accounts_traits.rs` (92 lines: `#[async_trait] pub trait XRepositoryTrait: Send + Sync` + `XServiceTrait`)                                                         | exact-pattern                                                                                         |
| `transactions_service.rs`       | service      | request-response + event emit   | `crates/core/src/accounts/accounts_service.rs` (259 lines, but ~600-1000 LOC expected for txn service — owns import path); refer also to `activities_service.rs` for import-flow LOC profile | role-match (bigger; absorb activities-service patterns for `import_csv` / `import_ofx` orchestration) |
| `transactions_service_tests.rs` | test         | various                         | `crates/core/src/accounts/accounts_service_tests.rs` (mock repo + scenario tables)                                                                                                           | exact-pattern                                                                                         |
| `transactions_errors.rs`        | error type   | error mapping                   | `crates/core/src/activities/activities_errors.rs` (557B — single thin enum)                                                                                                                  | exact-pattern                                                                                         |
| `compiler.rs`                   | utility      | transform: parsed-rows → domain | `crates/core/src/activities/compiler.rs` lines 36-67 (`compile_drip` etc.)                                                                                                                   | role-match (simpler — txns are 1:1, no leg expansion)                                                 |
| `csv_parser.rs`                 | utility      | file-I/O parse                  | `crates/core/src/activities/csv_parser.rs` (18.6K — full BOM + delimiter + header detection)                                                                                                 | re-export (RESEARCH §CSV Parsing Strategy mandates re-export verbatim)                                |
| `ofx_parser.rs`                 | utility      | file-I/O parse                  | (no analog) — uses `sgmlish` 0.2 per RESEARCH §OFX Parsing Strategy                                                                                                                          | fresh-build (see §No Analog Found)                                                                    |
| `ofx_parser_tests.rs`           | test         | parse fixtures                  | `crates/core/src/activities/csv_parser.rs` test block (fixture-driven)                                                                                                                       | weak-pattern                                                                                          |
| `duplicate_detector.rs`         | utility      | event-driven (import-time)      | (no analog) — uses `strsim::normalized_levenshtein` per RESEARCH §Don't Hand-Roll                                                                                                            | fresh-build                                                                                           |
| `duplicate_detector_tests.rs`   | test         | scenario table                  | `crates/core/src/accounts/accounts_model_tests.rs` (validation table)                                                                                                                        | weak-pattern                                                                                          |
| `idempotency.rs`                | utility      | hash compute                    | `crates/core/src/activities/idempotency.rs` (315 lines: `compute_idempotency_key()` + tests, SHA-256 of pipe-delimited normalized fields)                                                    | exact-pattern                                                                                         |
| `merchant_normalizer.rs`        | utility      | string transform                | (no analog — D-13 algo is novel; pure-Rust regex)                                                                                                                                            | fresh-build                                                                                           |
| `merchant_normalizer_tests.rs`  | test         | scenario table                  | (none)                                                                                                                                                                                       | fresh                                                                                                 |
| `reconciliation.rs`             | utility      | once-per-account hook           | (no analog — Phase 3 D-14 contract)                                                                                                                                                          | fresh-build                                                                                           |

### Layer 3 — Rust storage (NEW under `crates/storage-postgres/src/transactions/`)

| New file              | Role         | Data flow                    | Closest analog                                                                                                                                                                                                                                                                                              | Match quality            |
| --------------------- | ------------ | ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------ |
| `mod.rs`              | module index | re-exports                   | `crates/storage-postgres/src/accounts/mod.rs` (14 lines: `pub mod model; pub mod repository; pub use model::AccountDB; pub use repository::PgAccountRepository; #[cfg(test)] mod ...`)                                                                                                                      | exact                    |
| `model.rs`            | model        | Diesel mapping + From<>impls | `crates/storage-postgres/src/accounts/model.rs` (168 lines: `#[derive(Queryable, Identifiable, Insertable, AsChangeset, Selectable, ...)]` + `From<AccountDB> for Account` + `From<NewAccount> for AccountDB` + `From<AccountUpdate> for AccountDB`)                                                        | exact-pattern            |
| `repository.rs`       | repository   | CRUD + custom queries        | `crates/storage-postgres/src/accounts/repository.rs` (138 lines: `Arc<PgPool>` + `#[async_trait] impl AccountRepositoryTrait` with `create()`/`update()`/`get_by_id()`/`list()`/`delete()`); refer to `activities/repository.rs` for transaction-spanning ops (`Diesel transaction` for create-with-splits) | exact-pattern (extended) |
| `repository_tests.rs` | test         | testcontainers integration   | `crates/storage-postgres/src/accounts/repository_tests.rs` (8.1K)                                                                                                                                                                                                                                           | exact-pattern            |
| `migration_tests.rs`  | test         | up-down round-trip           | `crates/storage-postgres/src/accounts/migration_tests.rs` (2.0K)                                                                                                                                                                                                                                            | exact-pattern            |

### Layer 4 — Rust web routes (Axum)

| New / modified file                                                                                                                                                                                                              | Role              | Data flow        | Closest analog                                                                                                                         | Match quality |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------- | ---------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `apps/server/src/api/transactions.rs` (NEW)                                                                                                                                                                                      | controller        | request-response | `apps/server/src/api/activities.rs` (399 lines: search-body deser, multipart import, router builder) — closer match than `accounts.rs` | exact-pattern |
| `apps/server/src/api.rs` (MODIFY: add `.merge(transactions::router())` at line 110-area)                                                                                                                                         | router-aggregator | request-response | `apps/server/src/api.rs:92-110` (existing `.merge()` chain)                                                                            | exact-pattern |
| `apps/server/src/main_lib.rs` (MODIFY: build `transaction_service` + add to `AppState`)                                                                                                                                          | wiring            | startup          | `main_lib.rs:193` (`AccountService::new`) and `main_lib.rs:317` (`activity_service` build)                                             | exact-pattern |
| `apps/server/src/models.rs` (MODIFY: add HTTP-DTO `Transaction`/`NewTransaction`/`TransactionUpdate` + `From<>` impls if any HTTP-side sanitization needed; mirrors the H-03 fix pattern noted in `accounts_service.rs:111-124`) | DTO               | request-response | `apps/server/src/models.rs` (existing `Account`/`NewAccount`/`AccountUpdate` HTTP DTOs at line ~150-300)                               | exact-pattern |

### Layer 5 — Frontend types + hooks + adapters

| New / modified file                                                                                                                          | Role             | Data flow             | Closest analog                                                                                                                                                                        | Match quality                                                                 |
| -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `apps/frontend/src/lib/types/transaction.ts` (NEW)                                                                                           | type-only        | n/a                   | `apps/frontend/src/lib/types/account.ts` (79 lines: `interface Account` + `AccountSummaryView` etc., camelCase fields, `number` for money, `Date` for stamps, `string` ISO for dates) | exact-pattern                                                                 |
| `apps/frontend/src/lib/types/index.ts` (MODIFY: add `export * from "./transaction"`)                                                         | barrel           | n/a                   | existing barrel                                                                                                                                                                       | exact-pattern                                                                 |
| `apps/frontend/src/hooks/use-transactions.ts` (NEW)                                                                                          | hook             | TanStack-query CRUD   | `apps/frontend/src/hooks/use-accounts.ts` (33 lines: `useQuery<Account[], Error>` + `QueryKeys` constant + filter memo)                                                               | exact-pattern (extend with mutations: `useMutation` for create/update/delete) |
| `apps/frontend/src/hooks/use-merchant-categories.ts` (NEW)                                                                                   | hook             | TanStack-query lookup | `apps/frontend/src/hooks/use-taxonomies.ts` (7.4K — has full CRUD mutation set)                                                                                                       | role-match                                                                    |
| `apps/frontend/src/lib/query-keys.ts` (MODIFY: add `TRANSACTIONS`, `MERCHANT_CATEGORIES`, `TRANSACTION_TEMPLATES`)                           | constant         | n/a                   | existing `QueryKeys.ACCOUNTS` etc.                                                                                                                                                    | exact-pattern                                                                 |
| `apps/frontend/src/adapters/shared/transactions.ts` (NEW)                                                                                    | adapter shim     | request-response      | `apps/frontend/src/adapters/shared/activities.ts` (331 lines: typed wrappers around `invoke<T>("command_name", payload)` + `try/catch` + `logger.error`)                              | exact-pattern                                                                 |
| `apps/frontend/src/adapters/shared/index.ts` (MODIFY: add `export * from "./transactions"`)                                                  | barrel           | n/a                   | existing barrel                                                                                                                                                                       | exact-pattern                                                                 |
| `apps/frontend/src/adapters/web/core.ts` (MODIFY at line ~99 inside `COMMANDS`: add Phase-4 entries from RESEARCH §`COMMANDS` Map Additions) | command-registry | request-response      | `core.ts:84-99` (existing `Activities` block — same shape, paths under `/transactions/...`)                                                                                           | exact-pattern                                                                 |
| `apps/frontend/src/adapters/web/modules/transactions.ts` (NEW)                                                                               | command-handlers | request-response      | `apps/frontend/src/adapters/web/modules/activities.ts` (1.9K: `handle*` functions for payload→URL/body transform)                                                                     | exact-pattern                                                                 |
| `apps/frontend/src/adapters/web/core.ts` (MODIFY at line ~444 in `handleCommand` switch: add `case` arms for new commands)                   | router           | request-response      | `core.ts:445-473` (`Activities` block)                                                                                                                                                | exact-pattern                                                                 |

### Layer 6 — Frontend pages (global ledger, NEW under `apps/frontend/src/pages/transactions/`)

| New file                       | Role      | Data flow                       | Closest analog                                                                                                                                     | Match quality                                         |
| ------------------------------ | --------- | ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| `transactions-page.tsx`        | page      | request-response + filter state | `apps/frontend/src/pages/account/account-page.tsx` (Page+Header+Card layout, Sheet for detail, `useQuery` driven, mobile-aware)                    | role-match (different domain, same layout primitives) |
| `transaction-detail-sheet.tsx` | component | display + edit                  | `apps/frontend/src/pages/account/account-page.tsx` (`Sheet`/`SheetContent`/`SheetHeader` usage)                                                    | role-match                                            |
| `transaction-form.tsx`         | component | request-response (mutation)     | `apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx` (`MoneyInput` + `MoneyInput`/`PrivacyAmount` + Zod + form pattern) | role-match                                            |
| `transaction-row.tsx`          | component | display                         | `apps/frontend/src/pages/dashboard/accounts-summary.tsx` (row shape per CONTEXT `<code_context>`)                                                  | role-match (visual reference)                         |
| `transaction-list.tsx`         | component | display + virtualized list      | (no direct analog with date-grouping; closest is `apps/frontend/src/pages/activity/components/activity-table/*`)                                   | weak-pattern                                          |
| `filter-bar/` (multiple files) | component | state + UI                      | UI-SPEC §3 vocabulary uses existing `ToggleGroup`, chip pattern from `account-page.tsx` filter chips                                               | role-match                                            |
| `recent-transactions.tsx`      | component | embedded display                | `apps/frontend/src/pages/dashboard/accounts-summary.tsx` row pattern                                                                               | role-match                                            |
| `split-editor.tsx`             | component | state + repeater                | (no analog)                                                                                                                                        | fresh                                                 |
| `duplicate-banner.tsx`         | component | display + action                | `apps/frontend/src/pages/activity/import/import-validation-alert.tsx` (`AlertFeedback` usage)                                                      | role-match                                            |
| `duplicate-review-sheet.tsx`   | component | request-response (review)       | `apps/frontend/src/pages/activity/import/steps/review-step.tsx` (review-pattern)                                                                   | role-match                                            |

### Layer 7 — Frontend import wizard fork (NEW under `apps/frontend/src/pages/transactions/import/`)

| New file                                     | Role               | Data flow                                      | Closest analog                                                                                                                                                                      | Match quality                                                                                                                                                                                                                                      |
| -------------------------------------------- | ------------------ | ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `transaction-import-page.tsx`                | page (wizard root) | step-machine                                   | `apps/frontend/src/pages/activity/import/activity-import-page.tsx` (650 lines: `STEPS` const, `STEP_COMPONENTS` map, `ImportProvider` + `ImportWizardContent`, `useStepValidation`) | exact-pattern (FORK)                                                                                                                                                                                                                               |
| `context/transaction-import-context.tsx`     | context provider   | state                                          | `apps/frontend/src/pages/activity/import/context/import-context.tsx` (26.5K: reducer + `DraftActivity` type + `ImportState`)                                                        | exact-pattern (FORK — drop `assetCandidateKey`/`importAssetKey`/`exchangeMic`/`symbolName`/`quoteCcy`/`instrumentType`/`quoteMode`; add `direction`/`payee`/`categoryId`/`splits`/`duplicateConfidence`/`duplicateBucket`/`existingTransactionId`) |
| `components/file-dropzone.tsx`               | component          | re-export                                      | `apps/frontend/src/pages/activity/import/components/file-dropzone.tsx`                                                                                                              | share-verbatim (single-line re-export)                                                                                                                                                                                                             |
| `components/csv-file-viewer.tsx`             | component          | re-export                                      | `csv-file-viewer.tsx`                                                                                                                                                               | share-verbatim                                                                                                                                                                                                                                     |
| `components/wizard-step-indicator.tsx`       | component          | re-export                                      | `wizard-step-indicator.tsx`                                                                                                                                                         | share-verbatim                                                                                                                                                                                                                                     |
| `components/step-navigation.tsx`             | component          | re-export                                      | `step-navigation.tsx`                                                                                                                                                               | share-verbatim                                                                                                                                                                                                                                     |
| `components/help-tooltip.tsx`                | component          | re-export                                      | `help-tooltip.tsx`                                                                                                                                                                  | share-verbatim                                                                                                                                                                                                                                     |
| `components/cancel-confirmation-dialog.tsx`  | component          | re-export with prop-parameterized copy         | `cancel-confirmation-dialog.tsx`                                                                                                                                                    | share-with-extend (add optional `title`/`description`/`confirmLabel` props if not already there; otherwise pure re-export)                                                                                                                         |
| `components/transaction-mapping-table.tsx`   | component          | mapping editor                                 | `apps/frontend/src/pages/activity/import/components/mapping-table.tsx` (5.4K)                                                                                                       | exact-pattern (FORK — drop asset-resolution column; add `direction` inference column + `payee` column)                                                                                                                                             |
| `components/transaction-template-picker.tsx` | component          | template picker                                | `apps/frontend/src/pages/activity/import/components/template-picker.tsx` (145 lines: Popover + Command + system/user grouping)                                                      | exact-pattern (FORK — labels + ImportTemplateScope type stays the same, but template payload shape differs: D-17 adds `header_signature` field)                                                                                                    |
| `components/duplicate-review-list.tsx`       | component          | display                                        | (no direct analog — closest is `import-preview-table.tsx`)                                                                                                                          | weak-pattern                                                                                                                                                                                                                                       |
| `steps/upload-step.tsx`                      | step component     | file-I/O                                       | `apps/frontend/src/pages/activity/import/steps/upload-step.tsx` (32.3K)                                                                                                             | exact-pattern (FORK — drop instrument-type detection, add OFX MIME accept)                                                                                                                                                                         |
| `steps/transaction-mapping-step.tsx`         | step component     | mapping state                                  | `apps/frontend/src/pages/activity/import/steps/mapping-step-unified.tsx` (27.2K — D-17 header-mismatch detection lives here)                                                        | exact-pattern (FORK — drop asset/symbol UI, keep header detection)                                                                                                                                                                                 |
| `steps/transaction-review-step.tsx`          | step component     | request-response (validate) + duplicate inline | `apps/frontend/src/pages/activity/import/steps/review-step.tsx` (17.0K)                                                                                                             | exact-pattern (FORK — surface duplicates inline per UI-SPEC §6)                                                                                                                                                                                    |
| `steps/transaction-confirm-step.tsx`         | step component     | request-response (commit)                      | `apps/frontend/src/pages/activity/import/steps/confirm-step.tsx` (18.8K)                                                                                                            | exact-pattern (FORK — simpler result stats)                                                                                                                                                                                                        |
| `utils/transaction-draft-utils.ts`           | utility            | transform                                      | `apps/frontend/src/pages/activity/import/utils/draft-utils.ts` (19.2K)                                                                                                              | exact-pattern (FORK — `createDraftTransactions` instead of `createDraftActivities`)                                                                                                                                                                |
| `utils/transaction-default-template.ts`      | utility            | factory                                        | `apps/frontend/src/pages/activity/import/utils/default-activity-template.ts` (2.2K)                                                                                                 | exact-pattern (FORK — different field set)                                                                                                                                                                                                         |
| `hooks/use-transaction-import-mapping.ts`    | hook               | state                                          | `apps/frontend/src/pages/activity/import/hooks/use-import-mapping.ts` (12.9K — `computeFieldMappings` exported here)                                                                | exact-pattern (FORK)                                                                                                                                                                                                                               |

### Layer 8 — Frontend integration into existing screens

| Modified file                                                                                                                                                                                                                                           | Role       | Data flow        | Closest analog                                                                                                   | Match quality                                                                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ---------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ------------ | ------------------------------------------------------------- |
| `apps/frontend/src/routes.tsx` (MODIFY at line ~155)                                                                                                                                                                                                    | router     | n/a              | line 149-154 (existing `/activities` + `/import` routes nested under `<Route path="/" element={<AppLayout />}>`) | exact-pattern (add `<Route path="transactions" element={<TransactionsPage />} />` and `<Route path="transactions/import" element={<TransactionImportPage />} />` adjacent to line 154) |
| `apps/frontend/src/pages/layouts/navigation/app-navigation.tsx` (MODIFY: add primary nav slot for Transactions per UI-SPEC §Responsive)                                                                                                                 | nav-config | static           | `app-navigation.tsx:20-49` (existing `primary` array of `{icon, title, href, label, keywords}`)                  | exact-pattern (insert between Activities (line 44) and AI Assistant (line 59); slot order `Dashboard                                                                                   | Activities | Transactions | Settings`— UI-SPEC requested`Reports` slot waits for Phase 6) |
| `apps/frontend/src/pages/account/account-page.tsx` (MODIFY: mount `<RecentTransactions accountId={id} />` per UI-SPEC §account-page integration)                                                                                                        | page       | request-response | existing imports of `AccountHoldings`, `AccountMetrics` (sibling components)                                     | exact-pattern                                                                                                                                                                          |
| `apps/frontend/src/pages/account/account-page.tsx` (MODIFY: replace Phase-3 manual "Update balance" CTA with computed-from-txns balance + read-only display, per CONTEXT `<decisions>` "Transactions become the source of truth for `current_balance`") | page       | display          | existing `UpdateBalanceModal` import                                                                             | role-match                                                                                                                                                                             |
| `apps/frontend/src/components/header.tsx`                                                                                                                                                                                                               | header     | static           | (NO change — header has no nav-items; nav lives in `app-navigation.tsx`)                                         | n/a                                                                                                                                                                                    |

## Pattern Assignments

> Below: per-file concrete code excerpts with file:line citations. **Every
> excerpt is verbatim from the analog at the cited line range.** Planner copies
> these into action specs.

---

### `crates/storage-postgres/migrations/20260501000000_transactions_initial/up.sql`

**Analog:**
`crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql:1-22`

**Header-comment + ALTER pattern** (lines 1-7 of analog):

```sql
-- Phase 3: Bank accounts, credit cards, and balance fields.
-- Adds 11 nullable columns to accounts. Money columns use NUMERIC(20,8) per
-- decision D-10 (resolved 2026-04-25). This diverges from the existing TEXT
-- pattern used in 20260101000000_initial_schema for money columns; only NEW
-- Phase 3 columns use NUMERIC. Existing TEXT-stored money columns are
-- unchanged.
```

**Diff hint:** Phase 4 file is a CREATE TABLE (not ALTER) — full schema is
verbatim from RESEARCH §Schema Design lines 546-689. Header-comment pattern is
preserved (start with phase + decision references). Money columns use
`NUMERIC(20,8)` (matches Phase 3 D-10).

**Seed pattern within migration up.sql** — RESEARCH §Seed lines 691-731 (the
seed lives **inside** `up.sql`, not in `taxonomy_service::initialize()`). Use
`INSERT INTO ... ON CONFLICT (id) DO NOTHING` for idempotency.

---

### `crates/storage-postgres/migrations/20260501000000_transactions_initial/down.sql`

**Analog:**
`crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql:1-12`

**Reverse-DDL pattern** (full file):

```sql
ALTER TABLE accounts
    DROP COLUMN IF EXISTS institution,
    ...
```

**Diff hint:** Reverse Phase 4 in dependency order —
`DROP VIEW v_transactions_with_running_balance`,
`DROP TABLE transaction_splits`, `DROP TABLE payee_category_memory`,
`DROP TABLE transactions`,
`DELETE FROM taxonomy_categories WHERE taxonomy_id = 'sys_taxonomy_transaction_categories'`,
`DELETE FROM taxonomies WHERE id = 'sys_taxonomy_transaction_categories'`. Use
`DROP TABLE IF EXISTS` and `DELETE` (not `DROP TABLE`) so reruns are safe.

---

### `crates/core/src/transactions/mod.rs`

**Analog:** `crates/core/src/activities/mod.rs:1-45`

**Re-export pattern** (lines 1-45 of analog):

```rust
//! Activities module - domain models, services, and traits.

mod activities_constants;
mod activities_errors;
mod activities_model;
mod activities_service;
mod activities_traits;
mod compiler;
mod csv_parser;
mod idempotency;
mod import_run_model;

#[cfg(test)]
mod activities_service_tests;

#[cfg(test)]
mod activities_model_tests;

pub use activities_constants::*;
pub use activities_errors::ActivityError;
pub use activities_model::{Activity, NewActivity, ActivityUpdate, ...};
pub use activities_service::ActivityService;
pub use activities_traits::{ActivityRepositoryTrait, ActivityServiceTrait};
pub use compiler::{ActivityCompiler, DefaultActivityCompiler};
pub use csv_parser::{parse_csv, ParseConfig, ParseError, ParsedCsvResult};
pub use idempotency::{compute_idempotency_key, ...};
```

**Diff hint:** Phase 4 adds
`mod ofx_parser; mod duplicate_detector; mod merchant_normalizer; mod reconciliation;`
plus their `#[cfg(test)] mod *_tests;`. Re-export `Transaction`,
`NewTransaction`, `TransactionUpdate`, `TransactionSplit`, `NewSplit`,
`PayeeCategoryMemory`, plus `parse_ofx`, `detect_duplicates`,
`normalize_merchant`, `compute_transaction_idempotency_key`,
`synthesize_reconciliation_rows`.

---

### `crates/core/src/transactions/transactions_model.rs`

**Analog:** `crates/core/src/accounts/accounts_model.rs:1-270`

**Imports + struct shape** (lines 1-71 of analog):

```rust
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::accounts::account_types;
use crate::{errors::ValidationError, Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    ...
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    ...
}
```

**`NewX` validate impl pattern** (lines 118-193 of analog):

```rust
impl NewAccount {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account name cannot be empty".to_string(),
            )));
        }
        // ... domain-specific D-rules
        Ok(())
    }
}
```

**`XUpdate` validate impl pattern** (lines 236-269 of analog):

```rust
impl AccountUpdate {
    pub fn validate(&self) -> Result<()> {
        if self.id.is_none() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account ID is required for updates".to_string(),
            )));
        }
        ...
    }
}
```

**Diff hint:**

- `Transaction` struct fields = full set in RESEARCH §Schema Design lines
  552-600 (`id`, `account_id`, `direction`, `amount: Decimal`, `currency`,
  `transaction_date: NaiveDate`, `payee: Option<String>`, `notes`,
  `category_id: Option<String>`, `has_splits: bool`, `fx_rate: Option<Decimal>`,
  `fx_rate_source: Option<String>`, `transfer_group_id`,
  `counterparty_account_id`, `transfer_leg_role: Option<String>`,
  `idempotency_key`, `import_run_id`, `source`, `external_ref`,
  `is_system_generated`, `is_user_modified`, `category_source`, `created_at`,
  `updated_at`).
- `NewTransaction.validate()` enforces:
  `direction in {INCOME, EXPENSE, TRANSFER}`; `amount > 0`;
  `direction == TRANSFER` ⟺
  `transfer_group_id.is_some() && counterparty_account_id.is_some() && transfer_leg_role.is_some()`;
  `direction != TRANSFER` ⟹ `payee.is_some()` (mirrors PG check
  `non_transfer_must_have_payee`); SUM(splits) == amount when has_splits.
- Add `TransactionSplit`, `NewSplit`, `SplitUpdate`, `PayeeCategoryMemory`
  structs in same file (mirrors `accounts_model.rs` containing both `Account`
  and the related `TrackingMode` enum on line 11-21).

---

### `crates/core/src/transactions/transactions_traits.rs`

**Analog:** `crates/core/src/accounts/accounts_traits.rs:1-92`

**Repository trait pattern** (lines 16-46 of analog):

```rust
#[async_trait]
pub trait AccountRepositoryTrait: Send + Sync {
    async fn create(&self, new_account: NewAccount) -> Result<Account>;
    async fn update(&self, account_update: AccountUpdate) -> Result<Account>;
    async fn delete(&self, account_id: &str) -> Result<usize>;
    async fn get_by_id(&self, account_id: &str) -> Result<Account>;
    async fn list(
        &self,
        is_active_filter: Option<bool>,
        is_archived_filter: Option<bool>,
        account_ids: Option<&[String]>,
    ) -> Result<Vec<Account>>;
}
```

**Service trait pattern** (lines 52-91 of analog):

```rust
#[async_trait]
pub trait AccountServiceTrait: Send + Sync {
    async fn create_account(&self, new_account: NewAccount) -> Result<Account>;
    async fn update_account(&self, account_update: AccountUpdate) -> Result<Account>;
    async fn delete_account(&self, account_id: &str) -> Result<()>;
    async fn get_account(&self, account_id: &str) -> Result<Account>;
    async fn list_accounts(
        &self,
        is_active_filter: Option<bool>,
        is_archived_filter: Option<bool>,
        account_ids: Option<&[String]>,
    ) -> Result<Vec<Account>>;
    ...
    fn get_base_currency(&self) -> Option<String>;
}
```

**Diff hint:** `TransactionRepositoryTrait` adds:
`create_with_splits(NewTransaction, Vec<NewSplit>) -> Result<Transaction>`,
`update_with_splits(TransactionUpdate, Option<Vec<SplitUpdate>>) -> Result<Transaction>`,
`search(filters, page, page_size, sort) -> Result<TransactionSearchResponse>`,
`list_in_window(account_id, date_from, date_to) -> Result<Vec<Transaction>>`
(3-day duplicate window),
`list_with_running_balance(account_id, date_from, date_to) -> Result<Vec<TransactionWithRunningBalance>>`
(queries `v_transactions_with_running_balance`),
`lookup_payee_memory(normalized_merchant, account_id) -> Result<Option<PayeeCategoryMemory>>`,
`upsert_payee_memory(normalized_merchant, account_id, category_id) -> Result<()>`
(D-14 last-write-wins), `delete_pair(transfer_group_id) -> Result<()>` (D-01
cascade delete),
`get_existing_idempotency_keys(keys: &[String]) -> Result<HashMap<String, String>>`
(re-import dedup; mirrors `activities_service.rs::check_existing_duplicates`).

`TransactionServiceTrait` adds the same with high-level business-flow methods:
`import_csv(file, mapping, account_id)`, `import_ofx(file, account_id)`,
`detect_duplicates(candidates) -> Vec<DuplicateMatch>`,
`create_transfer(source_id, dest_id, source_amount, dest_amount, ...) -> (Transaction, Transaction)`,
`break_transfer_pair(leg_id) -> Transaction`.

---

### `crates/core/src/transactions/transactions_service.rs`

**Analog:** `crates/core/src/accounts/accounts_service.rs:1-258` plus the
import-flow methods on `crates/core/src/activities/activities_service.rs`.

**Service struct + new() pattern** (lines 14-43 of analog):

```rust
pub struct AccountService {
    repository: Arc<dyn AccountRepositoryTrait>,
    fx_service: Arc<dyn FxServiceTrait>,
    base_currency: Arc<RwLock<String>>,
    event_sink: Arc<dyn DomainEventSink>,
    asset_repository: Arc<dyn AssetRepositoryTrait>,
    sync_state_store: Arc<dyn SyncStateStore>,
}

impl AccountService {
    pub fn new(
        repository: Arc<dyn AccountRepositoryTrait>,
        fx_service: Arc<dyn FxServiceTrait>,
        base_currency: Arc<RwLock<String>>,
        event_sink: Arc<dyn DomainEventSink>,
        asset_repository: Arc<dyn AssetRepositoryTrait>,
        sync_state_store: Arc<dyn SyncStateStore>,
    ) -> Self {
        Self { repository, fx_service, base_currency, event_sink, asset_repository, sync_state_store }
    }
}
```

**Mutate-and-emit pattern** (lines 47-77 of analog):

```rust
async fn create_account(&self, new_account: NewAccount) -> Result<Account> {
    let base_currency = self.base_currency.read().unwrap().clone();
    if new_account.currency != base_currency {
        self.fx_service.register_currency_pair(...).await?;
    }
    let result = self.repository.create(new_account).await?;
    let currency_changes = vec![CurrencyChange { ... }];
    self.event_sink.emit(DomainEvent::accounts_changed(
        vec![result.id.clone()],
        currency_changes,
    ));
    Ok(result)
}
```

**Server-stamping rule (D-12 mirror)** (lines 109-124 of analog):

```rust
// D-12: auto-stamp balance_updated_at when current_balance changes.
// The client never gets to set this field — server is the source of truth
// ...
if account_update.current_balance.is_some()
    && account_update.current_balance != existing.current_balance
{
    account_update.balance_updated_at = Some(chrono::Utc::now().naive_utc());
} else {
    // Belt and suspenders for D-12 ...
    account_update.balance_updated_at = None;
}
```

**Diff hint:** `TransactionService` adds these dependencies in the constructor:
`payee_memory_repository: Arc<dyn PayeeCategoryMemoryRepositoryTrait>`,
`taxonomy_repository: Arc<dyn TaxonomyRepositoryTrait>` (for category
validation), `account_repository: Arc<dyn AccountRepositoryTrait>` (for D-14
reconciliation hook).

Critical service-layer behaviors:

- `create_transaction()`: validate; FX-snapshot if
  `currency != account.currency` (call
  `fx_service.get_rate(date, txn.currency, account.currency)` and write to
  `fx_rate` + `fx_rate_source = SYSTEM`); idempotency key compute; if first
  non-system row → call `reconciliation::synthesize_reconciliation_rows()`
  before insert; if direction != TRANSFER and payee.is_some() → upsert payee
  memory; emit `DomainEvent::transactions_changed(vec![id])`.
- `update_transaction()`: server-stamp `updated_at`; if amount or category_id
  changed and is leg of pair → run D-04 alert dialog server-side contract
  (return a typed `EditConflictRequiresConfirmation` error or the client always
  passes the resolution); D-14 last-write-wins on category change → upsert payee
  memory.
- `delete_transaction()`: if `transfer_group_id.is_some()` → cascade delete
  sibling row first via `repository.delete_pair(group_id)`; emit
  `TransactionsChanged`.
- `import_csv()` / `import_ofx()`: parse → `compiler::compile_*` → assign
  idempotency keys → check existing keys → run
  `duplicate_detector::detect_duplicates` for the user-visible review step →
  return `ImportPreview { duplicates, ready }`. Final commit method takes the
  user's overrides (`force_import` flags, category overrides) and calls
  `repository.create_with_splits` in a Diesel transaction per row.

Reuse `compute_transaction_idempotency_key` from `idempotency.rs` (FITID-aware).

---

### `crates/storage-postgres/src/transactions/model.rs`

**Analog:** `crates/storage-postgres/src/accounts/model.rs:1-167`

**Diesel struct pattern** (lines 9-44 of analog):

```rust
#[derive(Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccountDB {
    #[diesel(column_name = id)]
    pub id: String,
    pub name: String,
    ...
    #[diesel(skip_insertion)]
    pub created_at: NaiveDateTime,
    #[diesel(skip_insertion)]
    pub updated_at: NaiveDateTime,
    ...
    pub opening_balance: Option<Decimal>,
    pub current_balance: Option<Decimal>,
    pub balance_updated_at: Option<NaiveDateTime>,
    ...
}
```

**`From<DB> for Domain` pattern** (lines 46-83):

```rust
impl From<AccountDB> for Account {
    fn from(db: AccountDB) -> Self {
        let tracking_mode = match db.tracking_mode.as_str() {
            "TRANSACTIONS" => TrackingMode::Transactions,
            "HOLDINGS" => TrackingMode::Holdings,
            _ => TrackingMode::NotSet,
        };
        Self { id: db.id, name: db.name, ..., tracking_mode }
    }
}
```

**`From<NewDomain> for DB` pattern** (lines 85-124):

```rust
impl From<NewAccount> for AccountDB {
    fn from(domain: NewAccount) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: domain.id.unwrap_or_default(),  // repository overwrites with Uuid::now_v7()
            name: domain.name,
            ...
            created_at: now,
            updated_at: now,
            ...
        }
    }
}
```

**Diff hint:** Add three `*DB` structs: `TransactionDB`, `TransactionSplitDB`,
`PayeeCategoryMemoryDB`. Each gets `From<*DB> for Domain` and
`From<New*> for *DB` plus `From<*Update> for *DB` (mirrors
`From<AccountUpdate> for AccountDB` at line 126-167). Direction string mapping
is straightforward (the domain already uses String per the schema's CHECK
constraint values, so no enum→string mapping needed unless you choose to model
`Direction` as a Rust enum like `TrackingMode`).

---

### `crates/storage-postgres/src/transactions/repository.rs`

**Analog:** `crates/storage-postgres/src/accounts/repository.rs:1-138`

**Imports + struct pattern** (lines 1-25 of analog):

```rust
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use super::model::AccountDB;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::accounts;
use crate::schema::accounts::dsl::*;
use whaleit_core::accounts::{Account, AccountRepositoryTrait, AccountUpdate, NewAccount};
use whaleit_core::errors::Result;

pub struct PgAccountRepository {
    pool: Arc<PgPool>,
}

impl PgAccountRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
```

**`create()` UUIDv7 pattern** (lines 28-43):

```rust
async fn create(&self, new_account: NewAccount) -> Result<Account> {
    new_account.validate()?;
    let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;

    let mut account_db: AccountDB = new_account.into();
    account_db.id = Uuid::now_v7().to_string();

    diesel::insert_into(accounts::table)
        .values(&account_db)
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

    Ok(account_db.into())
}
```

**`list()` boxed-query filter pattern** (lines 97-127):

```rust
async fn list(...) -> Result<Vec<Account>> {
    let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
    let mut query = accounts::table.into_boxed();
    if let Some(active) = is_active_filter { query = query.filter(is_active.eq(active)); }
    ...
    let results = query.select(AccountDB::as_select())
        .order((is_active.desc(), is_archived.asc(), name.asc()))
        .load::<AccountDB>(&mut conn).await
        .map_err(StoragePgError::from)?;
    Ok(results.into_iter().map(Account::from).collect())
}
```

**Diff hint:** `PgTransactionRepository` adds:

- `create_with_splits()` — wrap in
  `conn.transaction(|c| async move { ... }.scope_boxed()).await` (Diesel-async
  transaction). Insert parent row first, then bulk-insert splits via
  `diesel::insert_into(transaction_splits::table).values(&split_dbs).execute(c)`.
- `search()` — boxed query with all `TransactionFilters` fields per RESEARCH
  §Contract-Level Types lines 1486-1497. Pagination via
  `.limit(page_size).offset(page * page_size)`. Return
  `TransactionSearchResponse { data, meta }`.
- `list_with_running_balance()` — query the VIEW
  `v_transactions_with_running_balance` instead of base table. Add a
  `RunningBalanceDB` queryable struct in `model.rs` (mirrors PR pattern:
  `#[derive(Queryable, Selectable)] #[diesel(table_name = crate::schema::v_transactions_with_running_balance)]`).
- `delete_pair()` — single statement:
  `diesel::delete(transactions::table.filter(transfer_group_id.eq(group_id))).execute(c)`.
  Returns `usize` for affected rows.
- `lookup_payee_memory()` / `upsert_payee_memory()` — Diesel
  `INSERT ... ON CONFLICT (account_id, normalized_merchant) DO UPDATE SET category_id = EXCLUDED.category_id, last_seen_at = EXCLUDED.last_seen_at, seen_count = payee_category_memory.seen_count + 1`
  (use `diesel::pg::upsert::on_constraint`).

---

### `crates/core/src/transactions/idempotency.rs`

**Analog:** `crates/core/src/activities/idempotency.rs:1-315`

**Hash composition pattern** (lines 25-91 of analog):

```rust
#[allow(clippy::too_many_arguments)]
pub fn compute_idempotency_key(
    account_id: &str,
    activity_type: &str,
    activity_date: &DateTime<Utc>,
    asset_id: Option<&str>,
    quantity: Option<Decimal>,
    unit_price: Option<Decimal>,
    amount: Option<Decimal>,
    currency: &str,
    provider_reference_id: Option<&str>,
    description: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"|");
    hasher.update(activity_type.as_bytes());
    hasher.update(b"|");

    let date_str = activity_date.format("%Y-%m-%d").to_string();
    hasher.update(date_str.as_bytes());
    hasher.update(b"|");

    if let Some(aid) = asset_id { hasher.update(aid.as_bytes()); }
    hasher.update(b"|");
    ...
    if let Some(amt) = amount { hasher.update(normalize_decimal(amt).as_bytes()); }
    hasher.update(b"|");

    hasher.update(currency.as_bytes());
    hasher.update(b"|");

    if let Some(ref_id) = provider_reference_id { hasher.update(ref_id.as_bytes()); }
    hasher.update(b"|");

    if let Some(desc) = description {
        let normalized = normalize_description(desc);
        hasher.update(normalized.as_bytes());
    }

    let result = hasher.finalize();
    hex::encode(result)
}

fn normalize_decimal(d: Decimal) -> String { d.normalize().to_string() }
fn normalize_description(s: &str) -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") }
```

**Diff hint (per RESEARCH Pattern 3):**

```rust
pub fn compute_transaction_idempotency_key(
    account_id: &str,
    direction: &str,            // "INCOME" | "EXPENSE" | "TRANSFER"
    transaction_date: &NaiveDate,
    amount: Decimal,
    currency: &str,
    payee: Option<&str>,
    external_ref: Option<&str>, // OFX FITID or CSV bank-ref column
) -> String { /* SHA-256 hex */ }
```

Use `NaiveDate` (transactions are date-only; activities use `DateTime<Utc>` —
date is what matters for matching, time is irrelevant). FITID goes into
`external_ref`. Keep the `normalize_decimal` and pipe-delimiter pattern
verbatim. Reuse `generate_manual_idempotency_key()` from activities (it's
currently generic), or duplicate-with-rename-prefix `txn_manual:` if you want
phase isolation; either works.

---

### `apps/server/src/api/transactions.rs`

**Analog:** `apps/server/src/api/activities.rs:1-399`

**Imports + body-deser pattern** (lines 1-54):

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::{error::ApiResult, main_lib::AppState};
use axum::{
    extract::{Multipart, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use whaleit_core::activities::{ ... };

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum SortWrapper { One(...), Many(Vec<...>) }

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum StringOrVec { One(String), Many(Vec<String>) }

#[derive(serde::Deserialize)]
struct ActivitySearchBody {
    page: i64,
    #[serde(rename = "pageSize")]
    page_size: i64,
    #[serde(rename = "accountIdFilter")]
    account_id_filter: Option<StringOrVec>,
    ...
    #[serde(rename = "dateFrom")]
    date_from: Option<String>,
    #[serde(rename = "dateTo")]
    date_to: Option<String>,
    ...
}
```

**Handler + service-call pattern** (lines 56-101):

```rust
async fn search_activities(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ActivitySearchBody>,
) -> ApiResult<Json<ActivitySearchResponse>> {
    let date_from_parsed = parse_date_optional(body.date_from, "dateFrom")?;
    let date_to_parsed = parse_date_optional(body.date_to, "dateTo")?;
    let resp = state.activity_service.search_activities(...).await?;
    Ok(Json(resp))
}
```

**Multipart upload pattern** (lines 318-362):

```rust
async fn parse_csv_endpoint(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ApiResult<Json<ParsedCsvResult>> {
    let mut file_content: Option<Vec<u8>> = None;
    let mut config = ParseConfig::default();
    while let Some(field) = multipart.next_field().await.map_err(...)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => { file_content = Some(field.bytes().await.map_err(...)?.to_vec()); }
            "config" => { config = serde_json::from_slice(&config_bytes).map_err(...)?; }
            _ => {}
        }
    }
    let content = file_content.ok_or_else(|| crate::error::ApiError::BadRequest(...))?;
    let result = whaleit_core::activities::parse_csv(&content, &config)?;
    Ok(Json(result))
}
```

**Router builder pattern** (lines 364-399):

```rust
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/activities/search", post(search_activities))
        .route("/activities", post(create_activity).put(update_activity))
        .route("/activities/bulk", post(save_activities))
        .route("/activities/{id}", delete(delete_activity))
        .route("/activities/import/check", post(check_activities_import))
        ...
        .route("/activities/import/templates", get(list_import_templates).post(save_import_template).delete(delete_import_template))
        .route("/activities/import/templates/item", get(get_import_template))
        ...
}
```

**Diff hint:**

- Body structs: `TransactionSearchBody` mirrors `ActivitySearchBody` but field
  set per RESEARCH §Contract-Level Types. Reuse `StringOrVec` and
  `parse_date_optional` (already in `super::shared`).
- `TransferBody` is new:
  `{ source_account_id, dest_account_id, source_amount, dest_amount, source_currency, dest_currency, fx_rate, transaction_date, notes }`.
- `BreakTransferPairBody` = `{ leg_id }`.
- Route map per RESEARCH §`COMMANDS` Map Additions lines 1346-1376:
  ```rust
  pub fn router() -> Router<Arc<AppState>> {
      Router::new()
          .route("/transactions/search", post(search_transactions))
          .route("/transactions/item", get(get_transaction))
          .route("/transactions", post(create_transaction).put(update_transaction).delete(delete_transaction))
          .route("/transactions/running-balance", post(list_running_balance))
          .route("/transactions/by-account/recent", get(get_account_recent_transactions))
          .route("/transactions/import/csv", post(import_transactions_csv))
          .route("/transactions/import/ofx", post(import_transactions_ofx))
          .route("/transactions/import/preview", post(preview_transaction_import))
          .route("/transactions/import/duplicates", post(detect_transaction_duplicates))
          .route("/transactions/import/templates", get(list_transaction_templates).post(save_transaction_template).delete(delete_transaction_template))
          .route("/transactions/import/templates/item", get(get_transaction_template))
          .route("/transactions/transfer", post(create_transfer))
          .route("/transactions/transfer/leg", put(update_transfer_leg))
          .route("/transactions/transfer/break", post(break_transfer_pair))
          .route("/transactions/payee-category-memory/lookup", post(lookup_payee_category))
          .route("/transactions/payee-category-memory", get(list_payee_category_memory))
  }
  ```
- Multipart for `/transactions/import/csv` and `/transactions/import/ofx` —
  reuse the `parse_csv_endpoint` multipart shape.

---

### `apps/server/src/api.rs` (MODIFY)

**Analog:** `apps/server/src/api.rs:92-110`

**Existing merge pattern** (lines 92-110):

```rust
.merge(accounts::router())
.merge(settings::router())
.merge(portfolio::router())
.merge(holdings::router())
.merge(performance::router())
.merge(activities::router())
.merge(goals::router())
.merge(exchange_rates::router())
.merge(market_data::router())
.merge(assets::router())
.merge(secrets::router())
.merge(limits::router())
.merge(addons::router())
.merge(taxonomies::router())
.merge(net_worth::router())
.merge(alternative_assets::router())
.merge(ai_providers::router())
.merge(ai_chat::router())
```

**Diff hint:** Add `.merge(transactions::router())` at line ~98 (immediately
after `activities::router()` to keep alphabetical-by-domain ordering inside the
"core" block). Also add `pub mod transactions;` to `apps/server/src/api/mod.rs`
or wherever the api modules are declared (search for `pub mod activities;`).

---

### `apps/server/src/main_lib.rs` (MODIFY)

**Analog:** `main_lib.rs:317` (`activity_service` build) and `:193`
(`AccountService::new`).

**Diff hint:** Add `transaction_service` field to `pub struct AppState` (line
~70-127); construct in `build_state()` after `activity_service` is built so
event_sink, fx_service, and account_repository are all already available;
register on the AppState init at line 437-454.

---

### `apps/frontend/src/lib/types/transaction.ts`

**Analog:** `apps/frontend/src/lib/types/account.ts:1-79`

**Type pattern** (lines 7-36):

```typescript
export interface Account {
  id: string;
  name: string;
  accountType: AccountType;
  group?: string;
  currentBalance?: number; // Decimal serialized as JSON number (rust_decimal serde-float)
  currency: string;
  isDefault: boolean;
  isActive: boolean;
  isArchived: boolean;
  trackingMode: TrackingMode;
  createdAt: Date;
  updatedAt: Date;
  ...
  // Phase 3 additions:
  institution?: string;
  openingBalance?: number;
  balanceUpdatedAt?: Date;
  ...
  statementDueDate?: string; // ISO date
  ...
}
```

**Diff hint:** Use the **full set** from RESEARCH §Contract-Level Types lines
1429-1497 verbatim (TransactionDirection, TransactionSource, FxRateSource,
Transaction, TransactionSplit, TransactionWithRunningBalance,
DuplicateCandidate, TransactionFilters). Money fields use `number`. Date-only
columns use `string` (YYYY-MM-DD). DateTime columns use `string` (ISO 8601).
Mirror the `// Decimal serialized as JSON number` comment for amount/fxRate
fields.

---

### `apps/frontend/src/hooks/use-transactions.ts`

**Analog:** `apps/frontend/src/hooks/use-accounts.ts:1-33`

**Hook pattern** (full file, 33 lines):

```typescript
import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { Account } from "@/lib/types";
import { getAccounts } from "@/adapters";
import { QueryKeys } from "@/lib/query-keys";

export function useAccounts(options?: {
  filterActive?: boolean;
  includeArchived?: boolean;
}) {
  const { filterActive = true, includeArchived = false } = options ?? {};
  const {
    data: fetchedAccounts = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery<Account[], Error>({
    queryKey: [QueryKeys.ACCOUNTS, includeArchived],
    queryFn: () => getAccounts(includeArchived),
  });
  const filteredAccounts = useMemo(() => {
    let accounts = fetchedAccounts;
    if (filterActive) accounts = accounts.filter((a) => a.isActive);
    return accounts;
  }, [fetchedAccounts, filterActive]);
  return { accounts: filteredAccounts, isLoading, isError, error, refetch };
}
```

**Diff hint:** Multi-hook file. Export:

- `useTransactions(filters: TransactionFilters, page, pageSize, sort)` —
  `useQuery` against `searchTransactions` adapter; key
  `[QueryKeys.TRANSACTIONS, filters, page, pageSize, sort]`.
- `useTransaction(id)` — single-row fetch; key `[QueryKeys.TRANSACTIONS, id]`.
- `useCreateTransaction()` / `useUpdateTransaction()` / `useDeleteTransaction()`
  — mirrors
  `apps/frontend/src/pages/activity/import/hooks/use-activity-import-mutations.ts`
  pattern (1.8K analog, useMutation with onSuccess invalidate).
- `useRunningBalance(accountId, fromDate, toDate)`.
- `useRecentTransactions(accountId, limit)` — for `account-page.tsx` embed.

---

### `apps/frontend/src/adapters/shared/transactions.ts`

**Analog:** `apps/frontend/src/adapters/shared/activities.ts:1-331`

**Adapter wrapper pattern** (lines 100-141):

```typescript
export const createActivity = async (
  activity: ActivityCreate,
): Promise<Activity> => {
  try {
    return await invoke<Activity>("create_activity", { activity });
  } catch (err) {
    logger.error("Error creating activity.");
    throw err;
  }
};

export const updateActivity = async (
  activity: ActivityUpdate,
): Promise<Activity> => {
  try {
    return await invoke<Activity>("update_activity", { activity });
  } catch (err) {
    logger.error("Error updating activity.");
    throw err;
  }
};

export const deleteActivity = async (activityId: string): Promise<Activity> => {
  try {
    return await invoke<Activity>("delete_activity", { activityId });
  } catch (err) {
    logger.error("Error deleting activity.");
    throw err;
  }
};
```

**Search wrapper pattern** (lines 38-98):

```typescript
function normalizeStringArray(input?: string | string[]): string[] | undefined {
  if (!input) return undefined;
  if (Array.isArray(input)) return input.length > 0 ? input : undefined;
  return input.length > 0 ? [input] : undefined;
}

export const searchActivities = async (
  page: number, pageSize: number, filters: ActivityFilters,
  searchKeyword: string, sort?: ActivitySort,
): Promise<ActivitySearchResponse> => {
  const accountIdFilter = normalizeStringArray(filters?.accountIds);
  ...
  try {
    return await invoke<ActivitySearchResponse>("search_activities", {
      page, pageSize, accountIdFilter, ..., sort: sortOption, ..., dateFrom, dateTo
    });
  } catch (err) { logger.error("Error fetching activities."); throw err; }
};
```

**Diff hint:** Per RESEARCH §`adapters/shared/transactions.ts` lines 1378-1427.
Use the same `normalizeStringArray` helper (consider hoisting to a shared util
if it grows; for v1 inline-copy is fine — Project CLAUDE.md says "minimum code
that solves the problem"). Add typed wrappers per command in RESEARCH
§`COMMANDS` Map Additions.

---

### `apps/frontend/src/adapters/web/core.ts` (MODIFY)

**Analog:** `apps/frontend/src/adapters/web/core.ts:84-99` (existing Activities
block).

**Block pattern** (lines 83-99):

```typescript
// Activities
search_activities: { method: "POST", path: "/activities/search" },
create_activity: { method: "POST", path: "/activities" },
update_activity: { method: "PUT", path: "/activities" },
save_activities: { method: "POST", path: "/activities/bulk" },
delete_activity: { method: "DELETE", path: "/activities" },
// Activity import
check_activities_import: { method: "POST", path: "/activities/import/check" },
preview_import_assets: { method: "POST", path: "/activities/import/assets/preview" },
import_activities: { method: "POST", path: "/activities/import" },
get_account_import_mapping: { method: "GET", path: "/activities/import/mapping" },
save_account_import_mapping: { method: "POST", path: "/activities/import/mapping" },
link_account_template: { method: "POST", path: "/activities/import/templates/link" },
list_import_templates: { method: "GET", path: "/activities/import/templates" },
get_import_template: { method: "GET", path: "/activities/import/templates/item" },
save_import_template: { method: "POST", path: "/activities/import/templates" },
delete_import_template: { method: "DELETE", path: "/activities/import/templates" },
```

**Switch dispatch pattern** (lines 444-473):

```typescript
// ── Activities ──────────────────────────────────────────
case "search_activities":
  return activityHandlers.handleSearchActivities(url, p!);
case "create_activity":
  return activityHandlers.handleCreateActivity(url, p!);
...
```

**Diff hint:** Insert the full Phase-4 `COMMANDS` block from RESEARCH lines
1346-1376 immediately after the existing Activity-import block (line 99). For
each new command that doesn't need URL/body transformation (most of them; they
use simple JSON POST/PUT or query-string GET), add the `case` arm in
`handleCommand`. Most can fall through to `{ url, body: undefined }` — only
search and update commands need a `transactionHandlers.*` wrapper for camelCase
serialization. Mirror the activities-handlers shape verbatim.

---

### `apps/frontend/src/adapters/web/modules/transactions.ts` (NEW)

**Analog:** `apps/frontend/src/adapters/web/modules/activities.ts` (1.9K).

**Diff hint:** Single-purpose handle\* exports per command that need url/body
massaging. For most commands this file is small (each handler is ~5 lines):
extract payload, JSON.stringify, return `{ url, body }`.

---

### `apps/frontend/src/pages/transactions/import/transaction-import-page.tsx`

**Analog:**
`apps/frontend/src/pages/activity/import/activity-import-page.tsx:1-650`

**STEPS const + STEP_COMPONENTS pattern** (lines 62-77):

```typescript
const STEPS: WizardStep[] = [
  { id: "upload", label: "Upload" },
  { id: "mapping", label: "Mapping" },
  { id: "assets", label: "Review Assets" },
  { id: "review", label: "Review Activities" },
  { id: "confirm", label: "Import" },
];

const STEP_COMPONENTS: Record<ImportStep, React.ComponentType> = {
  upload: UploadStep,
  mapping: MappingStepUnified,
  assets: AssetReviewStep,
  review: ReviewStep,
  confirm: ConfirmStep,
  result: ContextResultStep,
};
```

**Wizard scaffolding** (lines 527-602): full
Page+Card+WizardStepIndicator+ErrorBoundary structure verbatim.

**Provider mount pattern** (lines 638-647):

```typescript
export function ActivityImportPageV2() {
  const [searchParams] = useSearchParams();
  const accountId = searchParams.get("account") || "";
  return (
    <ImportProvider initialAccountId={accountId}>
      <ImportWizardContent />
    </ImportProvider>
  );
}
```

**Diff hint:**

- STEPS array drops the `assets` entry (no asset review). New shape:
  `[{ id: "upload" }, { id: "mapping" }, { id: "review" }, { id: "confirm" }]`
  - UI-SPEC §6 says a dedicated "Review duplicates" step lives between Mapping
    and Confirm — implement as a sub-section of the Review step rather than a
    separate STEPS entry (UI-SPEC §6: "duplicate review inline within Review
    activities step"; revalidate against UI-SPEC).
- STEP_COMPONENTS map is forked: drop `assets`, drop `result` if not in scope
  (CONFIRM step shows result inline). Verify with UI-SPEC §6.
- `useStepValidation` hook (analog lines 107-297): drop the asset-resolution
  branches; add a "duplicate-resolved" gate (each duplicate row has either
  `keep_both: true`, `discard: true`, or `force_import: true` flag).
- `handleNext` `state.step === "mapping"` branch (lines 379-422): drop
  `previewAssets()` call; replace with `void detectDuplicates(drafts)`
  fire-and-forget that fills `state.duplicateCandidates`.
- Initial query param: `searchParams.get("accountId")` (per CONTEXT decisions
  which use camelCase) instead of `account`.

---

### `apps/frontend/src/pages/transactions/import/context/transaction-import-context.tsx`

**Analog:**
`apps/frontend/src/pages/activity/import/context/import-context.tsx:1-120` (and
remaining 26.5K file).

**Type shape** (lines 27-110):

```typescript
export type ImportStep = "upload" | "mapping" | "assets" | "review" | "confirm" | "result";

export interface ParseConfig {
  hasHeaderRow: boolean;
  headerRowIndex: number;
  delimiter: string;
  skipTopRows: number;
  skipBottomRows: number;
  skipEmptyRows: boolean;
  dateFormat: string;
  decimalSeparator: string;
  thousandsSeparator: string;
  defaultCurrency: string;
}

export type DraftActivityStatus = "valid" | "warning" | "error" | "skipped" | "duplicate";

export interface DraftActivity {
  rowIndex: number;
  rawRow: string[];
  activityDate: string;
  activityType: string;
  symbol?: string;
  ...
  // Asset resolution
  assetCandidateKey?: string;
  importAssetKey?: string;
  // Validation state
  status: DraftActivityStatus;
  errors: Record<string, string[]>;
  warnings: Record<string, string[]>;
  ...
}

export interface ImportState {
  step: ImportStep;
  file: File | null;
  parseConfig: ParseConfig;
  headers: string[];
  parsedRows: string[][];
  mapping: ImportMappingData | null;
  draftActivities: DraftActivity[];
  ...
}
```

**Diff hint:**

- `ImportStep` drops `"assets"` and possibly `"result"`.
- `DraftTransaction` (renamed `DraftActivity`) field set: keep `rowIndex`,
  `rawRow`, `transactionDate` (renamed from `activityDate`), `currency`,
  `accountId`, `comment`/`notes`, `status`, `errors`, `warnings`, `isEdited`.
  Drop: `activityType`, `symbol`, `assetId`, `quantity`, `unitPrice`, `subtype`,
  `fxRate`, `isExternal`, `isin`, `exchangeMic`, `symbolName`, `quoteCcy`,
  `instrumentType`, `quoteMode`, `assetCandidateKey`, `importAssetKey`. Add:
  `direction: TransactionDirection`, `amount`, `payee`, `categoryId`,
  `splits?: NewSplit[]`, `duplicateConfidence?: number`,
  `duplicateBucket?: "ALMOST_CERTAIN" | "LIKELY" | "POSSIBLE"`,
  `existingTransactionId?: string`,
  `userResolution?: "keep_both" | "discard" | "force_import"`.
- `ImportState` adds: `duplicateCandidates: DuplicateCandidate[]`,
  `isDetectingDuplicates: boolean`, `duplicateDetectionError: string | null`,
  `reconciliationWarning?: { openingBalance: number; balanceAdjustment: number }`.

---

### `apps/frontend/src/pages/transactions/import/components/transaction-template-picker.tsx`

**Analog:**
`apps/frontend/src/pages/activity/import/components/template-picker.tsx:1-145`

**Picker shape** (lines 25-144 of analog):

```typescript
export function TemplatePicker({
  templates, selectedTemplateId, onSelect, onClear, placeholder = "Select format…",
}: TemplatePickerProps) {
  const [open, setOpen] = useState(false);
  const systemTemplates = templates.filter((t) => t.scope === "SYSTEM");
  const userTemplates = templates.filter((t) => t.scope === "USER");
  const selected = templates.find((t) => t.id === selectedTemplateId);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" role="combobox" ... className="w-full justify-between rounded-lg font-normal">
          {selected ? (
            <span className="flex items-center gap-2 truncate">
              {selected.scope === "SYSTEM"
                ? <Icons.Building className="text-muted-foreground h-3.5 w-3.5 shrink-0" />
                : <Icons.User className="text-muted-foreground h-3.5 w-3.5 shrink-0" />}
              {selected.name}
            </span>
          ) : (
            <span className="text-muted-foreground">{placeholder}</span>
          )}
          ...
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[var(--radix-popover-trigger-width)] p-0" align="start">
        <Command>
          <CommandInput placeholder="Search formats…" className="h-9" />
          <CommandEmpty>...</CommandEmpty>
          {userTemplates.length > 0 && (
            <CommandGroup heading="Custom">{userTemplates.map(...)}</CommandGroup>
          )}
          {userTemplates.length > 0 && systemTemplates.length > 0 && <CommandSeparator />}
          {systemTemplates.length > 0 && (
            <CommandGroup heading="Built-in">{systemTemplates.map(...)}</CommandGroup>
          )}
        </Command>
      </PopoverContent>
    </Popover>
  );
}
```

**Diff hint:** Per D-16 (no starter pack), the `systemTemplates` filter will
ALWAYS be empty for transactions. The component still keeps the SYSTEM/USER
bifurcation logic — fold the heading rendering, but do NOT add a "no system
templates" branch (the existing `null` render via
`systemTemplates.length > 0 &&` is correct; nothing changes). Add
`headerSignature?: string` field handling per D-17 — when picker selects a
template, page-level state validates and returns
`"Your saved 'X' template doesn't match this file's columns. Re-map?"`.

---

### `apps/frontend/src/routes.tsx` (MODIFY)

**Analog:** `apps/frontend/src/routes.tsx:148-162`

**Existing route block**:

```tsx
<Route path="dashboard" element={<PortfolioPage />} />
<Route path="activities" element={<ActivityPage />} />
<Route path="activities/manage" element={<ActivityManagerPage />} />
<Route path="holdings" element={<HoldingsPage />} />
...
<Route path="import" element={<ActivityImportPage />} />
<Route path="accounts/:id" element={<AccountPage />} />
```

**Diff hint:** Add adjacent to line 154 (after `import`):

```tsx
<Route path="transactions" element={<TransactionsPage />} />
<Route path="transactions/import" element={<TransactionImportPage />} />
```

Also add the imports at the top of the file (currently lines 13-15 for activity
imports).

---

### `apps/frontend/src/pages/layouts/navigation/app-navigation.tsx` (MODIFY)

**Analog:** `app-navigation.tsx:20-49`

**Existing primary nav items**:

```tsx
primary: [
  {
    icon: <Icons.Dashboard className="size-6" />,
    title: "Dashboard",
    href: "/dashboard",
    label: "View Dashboard",
  },
  ...
  {
    title: "Activities",
    href: "/activities",
    label: "View Activities",
  },
  ...
]
```

**Diff hint:** Insert per UI-SPEC §Responsive
(`Dashboard | Accounts | Transactions | Reports | Settings` was the suggested
order; existing order is
`Dashboard | Insights | Holdings | Activities | AI Assistant | Settings`). Per
UI-SPEC `Reports waits for Phase 6` — add Transactions between Activities and AI
Assistant:

```tsx
{
  icon: <Icons.ArrowLeftRight className="size-6" />, // verify icon exists in @whaleit/ui Icons; fallback to Icons.Wallet or Icons.CreditCard
  title: "Transactions",
  href: "/transactions",
  keywords: ["Transactions", "income", "expense", "transfer", "ledger", "spending"],
  label: "View Transactions",
},
```

---

### `apps/frontend/src/pages/account/account-page.tsx` (MODIFY)

**Analog (sibling-component import pattern):** existing imports of
`AccountHoldings`, `AccountMetrics`, `AccountContributionLimit` near the top of
the file.

**Diff hint:**

- Add
  `import { RecentTransactions } from "@/pages/transactions/recent-transactions"`.
- Mount `<RecentTransactions accountId={id} />` per UI-SPEC §Account-page embed
  (UI-SPEC §10 — "Recent transactions" section, position adjacent to
  AccountMetrics).
- Per CONTEXT `<decisions>` "Transactions become the source of truth for
  `current_balance` (replacing the manual 'Update balance' flow from Phase 3)" —
  replace the `UpdateBalanceModal` button with a read-only computed display. The
  Phase 3 D-12 `current_balance` column stays in the schema but is now derived
  from transactions; planner decides whether the reconciliation hook writes back
  to `current_balance` or whether the UI computes from transactions. Cross-check
  against `accounts_service.rs: 118` (D-12 server-stamping rule remains the
  contract for any non-Phase-4 caller). Recommended: the Phase 4 reconciliation
  hook (`reconciliation::synthesize_reconciliation_rows`) plus first-insert
  service flow updates `current_balance` to match the running balance,
  preserving the column's contract for Phase 3 callers.

---

## Shared Patterns

> Cross-cutting patterns applied across multiple Phase 4 files. Planner threads
> these into individual plan steps.

### Authentication

**Source:** `apps/server/src/main_lib.rs:128-134` (existing `protected_api`
builder + `auth::require_auth_layer()` pattern); also
`apps/server/src/api/accounts.rs` (no per-handler auth — auth is layered at the
router level). **Apply to:** all `apps/server/src/api/transactions.rs` routes —
add `transactions::router()` to the `protected_api` block at `api.rs:128`, NOT
the open `api` builder.

### Error handling

**Source:** `crates/storage-postgres/src/errors.rs` (`StoragePgError::from`
mapping); `crates/core/src/errors/` (`Error::Validation`, `Result<T>` alias);
`apps/server/src/error.rs` (`ApiResult<T>`, `ApiError::BadRequest`). **Apply
to:** every Rust file. `?` operator for propagation; explicit mapping at the
Diesel boundary: `.map_err(StoragePgError::from)?`. Mirrors
`accounts/repository.rs:31`
`let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;`.

### Validation

**Source:** `crates/core/src/accounts/accounts_model.rs:118-193`
(`NewAccount::validate()` returning `Result<()>` with
`Error::Validation(ValidationError::InvalidInput(String))`). **Apply to:** all
`New*` and `*Update` structs in `transactions_model.rs`. Repository
`create()`/`update()` calls `validate()` before pool checkout (see
`accounts/repository.rs:30,46`).

### UUID generation

**Source:** `crates/storage-postgres/src/accounts/repository.rs:34`
`account_db.id = Uuid::now_v7().to_string();` — time-ordered. **Apply to:** all
repository `create*()` paths in
`crates/storage-postgres/src/transactions/repository.rs`. Override the
`unwrap_or_default()` empty string from `From<NewX> for XDB`.

### FX rate snapshotting

**Source:** existing `fx_service.register_currency_pair(...)` at
`accounts_service.rs:55-60` and `accounts_service.rs:128-135`. **Apply to:**
`transactions_service.rs::create_transaction` and `create_transfer` — per D-02,
snapshot the rate at the time the transaction is committed and write to
`fx_rate` + `fx_rate_source = SYSTEM` (or `MANUAL_OVERRIDE` if the user supplied
an explicit rate).

### Domain-event emission

**Source:** `accounts_service.rs:71-74,146-149,251-254`

```rust
self.event_sink.emit(DomainEvent::accounts_changed(
    vec![result.id.clone()],
    currency_changes,
));
```

**Apply to:** every mutation in `transactions_service.rs`. Add
`DomainEvent::transactions_changed(vec![id])` variant in
`crates/core/src/events/domain_event.rs` (mirror `accounts_changed` — no extra
payload needed for v1 unless reconciliation needs to broadcast a synthetic-row
insertion; recommended: include `is_system_generated: bool` flag in payload so
the downstream listener can skip portfolio-recalc on reconciliation rows).

### Server-side timestamp stamping

**Source:** `accounts_service.rs:109-124` (D-12 pattern; client cannot set
timestamp fields). **Apply to:** `transactions.created_at` / `updated_at` (set
in `From<NewTransaction> for TransactionDB`);
`payee_category_memory.last_seen_at` (set in `upsert_payee_memory`);
`transactions.updated_at` on every update path.

### Frontend invoke wrapper

**Source:** `apps/frontend/src/adapters/shared/activities.ts:100-107`

```typescript
export const createActivity = async (
  activity: ActivityCreate,
): Promise<Activity> => {
  try {
    return await invoke<Activity>("create_activity", { activity });
  } catch (err) {
    logger.error("Error creating activity.");
    throw err;
  }
};
```

**Apply to:** every export in
`apps/frontend/src/adapters/shared/transactions.ts`.

### Frontend QueryKey conventions

**Source:** `apps/frontend/src/lib/query-keys.ts` (existing
`QueryKeys.ACCOUNTS`). **Apply to:** Phase 4 hooks. New keys:

- `QueryKeys.TRANSACTIONS`
- `QueryKeys.TRANSACTION_TEMPLATES`
- `QueryKeys.MERCHANT_CATEGORIES` (for payee memory lookups)
- `QueryKeys.RUNNING_BALANCE` (for `useRunningBalance(accountId)`)
- Mutation invalidations: invalidate `[QueryKeys.TRANSACTIONS]` AND
  `[QueryKeys.ACCOUNTS]` AND `[QueryKeys.RUNNING_BALANCE]` on every
  `create_transaction`/`update_transaction`/`delete_transaction` because account
  current_balance is derived from transactions.

### Friendly-companion brand voice (CONTEXT `<specifics>`)

**Source:**
`apps/frontend/src/pages/activity/import/components/cancel-confirmation-dialog.tsx`
(existing copy tone). **Apply to:** all new copy strings:

- Duplicate review: `"Discard new"` / `"Keep both"` (NOT "Reject" / "Accept")
- Template mismatch:
  `"Your saved 'Chase Checking CSV' template doesn't match this file's columns. Re-map?"`
- Transfer-pair edit: AlertDialog body "Apply to both legs (preserves transfer
  pairing) or only this leg (breaks the link)?"

## No Analog Found

Files with **no close codebase analog** — planner uses RESEARCH.md patterns and
external documentation:

| File                                                                               | Role      | Data flow                     | Reason                                                                                                                                                                | RESEARCH section to use                       |
| ---------------------------------------------------------------------------------- | --------- | ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `crates/core/src/transactions/ofx_parser.rs`                                       | parser    | file-I/O                      | No OFX parsing exists in workspace; uses `sgmlish` 0.2                                                                                                                | RESEARCH §OFX Parsing Strategy lines 752-810  |
| `crates/core/src/transactions/ofx_parser_tests.rs`                                 | test      | parse fixtures                | Fresh; planner generates anonymized OFX 1.x and 2.x fixtures                                                                                                          | RESEARCH §OFX Edge Cases lines 801-810        |
| `crates/core/src/transactions/duplicate_detector.rs`                               | detector  | event-driven                  | New algorithm; uses `strsim::normalized_levenshtein` + 3-key gate                                                                                                     | RESEARCH §Duplicate Detection lines 853-948   |
| `crates/core/src/transactions/duplicate_detector_tests.rs`                         | test      | scenario table                | Fresh                                                                                                                                                                 | RESEARCH §Duplicate Detection lines 920-948   |
| `crates/core/src/transactions/merchant_normalizer.rs`                              | utility   | string transform              | Pure D-13 algorithm                                                                                                                                                   | CONTEXT D-13                                  |
| `crates/core/src/transactions/merchant_normalizer_tests.rs`                        | test      | scenario table                | Fresh                                                                                                                                                                 | CONTEXT D-13 examples + RESEARCH expansion    |
| `crates/core/src/transactions/reconciliation.rs`                                   | utility   | once-per-account hook         | Phase 3 D-14 contract; novel                                                                                                                                          | RESEARCH §Reconciliation Hook lines 1147-1257 |
| `apps/frontend/src/pages/transactions/transaction-list.tsx`                        | component | virtualized date-grouped list | Closest analog (`activity-table-mobile.tsx`) is a flat table; transaction-list adds per-day subtotal headers + running balance — UI-SPEC §1 visual, no existing match | UI-SPEC §1 list visual                        |
| `apps/frontend/src/pages/transactions/split-editor.tsx`                            | component | repeater                      | No existing split editor in codebase                                                                                                                                  | UI-SPEC §7 split editor visual                |
| `apps/frontend/src/pages/transactions/duplicate-banner.tsx`                        | component | display + action              | Closest is `import-validation-alert.tsx` (different shape); UI-SPEC §6 dedicated component                                                                            | UI-SPEC §6 banner visual                      |
| `apps/frontend/src/pages/transactions/import/components/duplicate-review-list.tsx` | component | display                       | Companion to review-step; no analog                                                                                                                                   | UI-SPEC §6                                    |

## Metadata

**Analog search scope:**

- `crates/core/src/{accounts,activities,fx,taxonomies,events}/`
- `crates/storage-postgres/src/{accounts,activities,taxonomies}/`,
  `migrations/`, `schema.rs`
- `apps/server/src/api/{accounts,activities,taxonomies}.rs`, `api.rs`,
  `main_lib.rs`, `models.rs`
- `apps/frontend/src/adapters/{shared,web}/`
- `apps/frontend/src/pages/activity/import/` (full subtree)
- `apps/frontend/src/pages/{account,settings/accounts}/`
- `apps/frontend/src/{hooks,lib/types,components,routes.tsx,App.tsx}`
- `apps/frontend/src/pages/layouts/navigation/`

**Files scanned:** ~38 files read in full or targeted; ~50 directory listings.

**Pattern extraction date:** 2026-04-30
