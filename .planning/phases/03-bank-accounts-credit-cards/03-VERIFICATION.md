---
phase: 03-bank-accounts-credit-cards
verified: 2026-04-25T08:49:51Z
status: gaps_found
score:
  4/5 ROADMAP success criteria verified; 6/7 ACCT requirements satisfied (1
  partial via H-01)
must_haves_total: 5
must_haves_verified: 4
requirements_total: 7
requirements_verified: 6
overrides_applied: 0
re_verification: false
gaps:
  - truth:
      "User can edit and archive accounts without losing any historical
      transaction data (ROADMAP SC-4 / ACCT-04)"
    status: partial
    reason: |
      H-01 (code-review) confirmed in disk inspection: AccountEditModal does not seed any of the 9 new Phase 3 fields into form `defaultValues`. Combined with the form's superRefine ("Opening balance is required for this account type"), opening the edit dialog on any existing CHECKING / SAVINGS / CREDIT_CARD / LOAN account will block submit until the user manually re-enters openingBalance, and additionally creditLimit + statementCycleDay for credit cards. Historical data is not lost (DB persists), but the edit user-flow is broken for the four new account types.
    artifacts:
      - path: "apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx"
        issue:
          "defaultValues object (lines 17-29) omits institution, openingBalance,
          creditLimit, statementCycleDay, statementBalance, minimumPayment,
          statementDueDate, rewardPointsBalance, cashbackBalance"
      - path: "apps/frontend/src/lib/schemas.ts"
        issue:
          "newAccountSchema.superRefine at line 157 requires openingBalance for
          bank/CC/LOAN — combined with the modal omission this blocks all edits"
    missing:
      - "Forward all 9 Phase 3 fields into AccountEditModal `defaultValues` from
        the `account` prop"
      - "Add an accounts-page.test.tsx case that opens the edit dialog on a CC
        account and asserts creditLimit + openingBalance are pre-filled
        (negative test for H-01 regression)"
human_verification:
  - test:
      "E2E spec e2e/11-accounts.spec.ts (compile-only verified) — full create
      CHECKING / create CREDIT_CARD with required CC fields / Update balance via
      modal / archive / Show-archived toggle"
    expected: "All 6 tests pass against a fresh PG database"
    why_human:
      "Port 8088 occupied by OrbStack on the executor host blocked dev server
      boot. Run on a clean host with `node scripts/prep-e2e.mjs && pnpm run
      dev:web && npx playwright test e2e/11-accounts.spec.ts`"
  - test:
      "PG integration tests with DATABASE_URL (cargo test -p
      whaleit-storage-postgres accounts)"
    expected:
      "5 round-trip tests + migration up/down test all pass against a real PG
      instance"
    why_human:
      "DATABASE_URL not set in verification environment; tests gracefully skip
      without it. Compile-time AccountDB::as_select() validation passes against
      schema.rs."
  - test:
      "Visual fidelity of Available credit chip + CC sections vs UI-SPEC §1 + §6"
    expected:
      "Chip placement, color tier, hover state, and progress bar match spec"
    why_human: "Pixel-level visual checks not in scope of unit/E2E"
  - test:
      "Manual: open account-edit dialog on existing CHECKING and CREDIT_CARD
      accounts"
    expected:
      "Form pre-fills openingBalance, institution, and CC fields; submit
      succeeds without re-entering data"
    why_human:
      "Confirms H-01 severity in real UI; structural inspection already confirms
      the bug exists in source"
overrides: []
---

# Phase 3: Bank Accounts & Credit Cards — Verification Report

**Phase Goal:** Users can manage checking, savings, and credit card accounts
alongside existing investment accounts **Verified:** 2026-04-25T08:49:51Z
**Status:** gaps_found (1 partial truth, 1 ACCT requirement degraded)
**Re-verification:** No — initial verification

## Goal Achievement Summary

Phase 3 lands the bank-accounts and credit-card domain end-to-end: PG migration
with 11 nullable columns (NUMERIC(20,8) for money), Rust core extension (4 new
AccountType leaves + AccountKind helper), DTO/service/repo/zod/component layers,
and a Playwright spec. Goal is **substantially achieved at the data layer and
create flow**, but **edit flow is broken (H-01)** for the four new account types
until AccountEditModal seeds Phase 3 fields. This blocks ACCT-04 cleanly
satisfying its success criterion.

| ACCT-\* | Goal                                                                               | Status                                                                                        |
| ------- | ---------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| ACCT-01 | User can create CHECKING/SAVINGS with name, institution, currency, opening balance | ✓ SATISFIED                                                                                   |
| ACCT-02 | User can create CREDIT_CARD with credit limit and statement cycle day              | ✓ SATISFIED                                                                                   |
| ACCT-03 | All account types appear in unified list with current balances                     | ✓ SATISFIED                                                                                   |
| ACCT-04 | User can edit and archive accounts                                                 | ⚠️ PARTIAL — archive works; edit blocked by H-01 for the 4 new types                          |
| ACCT-05 | CC shows balance, available credit, utilization, due date                          | ✓ SATISFIED                                                                                   |
| ACCT-06 | User can record statement balance, minimum payment, due date                       | ✓ SATISFIED (data round-trip; first record via create works; subsequent edit blocked by H-01) |
| ACCT-07 | User can track reward points/cashback per CC                                       | ✓ SATISFIED (same caveat as ACCT-06)                                                          |

## Observable Truths (ROADMAP Success Criteria)

| #   | Truth                                                                                                         | Status          | Evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| --- | ------------------------------------------------------------------------------------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | User can create and manage bank accounts (CHECKING/SAVINGS) with name, institution, currency, opening balance | ✓ VERIFIED      | `account-form.tsx` renders Institution + Opening balance inputs gated on type; `newAccountSchema` enforces openingBalance for bank/CC/LOAN; `crates/core/src/accounts/accounts_model.rs::NewAccount::validate` enforces server-side; PG round-trip tests landed in `repository_tests.rs`                                                                                                                                                                                                                                                         |
| 2   | User can create credit card accounts with limit, utilization, statement cycle                                 | ✓ VERIFIED      | `account-form.tsx` renders 7 CC-only FormFields when CREDIT_CARD selected; `creditLimit > 0` and `statementCycleDay 1..31` enforced in both schemas.ts (zod superRefine) and accounts_model.rs (Rust validate); migration adds CHECK constraints                                                                                                                                                                                                                                                                                                 |
| 3   | All account types (bank, CC, investment) appear in unified list with current balances                         | ✓ VERIFIED      | `accounts-page.tsx:147` uses `account.group ?? defaultGroupForAccountType(acc.accountType)`; `account-item.tsx:151-155` renders Available credit chip on CC rows; `accounts-page.test.tsx` covers group-by + archive toggle                                                                                                                                                                                                                                                                                                                      |
| 4   | User can edit and archive accounts without losing transaction data                                            | ✗ FAILED (H-01) | Archive flow works (accounts-page.tsx:104, useAccounts hook respects includeArchived). **Edit flow broken**: `account-edit-modal.tsx:17-29` does not seed institution/openingBalance/creditLimit/statementCycleDay/statementBalance/minimumPayment/statementDueDate/rewardPointsBalance/cashbackBalance. Combined with `schemas.ts:157` "Opening balance is required for this account type", the edit dialog refuses to submit on any existing bank/CC/LOAN account. Historical data is preserved in DB, but the form-level UX prevents re-edit. |
| 5   | CC accounts show outstanding balance, available credit, statement details, rewards                            | ✓ VERIFIED      | `account-page.tsx:659-822` renders Credit overview (balance + available credit + utilization Progress + tier-colored), Statement snapshot (statement_balance + minimum_payment + due_date), and Rewards (points + cashback) sections gated on `accountType === CREDIT_CARD`. credit-helpers.ts derives availableCredit + utilizationPercent + utilizationTier with edge-case handling (14 unit tests pass).                                                                                                                                      |

**Score: 4/5 ROADMAP success criteria verified.**

## Required Artifacts

### Schema & Storage

| Artifact                                                                                        | Expected                                    | Status     | Details                                                                                                       |
| ----------------------------------------------------------------------------------------------- | ------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------- |
| `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql`   | 11 ADD COLUMN clauses + 2 CHECK constraints | ✓ VERIFIED | All 11 columns present; CHECKs on statement_cycle_day (1..=31) and reward_points_balance (>= 0) inline        |
| `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/down.sql` | 11 DROP COLUMN IF EXISTS                    | ✓ VERIFIED | All 11 reversible                                                                                             |
| `crates/storage-postgres/src/schema.rs` accounts block                                          | DSL with 11 new Nullable columns            | ✓ VERIFIED | Lines 30-40 contain all 11 columns with correct sql_types (Numeric, SmallInt, Date, Integer, Text, Timestamp) |
| `Cargo.toml` rust_decimal `db-diesel2-postgres` feature                                         | Enables ToSql/FromSql for Numeric           | ✓ VERIFIED | Line 48 contains the feature                                                                                  |

### Rust Core Domain

| Artifact                                           | Expected                                                                                                 | Status     | Details                                                                                                                                                                                                           |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/core/src/accounts/accounts_constants.rs`   | AccountType constants + AccountKind enum + account_kind helper + extended default_group_for_account_type | ✓ VERIFIED | `account_types::{CHECKING,SAVINGS,CREDIT_CARD,LOAN}` constants; `AccountKind { Asset, Liability, Investment }`; `account_kind` mapping; `default_group_for_account_type` returns "Banking"/"Credit Cards"/"Loans" |
| `crates/core/src/accounts/accounts_model.rs`       | 11 fields on Account/NewAccount/AccountUpdate + validate rules                                           | ✓ VERIFIED | All 11 fields present on all 3 structs; `NewAccount::validate` enforces D-06 + D-11 + cycle_day; `AccountUpdate::validate` enforces D-06 null-rule (but see H-02 below for stale CC data on type-transition)      |
| `crates/core/src/accounts/accounts_model_tests.rs` | Wave 0 tests                                                                                             | ✓ VERIFIED | `cargo test -p whaleit-core accounts::` → 16 passed (validated by verifier)                                                                                                                                       |

### Storage-Postgres Layer

| Artifact                                                   | Expected                          | Status                    | Details                                                                                                                                       |
| ---------------------------------------------------------- | --------------------------------- | ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/storage-postgres/src/accounts/model.rs`            | AccountDB + 3 From impls extended | ✓ VERIFIED                | All 11 fields on AccountDB; From<AccountDB> for Account, From<NewAccount> for AccountDB, From<AccountUpdate> for AccountDB all copy 11 fields |
| `crates/storage-postgres/src/accounts/repository_tests.rs` | PG round-trip tests               | ✓ VERIFIED (compile-only) | File exists; tests gracefully skip without DATABASE_URL. `cargo check -p whaleit-storage-postgres --tests` clean                              |
| `crates/storage-postgres/src/accounts/migration_tests.rs`  | Migration smoke test              | ✓ VERIFIED (compile-only) | File exists; same skip-on-no-DB pattern                                                                                                       |

### Server DTO + Service

| Artifact                                             | Expected                                                         | Status                        | Details                                                                                                                                                                                                                                                                                                                                          |
| ---------------------------------------------------- | ---------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `apps/server/src/models.rs`                          | DTO Account/NewAccount/AccountUpdate with 11 fields + From impls | ✓ VERIFIED (with H-03 caveat) | All 3 DTOs carry 11 new fields; From impls copy them. **H-03 caveat:** `AccountUpdate.balance_updated_at` at line 189 is client-writable                                                                                                                                                                                                         |
| `crates/core/src/accounts/accounts_service.rs`       | balance_updated_at auto-bump on current_balance change (D-12)    | ⚠️ PARTIAL                    | Lines 89-97 conditionally override only when `current_balance.is_some() && current_balance != existing.current_balance`. **H-03 confirmed:** if client sends `balanceUpdatedAt: <past>` with `currentBalance: undefined`, the server passes the client value through. D-12 invariant ("server is source of truth for last-touched") is breached. |
| `crates/core/src/accounts/accounts_service_tests.rs` | Wave 0 service-level test for auto-bump                          | ✓ VERIFIED                    | File exists; auto-bump test passes (`cargo test -p whaleit-core accounts::` 16 passed)                                                                                                                                                                                                                                                           |

### Frontend

| Artifact                                                             | Expected                                                             | Status     | Details                                                                                                                                                                              |
| -------------------------------------------------------------------- | -------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `apps/frontend/src/lib/constants.ts`                                 | AccountType + AccountKind + accountKind + defaultGroupForAccountType | ✓ VERIFIED | Lines 44-119 contain extended AccountType (7 variants), AccountKind enum, accountKind helper with exhaustive `_exhaustive: never` check, defaultGroupForAccountType with all 7 cases |
| `apps/frontend/src/lib/types/account.ts`                             | Account interface with 11 new fields + currentBalance rename         | ✓ VERIFIED | Lines 12 (currentBalance), 26-35 (institution + 9 Phase 3 fields). Legacy `balance: number` removed                                                                                  |
| `apps/frontend/src/lib/schemas.ts`                                   | newAccountSchema with CC-gated superRefine                           | ✓ VERIFIED | Lines 110-164: superRefine enforces D-06 (CC fields null on non-CC) + D-11 (openingBalance required for bank/CC/LOAN) + creditLimit/statementCycleDay required for CC                |
| `apps/frontend/src/lib/constants.test.ts`                            | accountKind + group tests                                            | ✓ VERIFIED | Test file exists; 7 tests pass                                                                                                                                                       |
| `apps/frontend/src/lib/schemas.test.ts`                              | CC-gated zod tests                                                   | ✓ VERIFIED | 12 tests pass                                                                                                                                                                        |
| `apps/frontend/src/components/account-selector.tsx` + mobile variant | Hides archived by default                                            | ✓ VERIFIED | `useAccounts({ filterActive, includeArchived: false })` (line 131-134) — archived filtered at the hook layer via `getAccounts(includeArchived=false)` IPC arg                        |

### Frontend UI Glue

| Artifact                                                                        | Expected                                                                             | Status     | Details                                                                                                                                             |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `apps/frontend/src/pages/settings/accounts/credit-helpers.ts` + .test.ts        | availableCredit + utilizationPercent + utilizationTier helpers                       | ✓ VERIFIED | Helpers correctly handle missing limit / division by zero / range clamp; 14 unit tests pass                                                         |
| `apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx` | Update balance modal wraps update_account                                            | ✓ VERIFIED | Wires `updateAccountMutation.mutate` (line 39); only sets currentBalance + balance_updated_at NOT included → server can detect the diff             |
| `apps/frontend/src/pages/settings/accounts/components/account-form.tsx`         | Dynamic CC sections + Institution + Opening balance inputs                           | ✓ VERIFIED | Lines 97-102: requiresInstitution + requiresOpeningBalance booleans; lines 225-308 conditionally render Institution / Opening balance / 7 CC fields |
| `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`   | defaultValues seed all 9 Phase 3 fields                                              | ✗ STUB     | **H-01:** lines 17-29 omit all 9 Phase 3 fields, breaking edit                                                                                      |
| `apps/frontend/src/pages/settings/accounts/components/account-item.tsx`         | Available credit chip on CC rows                                                     | ✓ VERIFIED | Line 151-155: imports availableCredit from credit-helpers; renders chip when accountType === CREDIT_CARD                                            |
| `apps/frontend/src/pages/settings/accounts/accounts-page.tsx`                   | group-by + Show archived Switch                                                      | ✓ VERIFIED | Lines 142-147 group-by `defaultGroupForAccountType`; lines 243-251 Show archived Switch                                                             |
| `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`              | group-by + archive toggle coverage                                                   | ✓ VERIFIED | 5 tests pass                                                                                                                                        |
| `apps/frontend/src/pages/account/account-page.tsx`                              | CC sections (Credit overview, Statement snapshot, Rewards) + Balance card for non-CC | ✓ VERIFIED | Lines 659-822 render all 3 CC sections gated on isCreditCard; investment-only modules hidden for CHECKING/SAVINGS/LOAN                              |

### E2E

| Artifact                  | Expected                                          | Status   | Details                                                                                                                                                                                                                                           |
| ------------------------- | ------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `e2e/11-accounts.spec.ts` | 6 Playwright tests covering ACCT-01..07 user flow | ⚠️ unrun | File exists with 6 tests (login, create CHECKING, create CREDIT_CARD, update balance, archive, show-archived). Compile-only verified — port 8088 (Axum) occupied by OrbStack on executor host blocked in-place run. Routed to human verification. |

## Key Link Verification

| From                                     | To                               | Via                                                | Status      | Details                                                                                                                                         |
| ---------------------------------------- | -------------------------------- | -------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `Cargo.toml rust_decimal`                | `diesel::sql_types::Numeric`     | `db-diesel2-postgres` feature                      | ✓ WIRED     | Feature flag in line 48                                                                                                                         |
| `schema.rs accounts block`               | `migrations/.../up.sql`          | manual schema regen                                | ✓ WIRED     | All 11 columns match types between SQL and DSL                                                                                                  |
| `From<NewAccount> for AccountDB`         | `repository.rs::create()`        | `domain.into()`                                    | ✓ WIRED     | repository.rs unchanged; insert path uses From impl                                                                                             |
| `apps/server/src/api/accounts.rs`        | `apps/server/src/models.rs` DTOs | `Json<NewAccount>` deserialization                 | ✓ WIRED     | DTOs serde-deserialize from camelCase JSON                                                                                                      |
| `AccountService::update_account`         | `Account::balance_updated_at`    | service-layer mutation                             | ⚠️ PARTIAL  | Auto-stamp fires only when current_balance changes (H-03 — client can backdate when sending balance_updated_at without changing currentBalance) |
| `AccountType union (TS)`                 | `Record<AccountType, ...>` sites | exhaustiveness                                     | ✓ WIRED     | app-launcher, account-page, account-item, account-form all enumerate 7 keys; type-check exits 0                                                 |
| `account-edit-modal`                     | `AccountForm defaultValues`      | spread of 9 Phase 3 fields                         | ✗ NOT_WIRED | **H-01** — defaultValues object omits all 9 fields                                                                                              |
| `account-form CREDIT_CARD branch`        | `newAccountSchema.superRefine`   | react-hook-form + zod                              | ✓ WIRED     | Verified by 12 schemas.test.ts cases                                                                                                            |
| `Update balance modal submit`            | `updateAccount` adapter          | `useAccountMutations.updateAccountMutation.mutate` | ✓ WIRED     | Spread of full account into payload, only currentBalance changed                                                                                |
| `account-item.tsx Available credit chip` | `credit-helpers.availableCredit` | named import                                       | ✓ WIRED     | Import + call at line 8/151                                                                                                                     |
| `account-page.tsx CC sections`           | `update-balance-modal`           | `<UpdateBalanceModal>`                             | ✓ WIRED     | Lines 79/895                                                                                                                                    |
| `e2e/11-accounts.spec.ts`                | `/settings/accounts`             | `page.goto(BASE_URL + "/settings/accounts")`       | ✓ WIRED     | Spec correctly targets D-15 route                                                                                                               |

## Data-Flow Trace (Level 4)

| Artifact                                 | Data Variable                                   | Source                                                                                                                      | Produces Real Data                             | Status               |
| ---------------------------------------- | ----------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | -------------------- |
| `accounts-page.tsx`                      | `accounts`                                      | `useAccounts({ filterActive: false, includeArchived: showArchived })` → `getAccounts(includeArchived)` → IPC `get_accounts` | Yes (real DB query via PG repository `list()`) | ✓ FLOWING            |
| `account-page.tsx` (CC sections)         | `account`                                       | `useAccount(id)` → `getAccount(id)` IPC                                                                                     | Yes                                            | ✓ FLOWING            |
| `account-item.tsx` Available credit chip | `account.creditLimit`, `account.currentBalance` | Account interface (real PG NUMERIC columns round-tripped via Decimal)                                                       | Yes                                            | ✓ FLOWING            |
| `update-balance-modal.tsx`               | `account.currentBalance` + new value            | Real `update_account` mutation invalidates accounts cache                                                                   | Yes                                            | ✓ FLOWING            |
| `account-edit-modal.tsx`                 | `defaultValues` (Phase 3 fields)                | **none** — fields omitted from defaultValues                                                                                | **No — disconnected**                          | ✗ HOLLOW_PROP (H-01) |

## Behavioral Spot-Checks

| Behavior                                           | Command                                                                                                                | Result                             | Status         |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------- | -------------- |
| Rust core tests pass                               | `cargo test -p whaleit-core accounts::`                                                                                | 16 passed, 796 filtered out        | ✓ PASS         |
| storage-postgres compiles with all 11 fields wired | `cargo check -p whaleit-storage-postgres --tests`                                                                      | Clean (6 crates compiled in 28s)   | ✓ PASS         |
| server compiles with extended DTOs                 | `cargo check -p whaleit-server`                                                                                        | Clean (cached, 0.80s)              | ✓ PASS         |
| Frontend type-check                                | `pnpm --filter frontend type-check` (`tsc --noEmit`)                                                                   | Exit 0                             | ✓ PASS         |
| Frontend Phase 3 unit tests                        | `pnpm --filter frontend test -- --run constants.test.ts schemas.test.ts credit-helpers.test.ts accounts-page.test.tsx` | 537/537 tests pass across 45 files | ✓ PASS         |
| PG integration round-trips                         | `cargo test -p whaleit-storage-postgres accounts` (with DATABASE_URL)                                                  | Skipped without DATABASE_URL       | ? SKIP (human) |
| E2E user flow                                      | `npx playwright test e2e/11-accounts.spec.ts`                                                                          | Skipped — port 8088 conflict       | ? SKIP (human) |

## Requirements Coverage

| Requirement | Source Plans                                                   | Description                                                                                     | Status            | Evidence                                                                                                                                                                                                                                                                               |
| ----------- | -------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ACCT-01     | 03-01, 03-02, 03-03, 03-04, 03-05, 03-07, 03-08                | Create bank accounts (checking, savings) with name, institution, currency, opening balance      | ✓ SATISFIED       | Migration adds columns; `NewAccount::validate` enforces openingBalance for bank/LOAN; account-form renders Institution + Opening balance inputs; e2e Test 2 covers create CHECKING                                                                                                     |
| ACCT-02     | 03-01, 03-02, 03-03, 03-04, 03-05, 03-07, 03-08                | Create credit card accounts with name, institution, currency, credit limit, statement cycle day | ✓ SATISFIED       | Migration adds CC columns + CHECK constraints; both Rust and zod superRefine enforce creditLimit > 0 + cycle_day 1..31; account-form renders 7 CC FormFields when CREDIT_CARD selected; e2e Test 3 covers create CREDIT_CARD                                                           |
| ACCT-03     | 03-02, 03-05, 03-06, 03-07b, 03-08                             | View all accounts in unified list with current balances                                         | ✓ SATISFIED       | accounts-page.tsx group-by axis using `defaultGroupForAccountType` (Banking/Credit Cards/Loans/Investments/Cash/Crypto); accountKind helper exhaustive; accounts-page.test.tsx covers group-by ordering                                                                                |
| ACCT-04     | 03-03, 03-04, 03-06, 03-07b, 03-08                             | Edit and archive accounts                                                                       | ⚠️ PARTIAL — H-01 | Archive flow: ✓ verified (useAccounts hook respects includeArchived; accounts-page Show archived Switch reveals; selectors hide). **Edit flow: BROKEN** for CHECKING/SAVINGS/CREDIT_CARD/LOAN — AccountEditModal omits 9 Phase 3 fields from defaultValues; superRefine refuses submit |
| ACCT-05     | 03-01, 03-02, 03-03, 03-04, 03-05, 03-06, 03-07, 03-07b, 03-08 | CC tracking shows outstanding balance, available credit, utilization, due date                  | ✓ SATISFIED       | account-page.tsx Credit overview section renders balance + available credit + utilization Progress + tier color + due date in Statement snapshot; credit-helpers.ts derivation tested (14 cases)                                                                                       |
| ACCT-06     | 03-01, 03-03, 03-04, 03-08                                     | Record statement balance, minimum payment, due date                                             | ✓ SATISFIED       | All 3 fields land as NUMERIC(20,8)/DATE columns; round-trip test in repository_tests.rs::test_cc_statement_roundtrip; account-form fields land via Plan 03-07; account-page.tsx Statement snapshot section renders them                                                                |
| ACCT-07     | 03-01, 03-03, 03-04, 03-08                                     | Track reward points/cashback per CC                                                             | ✓ SATISFIED       | reward_points_balance INTEGER + cashback_balance NUMERIC; round-trip test in repository_tests.rs::test_cc_rewards_roundtrip; account-page.tsx Rewards section renders both                                                                                                             |

**Score: 6/7 requirements fully satisfied; ACCT-04 partial (archive ✓, edit
✗).**

No orphaned requirements: every ACCT-01..07 is claimed by at least one plan's
`requirements_addressed` field.

## Anti-Patterns Found

Scanned files modified by Phase 3 plans. Findings:

| File                     | Line                    | Pattern                                                                               | Severity          | Impact                                                                                                                                                                                                                      |
| ------------------------ | ----------------------- | ------------------------------------------------------------------------------------- | ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `account-form.tsx`       | 183, 196, 214, 234, 308 | `placeholder="..."`                                                                   | ℹ️ Info           | False positive — these are `<Input placeholder>` UX text, not stubs                                                                                                                                                         |
| `accounts-page.tsx`      | 224                     | `placeholder="Search accounts..."`                                                    | ℹ️ Info           | False positive — Search UI placeholder                                                                                                                                                                                      |
| `accounts_service.rs`    | 96                      | `chrono::Utc::now().naive_utc()` only set when `current_balance` changes              | 🛑 Blocker (H-03) | D-12 invariant violation — client can supply backdated `balance_updated_at` when not changing balance. Documented as code-review High; root cause is DTO accepting a server-only field                                      |
| `account-edit-modal.tsx` | 17-29                   | `defaultValues` omits Phase 3 fields                                                  | 🛑 Blocker (H-01) | Edit flow broken for the 4 new account types — directly degrades ACCT-04                                                                                                                                                    |
| `accounts_model.rs`      | 250-265                 | `AccountUpdate::validate` does not require explicit None for CC fields on type-change | ⚠️ Warning (H-02) | Diesel `AsChangeset` skip-on-None means stale CC values persist when CREDIT_CARD → CHECKING. Latent data integrity issue; D-06 invariant violated. Not currently surfaced via UI flow because edit is already broken (H-01) |

No TODO/FIXME/XXX/HACK or empty-stub returns found in Phase 3 files.

## Code-Review Severity Triage

| Issue                                   | Severity | Blocks Goal?                                          | Phase 3 Verdict                                       |
| --------------------------------------- | -------- | ----------------------------------------------------- | ----------------------------------------------------- |
| H-01 Edit modal drops Phase 3 fields    | High     | **Yes — ACCT-04 partial**                             | GAP — must close in same phase or accept overrides    |
| H-02 Stale CC fields on type transition | High     | No — not exercised by UI given H-01; latent integrity | Carry as known issue (could be deferred to follow-up) |
| H-03 Client-supplied balance_updated_at | High     | No — not user-visible v1                              | Carry as known issue; doc D-12 invariant breach       |
| M-01..M-07 Various                      | Medium   | No                                                    | Tracked in 03-REVIEW.md                               |
| L-01..L-08 Various                      | Low      | No                                                    | Tracked in 03-REVIEW.md                               |

## Gaps Summary

**1 blocking gap** found that prevents the phase goal from being fully achieved:

- **H-01: AccountEditModal field omission** — directly breaks edit flow for
  CHECKING / SAVINGS / CREDIT_CARD / LOAN. ROADMAP SC-4 ("User can edit and
  archive accounts without losing any historical transaction data") is partially
  failed at the form layer even though the database preserves history. The fix
  is mechanical (forward 9 fields to defaultValues) and contained to one file
  plus a regression test.

**2 known issues carry-forward** (not goal-blocking, but flagged for follow-up):

- **H-02 Stale CC fields on type transition** — D-06 invariant breach when
  changing CREDIT_CARD → non-CC. Not user-reachable today because H-01 blocks
  edit; will surface once H-01 closes. Recommend bundling H-02 fix with H-01
  closure plan.
- **H-03 Client-supplied balance_updated_at** — D-12 invariant breach. Server
  accepts the field; auto-stamp only overwrites on currentBalance change.
  Recommend dropping `balance_updated_at` from inbound DTO.

## Human Verification Required

See frontmatter `human_verification:` section. Key items:

### 1. E2E Spec Run (Plan 03-08 carry-over)

**Test:** `npx playwright test e2e/11-accounts.spec.ts` against a fresh PG
database **Expected:** 6/6 tests pass (login, create CHECKING, create
CREDIT_CARD with required CC fields, update balance via modal, archive,
show-archived toggle) **Why human:** Port 8088 occupied by OrbStack on executor
host. Run with:
`node scripts/prep-e2e.mjs && pnpm run dev:web && ./scripts/wait-for-both-servers-to-be-ready.sh && npx playwright test e2e/11-accounts.spec.ts`

### 2. PG Integration Tests

**Test:**
`DATABASE_URL=postgres://... cargo test -p whaleit-storage-postgres accounts`
**Expected:** 5 round-trip tests + migration up/down test all pass **Why
human:** DATABASE_URL not set in verification environment; tests gracefully skip
without it

### 3. Visual Fidelity (UI-SPEC §1 + §6)

**Test:** Open `/settings/accounts` after creating a CC; verify Available credit
chip placement, color tier, hover state, Progress bar utilization color match
UI-SPEC **Expected:** Pixel-level visual match **Why human:** Pixel-level visual
checks not in scope of unit/E2E tests

### 4. Edit-Flow Manual Confirmation (H-01 severity)

**Test:** Open the account-edit dialog on an existing CHECKING account, then on
an existing CREDIT_CARD account **Expected:** Form pre-fills openingBalance,
institution; CC form pre-fills creditLimit + statementCycleDay; submit succeeds
without re-entering data **Why human:** Confirms H-01 user-visible severity
(structural inspection has already confirmed the bug exists in source)

---

_Verified: 2026-04-25T08:49:51Z_ _Verifier: Claude (gsd-verifier)_
