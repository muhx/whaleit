# Phase 3: Bank Accounts & Credit Cards — Research

**Researched:** 2026-04-25 **Domain:** account domain extension (Rust core +
storage-postgres + apps frontend) **Confidence:** HIGH

## Executive Summary

- **Decimal storage is TEXT, not NUMERIC.** CONTEXT.md D-10 says "NUMERIC" but
  every existing money column in
  `crates/storage-postgres/migrations/20260101000000_initial_schema/up.sql` is
  declared `TEXT` and deserialized via `Decimal::from_str`. Planner must
  reconcile: either follow established pattern (TEXT + `String` in Diesel model)
  or explicitly diverge with `NUMERIC(20,8)` + `rust_decimal::Decimal` diesel
  type. **Recommendation: follow existing pattern (TEXT)** — diverging requires
  also adding `diesel-numeric` feature wiring and breaks the single-dialect rule
  the project already uses across 30+ money columns. This is the single biggest
  decision for the planner.
- **No Rust exhaustive-match landmine.** `AccountType` is `String` in the domain
  and `&'static str` constants in `accounts_constants.rs`. There is NO Rust
  enum, no `match AccountType::*` to break when adding
  CHECKING/SAVINGS/CREDIT_CARD/LOAN.
- **One categorization landmine.**
  `crates/core/src/portfolio/net_worth/net_worth_service.rs:63-69` has
  `categorize_by_account_type` with fallback `_ => AssetCategory::Investment`.
  New liability types will be mis-categorized as Investment unless this function
  is updated to use the new `account_kind()` helper (D-03). Net-worth totals are
  Phase 6 scope, but the helper must land in Phase 3 so Phase 6 can adopt it
  cleanly.
- **Web-only runtime.** Desktop IPC commands for accounts don't exist in this
  codebase — `apps/tauri/src/lib.rs` registers NO `#[tauri::command]` handlers.
  Both desktop and web use the same React adapter path that invokes the Axum
  HTTP server at `/api/v1/accounts`. This means there is exactly **one** backend
  surface to extend (Axum), and all `COMMANDS` mapping in
  `apps/frontend/src/adapters/web/core.ts` flows to it.
- **Migrations run embedded on server startup.**
  `crates/storage-postgres/src/db/mod.rs:15` uses `embed_migrations!()`. No
  separate push command required — `run_migrations()` runs at
  `apps/server/src/main_lib.rs` boot. Planner DOES still need a Diesel CLI step
  to regenerate `schema.rs` after writing the new migration.
- **UI already has all needed primitives.** `@whaleit/ui` exports every
  component the UI-SPEC asks for. Icons `Landmark` and `MoreHorizontal` are the
  only gaps — UI-SPEC accepts `Building` / `Building2` fallback; `Ellipsis`
  (already present) can substitute for `MoreHorizontal`.

**Primary recommendation:** Treat this phase as an **additive extension** of an
existing, stable domain. Every new piece of data goes into an already-wired pipe
(model → DB → repository → service → HTTP handler → shared adapter → hook →
page). The phase's complexity is not in wiring — it's in (a) the 11-column
migration with correct constraints, (b) the service-layer validation rules for
CC-field nullability per `account_type`, and (c) building the new `/accounts`
route and dynamic form UI.

## Phase Requirements

| ID      | Description                                                                                         | Research Support                                                                                                        |
| ------- | --------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| ACCT-01 | User can create bank accounts (CHECKING, SAVINGS) with name, institution, currency, opening balance | Extend `NewAccount` + `accounts` table columns; new AccountType constants; form section in unified create flow          |
| ACCT-02 | User can create credit card accounts with limit and statement cycle day                             | CC-specific nullable columns on `accounts`; service-layer validation enforces required-when-CC                          |
| ACCT-03 | Unified list of all account types with current balances                                             | New `/accounts` route; reuse `accounts-summary.tsx` row shape; `current_balance` column                                 |
| ACCT-04 | Edit and archive while preserving history                                                           | `is_archived` already in schema; `update_account` already works end-to-end; need archive-hidden-by-default on selectors |
| ACCT-05 | CC shows outstanding, available credit, utilization%, next payment due                              | Derived helpers in frontend (D-08); new CC detail sections                                                              |
| ACCT-06 | Statement balance, minimum payment, due date                                                        | Three nullable columns on `accounts` (D-07, current-snapshot-only)                                                      |
| ACCT-07 | Reward points / cashback balance                                                                    | Two nullable columns (D-09)                                                                                             |

## User Constraints (from CONTEXT.md)

### Locked Decisions (D-01..D-19)

- **D-01:** Add `CHECKING`, `SAVINGS`, `CREDIT_CARD`, `LOAN` to `AccountType`.
  `SECURITIES`, `CASH`, `CRYPTOCURRENCY` stay.
- **D-02:** `LOAN` is enum-slot-only — basic CRUD only, no amortization.
- **D-03:** Derive asset/liability/investment via `AccountKind` enum +
  `account_kind(t: &str) -> AccountKind` helper in `crates/core`. Mirror helper
  in `apps/frontend/src/lib/constants.ts`.
- **D-04:** Default `tracking_mode` = `TRANSACTIONS` for new CHECKING / SAVINGS
  / CREDIT_CARD / LOAN.
- **D-05:** Existing `CASH` accounts untouched — no migration.
- **D-06:** CC-specific fields live as nullable columns on `accounts`. No side
  table, no JSON blob.
- **D-07:** Statement snapshot is current-only (no history table this phase).
- **D-08:** Utilization derived on the fly
  (`current_balance / credit_limit * 100`).
- **D-09:** Rewards are manual balance fields only.
- **D-10:** All money columns use `NUMERIC` (see Executive Summary — conflicts
  with established TEXT pattern; planner must decide explicitly).
- **D-11:** `opening_balance` is a required field on `NewAccount` for
  bank/CC/LOAN.
- **D-12:** `current_balance` + `balance_updated_at` columns. Manual "Update
  balance" action bumps both.
- **D-13:** CC balances stored as POSITIVE values. `account_kind` sign-flips for
  net worth.
- **D-14:** Phase 4 will auto-generate an "Opening Balance" transaction +
  optional "Balance adjustment" reconciliation transaction. Phase 3 records only
  `opening_balance`, `current_balance`, `balance_updated_at`.
- **D-15:** Keep `accounts-summary.tsx`. Add NEW `/accounts` route. Per-account
  page stays.
- **D-16:** Default groups: CHECKING / SAVINGS → `"Banking"`, `CREDIT_CARD` →
  `"Credit Cards"`, `LOAN` → `"Loans"`. Unchanged: SECURITIES / CASH /
  CRYPTOCURRENCY.
- **D-17:** Row shape per D-17: name, institution, balance (native),
  base-currency equivalent, CC-only "Available credit" chip.
- **D-18:** `institution` is a free-text `VARCHAR` column, distinct from
  existing `platform_id`.
- **D-19:** Archived hidden by default. "Show archived" toggle on `/accounts`.
  No un-archive confirm.

### Claude's Discretion

- Exact Diesel column types in migration (precision/scale + CHECK wording).
- Validation rules: `credit_limit > 0`, `statement_cycle_day BETWEEN 1 AND 31`,
  `opening_balance >= 0` for bank/LOAN, any NUMERIC for CC, `statement_due_date`
  vs `statement_cycle_day` sanity.
- Service-layer enforcement of "CC fields must all be null for non-CC types and
  must be present for CC type on create".
- Empty-state copy, error messages, button labels (UI-SPEC provides most).
- UI polish: "Update balance" surfacing (inline vs modal — UI-SPEC says modal),
  archive toggle placement.
- Whether `/accounts` replaces or augments the existing `/settings/accounts`
  page.
- Migration file numbering and `schema.rs` regeneration strategy (mirror Phase
  2).

### Deferred Ideas (OUT OF SCOPE)

- Full loan tracking (principal, APR, amortization) — future phase.
- Statement history table — future phase.
- Rewards rules engine — future phase.
- Institution lookup table — future phase.
- Separate `/accounts/archived` route — deferred.
- Un-archive confirmation — deferred.
- Balance reconciliation tool — Phase 4+.
- Bank sync via API (Plaid) — out of scope per PROJECT.md.

## Project Constraints (from CLAUDE.md)

- **Simplicity First:** minimum code, no abstractions for single-use, no
  speculative configurability. [CITED: .claude/CLAUDE.md §2]
- **Surgical Changes:** every changed line traces to the user's request; don't
  "improve" adjacent code. [CITED: .claude/CLAUDE.md §3]
- **Goal-Driven Execution:** each task needs a verifiable success criterion.
  [CITED: .claude/CLAUDE.md §4]
- **Tech stack is established:** Rust crates, React/TS frontend, Diesel ORM,
  Tauri v2 + Axum. [CITED: .planning/PROJECT.md §Constraints]
- **PostgreSQL-only storage** — SQLite was removed in `a5f0515e`. No SQLite
  migrations, no parity tests. [CITED: memory/project_storage_pivot_pg_only.md]

## Architectural Responsibility Map

| Capability                                                                                  | Primary Tier                                                 | Secondary Tier                                                  | Rationale                                                                                                                                                                                                                |
| ------------------------------------------------------------------------------------------- | ------------------------------------------------------------ | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Account type enum extension                                                                 | Rust core (`crates/core/src/accounts/accounts_constants.rs`) | Frontend constants (`apps/frontend/src/lib/constants.ts`)       | `&str` constants in Rust, zod + const object in TS — both define the canonical set                                                                                                                                       |
| `AccountKind` helper (asset/liability/investment)                                           | Rust core (`crates/core/src/accounts/`)                      | Frontend constants                                              | Pure function keyed by account_type string; mirrored in TS for UI grouping without round-trip                                                                                                                            |
| New columns: opening_balance, current_balance, balance_updated_at, institution, CC-specific | Database / Storage                                           | Diesel model + core `Account` domain model                      | 11 additive nullable columns in one migration; `AccountDB` + domain `Account` extend; `From<AccountDB> for Account` gains new field copies                                                                               |
| Service-layer validation (CC-required-when-type, null-when-not)                             | Rust core (`crates/core/src/accounts/accounts_service.rs`)   | —                                                               | Business rule lives at service layer; repo is dumb persistence. Existing pattern: `new_account.validate()` → extend with type-gated rules                                                                                |
| Repository CRUD                                                                             | storage-postgres                                             | —                                                               | Existing `create` / `update` / `list` signatures unchanged — new fields flow through struct literals                                                                                                                     |
| HTTP endpoints                                                                              | Axum (`apps/server/src/api/accounts.rs`)                     | `apps/server/src/models.rs` DTOs                                | Existing `GET/POST/PUT/DELETE /accounts[/{id}]` carry the new fields automatically once DTOs are extended                                                                                                                |
| `update_account_balance` command (if separated per D-12)                                    | Axum (new handler) + core service + repo                     | —                                                               | Recommendation: **do NOT add a separate command.** Reuse `update_account` with a dedicated frontend modal that sets `current_balance` + lets backend auto-stamp `balance_updated_at`. See "Balance Update Flow" section. |
| `/accounts` unified list route                                                              | Frontend page                                                | shared hook `useAccounts`, shared `adapters/shared/accounts.ts` | Reuses existing commands; adds a new page component only                                                                                                                                                                 |
| New-account form (dynamic by type)                                                          | Frontend page (`/accounts` or route)                         | react-hook-form + zod (`newAccountSchema`)                      | Extends existing form pattern; conditional fields based on `accountType` watch                                                                                                                                           |
| Archive default-hidden filter                                                               | Frontend hook (`use-accounts.ts`)                            | backend `list(is_archived_filter)`                              | Hook already supports `includeArchived` opt-in. Audit selectors to ensure `includeArchived: false` is the default — it already is.                                                                                       |
| CC-specific detail sections                                                                 | Frontend page (`account-page.tsx`)                           | helper: `account_kind()`                                        | Conditional render when `account_kind(accountType) === Liability` or `accountType === "CREDIT_CARD"`                                                                                                                     |

## Current Account Domain Audit

### Rust — all files touching `account_type` / `AccountType`

| File                                                       | Line          | What it does                                                                                                         | Needs update?                                                                                                                                                                     |
| ---------------------------------------------------------- | ------------- | -------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/core/src/accounts/accounts_model.rs`               | 24-47         | `Account` struct — `account_type: String`                                                                            | YES — add 11 new fields (D-06, D-11, D-12, D-18)                                                                                                                                  |
| `crates/core/src/accounts/accounts_model.rs`               | 52-70         | `NewAccount` struct                                                                                                  | YES — add same fields + `institution` + CC fields + `opening_balance` required                                                                                                    |
| `crates/core/src/accounts/accounts_model.rs`               | 72-87         | `NewAccount::validate()`                                                                                             | YES — extend to enforce CC-field rules by account_type (D-06)                                                                                                                     |
| `crates/core/src/accounts/accounts_model.rs`               | 90-123        | `AccountUpdate` struct + validate                                                                                    | YES — add optional fields for updates + CC fields                                                                                                                                 |
| `crates/core/src/accounts/accounts_constants.rs`           | 2-9           | `DEFAULT_ACCOUNT_TYPE` + `account_types` module with 3 `&str` constants                                              | YES — add `CHECKING`, `SAVINGS`, `CREDIT_CARD`, `LOAN`                                                                                                                            |
| `crates/core/src/accounts/accounts_constants.rs`           | 18-25         | `default_group_for_account_type(&str) -> &'static str` with 3 arms + `_ => "Investments"`                            | YES — add 4 new arms (D-16)                                                                                                                                                       |
| `crates/core/src/accounts/accounts_service.rs`             | 45-126        | `AccountServiceTrait` impl — `create_account` and `update_account`                                                   | YES — add type-gated validation before repo call                                                                                                                                  |
| `crates/core/src/accounts/accounts_traits.rs`              | 17-46         | `AccountRepositoryTrait` — `create` / `update` / `get_by_id` / `list` signatures                                     | NO — signatures unchanged                                                                                                                                                         |
| `crates/core/src/accounts/accounts_traits.rs`              | 53-91         | `AccountServiceTrait`                                                                                                | NO — signatures unchanged                                                                                                                                                         |
| `crates/core/src/accounts/accounts_model_tests.rs`         | 74-95         | `create_test_account` helper                                                                                         | YES — add new fields to struct literal (affects all downstream tests)                                                                                                             |
| `crates/core/src/portfolio/net_worth/net_worth_service.rs` | 63-69         | `categorize_by_account_type(&str) -> AssetCategory` with `_ => Investment` fallback                                  | **LANDMINE** — DON'T fix in Phase 3 (Phase 6 scope), but the `account_kind()` helper MUST land in this phase so Phase 6 can adopt. Keep the current fallback; file it in Phase 6. |
| `crates/core/src/health/service.rs`                        | 377, 415      | `a.account_type == CASH` / `!= CASH` filters                                                                         | NO — pattern-matches only on CASH, stays correct                                                                                                                                  |
| `crates/ai/src/tools/accounts.rs`                          | 32, 112       | AI tool echoes `account_type: String`                                                                                | NO                                                                                                                                                                                |
| `crates/connect/src/broker/service.rs`                     | 229, 262, 244 | Broker sync sets `account_type` from string returned by provider                                                     | NO — providers won't return new types                                                                                                                                             |
| `crates/connect/src/broker/models.rs`                      | 68            | `BrokerAccount.account_type: Option<String>`                                                                         | NO                                                                                                                                                                                |
| `crates/storage-postgres/src/accounts/model.rs`            | 12-122        | `AccountDB` + `From<AccountDB> for Account` + `From<NewAccount> for AccountDB` + `From<AccountUpdate> for AccountDB` | YES — add 11 fields to all four sites                                                                                                                                             |
| `crates/storage-postgres/src/accounts/repository.rs`       | 28-137        | CRUD via Diesel                                                                                                      | NO — struct literals updated via model changes                                                                                                                                    |
| `crates/storage-postgres/src/schema.rs`                    | 13-31         | `diesel::table! { accounts { ... } }`                                                                                | YES — regenerate after migration                                                                                                                                                  |
| `apps/server/src/models.rs`                                | 8-147         | HTTP DTOs `Account` / `NewAccount` / `AccountUpdate` + 3 `From` impls                                                | YES — add 11 fields to all four sites                                                                                                                                             |
| `apps/server/src/api/accounts.rs`                          | 22-74         | Axum handlers                                                                                                        | NO — DTOs flow through unchanged                                                                                                                                                  |

### TypeScript — all files touching `accountType` / `AccountType`

| File                                                                          | Line                   | What it does                                                                                                          | Needs update?                                                                                                                    |
| ----------------------------------------------------------------------------- | ---------------------- | --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `apps/frontend/src/lib/constants.ts`                                          | 44-56                  | `AccountType` const object + `accountTypeSchema` zod enum                                                             | YES — add 4 variants                                                                                                             |
| `apps/frontend/src/lib/constants.ts`                                          | 61-72                  | `defaultGroupForAccountType` with `default: "Investments"`                                                            | YES — add 4 arms (D-16)                                                                                                          |
| `apps/frontend/src/lib/constants.ts`                                          | 492-511                | `HOLDING_GROUP_DISPLAY_NAMES` / `HOLDING_GROUP_ORDER`                                                                 | NO — these are Holdings-page-only, new types don't hold securities                                                               |
| `apps/frontend/src/lib/types/account.ts`                                      | 7-25                   | `Account` TS interface                                                                                                | YES — add 11 fields                                                                                                              |
| `apps/frontend/src/lib/schemas.ts`                                            | 78-96                  | `newAccountSchema` zod                                                                                                | YES — add 11 fields + conditional validation via `.refine()` or `.superRefine()` for CC-gated rules                              |
| `apps/frontend/src/hooks/use-accounts.ts`                                     | 7-33                   | Hook wrapping `getAccounts(includeArchived)`                                                                          | NO — `includeArchived: false` is already the default                                                                             |
| `apps/frontend/src/components/account-selector.tsx`                           | 26-31, 127-130         | `accountTypeIcons` map + `includeArchived: false` default                                                             | YES — extend icon map with 4 new types; archive default already correct                                                          |
| `apps/frontend/src/components/account-selector-mobile.tsx`                    | 24-29, 110-123         | `accountTypeIcons` + `getAccountTypeLabel` switch                                                                     | YES — extend icon map + add labels for new types                                                                                 |
| `apps/frontend/src/components/app-launcher.tsx`                               | 65-70                  | `accountTypeIcons: Record<AccountType, Icon>` — **exhaustive type**                                                   | YES — EXHAUSTIVE RECORD KEY will break TypeScript compilation when new enum variants land unless extended.                       |
| `apps/frontend/src/pages/account/account-page.tsx`                            | 82-86                  | `accountTypeIcons: Record<AccountType, Icon>` — **exhaustive type**                                                   | YES — same TS-exhaustive concern                                                                                                 |
| `apps/frontend/src/pages/settings/accounts/components/account-form.tsx`       | 47-51                  | `accountTypes: ResponsiveSelectOption[]` array                                                                        | YES — add 4 new options (if Settings page keeps showing the full type picker)                                                    |
| `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx` | 20                     | hard-coded `"SECURITIES" \| "CASH" \| "CRYPTOCURRENCY"` cast                                                          | YES — extend union                                                                                                               |
| `apps/frontend/src/pages/fire-planner/pages/settings-page.tsx`                | 624, 643-644           | String-literal filter `accountType === "SECURITIES" \|\| accountType === "CRYPTOCURRENCY"`                            | NO — fire planner is investment-oriented; new types correctly excluded                                                           |
| `apps/frontend/src/pages/dashboard/accounts-summary.tsx`                      | 330-332, 424-425       | Groups by `acc.group ?? "Uncategorized"` — NO accountType pattern match                                               | NO — confirm by code reading: only passes `accountType` through as data, never switches on it. New types flow through correctly. |
| `apps/frontend/src/adapters/shared/accounts.ts`                               | 8, 22-32, 34-42, 44-59 | `NewAccount = z.infer<typeof newAccountSchema>` + `createAccount` / `updateAccount` / `getAccounts` / `deleteAccount` | NO — schema change cascades automatically                                                                                        |
| `apps/frontend/src/adapters/web/modules/accounts.ts`                          | 3-41                   | 4 HTTP request builders                                                                                               | NO                                                                                                                               |
| `apps/frontend/src/adapters/web/core.ts`                                      | 39-42, 435-442         | `COMMANDS` map + `handleCommand` switch                                                                               | NO for the 4 existing commands; YES IF planner introduces `update_account_balance` (we recommend AGAINST)                        |
| `apps/frontend/src/routes.tsx`                                                | 155, 179               | `accounts/:id` + `settings/accounts`                                                                                  | YES — add new `/accounts` route                                                                                                  |

## Migration Pattern

### Naming Convention

Existing migrations follow **timestamp-prefixed directories containing
`up.sql` + `down.sql`**:

```
crates/storage-postgres/migrations/
├── 20260101000000_initial_schema/
│   ├── up.sql
│   └── down.sql
└── 20260422000000_auth_users/
    ├── up.sql
    └── down.sql
```

Phase 3 migration should be named e.g.
`20260425000000_accounts_extend_types_and_balances/` (today's date
`2026-04-25` + sequential `000000`). The `diesel migration generate` CLI
produces this shape automatically.

### Existing Money-Column Declaration Pattern

**Critical finding:** every monetary column in the initial schema is declared as
plain `TEXT`, with a header comment explaining why:

```sql
-- crates/storage-postgres/migrations/20260101000000_initial_schema/up.sql:7
--   - Decimals: TEXT (serialized Rust Decimal)
```

Examples:

```sql
-- activities table (line 90+)
quantity TEXT,
unit_price TEXT,
amount TEXT,
fee TEXT,
fx_rate TEXT,

-- holdings_snapshots (line 124+)
cost_basis TEXT NOT NULL DEFAULT '0',
net_contribution TEXT NOT NULL DEFAULT '0',

-- daily_account_valuation (line 141+)
fx_rate_to_base TEXT NOT NULL DEFAULT '1',
cash_balance TEXT NOT NULL DEFAULT '0',
investment_market_value TEXT NOT NULL DEFAULT '0',

-- quotes (line 248+)
open TEXT,
high TEXT,
low TEXT,
close TEXT NOT NULL,
```

The Rust deserialization pattern is **uniform**:

```rust
// crates/storage-postgres/src/fx/model.rs:49-71
use rust_decimal::Decimal;
use std::str::FromStr;

close: Decimal::from_str(&db.close).unwrap_or(Decimal::ZERO),
open: db.open.as_deref().and_then(|s| Decimal::from_str(s).ok()).unwrap_or(Decimal::ZERO),
```

Diesel models declare money columns as `String` / `Option<String>`, and
conversion to/from `Decimal` happens at the `From<DB> for DomainModel` boundary.

### D-10 Reconciliation

CONTEXT.md D-10 says "All money columns use `NUMERIC`". The codebase says TEXT.
These are incompatible. Options:

| Option                                            | Pros                                                                                | Cons                                                                                                                                                                                                                                            |
| ------------------------------------------------- | ----------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A: Follow existing TEXT pattern (recommended)** | Zero-risk — mirrors 30+ existing columns; no new Diesel infra; migration is trivial | Technically diverges from D-10 literal text; loses DB-side numeric range checks — planner must add CHECK via regex or convert+cast in triggers (painful)                                                                                        |
| B: Use `NUMERIC(20,8)` as D-10 states             | DB-side precision + range checks trivially expressible                              | Requires feature flip on `rust_decimal` (`db-diesel2-postgres`), adds `diesel::sql_types::Numeric` type to schema, breaks the "all money is TEXT" consistency the codebase currently has — risk of deserialization subtleties (scale/precision) |
| C: Hybrid — NUMERIC only for the new columns      | Keeps existing TEXT columns stable, adds NUMERIC only where explicitly needed       | Creates two conventions; future contributors won't know which to follow                                                                                                                                                                         |

**Recommendation:** **Option A.** Treat D-10 as "money is persisted with
arbitrary precision" — the existing TEXT-serialized-Decimal approach satisfies
that spirit without introducing a new dialect. Range validation (e.g.
`credit_limit > 0`) moves to the service layer (already how
`NewAccount::validate()` works). Planner should flag this explicitly in the
discuss-phase summary to get user confirmation. [ASSUMED: recommendation — not a
locked decision]

### CHECK Constraints

Initial schema uses CHECK sparingly. Examples found: NONE in
`20260101000000_initial_schema/up.sql`. Constraint-like semantics are usually
enforced via UNIQUE + application validation.

For Phase 3, suggested inline constraints (only for values guaranteed at DB
level regardless of type):

```sql
statement_cycle_day SMALLINT CHECK (statement_cycle_day BETWEEN 1 AND 31),
reward_points_balance INTEGER CHECK (reward_points_balance >= 0),
```

For NUMERIC-as-TEXT, range checks are NOT practical at DB level — service layer
owns them.

### Enum-as-String Storage Pattern

`account_type` is already stored as `TEXT NOT NULL` (schema line 35). No PG
`ENUM` type, no CHECK list. Adding new values is zero DDL cost — just accept
them in service validation. **If the planner wants DB-level enforcement**, add:

```sql
ALTER TABLE accounts
  ADD CONSTRAINT accounts_type_whitelist
  CHECK (account_type IN ('SECURITIES','CASH','CRYPTOCURRENCY','CHECKING','SAVINGS','CREDIT_CARD','LOAN'));
```

This is a **nice-to-have**, not required. Existing `CASH`/`SECURITIES` has no
such constraint and the app has been fine.

### schema.rs Regeneration

`crates/storage-postgres/diesel.toml` is minimal:

```toml
[migrations_directory]
dir = "migrations"
```

There is no `[print_schema]` block — `schema.rs` is NOT auto-regenerated by
diesel-cli on migration run. It appears hand-maintained. The existing
`schema.rs` matches the initial migration 1:1. **Planner must include an
explicit task to regenerate `schema.rs` after the migration** (either via
`diesel print-schema > src/schema.rs` with DATABASE_URL set, or hand-edit to
mirror the new migration). Embedded `MIGRATIONS` (`db/mod.rs:15`) applies
migrations at runtime — this works regardless of `schema.rs` state — but
Diesel's query DSL compile-time checking REQUIRES `schema.rs` to match.

## Repository/Service Wiring — The New-Field Path

For every new column, the full touch-list is:

1. **Migration** —
   `crates/storage-postgres/migrations/YYYYMMDDHHMMSS_accounts_extend/up.sql` —
   `ALTER TABLE accounts ADD COLUMN ...`
2. **schema.rs regeneration** — `crates/storage-postgres/src/schema.rs:13-31` —
   new fields in the `accounts (id) { ... }` block
3. **Diesel model** — `crates/storage-postgres/src/accounts/model.rs:12-32` —
   `AccountDB` struct gains the fields (`Option<String>` for NUMERIC/DATE
   columns, `Option<i16>` for SMALLINT, etc.)
4. **`From<AccountDB> for Account`** —
   `crates/storage-postgres/src/accounts/model.rs:34-60` — copy new fields into
   domain model (with `Decimal::from_str` conversion if applicable)
5. **`From<NewAccount> for AccountDB`** —
   `crates/storage-postgres/src/accounts/model.rs:62-90` — copy new fields from
   domain into DB model
6. **`From<AccountUpdate> for AccountDB`** —
   `crates/storage-postgres/src/accounts/model.rs:92-122` — same for updates.
   NOTE: the current update path (`repository.rs:45-84`) preserves some existing
   fields by re-reading them from DB; planner must decide per-field whether
   CC-specific updates should be allowed via the standard `update_account` path
   or via a separate operation.
7. **Core domain model** — `crates/core/src/accounts/accounts_model.rs` —
   `Account`, `NewAccount`, `AccountUpdate` all gain the fields. Fields are
   plain `Option<Decimal>` / `Option<String>` / `Option<i16>` /
   `Option<NaiveDate>` / `Option<i32>` — no DB-type leakage.
8. **Core `NewAccount::validate()`** — extend to enforce "CC fields all present
   when `account_type == CREDIT_CARD`, all null otherwise".
9. **Core `AccountUpdate::validate()`** — extend similarly (but consider:
   updates often don't supply the full record).
10. **Service layer (`accounts_service.rs`)** — no signature change. The
    existing service already calls `repository.create`/`update` with the full
    `NewAccount` / `AccountUpdate` — new fields flow through as struct data.
11. **Axum DTO (`apps/server/src/models.rs`)** — `Account`, `NewAccount`,
    `AccountUpdate` + 3 `From` impls — gain the fields.
12. **HTTP handlers (`apps/server/src/api/accounts.rs`)** — no signature change.
    DTOs flow through.
13. **TS shared adapter (`apps/frontend/src/adapters/shared/accounts.ts`)** — no
    signature change. The type flows from `newAccountSchema` inference.
14. **TS zod schema (`apps/frontend/src/lib/schemas.ts`)** — `newAccountSchema`
    gains the fields. Use `.superRefine()` for the "CC-fields-when-CC" rule.
15. **TS `Account` interface (`apps/frontend/src/lib/types/account.ts`)** — gain
    the fields.

**Key insight:** There are NO trait-signature changes needed. The repository
`create(new_account: NewAccount)` / `update(account_update: AccountUpdate)` /
`get_by_id` / `list` signatures already carry the full record. Adding fields is
a pure data change propagating through three serialization boundaries (SQL ↔
Rust struct, Rust struct ↔ JSON DTO, JSON ↔ TS).

### Validation Placement (Canonical Example)

Current validation in `accounts_model.rs:72-87`:

```rust
impl NewAccount {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account name cannot be empty".to_string(),
            )));
        }
        if self.currency.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Currency cannot be empty".to_string(),
            )));
        }
        Ok(())
    }
}
```

**Extend this exact function** with CC-field rules. No need to add service-layer
validation on top — the `repository.create()` calls `new_account.validate()?` at
`repository.rs:30` already.

## Adapter Layer for Tauri + Web

### Desktop (Tauri): NO IPC COMMANDS

**Surprising finding:** there are zero `#[tauri::command]` attributes,
`invoke_handler`, or `generate_handler` calls anywhere in this repository.
`apps/tauri/src/lib.rs` is a thin Tauri shell (menu, deep links, plugins) — no
command registry.

This means both **desktop** and **web** builds use the `web/core.ts` HTTP
invoker talking to Axum. The `adapters/tauri/index.ts` module merely re-exports
`createAccount` / `updateAccount` / `getAccounts` / `deleteAccount` from
`../shared/accounts` — and the shared module's `invoke()` points to the same
HTTP invoker via build-time alias resolution (`adapters/index.ts`).

Therefore:

- **NEW backend work required:** Axum handlers only.
- **Tauri-specific work required:** None. (If the project later adds IPC
  commands for bundled desktop mode, that's a separate migration.)

### Backend Surface

| File                              | Current                                                                                | New for Phase 3                              |
| --------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------- |
| `apps/server/src/models.rs`       | `Account`, `NewAccount`, `AccountUpdate` DTOs (lines 8-147)                            | Add 11 fields to each; extend 3 `From` impls |
| `apps/server/src/api/accounts.rs` | `list_accounts` / `create_account` / `update_account` / `delete_account` (lines 22-74) | No change in signatures                      |
| `apps/server/src/api.rs`          | OpenAPI registration (line 65)                                                         | No change                                    |

If the planner decides to introduce a dedicated `update_account_balance` command
(we recommend against — see "Balance Update Flow"), it would require:

- New handler in `api/accounts.rs` (e.g. `PATCH /accounts/{id}/balance`)
- New service method in `accounts_traits.rs` + `accounts_service.rs`
- New repository method in `accounts_traits.rs` +
  `storage-postgres/src/accounts/repository.rs`
- New `COMMANDS` entry in `web/core.ts`
- New handler function in `web/modules/accounts.ts`
- New `switch` arm in `web/core.ts` `handleCommand`
- New `updateAccountBalance` export in `adapters/shared/accounts.ts`

That's a 7-file touch just for one command. **A `update_account` call with a
modal-constrained payload reuses the existing pipe and adds ZERO new
endpoints.** See Balance Update Flow.

### Frontend Command Registration Pattern

To add any new command (if the planner chooses to):

```typescript
// 1. apps/frontend/src/adapters/web/core.ts: COMMANDS map
update_account_balance: { method: "PATCH", path: "/accounts/balance" },

// 2. apps/frontend/src/adapters/web/modules/accounts.ts: request builder
export function handleUpdateAccountBalance(url: string, payload: ...) {
  const data = payload as { accountId: string, body: {...} };
  return { url: `${url}/${encodeURIComponent(data.accountId)}/balance`, body: JSON.stringify(data.body) };
}

// 3. apps/frontend/src/adapters/web/core.ts: handleCommand switch
case "update_account_balance":
  return accountHandlers.handleUpdateAccountBalance(url, p!);

// 4. apps/frontend/src/adapters/shared/accounts.ts: typed wrapper
export const updateAccountBalance = async (...) => invoke<Account>("update_account_balance", {...});
```

## Frontend Integration Points

### Must extend (core account type set)

| File                                     | Line range | Change                                                                                                        |
| ---------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------- |
| `apps/frontend/src/lib/constants.ts`     | 44-56      | Extend `AccountType` const and `accountTypeSchema` zod enum with `CHECKING`, `SAVINGS`, `CREDIT_CARD`, `LOAN` |
| `apps/frontend/src/lib/constants.ts`     | 61-72      | Extend `defaultGroupForAccountType` with new arms per D-16                                                    |
| `apps/frontend/src/lib/constants.ts`     | NEW        | Add `AccountKind` enum and `accountKind(t: AccountType)` helper mirroring Rust                                |
| `apps/frontend/src/lib/types/account.ts` | 7-25       | Add 11 new optional fields to `Account` interface                                                             |
| `apps/frontend/src/lib/schemas.ts`       | 78-96      | Extend `newAccountSchema` with new fields + `.superRefine()` for CC-gated rules                               |

### Must extend (selectors / icon maps with exhaustive keys)

| File                                                                          | Line range | Change                                                                                                                    |
| ----------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------- |
| `apps/frontend/src/components/account-selector.tsx`                           | 26-31      | Extend `accountTypeIcons` map with 4 new types (record has string keys — loose, only runtime lookup fail to `CreditCard`) |
| `apps/frontend/src/components/account-selector-mobile.tsx`                    | 24-29      | Same                                                                                                                      |
| `apps/frontend/src/components/account-selector-mobile.tsx`                    | 110-123    | Extend `getAccountTypeLabel` switch with new type labels                                                                  |
| `apps/frontend/src/components/app-launcher.tsx`                               | 65-70      | **`Record<AccountType, Icon>` — EXHAUSTIVE key type will fail TS compile** — must add entries for all 4 new variants      |
| `apps/frontend/src/pages/account/account-page.tsx`                            | 82-86      | Same exhaustive `Record<AccountType, Icon>` concern                                                                       |
| `apps/frontend/src/pages/settings/accounts/components/account-form.tsx`       | 47-51      | `accountTypes: ResponsiveSelectOption[]` — add 4 options (or only if settings page is kept)                               |
| `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx` | 20         | Replace hard-coded union string with `AccountType`                                                                        |

### Must confirm (archive default)

| File                                                          | Line range | Status                                                                              |
| ------------------------------------------------------------- | ---------- | ----------------------------------------------------------------------------------- |
| `apps/frontend/src/hooks/use-accounts.ts`                     | 7          | `includeArchived = false` is already default ✓                                      |
| `apps/frontend/src/components/account-selector.tsx`           | 127-130    | `includeArchived: false` passed explicitly ✓                                        |
| `apps/frontend/src/components/account-selector-mobile.tsx`    | 79-82      | `includeArchived: false` passed explicitly ✓                                        |
| `apps/frontend/src/pages/dashboard/accounts-summary.tsx`      | 264        | Calls `useAccounts()` with NO options → falls to default `includeArchived: false` ✓ |
| `apps/frontend/src/pages/settings/accounts/accounts-page.tsx` | 25         | `includeArchived: true` — correct for the settings page which toggles visibility    |

**Archive default is already correct everywhere.** No audit-required changes.

### Must create (new routes / pages)

| File                                                              | Status                                          | Purpose                                                                                                     |
| ----------------------------------------------------------------- | ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `apps/frontend/src/pages/accounts/accounts-page.tsx` (or similar) | NEW                                             | The `/accounts` unified list route per UI-SPEC section 1                                                    |
| `apps/frontend/src/pages/accounts/account-form.tsx` (or similar)  | NEW (or refactor existing settings AccountForm) | Dynamic new-account form per UI-SPEC section 2                                                              |
| `apps/frontend/src/pages/accounts/update-balance-modal.tsx`       | NEW                                             | "Update balance" modal per UI-SPEC section 5                                                                |
| `apps/frontend/src/routes.tsx`                                    | EXTEND                                          | Add `<Route path="accounts" element={<AccountsPage />} />` (line 155 is `accounts/:id` — add sibling above) |

### Must extend (per-account detail for CC)

| File                                               | Line range | Change                                                                                                                                                                                                                                                                                        |
| -------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `apps/frontend/src/pages/account/account-page.tsx` | throughout | Render CC-specific sections per UI-SPEC section 3 when `account.accountType === "CREDIT_CARD"`. For `CHECKING`/`SAVINGS`/`LOAN`, hide investment-only modules (HistoryChart, AccountHoldings, AccountMetrics, AccountContributionLimit) and show single "Balance" card per UI-SPEC section 4. |

### Accounts-summary unchanged at code level

`accounts-summary.tsx` already groups by `account.group ?? "Uncategorized"` and
passes `accountType` through as data (never switches on it). New types will flow
through correctly without modification. **Confirmed safe.**

### UI-SPEC Component Reuse Opportunities

- **Row shape** → reuse `AccountSummaryComponent` from `accounts-summary.tsx`
  (lines 56-257) as-is.
- **Form primitives** → `@whaleit/ui` already exports: `Form`, `FormField`,
  `FormItem`, `FormLabel`, `FormControl`, `FormMessage`, `Input`, `Select`,
  `ResponsiveSelect`, `ToggleGroup`, `ToggleGroupItem`, `MoneyInput`,
  `CurrencyInput`, `DatePickerInput`, `Switch`, `Textarea`, `AlertDialog`,
  `Sheet`, `Dialog`, `Card`, `CardHeader`, `CardTitle`, `CardContent`,
  `Progress`, `Separator`, `Skeleton`, `EmptyPlaceholder`, `Tooltip`,
  `PrivacyAmount`, `GainAmount`, `Button`.
- **Empty state** → `EmptyPlaceholder` with
  `<EmptyPlaceholder.Icon name="Wallet" />` is the canonical pattern (already
  used in `accounts-page.tsx:235-244`).
- **Group/List toggle** → matches existing `accounts-summary.tsx` group toggle
  convention.
- **Icons** → All Phase 3 icons exist in `@whaleit/ui` Icons map EXCEPT
  `Landmark` (use `Building2` per UI-SPEC fallback) and `MoreHorizontal` (use
  `Ellipsis` which is present).

## Archive Filter Audit

| Call site                                                         | Default behavior                                | Phase 3 expectation                    | Status                           |
| ----------------------------------------------------------------- | ----------------------------------------------- | -------------------------------------- | -------------------------------- |
| `useAccounts()` hook default                                      | `includeArchived: false`                        | Archived hidden                        | ✓ correct                        |
| `AccountSelector` (`account-selector.tsx:127-130`)                | Explicit `includeArchived: false`               | Archived hidden from selector          | ✓ correct                        |
| `AccountSelectorMobile` (`account-selector-mobile.tsx:79-82`)     | Explicit `includeArchived: false`               | Archived hidden                        | ✓ correct                        |
| Dashboard `accounts-summary.tsx` (`accounts-summary.tsx:264`)     | Default (`includeArchived: false`)              | Archived hidden from dashboard         | ✓ correct                        |
| `SettingsAccountsPage` (`settings/accounts/accounts-page.tsx:25`) | Explicit `includeArchived: true` + local filter | Settings page shows all for management | ✓ correct (not affected by D-19) |
| `AppLauncher` (`app-launcher.tsx:78`)                             | Default                                         | Archived hidden from launcher          | ✓ correct                        |
| NEW `/accounts` page                                              | —                                               | Default hidden, toggle reveals         | NEW work                         |

**Result:** Archive default is already consistent across the codebase. The ONLY
new archive-related work is in the new `/accounts` page (D-19 "Show archived"
toggle) and ensuring the new page passes `includeArchived: showArchivedToggle`
through.

## Validation Layer

### Existing Pattern (Canonical)

```rust
// crates/core/src/accounts/accounts_model.rs:72-87
impl NewAccount {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Account name cannot be empty".to_string(),
            )));
        }
        if self.currency.trim().is_empty() {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Currency cannot be empty".to_string(),
            )));
        }
        Ok(())
    }
}
```

Called by `repository.rs:30`:

```rust
async fn create(&self, new_account: NewAccount) -> Result<Account> {
    new_account.validate()?;
    ...
}
```

### Phase 3 Extension (Recommended Shape)

```rust
// Add to NewAccount::validate()
use crate::accounts::account_types;

fn validate(&self) -> Result<()> {
    if self.name.trim().is_empty() { ... }
    if self.currency.trim().is_empty() { ... }
    if self.institution.as_ref().map(|s| s.trim().is_empty()).unwrap_or(false) {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "Institution cannot be empty".to_string(),
        )));
    }

    let is_credit_card = self.account_type == account_types::CREDIT_CARD;
    let is_bank_or_loan = matches!(
        self.account_type.as_str(),
        account_types::CHECKING | account_types::SAVINGS | account_types::LOAN
    );

    // D-06: CC-specific fields must all be null for non-CC accounts
    if !is_credit_card {
        if self.credit_limit.is_some()
            || self.statement_cycle_day.is_some()
            || self.statement_balance.is_some()
            || self.minimum_payment.is_some()
            || self.statement_due_date.is_some()
            || self.reward_points_balance.is_some()
            || self.cashback_balance.is_some()
        {
            return Err(Error::Validation(ValidationError::InvalidInput(
                "Credit card fields are only valid for CREDIT_CARD accounts".to_string(),
            )));
        }
    } else {
        // CC required: credit_limit > 0 and statement_cycle_day in 1..=31
        match self.credit_limit {
            Some(ref limit) if limit > &Decimal::ZERO => {}
            _ => return Err(Error::Validation(ValidationError::InvalidInput(
                "Credit limit must be greater than 0".to_string(),
            ))),
        }
        match self.statement_cycle_day {
            Some(d) if (1..=31).contains(&d) => {}
            _ => return Err(Error::Validation(ValidationError::InvalidInput(
                "Statement cycle day must be between 1 and 31".to_string(),
            ))),
        }
    }

    // D-11: opening_balance required for bank/CC/LOAN; optional semantics for SECURITIES/CASH/CRYPTOCURRENCY
    if (is_bank_or_loan || is_credit_card) && self.opening_balance.is_none() {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "Opening balance is required for bank, credit card, and loan accounts".to_string(),
        )));
    }

    // Context D (discretion): opening_balance >= 0 for bank/LOAN; any value for CC
    if is_bank_or_loan {
        if let Some(ref ob) = self.opening_balance {
            if ob < &Decimal::ZERO {
                return Err(Error::Validation(ValidationError::InvalidInput(
                    "Opening balance cannot be negative for bank or loan accounts".to_string(),
                )));
            }
        }
    }

    Ok(())
}
```

`AccountUpdate::validate()` gets a parallel but weaker shape — updates are
partial, so the CC-gated null rule applies only to fields the user actually
supplied.

### Frontend Mirror

`newAccountSchema.superRefine((data, ctx) => { ... })` implements the same
rules. This is belt-and-braces — the backend is still the authority.

## Balance Update Flow Recommendation

CONTEXT.md D-12: "A manual 'Update balance' action on the account edit page
writes a new `current_balance` and bumps the timestamp."

**Two implementations considered:**

| Option          | Description                                                                                                                                                                                                                     | Files touched                           | Pros                                                                                                                          | Cons                                                                                      |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| A               | New `PATCH /accounts/{id}/balance` HTTP endpoint + new `update_account_balance` core service method + new repo method                                                                                                           | 7 (see Adapter section)                 | Explicit intent; server auto-stamps `balance_updated_at`; ensures no caller accidentally sets balance via generic update      | 7-file surface for one behavior; duplicates trait/impl plumbing                           |
| B (recommended) | Reuse `update_account`. UI-only modal that constrains the payload to `{ current_balance, balance_updated_at: now() }`. Server bumps `balance_updated_at` automatically whenever `current_balance` changes (service-layer rule). | 0 new endpoints; ~2 service-layer lines | Zero new API surface; consistent with existing update semantics; intent conveyed via UI component, not by a separate endpoint | Intent is only "implicit" — a misbehaving client COULD set any field via `update_account` |

**Recommended: Option B.** Rationale:

1. CLAUDE.md §2 (Simplicity First): "No abstractions for single-use code."
2. `balance_updated_at` auto-bumping in the service layer is a 3-line addition
   in `AccountService::update_account`:

   ```rust
   // accounts_service.rs line ~89 (before update call)
   let mut update = account_update;
   if update.current_balance.is_some() && update.current_balance != Some(existing.current_balance.clone().unwrap_or_default()) {
       update.balance_updated_at = Some(chrono::Utc::now().naive_utc());
   }
   ```

3. The UI-SPEC's "Update balance" modal (section 5) is a UX component — not a
   distinct semantic operation. The modal prevents users from editing other
   fields; that is UX scope, not API scope.
4. Phase 4 reconciliation (D-14) is agnostic to which endpoint wrote the balance
   — it only needs `current_balance`, `opening_balance`, `balance_updated_at`,
   and transaction totals.

If the planner disagrees and picks Option A, flag it explicitly — it's a ~7-file
expansion per D-12's "distinct action" reading.

## Derived Helpers — `account_kind()`

### Rust

Add to `crates/core/src/accounts/accounts_constants.rs` (co-located with
existing constants + `default_group_for_account_type`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    Asset,
    Liability,
    Investment,
}

pub fn account_kind(account_type: &str) -> AccountKind {
    match account_type {
        account_types::CHECKING | account_types::SAVINGS | account_types::CASH => AccountKind::Asset,
        account_types::CREDIT_CARD | account_types::LOAN => AccountKind::Liability,
        account_types::SECURITIES | account_types::CRYPTOCURRENCY => AccountKind::Investment,
        _ => AccountKind::Asset, // conservative default for forward-compat
    }
}
```

Export via `mod.rs:9` (already `pub use accounts_constants::*`).

**`AccountKind` name collision check:** grep confirms zero existing uses. Safe.

### TypeScript

Mirror in `apps/frontend/src/lib/constants.ts` immediately after
`defaultGroupForAccountType`:

```typescript
export const AccountKind = {
  ASSET: "ASSET",
  LIABILITY: "LIABILITY",
  INVESTMENT: "INVESTMENT",
} as const;

export type AccountKind = (typeof AccountKind)[keyof typeof AccountKind];

export function accountKind(accountType: AccountType): AccountKind {
  switch (accountType) {
    case AccountType.CHECKING:
    case AccountType.SAVINGS:
    case AccountType.CASH:
      return AccountKind.ASSET;
    case AccountType.CREDIT_CARD:
    case AccountType.LOAN:
      return AccountKind.LIABILITY;
    case AccountType.SECURITIES:
    case AccountType.CRYPTOCURRENCY:
      return AccountKind.INVESTMENT;
    default: {
      const _exhaustive: never = accountType;
      return AccountKind.ASSET;
    }
  }
}
```

The `never` check guarantees TS compile errors if the enum gains another variant
without updating this helper — a good safety net.

## Validation Architecture

### Test Framework

| Property                | Value                                                                                                |
| ----------------------- | ---------------------------------------------------------------------------------------------------- |
| Rust unit/integration   | `cargo test` (default cargo test harness + `tokio::test` for async)                                  |
| Frontend unit           | Vitest 3.2.4 (`vitest`)                                                                              |
| Frontend component      | React Testing Library 16.3.2 + jest-dom 6.9.1                                                        |
| E2E                     | Playwright `^1.58.2` via `node scripts/run-e2e.mjs`                                                  |
| Config (frontend)       | `apps/frontend/vite.config.ts` (inferred — no separate vitest.config; vitest reads from vite.config) |
| Config (e2e)            | `/Users/muhamad.rohman/Workspace/github.com/muhx/whaleit/playwright.config.ts`                       |
| Rust quick run          | `cargo test -p whaleit-core accounts::`                                                              |
| Rust PG integration run | `cargo test -p whaleit-storage-postgres accounts` (requires DATABASE_URL)                            |
| Frontend quick run      | `pnpm --filter frontend test <file>`                                                                 |
| Frontend full suite     | `pnpm test`                                                                                          |
| E2E run                 | `pnpm test:e2e`                                                                                      |

### Phase Requirements → Test Map

| Req ID                | Behavior                                                                    | Test Type                     | Automated Command                                                                                                                       | File Exists?       |
| --------------------- | --------------------------------------------------------------------------- | ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ------------------ |
| ACCT-01               | Bank account create with name/institution/currency/opening_balance persists | Rust unit (domain + validate) | `cargo test -p whaleit-core accounts::accounts_model_tests::test_new_account_validate_bank`                                             | ❌ Wave 0 extend   |
| ACCT-01               | Bank account create end-to-end against PG                                   | Rust integration              | `cargo test -p whaleit-storage-postgres accounts::repository_tests`                                                                     | ❌ Wave 0 create   |
| ACCT-02               | CC create with credit_limit + statement_cycle_day validated                 | Rust unit                     | `cargo test -p whaleit-core accounts::accounts_model_tests::test_new_account_validate_credit_card`                                      | ❌ Wave 0          |
| ACCT-02               | CC create rejects missing credit_limit / invalid cycle_day                  | Rust unit                     | `cargo test -p whaleit-core accounts::accounts_model_tests::test_new_account_validate_credit_card_rejects_invalid`                      | ❌ Wave 0          |
| ACCT-02               | Non-CC create rejects CC fields present                                     | Rust unit                     | `cargo test -p whaleit-core accounts::accounts_model_tests::test_new_account_validate_non_cc_rejects_cc_fields`                         | ❌ Wave 0          |
| ACCT-03               | `/accounts` page renders all types with current_balance                     | Frontend component            | `pnpm --filter frontend test apps/frontend/src/pages/accounts/accounts-page.test.tsx`                                                   | ❌ Wave 0 create   |
| ACCT-04               | Archive toggles `is_archived`, archived hidden by default in selectors      | Frontend component            | Extend `apps/frontend/src/pages/dashboard/accounts-summary.test.tsx`                                                                    | ✓ partial (extend) |
| ACCT-04               | Edit CC + bank preserves unrelated fields                                   | Rust integration              | `cargo test -p whaleit-storage-postgres accounts::repository_tests::test_update_preserves_fields`                                       | ❌ Wave 0          |
| ACCT-05               | `account_kind()` maps types correctly (both Rust + TS)                      | Rust + Frontend unit          | `cargo test -p whaleit-core accounts::tests::test_account_kind` + `pnpm --filter frontend test apps/frontend/src/lib/constants.test.ts` | ❌ Wave 0          |
| ACCT-05               | Available credit derived helper (`credit_limit - current_balance`)          | Frontend unit                 | `pnpm --filter frontend test apps/frontend/src/pages/accounts/credit-helpers.test.ts`                                                   | ❌ Wave 0          |
| ACCT-06               | Statement fields optional on CC; NULL on non-CC                             | Rust integration              | Same test file as ACCT-02 end-to-end                                                                                                    | — (folded)         |
| ACCT-07               | Reward points / cashback read/write                                         | Rust integration              | Same test file                                                                                                                          | — (folded)         |
| D-12 (balance update) | Updating `current_balance` bumps `balance_updated_at`                       | Rust unit + integration       | `cargo test -p whaleit-core accounts::accounts_service_tests::test_update_bumps_balance_timestamp`                                      | ❌ Wave 0          |
| D-19 (archive UX)     | `/accounts` page: archive toggle reveals archived rows                      | E2E                           | Extend Playwright `e2e/01-happy-path.spec.ts` or new `e2e/11-accounts.spec.ts`                                                          | ❌ Wave 0 create   |
| ACCT-01 → ACCT-07     | End-to-end user flow: create bank → CC → archive → update balance           | E2E                           | `pnpm test:e2e -- e2e/11-accounts.spec.ts`                                                                                              | ❌ Wave 0 create   |

### Sampling Rate

- **Per task commit:** `cargo test -p whaleit-core accounts::` (Rust core) +
  `pnpm --filter frontend test -- --run apps/frontend/src/lib/` (frontend
  constants/schemas). Under 30 seconds combined.
- **Per wave merge:** `cargo test --workspace` + `pnpm test` + one Playwright
  spec run.
- **Phase gate:** Full `cargo test --workspace` + full `pnpm test` + full
  `pnpm test:e2e` green.

### Wave 0 Gaps

- [ ] `crates/core/src/accounts/accounts_model_tests.rs` — extend with
      CC-validation and account_kind tests
- [ ] `crates/core/src/accounts/accounts_service_tests.rs` — NEW — covers
      balance-timestamp auto-bump (D-12)
- [ ] `crates/storage-postgres/src/accounts/repository_tests.rs` — NEW — PG
      roundtrip for each new AccountType + field set. Requires `DATABASE_URL`
      fixture.
- [ ] `apps/frontend/src/lib/constants.test.ts` — NEW — tests for
      `accountKind()` and `defaultGroupForAccountType` extensions
- [ ] `apps/frontend/src/lib/schemas.test.ts` — extend existing file with
      CC-gated validation cases
- [ ] `apps/frontend/src/pages/accounts/accounts-page.test.tsx` — NEW — unified
      list rendering + archive toggle behavior
- [ ] `e2e/11-accounts.spec.ts` (or extend `01-happy-path.spec.ts`) — NEW —
      Playwright end-to-end flow for create / archive / balance update

Framework install: no new installs required. All test infra already present.

## Schema Push Command

**No standalone push command required.** Migrations run embedded at server
startup via `crates/storage-postgres/src/db/mod.rs:57-77`:

```rust
pub async fn run_migrations(database_url: &str) -> Result<()> {
    // ... MigrationHarness.run_pending_migrations(MIGRATIONS)
}
```

Called from `apps/server/src/main_lib.rs` boot path. New migration files in
`crates/storage-postgres/migrations/` are auto-picked-up by
`embed_migrations!()`.

For development / test workflow:

```bash
# If diesel-cli is installed and DATABASE_URL is set:
cd crates/storage-postgres
diesel migration run          # applies pending migrations
diesel print-schema > src/schema.rs  # regenerates schema.rs

# Alternatively, just start the server (auto-migrates):
pnpm run dev:web
```

## Environment Availability

| Dependency                             | Required By                            | Available                                  | Version | Fallback                                                                         |
| -------------------------------------- | -------------------------------------- | ------------------------------------------ | ------- | -------------------------------------------------------------------------------- |
| Rust toolchain                         | Cargo build + tests                    | Assumed ✓ (repo is workspace)              | stable  | —                                                                                |
| diesel-cli (for `diesel print-schema`) | Regenerate `schema.rs` after migration | Not verified in repo                       | —       | Hand-edit `schema.rs` to mirror migration SQL (feasible for 11 additive columns) |
| PostgreSQL                             | Runtime + `cargo test` integration     | Not verified                               | —       | Docker Compose already has `postgres` service (per Phase 2 D-21)                 |
| Node/pnpm                              | Frontend dev + tests                   | Assumed ✓                                  | —       | —                                                                                |
| Playwright                             | E2E                                    | Installed (`^1.58.2` in root package.json) | 1.58.2  | —                                                                                |

**Missing dependencies with no fallback:** None identified — Phase 2 established
the PG dev environment.

**Missing dependencies with fallback:** `diesel-cli` may not be installed
globally; schema regeneration has a manual fallback.

## Security Surface & Threats

### Applicable ASVS Categories

| ASVS Category         | Applies | Standard Control                                                                                                                                                |
| --------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V2 Authentication     | no      | Handled by existing `apps/server/src/auth.rs`                                                                                                                   |
| V3 Session Management | no      | Existing                                                                                                                                                        |
| V4 Access Control     | partial | Existing web-mode user ownership model applies; Phase 3 adds no new authorization boundaries                                                                    |
| V5 Input Validation   | **yes** | zod (frontend) + service-layer `NewAccount::validate()` (backend). Ensures CC fields are type-gated, monetary values well-formed, statement_cycle_day in range. |
| V6 Cryptography       | no      | No crypto work in this phase                                                                                                                                    |

### Known Threat Patterns for Rust + Axum + Diesel

| Pattern                                                                      | STRIDE                             | Standard Mitigation                                                                                                                                                                                                                                                                                                                    |
| ---------------------------------------------------------------------------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| SQL injection via account_type / institution free-text                       | Tampering                          | Diesel parameterized queries already the default (`.filter(account_type.eq(t))`). No raw string SQL.                                                                                                                                                                                                                                   |
| Server-side trust of client-supplied `current_balance` without audit         | Repudiation                        | `balance_updated_at` auto-stamped server-side; Phase 4 will add transaction audit trail                                                                                                                                                                                                                                                |
| CC fields leaked for non-CC account types via overwrite                      | Information Disclosure / Tampering | Service-layer validation rejects CC-field presence on non-CC types (D-06)                                                                                                                                                                                                                                                              |
| Integer overflow on statement_cycle_day or reward_points_balance             | Tampering                          | zod `.int().min(1).max(31)` + PG SMALLINT/INTEGER types bound the range                                                                                                                                                                                                                                                                |
| `Decimal::from_str` silently substituting `Decimal::ZERO` on malformed input | Tampering / Integrity              | Current pattern uses `.unwrap_or(Decimal::ZERO)` — planner should decide whether to preserve the pattern (lenient) or error out (strict). Recommendation: preserve the pattern (consistent with rest of codebase), surface a warning log, and add a stronger `Decimal::from_str` check at service-layer validation BEFORE persistence. |
| Credit-limit abuse to trigger integer overflow in utilization calc           | Availability                       | Frontend derives utilization as `current_balance / credit_limit * 100`; protect against division-by-zero when `credit_limit === 0` (already covered by CC validation `credit_limit > 0`, but defensive code in frontend is worth adding)                                                                                               |
| Free-text `institution` XSS via UI render                                    | Tampering                          | React escapes text content by default; no unsafe HTML rendering in the rendering path                                                                                                                                                                                                                                                  |

**No new attack surface at the network layer.** All new fields flow through the
same authenticated `/api/v1/accounts` endpoints.

## Landmines

1. **CONTEXT D-10 (NUMERIC) vs codebase (TEXT).** Follow existing
   TEXT-serialized-Decimal pattern unless user explicitly confirms NUMERIC
   divergence. Both interpretations satisfy "arbitrary precision for money."
2. **TS exhaustive `Record<AccountType, Icon>` in `app-launcher.tsx:65` and
   `account-page.tsx:82`.** Adding `AccountType` variants will fail TS compile
   until these maps get 4 new entries. Mechanical fix but easy to miss.
3. **`net_worth_service.rs:67` fallback `_ => Investment`.** Unchanged in Phase
   3 (net worth is Phase 6). But `account_kind()` MUST land in Phase 3 so Phase
   6 can use it. If Phase 3 forgets, Phase 6 will either re-derive the helper or
   miscategorize liabilities.
4. **`schema.rs` is hand-synchronized.** There's no `[print_schema]` in
   `diesel.toml`. Forgetting to update `schema.rs` after writing the migration
   will cause Diesel's compile-time DSL to reject the new columns at build time.
5. **`AccountUpdate` in storage-postgres re-reads and overwrites preserved
   fields (`repository.rs:61-75`).** New CC fields need to decide per-field:
   preserved on update (read from DB if not in payload) or always overwritten
   (trust the payload). UI-SPEC edit flow implies overwritable — but this means
   null-ing a CC field is indistinguishable from "don't touch" unless the DTO
   uses `Option<Option<T>>` or similar. Planner decision.
6. **`accounts_summary.tsx` group heading stays based on `account.group`.** New
   types default to group "Banking"/"Credit Cards"/"Loans" via
   `default_group_for_account_type`, but the column is nullable and can be
   overridden by the user. If users set a custom `group`, the D-16 defaults do
   not take effect — this is expected per D-16 wording but needs copywriting in
   the form hint.
7. **Frontend `balance` field on `Account` (types/account.ts:13).** This legacy
   field is a number, NOT the same as the new `current_balance`
   (`Decimal`/`string`). Planner must decide whether to (a) keep both and have
   them coexist, (b) deprecate `balance` and migrate callers (many), or (c) map
   `current_balance` onto `balance` at the serialization boundary. The existing
   `balance: 0` defaults in `createPortfolioAccount` (`constants.ts:23`,
   `account-selector.tsx:60`) suggest this field is barely used — a grep will
   confirm the blast radius.
8. **Mobile FAB requirement in UI-SPEC section 1.** The current app has no FAB
   pattern (confirmed by grep of `rounded-full` + `fixed bottom`) — building it
   is genuine new component work. It's styling-only (no new library) but must
   respect `pb-safe` + `--mobile-nav-ui-height`. Reference the existing
   settings/accounts mobile icon button as a starting point.
9. **`update_account` on the current repo preserves currency
   (`repository.rs:61`).** If the planner wants currency editable on CC before
   balance update, this preservation logic blocks it. The web adapter
   (`adapters/shared/accounts.ts:45-53`) already strips currency on desktop
   updates — indicating the established UX rule: **currency is immutable after
   creation**. Phase 3 should keep this constraint.
10. **`newAccountSchema.name.min(2)` (schemas.ts:82-84)** — tighter than backend
    (`name.trim().is_empty()`). Not a bug, but if the planner exposes the schema
    via a shared constant, make sure backend and frontend agree.

## Open Questions for Planner

1. **D-10 NUMERIC vs TEXT for money columns** — Must resolve before migration
   writing. See Executive Summary + Migration Pattern section. Recommended: TEXT
   (follow established pattern) with explicit flag in discuss-phase for user
   confirmation.
2. **`update_account_balance` as a distinct command?** — Recommendation: reuse
   `update_account`. See Balance Update Flow section. Planner decides.
3. **Settings AccountForm vs new /accounts form** — Should the new `/accounts`
   route's "New account" flow REPLACE `/settings/accounts` creation, or do both
   coexist? UI-SPEC section 1 implies one primary location; settings could
   redirect to `/accounts?new=true` to avoid duplication.
4. **`balance` legacy TS field** — Audit blast radius and decide: deprecate,
   alias, or coexist. See Landmine 7.
5. **Partial CC-field updates** — Does `AccountUpdate` allow nulling a CC field
   (e.g. clearing `statement_balance`)? If yes, the
   `From<AccountUpdate> for AccountDB` impl needs to distinguish "field absent
   in update" from "field explicitly set to null" — today the pattern is
   implicit (fields without `Option<Option<T>>` default to preservation). See
   Landmine 5.

## Sources

### Primary (HIGH confidence)

- `crates/core/src/accounts/accounts_model.rs` —
  Account/NewAccount/AccountUpdate structs; `NewAccount::validate` pattern
- `crates/core/src/accounts/accounts_constants.rs` — AccountType `&str`
  constants; `default_group_for_account_type`
- `crates/core/src/accounts/accounts_service.rs` — Service layer;
  `create_account` / `update_account` implementations
- `crates/core/src/accounts/accounts_traits.rs` — Repository + service traits
  (signatures unchanged this phase)
- `crates/storage-postgres/src/accounts/model.rs` — `AccountDB` Diesel model +
  `From` impls
- `crates/storage-postgres/src/accounts/repository.rs` — Diesel CRUD
  implementation
- `crates/storage-postgres/src/schema.rs:13-31` — Current `accounts` table DSL
- `crates/storage-postgres/migrations/20260101000000_initial_schema/up.sql` —
  Money-column convention (TEXT for Decimal)
- `crates/storage-postgres/migrations/20260422000000_auth_users/up.sql` —
  Additive migration pattern
- `crates/storage-postgres/src/db/mod.rs` — Embedded migration runner; pool
  setup
- `crates/storage-postgres/src/fx/model.rs:46-88` — Canonical Decimal
  deserialization pattern
- `apps/server/src/api/accounts.rs` — HTTP handlers
- `apps/server/src/models.rs` — HTTP DTOs + domain `From` impls
- `apps/frontend/src/lib/constants.ts:44-72` — Frontend AccountType +
  `defaultGroupForAccountType`
- `apps/frontend/src/lib/schemas.ts:76-96` — `newAccountSchema`
- `apps/frontend/src/hooks/use-accounts.ts` — Archive default behavior
- `apps/frontend/src/adapters/shared/accounts.ts` +
  `adapters/web/modules/accounts.ts` + `adapters/web/core.ts:38-42,435-442` —
  Full adapter path
- `apps/frontend/src/pages/dashboard/accounts-summary.tsx` — Grouping by
  `account.group`
- `apps/frontend/src/pages/settings/accounts/components/account-form.tsx` —
  Existing form reference
- `apps/frontend/src/routes.tsx` — Routing config
- `apps/tauri/src/lib.rs` — Confirms NO IPC commands registered
- `packages/ui/src/components/ui/icons.tsx` — Verified which icons exist
  (Wallet, Coins, CreditCard, Building2, etc.)
- `.planning/phases/02-dual-database-engine/02-CONTEXT.md` + `02-RESEARCH.md` —
  Storage conventions from Phase 2
- `.claude/CLAUDE.md` — Simplicity / Surgical-changes guidelines
- `memory/project_storage_pivot_pg_only.md` — PG-only confirmation

### Secondary (MEDIUM)

- None — all claims verified against current codebase.

### Tertiary (LOW / ASSUMED)

- `[ASSUMED]` D-10 reconciliation recommendation (TEXT pattern) — a judgment
  call about which interpretation of "NUMERIC" to honor.
- `[ASSUMED]` Recommendation to skip `update_account_balance` distinct command —
  trades explicit intent for fewer touched files; planner may override.
- `[ASSUMED]` `diesel-cli` availability on dev machine — not verified; fallback
  is hand-editing `schema.rs`.

## Assumptions Log

| #   | Claim                                                                                           | Section                  | Risk if Wrong                                                                                                                |
| --- | ----------------------------------------------------------------------------------------------- | ------------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| A1  | Follow established TEXT pattern for D-10 NUMERIC columns                                        | Migration Pattern        | Medium — if user intended native PG NUMERIC, migration re-work required; service-layer validation remains correct regardless |
| A2  | Reuse `update_account` for balance updates instead of dedicated command                         | Balance Update Flow      | Low — Option A migration is additive and can be done later if needed                                                         |
| A3  | `diesel-cli` available for `schema.rs` regeneration                                             | Environment Availability | Low — hand-editing is straightforward for 11 additive columns                                                                |
| A4  | UI-SPEC's mention of `MoreHorizontal` icon can be substituted with `Ellipsis` (already present) | Frontend Integration     | Low — purely cosmetic substitution                                                                                           |

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all libraries already wired; no version decisions
  required.
- Architecture: HIGH — pattern is established and additive; every new field has
  a clear home.
- Pitfalls: HIGH — landmines grep-verified against current source.
- UI-SPEC alignment: HIGH — every called-for primitive exists in `@whaleit/ui`
  with minor icon substitutions.
- D-10 NUMERIC ambiguity: MEDIUM — recommendation documented, user confirmation
  needed in discuss-phase review.

**Research date:** 2026-04-25 **Valid until:** 2026-05-25 (30 days — storage
layer has been stable since Phase 2)

## RESEARCH COMPLETE

**Phase:** 3 — Bank Accounts & Credit Cards **Confidence:** HIGH

### Key Findings

- Money columns are TEXT in this codebase (not NUMERIC); D-10 reconciliation
  needed before migration writing.
- `AccountType` is `String` in Rust — NO exhaustive match to break. TS has two
  exhaustive `Record<AccountType, Icon>` maps that WILL break compile
  (`app-launcher.tsx:65`, `account-page.tsx:82`) and must be extended.
- NO Tauri IPC commands exist for accounts — desktop uses the Axum HTTP path via
  the web adapter. Only Axum needs new wiring; "adapter layer" reduces to one
  path, not two.
- Migrations run embedded at server startup; no separate push command. BUT
  `schema.rs` is hand-synchronized and MUST be updated after the migration.
- Balance-update command can be expressed as a reuse of `update_account` with a
  UI-constrained modal (saves 7 files vs dedicated endpoint).

### File Created

`.planning/phases/03-bank-accounts-credit-cards/03-RESEARCH.md`

### Confidence Assessment

| Area                        | Level  | Reason                                                          |
| --------------------------- | ------ | --------------------------------------------------------------- |
| Standard Stack              | HIGH   | All libraries/components already present                        |
| Architecture                | HIGH   | Pattern well-established and additive                           |
| Pitfalls                    | HIGH   | Landmines grep-verified                                         |
| Validation plan             | HIGH   | Existing Vitest + Playwright + cargo test harnesses cover needs |
| D-10 NUMERIC interpretation | MEDIUM | User confirmation advised                                       |

### Open Questions

See §"Open Questions for Planner" — five items, chief among them the D-10
NUMERIC/TEXT question.

### Ready for Planning

Research complete. Planner can now create PLAN.md files, including an explicit
Wave 0 that:

1. Resolves D-10 (TEXT vs NUMERIC) in discuss-phase review.
2. Creates the missing test scaffolds listed in Wave 0 Gaps.
3. Sequences work as: migration + schema.rs + Diesel model → core
   model/validation/helpers → DTOs → frontend constants/schema → frontend pages.
