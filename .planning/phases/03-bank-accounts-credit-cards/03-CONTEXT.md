# Phase 3: Bank Accounts & Credit Cards - Context

**Gathered:** 2026-04-25 **Status:** Ready for planning

<domain>
## Phase Boundary

Extend the existing account domain to cover bank accounts (CHECKING, SAVINGS),
credit cards (CREDIT_CARD), and basic loans (LOAN, enum-slot-only) alongside the
existing investment/cash/crypto accounts. Deliver create, edit, and archive
flows for these new types and a unified account list that shows current balances
across every account family, with credit-card-specific fields (credit limit,
statement snapshot, rewards) captured directly on the account record. PostgreSQL
is the sole storage engine (SQLite has been removed).

**Out of scope for Phase 3** (belongs to later phases or deferred):

- Transaction recording / CSV / OFX import (Phase 4)
- Per-statement history table (v1 keeps only latest statement snapshot on the
  account)
- Loan principal / interest / amortization tracking (enum slot only)
- Rewards earning rules / category multipliers
- Institution lookup table or autocomplete
- Bank API / Plaid / SnapTrade-style sync for bank accounts (PROJECT.md
  out-of-scope)

</domain>

<decisions>
## Implementation Decisions

### Account Type Model

- **D-01:** Extend `AccountType` enum with three new leaf values: `CHECKING`,
  `SAVINGS`, `CREDIT_CARD`, `LOAN`. Existing `SECURITIES`, `CASH`,
  `CRYPTOCURRENCY` stay untouched.
- **D-02:** `LOAN` is enum-slot-only in Phase 3 — basic CRUD (name, institution,
  currency, opening balance, archive), categorized as a liability via the
  derived helper, but no principal/interest/amortization tracking. That
  expansion is a future phase.
- **D-03:** Asset / liability / investment semantics are expressed via a derived
  helper in `crates/core`:

  ```rust
  pub enum AccountKind { Asset, Liability, Investment }
  pub fn account_kind(t: &str) -> AccountKind
  ```

  - `CHECKING`, `SAVINGS`, `CASH` → `Asset`
  - `CREDIT_CARD`, `LOAN` → `Liability`
  - `SECURITIES`, `CRYPTOCURRENCY` → `Investment`

  No new column on the `accounts` table. Frontend gets an equivalent helper in
  `apps/frontend/src/lib/constants.ts`.

- **D-04:** Default `tracking_mode` for new bank (`CHECKING`/`SAVINGS`), credit
  card, and loan accounts = `TRANSACTIONS`. Before Phase 4 the field is set but
  unused; it declares the long-term model so Phase 4 doesn't need a migration
  sweep.
- **D-05:** Existing `CASH` accounts remain as-is. `CHECKING` and `SAVINGS` are
  additive. No data migration from `CASH` → anything.

### Credit-Card Field Storage

- **D-06:** All credit-card-specific fields live as **nullable columns on the
  existing `accounts` table**. No side table, no JSON blob. Fields added this
  phase:
  - `credit_limit` (NUMERIC) — positive; card's approved line
  - `statement_cycle_day` (SMALLINT, 1..=31) — day of month a cycle closes
  - `statement_balance` (NUMERIC) — latest statement balance
  - `minimum_payment` (NUMERIC) — latest minimum payment due
  - `statement_due_date` (DATE) — latest due date
  - `reward_points_balance` (INTEGER) — optional points balance
  - `cashback_balance` (NUMERIC) — optional cashback balance

  All are null for non-credit-card rows. Non-null constraints, if any, are
  enforced at the service layer based on `account_type`.

- **D-07:** Statement snapshot fields (`statement_balance`, `minimum_payment`,
  `statement_due_date`) are **current-snapshot-only**. User overwrites them each
  cycle via the account edit form. No statement-history table in Phase 3.
  Matches ACCT-05/06 wording.
- **D-08:** Credit utilization % is **derived on the fly**
  (`current_balance / credit_limit * 100`), computed in core service / frontend.
  No stored `utilization` column.
- **D-09:** Rewards tracking is **manual balance fields**
  (`reward_points_balance`, `cashback_balance`). User updates them manually. No
  earning rules, no category multipliers, no integration with transactions.
- **D-10:** All money columns (balances, limits, payments) use `NUMERIC`
  (arbitrary precision) matching the existing pattern for monetary values in
  `storage-postgres`. Points uses `INTEGER`.

### Balance Before Phase 4 (Transaction Core)

- **D-11:** Add **dedicated `opening_balance` (NUMERIC)** column on `accounts`.
  Captured at account creation via a required field on `NewAccount` for
  bank/CC/LOAN (defaults to 0 for new cards; equal to the current statement
  balance or zero for existing cards the user is importing).
- **D-12:** Add **`current_balance` (NUMERIC)** + **`balance_updated_at`
  (TIMESTAMPTZ)** columns on `accounts`. Before Phase 4, a manual "Update
  balance" action on the account edit page writes a new `current_balance` and
  bumps the timestamp. In Phase 4, transaction inserts take over as the source
  of truth.
- **D-13:** Credit-card balances are stored as **positive values**.
  `CREDIT_CARD` is interpreted as a liability by the derived helper
  (`account_kind`), so net-worth / report consumers subtract CC balances
  automatically. A card owing $500 stores `current_balance = 500.00`.
  Utilization and "available credit" chips use the same positive semantics.
- **D-14:** **Phase 4 reconciliation story (planned now so Phase 3 doesn't paint
  itself into a corner):** when the first real transaction is inserted against a
  Phase-3-era account, Phase 4 will auto-generate an **"Opening Balance"
  transaction** dated at `account.created_at` with amount = `opening_balance`.
  If the user has since manually updated `current_balance` (delta !=
  sum(transactions) between created_at and now), Phase 4 also materializes that
  delta as a second "Balance adjustment" transaction so totals reconcile
  exactly. Phase 3 records just the data needed for this (opening_balance,
  current_balance, balance_updated_at).

### Unified Account List UX

- **D-15:** **Keep** the existing `accounts-summary.tsx` group-based layout on
  the dashboard. Introduce a **dedicated `/accounts` route** for the full
  unified list + archive toggle. Account detail page (`account-page.tsx`) stays
  per-account.
- **D-16:** Pre-seeded **default `group` values per `AccountType`** (used when
  the user does not set a custom group):
  - `CHECKING`, `SAVINGS` → `"Banking"`
  - `CREDIT_CARD` → `"Credit Cards"`
  - `LOAN` → `"Loans"`
  - `SECURITIES` → `"Investments"` _(unchanged)_
  - `CASH` → `"Cash"` _(unchanged)_
  - `CRYPTOCURRENCY` → `"Crypto"` _(unchanged)_

  Update `default_group_for_account_type` in both
  `crates/core/src/accounts/accounts_constants.rs` and
  `apps/frontend/src/lib/constants.ts`.

- **D-17:** Each row in the unified list shows: account name, institution,
  current balance in account currency, FX-converted base-currency equivalent,
  and (credit-card rows only) an "Available credit" chip derived from
  `credit_limit - current_balance`. Matches the existing accounts-summary row
  shape.
- **D-18:** **Institution** is a **free-text `institution` VARCHAR column** on
  `accounts`. User types the bank or issuer name. No lookup table, no
  autocomplete, no curation. Distinct from the existing `platform_id` which
  remains broker/sync-specific.
- **D-19:** **Archive UX:** archived accounts are **hidden by default** from the
  unified list, dashboard summary, and all account selectors. A "Show archived"
  toggle on `/accounts` reveals them in a separate group. Opening an archived
  account preserves full access to its historical data and edit page. No
  un-archive lock — user can flip `is_archived` back off.

### Claude's Discretion

Downstream agents (researcher, planner) decide these within the decisions above:

- Exact Diesel column type specifics in migrations (precision / scale on
  NUMERIC, CHECK constraint wording).
- Validation rules: `credit_limit > 0`, `statement_cycle_day BETWEEN 1 AND 31`,
  `opening_balance >= 0` for bank/loan, any NUMERIC for CC (cards can start at 0
  or with existing debt), `statement_due_date` vs `statement_cycle_day` sanity.
- Service-layer enforcement of "CC fields must all be null for non-CC types and
  must be present for CC type on create".
- Exact wording of empty-state copy, error messages, and button labels.
- UI polish: how the "Update balance" action surfaces (inline edit vs modal),
  available-credit chip styling, archive toggle placement.
- Whether the new `/accounts` route replaces or augments existing per-account
  detail navigation.
- Migration file numbering and Diesel schema.rs regeneration strategy (same
  pattern as Phase 2).

### Folded Todos

None — no pending todos matched this phase's scope.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning docs

- `.planning/ROADMAP.md` §Phase 3 — goal, success criteria, dependency on Phase
  2, requirement IDs (ACCT-01..07)
- `.planning/REQUIREMENTS.md` §Accounts — full text of ACCT-01..07
- `.planning/PROJECT.md` §Constraints, §Out of Scope — dual-runtime /
  local-first / PG-only guidance
- `.planning/phases/01-codebase-health-rebrand/01-CONTEXT.md` — rebrand
  decisions, npm scope, directory renames
- `.planning/phases/02-dual-database-engine/02-CONTEXT.md` — storage
  architecture (PG-only post-pivot), repository trait conventions, async
  strategy

### Core account domain (existing, extend — do not duplicate)

- `crates/core/src/accounts/accounts_model.rs` — `Account`, `NewAccount`,
  `AccountUpdate`, `TrackingMode`
- `crates/core/src/accounts/accounts_traits.rs` — `AccountRepositoryTrait`,
  `AccountServiceTrait`
- `crates/core/src/accounts/accounts_service.rs` — business logic to extend
- `crates/core/src/accounts/accounts_constants.rs` — `AccountType` constants and
  `default_group_for_account_type`

### PostgreSQL storage (existing PG impl, extend)

- `crates/storage-postgres/src/accounts/mod.rs`
- `crates/storage-postgres/src/accounts/model.rs` — `AccountDB` diesel mapping
- `crates/storage-postgres/src/accounts/repository.rs` — `PgAccountRepository`
- `crates/storage-postgres/src/schema.rs` — regenerate after migration
- `crates/storage-postgres/migrations/` — add a new numbered migration for this
  phase's columns

### Frontend (existing, extend)

- `apps/frontend/src/lib/constants.ts` (lines 44–75) — `AccountType` enum and
  `defaultGroupForAccountType`
- `apps/frontend/src/lib/types/account.ts` — `Account`, `AccountSummaryView`,
  `AccountGroup` TS types
- `apps/frontend/src/hooks/use-accounts.ts` — shared accounts hook
- `apps/frontend/src/adapters/shared/accounts.ts` — transport-agnostic account
  commands
- `apps/frontend/src/components/account-selector.tsx` +
  `account-selector-mobile.tsx` — must respect archived-hidden default
- `apps/frontend/src/pages/dashboard/accounts-summary.tsx` — existing grouped
  summary, reuse rendering for unified list
- `apps/frontend/src/pages/account/account-page.tsx` — per-account detail;
  extend with CC-specific sections

### Memory context (non-obvious, must apply)

- `memory/project_storage_pivot_pg_only.md` — Phase 02's "dual-engine" framing
  is historical. Only `crates/storage-postgres` exists. No SQLite migrations, no
  parity tests.

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `Account` / `NewAccount` / `AccountUpdate` domain structs — add new fields
  instead of creating parallel structs.
- `AccountRepositoryTrait` / `AccountServiceTrait` — methods stay; new fields
  flow through existing `create`, `update`, `get_by_id`, `list`.
- `tracking_mode` field and `TrackingMode` enum — already handles TRANSACTIONS
  vs HOLDINGS semantics; no new mode needed.
- `is_archived` boolean — already in the schema; hook up the "hidden by default"
  filter in `list` callers.
- `accounts-summary.tsx` grouping logic — reuse as the core of the new
  `/accounts` unified list.
- `useAccounts` hook, `adapters/shared/accounts.ts` — shared across Tauri and
  web, any new IPC command follows the same pattern.
- `meta` JSONB column — available but intentionally **not used** for CC fields
  (D-06 chose typed columns).

### Established Patterns

- Repository trait + concrete storage crate: new fields require updates in
  `accounts_model.rs`, `AccountDB`, `From<AccountDB> for Account`,
  `From<NewAccount> for AccountDB`, schema.rs (regenerated), and the web/Tauri
  adapter layers.
- Money columns use `NUMERIC` in PG, mapped to `Decimal` or `String` at the
  Diesel boundary (follow whatever pattern existing balance fields use).
- Enum values are stored as `String` in DB rows and converted via match arms in
  `From` impls — follow the same pattern for the new AccountType leaves.
- Frontend `AccountType` is a string-literal union + zod enum — extend both
  `AccountType` const and `accountTypeSchema`.

### Integration Points

- New migration file under `crates/storage-postgres/migrations/` adds columns:
  `opening_balance`, `current_balance`, `balance_updated_at`, `institution`,
  `credit_limit`, `statement_cycle_day`, `statement_balance`, `minimum_payment`,
  `statement_due_date`, `reward_points_balance`, `cashback_balance`. All
  nullable; validation at service layer.
- Web adapter (`apps/frontend/src/adapters/web/core.ts`) and IPC command
  registry need to surface any new commands (e.g., `update_account_balance`, if
  introduced as a distinct command rather than `update_account` mutation).
- Dashboard `accounts-summary.tsx` and any account selector currently assumes
  investment-oriented accounts — confirm they don't break when liability
  accounts (negative-to-net-worth) appear in the summary.

</code_context>

<specifics>
## Specific Ideas

- LOAN is a liability with enum-only support; create/edit UI treats it like a
  bank account (institution, opening balance, currency). No amortization UI in
  Phase 3.
- CC "available credit" chip = `credit_limit - current_balance`, displayed
  inline on the unified list row and prominently on the CC detail page.
- Archive toggle lives on the `/accounts` page; archived accounts are not shown
  in selectors anywhere in the app.
- Manual "Update balance" is a distinct action (separate from generic account
  edit) to make intent explicit and time-stamped via `balance_updated_at`.

</specifics>

<deferred>
## Deferred Ideas

Captured so the roadmap doesn't lose them. None is in Phase 3 scope.

- **Full loan tracking:** principal, APR, amortization schedule, payment
  breakdowns, payoff projections. Future phase.
- **Statement history table:** per-cycle rows for credit cards (foundation for
  subscription/bill detection in Phase 7 and reporting in Phase 6). Future
  phase.
- **Rewards rules engine:** category multipliers, earning rules per card,
  auto-apply rewards on transactions. Likely after Phase 4 (Transactions) and
  Phase 8 (AI Smart Entry).
- **Institution lookup table:** curated bank/issuer list with logos and
  autocomplete, possibly combined with `platforms`. Upgrade from free-text
  whenever needed.
- **Separate `/accounts/archive` page:** dedicated archive route if the inline
  toggle proves insufficient.
- **Un-archive confirmation UX:** no protection in v1; user can flip freely.
- **Balance reconciliation tool:** in Phase 4 or later, surface a UI for users
  to compare statement totals against computed running balance and fix
  discrepancies.
- **Bank sync via API (Plaid, etc.):** explicitly out of scope per PROJECT.md.

### Reviewed Todos (not folded)

None — no pending todos matched this phase.

</deferred>

---

_Phase: 03-bank-accounts-credit-cards_ _Context gathered: 2026-04-25_
