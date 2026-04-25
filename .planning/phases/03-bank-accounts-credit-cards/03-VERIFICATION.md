---
phase: 03-bank-accounts-credit-cards
verified: 2026-04-25T18:30:00Z
status: human_needed
score: 5/5 ROADMAP success criteria verified; 7/7 ACCT requirements satisfied
must_haves_total: 5
must_haves_verified: 5
requirements_total: 7
requirements_verified: 7
overrides_applied: 0
re_verification: true
re_verification_previous_status: gaps_found
re_verification_previous_score: "4/5 (1 partial)"
re_verification_gaps_closed:
  - "H-01: AccountEditModal defaultValues now seeds all 9 Phase 3 fields from
    account prop (account-edit-modal.tsx lines 29-38)"
  - "H-02: D-06 type-transition invariant restored — service clears all 7 CC
    columns on CREDIT_CARD→non-CC transition (accounts_service.rs lines 96-107)"
  - "H-03: D-12 server-only timestamp invariant restored — From impls hard-code
    balance_updated_at: None; service else-branch provides defense-in-depth
    (models.rs lines 156, 222; accounts_service.rs line 123)"
re_verification_regressions: []
human_verification:
  - test:
      "E2E spec e2e/11-accounts.spec.ts — full create CHECKING / create
      CREDIT_CARD with required CC fields / Update balance via modal / archive /
      Show-archived toggle"
    expected: "All 6 tests pass against a fresh PG database"
    why_human:
      "Port 8088 occupied by OrbStack on the executor host blocked dev server
      boot. Run on a clean host: node scripts/prep-e2e.mjs && pnpm run dev:web
      && npx playwright test e2e/11-accounts.spec.ts"
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
      "UAT golden path: create CHECKING + SAVINGS + CREDIT_CARD accounts, edit
      each (verify pre-fill), archive each, observe Available credit chip on CC
      row"
    expected:
      "All create/edit/archive actions succeed; CHECKING/SAVINGS pre-fill
      institution + openingBalance; CREDIT_CARD pre-fills all 9 Phase 3 fields;
      Available credit chip visible on CC row; archived accounts disappear from
      list by default"
    why_human:
      "Confirms H-01 regression closure in live UI; structural + unit assertions
      confirm source-level correctness but cannot substitute for real browser
      interaction"
overrides: []
---

# Phase 3: Bank Accounts & Credit Cards — Re-Verification Report

**Phase Goal:** Users can manage checking, savings, and credit card accounts
alongside existing investment accounts **Verified:** 2026-04-25T18:30:00Z
**Status:** human_needed — all automated checks pass; 1 UAT golden path +
existing E2E/PG items pending human **Re-verification:** Yes — after gap closure
plans 03-09 (H-01) and 03-10 (H-02, H-03)

All three blocking gaps from the prior verification are closed. ROADMAP SC-4
("User can edit and archive accounts without losing any historical transaction
data") is now fully verified at both source and test level. Score moves from 4/5
(1 partial) to 5/5.

## Re-Verification Summary

| Gap                                                | Closed By                              | Verification Method                                                                | Result |
| -------------------------------------------------- | -------------------------------------- | ---------------------------------------------------------------------------------- | ------ |
| H-01: AccountEditModal omits Phase 3 defaultValues | 03-09-gap (commits d216f7c3, 5b2d5d26) | Direct code inspection + 2 regression tests in accounts-page.test.tsx              | CLOSED |
| H-02: Stale CC fields on type transition (D-06)    | 03-10-gap (commits d85f1aec, 8528d463) | Direct code inspection + test_update_clears_cc_fields_on_type_transition_out_of_cc | CLOSED |
| H-03: Client-supplied balance_updated_at (D-12)    | 03-10-gap (commits d85f1aec, 8528d463) | Direct code inspection + test_update_ignores_client_supplied_balance_updated_at    | CLOSED |

## Observable Truths (ROADMAP Success Criteria)

| #   | Truth                                                                                                         | Status     | Evidence                                                                                                                                                                                                                                                                                                                                                |
| --- | ------------------------------------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | User can create and manage bank accounts (CHECKING/SAVINGS) with name, institution, currency, opening balance | ✓ VERIFIED | account-form.tsx renders Institution + Opening balance inputs gated on type; newAccountSchema enforces openingBalance for bank/CC/LOAN; NewAccount::validate enforces server-side; PG round-trip tests in repository_tests.rs                                                                                                                           |
| 2   | User can create credit card accounts with limit, utilization percentage, and statement cycle tracking         | ✓ VERIFIED | account-form.tsx renders 7 CC-only FormFields when CREDIT_CARD selected; creditLimit > 0 and statementCycleDay 1..31 enforced in both schemas.ts superRefine and accounts_model.rs validate                                                                                                                                                             |
| 3   | All account types (bank, CC, investment) appear in unified account list with current balances                 | ✓ VERIFIED | accounts-page.tsx group-by using defaultGroupForAccountType; account-item.tsx renders Available credit chip on CC rows; accounts-page.test.tsx covers all 6 groups + archive toggle                                                                                                                                                                     |
| 4   | User can edit and archive accounts without losing any historical transaction data                             | ✓ VERIFIED | **H-01 CLOSED:** account-edit-modal.tsx lines 29-38 forward all 9 Phase 3 fields from account prop. **H-02 CLOSED:** service clears CC fields on type transition. Archive flow already verified. Regression tests: accounts-page.test.tsx lines 410-456 (2 new H-01 regression cases pass). 539/539 frontend tests pass (orchestrator post-merge gate). |
| 5   | CC accounts show outstanding balance, available credit, statement details, and reward points/cashback         | ✓ VERIFIED | account-page.tsx lines 659-822 render Credit overview, Statement snapshot, and Rewards sections gated on isCreditCard; credit-helpers.ts derivation tested (14 unit tests pass)                                                                                                                                                                         |

**Score: 5/5 ROADMAP success criteria verified.**

## Gap Closure Detail

### H-01: AccountEditModal defaultValues (CLOSED)

**What changed:**
`apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`

Lines 29-38 now read:

```
institution: account?.institution,
openingBalance: account?.openingBalance,
creditLimit: account?.creditLimit,
statementCycleDay: account?.statementCycleDay,
statementBalance: account?.statementBalance,
minimumPayment: account?.minimumPayment,
statementDueDate: account?.statementDueDate,
rewardPointsBalance: account?.rewardPointsBalance,
cashbackBalance: account?.cashbackBalance,
```

`balanceUpdatedAt` intentionally excluded — server-only per D-12.

**Regression tests added:** `accounts-page.test.tsx` lines 410-456, describe
block `"AccountEditModal pre-fill regression (H-01)"`:

- `"pre-fills CC fields from the account prop"` — asserts institution,
  openingBalance, creditLimit, statementCycleDay all pre-fill from CC account
  fixture
- `"pre-fills CHECKING institution + openingBalance from the account prop"` —
  asserts institution + openingBalance pre-fill from CHECKING fixture

Both tests pass in the 539/539 orchestrator run.

### H-02: D-06 Type-Transition CC Field Clearing (CLOSED)

**What changed:** `crates/core/src/accounts/accounts_service.rs` lines 91-107

Service detects
`existing.account_type == CREDIT_CARD && account_update.account_type != CREDIT_CARD`
and forces all 7 CC columns (`credit_limit`, `statement_cycle_day`,
`statement_balance`, `minimum_payment`, `statement_due_date`,
`reward_points_balance`, `cashback_balance`) to `None` on the `AccountUpdate`
before forwarding to repository. This happens BEFORE validate(), so a buggy
client sending CC fields with a non-CC type gets sanitized rather than rejected.

**Pinned by:** `test_update_clears_cc_fields_on_type_transition_out_of_cc` in
`accounts_service_tests.rs` lines 454-498 — asserts `result.is_ok()` AND all 7
CC fields `None` in the captured update. Part of the 18/18 cargo test passing
set.

### H-03: D-12 Server-Only balance_updated_at (CLOSED)

**What changed (dual seam):**

1. `apps/server/src/models.rs` line 156 (From<NewAccount>) and line 222
   (From<AccountUpdate>): both hard-code
   `balance_updated_at: None, // D-12: server-only field, client value discarded`
2. `crates/core/src/accounts/accounts_service.rs` line 123: else-branch
   unconditionally sets `account_update.balance_updated_at = None` for all
   callers that bypass the DTO seam (Tauri IPC, future MCP)

**Pinned by:**
`test_update_ignores_client_supplied_balance_updated_at_when_balance_unchanged`
in `accounts_service_tests.rs` lines 437-451 — sends backdated timestamp with
unchanged balance and asserts captured update has `balance_updated_at == None`.
Part of the 18/18 cargo test passing set.

**Note on struct field vs inbound handling:** The `AccountUpdate` struct in
`apps/server/src/models.rs` retains
`pub balance_updated_at: Option<NaiveDateTime>` as a field (line 189). This is
intentional — the struct serves both request deserialization and response
serialization. The From impl discards the client-supplied value at the
conversion boundary. This is the correct dual-seam pattern; the field presence
in the struct does not constitute a vulnerability.

## Behavioral Spot-Checks

| Behavior                                           | Command                                                               | Result                                             | Status         |
| -------------------------------------------------- | --------------------------------------------------------------------- | -------------------------------------------------- | -------------- |
| Rust core tests (Phase 3 accounts suite)           | `cargo test -p whaleit-core accounts::`                               | 18/18 passing (per orchestrator post-merge gate)   | ✓ PASS         |
| storage-postgres compiles with all 11 fields wired | `cargo check -p whaleit-storage-postgres --tests`                     | Clean (per orchestrator)                           | ✓ PASS         |
| server compiles with extended DTOs                 | `cargo check -p whaleit-server`                                       | Clean, 0 errors (per orchestrator)                 | ✓ PASS         |
| Frontend Phase 3 unit tests                        | `pnpm --filter=frontend test --run accounts-page`                     | 539/539 passing across 45 files (per orchestrator) | ✓ PASS         |
| PG integration round-trips                         | `cargo test -p whaleit-storage-postgres accounts` (with DATABASE_URL) | Skipped without DATABASE_URL                       | ? SKIP (human) |
| E2E user flow                                      | `npx playwright test e2e/11-accounts.spec.ts`                         | Skipped — port 8088 conflict                       | ? SKIP (human) |

## Required Artifacts

### Key Files Modified by Gap Plans

| Artifact                                                                      | Expected                                                 | Status     | Details                                                                                                                                                                                                             |
| ----------------------------------------------------------------------------- | -------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx` | All 9 Phase 3 fields in defaultValues                    | ✓ VERIFIED | Lines 29-38 forward institution, openingBalance, creditLimit, statementCycleDay, statementBalance, minimumPayment, statementDueDate, rewardPointsBalance, cashbackBalance from account prop                         |
| `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`            | H-01 regression tests                                    | ✓ VERIFIED | Lines 410-456: describe "AccountEditModal pre-fill regression (H-01)" with 2 test cases covering CC + CHECKING pre-fill assertions                                                                                  |
| `crates/core/src/accounts/accounts_service.rs`                                | D-06 type-transition clearing + D-12 belt-and-suspenders | ✓ VERIFIED | Lines 91-107: type_transition_out_of_cc detection + 7 CC field force-NULL; lines 119-124: D-12 else-branch discard                                                                                                  |
| `crates/core/src/accounts/accounts_service_tests.rs`                          | 5 service-level tests (3 original + 2 new)               | ✓ VERIFIED | 5 test functions present: bumps_balance_timestamp, no_bump_when_balance_unchanged, no_bump_when_no_balance_in_update, ignores_client_supplied_balance_updated_at (D-12), clears_cc_fields_on_type_transition (D-06) |
| `apps/server/src/models.rs`                                                   | From impls discard client balance_updated_at             | ✓ VERIFIED | Lines 156 and 222: `balance_updated_at: None, // D-12: server-only field, client value discarded`                                                                                                                   |

### Phase 3 Core Artifacts (unchanged from initial verification — carry-forward)

| Artifact                                                                          | Status                    |
| --------------------------------------------------------------------------------- | ------------------------- |
| PG migration (11 ADD COLUMN + 2 CHECK constraints)                                | ✓ VERIFIED                |
| schema.rs DSL with 11 new nullable columns                                        | ✓ VERIFIED                |
| accounts_model.rs: 11 fields on Account/NewAccount/AccountUpdate + validate rules | ✓ VERIFIED                |
| accounts_constants.rs: AccountType + AccountKind + helpers                        | ✓ VERIFIED                |
| storage-postgres AccountDB + 3 From impls extended                                | ✓ VERIFIED                |
| Frontend Account interface with 11 new fields                                     | ✓ VERIFIED                |
| newAccountSchema with CC-gated superRefine                                        | ✓ VERIFIED                |
| credit-helpers.ts: availableCredit + utilizationPercent + utilizationTier         | ✓ VERIFIED                |
| account-form.tsx: dynamic CC sections + Institution + Opening balance             | ✓ VERIFIED                |
| account-item.tsx: Available credit chip on CC rows                                | ✓ VERIFIED                |
| accounts-page.tsx: group-by + Show archived Switch                                | ✓ VERIFIED                |
| account-page.tsx: CC sections (Credit overview, Statement snapshot, Rewards)      | ✓ VERIFIED                |
| e2e/11-accounts.spec.ts                                                           | ✓ VERIFIED (compile-only) |

## Key Link Verification

| From                                            | To                                       | Via                                             | Status  | Details                                                                                            |
| ----------------------------------------------- | ---------------------------------------- | ----------------------------------------------- | ------- | -------------------------------------------------------------------------------------------------- |
| `account-edit-modal.tsx defaultValues`          | `AccountForm`                            | `<AccountForm defaultValues={defaultValues} />` | ✓ WIRED | **H-01 CLOSED** — all 9 Phase 3 fields in defaultValues object, line 44 passes them to AccountForm |
| `accounts_service::update_account`              | `AccountUpdate` CC fields                | `type_transition_out_of_cc` guard               | ✓ WIRED | **H-02 CLOSED** — lines 96-107 clear all 7 CC fields before repository call                        |
| `apps/server/src/models.rs From<AccountUpdate>` | `core::AccountUpdate.balance_updated_at` | `balance_updated_at: None`                      | ✓ WIRED | **H-03 CLOSED** — lines 156 + 222 hard-code None at DTO boundary                                   |
| `AccountService::update_account`                | `account_update.balance_updated_at`      | else-branch discard                             | ✓ WIRED | **H-03 defense-in-depth** — line 123 discards for all callers (Tauri IPC, future MCP)              |

## Requirements Coverage

| Requirement | Plans        | Description                                                                                       | Status      | Evidence                                                                                                                                                  |
| ----------- | ------------ | ------------------------------------------------------------------------------------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ACCT-01     | 03-01..03-08 | Create bank accounts (checking, savings) with name, institution, currency, opening balance        | ✓ SATISFIED | Migration + Rust validate + account-form institution/openingBalance fields + E2E Test 2                                                                   |
| ACCT-02     | 03-01..03-08 | Create credit card accounts with name, institution, currency, credit limit, statement cycle day   | ✓ SATISFIED | CC columns + CHECK constraints + superRefine + 7 CC form fields + E2E Test 3                                                                              |
| ACCT-03     | 03-02..03-08 | View all accounts in unified list with current balances                                           | ✓ SATISFIED | accounts-page.tsx group-by with defaultGroupForAccountType; all 6 type groups render                                                                      |
| ACCT-04     | 03-03..03-10 | Edit and archive accounts while preserving historical transaction data                            | ✓ SATISFIED | **H-01 closed (03-09):** edit modal pre-fills all fields. **H-02 closed (03-10):** type-transition sanitizes CC fields. Archive flow unchanged + working. |
| ACCT-05     | 03-01..03-08 | CC tracking shows outstanding balance, available credit, utilization %, and next payment due date | ✓ SATISFIED | account-page.tsx Credit overview section; credit-helpers.ts derivation; 14 unit tests pass                                                                |
| ACCT-06     | 03-01..03-08 | Record CC statement details including statement balance, minimum payment, and due date            | ✓ SATISFIED | PG columns + account-form fields + account-page Statement snapshot section                                                                                |
| ACCT-07     | 03-01..03-08 | Track reward points/cashback balance per CC account                                               | ✓ SATISFIED | reward_points_balance INTEGER + cashback_balance NUMERIC columns + account-page Rewards section                                                           |

**Score: 7/7 requirements fully satisfied.**

## Anti-Patterns Scan (Gap Plan Files)

Scanned files modified by 03-09 and 03-10:

| File                        | Pattern                                    | Severity | Assessment                                 |
| --------------------------- | ------------------------------------------ | -------- | ------------------------------------------ |
| `account-edit-modal.tsx`    | No TODO/FIXME/empty stubs                  | —        | Clean — all 9 fields wired                 |
| `accounts-page.test.tsx`    | No empty stubs                             | —        | Clean — 2 new regression tests substantive |
| `accounts_service.rs`       | `let type_transition_out_of_cc = ...`      | ℹ️ Info  | Correct implementation, not a stub         |
| `accounts_service_tests.rs` | No TODO/FIXME/stubs                        | —        | Clean — 5 substantive async tests          |
| `apps/server/src/models.rs` | `balance_updated_at: None` in 2 From impls | ℹ️ Info  | Correct intentional discard, not a stub    |

No blockers or warnings found in gap plan files.

## Human Verification Required

### 1. UAT Golden Path (new — confirms H-01 closure in live browser)

**Test:** In the running web app (`pnpm run dev:web`), perform the following
sequence:

1. Create a CHECKING account with institution "Wells Fargo" and opening balance
   $1,234.56
2. Create a SAVINGS account with institution "Chase" and opening balance $5,000
3. Create a CREDIT_CARD account with institution "Amex", credit limit $10,000,
   statement cycle day 15
4. Open the edit dialog on the CHECKING account — verify "Wells Fargo" and
   $1,234.56 are pre-filled; submit without changes
5. Open the edit dialog on the CREDIT_CARD account — verify institution,
   creditLimit, statementCycleDay all pre-filled; submit without changes
6. Archive each of the three accounts; verify they disappear from the list by
   default
7. Toggle "Show archived" and verify all three reappear
8. On the CC account row, verify the "Available credit" chip displays ($10,000 -
   current_balance)

**Expected:** All steps succeed; form pre-fill works for all account types;
archive/unarchive toggle works; Available credit chip renders correctly

**Why human:** Structural inspection + unit regression tests confirm
source-level correctness; live browser interaction is the only way to confirm
Radix form wiring, portal behavior, and the actual DOM state under real
conditions

### 2. E2E Spec Run (carry-forward from initial verification)

**Test:**
`node scripts/prep-e2e.mjs && pnpm run dev:web && ./scripts/wait-for-both-servers-to-be-ready.sh && npx playwright test e2e/11-accounts.spec.ts`

**Expected:** 6/6 tests pass (login, create CHECKING, create CREDIT_CARD with
required CC fields, update balance via modal, archive, show-archived toggle)

**Why human:** Port 8088 occupied by OrbStack on the executor host blocked
in-place run. Run on a clean host with no conflicting services.

### 3. PG Integration Tests (carry-forward from initial verification)

**Test:**
`DATABASE_URL=postgres://... cargo test -p whaleit-storage-postgres accounts`

**Expected:** 5 round-trip tests + migration up/down test all pass against a
real PG instance

**Why human:** DATABASE_URL not set in verification environment; tests
gracefully skip without it. Compile-time validation passes.

**Recommended enhancement:** Add
`test_update_clears_cc_columns_on_type_change_pg` to
`crates/storage-postgres/src/accounts/repository_tests.rs` to assert at the DB
level that all 7 CC columns are NULL after a type transition — this would close
the gap between the service-layer contract (verified by unit test) and actual PG
row state.

### 4. Visual Fidelity (carry-forward from initial verification)

**Test:** Open `/settings/accounts` after creating a CC account. Verify
Available credit chip placement, color tier (green/yellow/red), hover state, and
Progress bar utilization color match UI-SPEC §1 + §6.

**Expected:** Pixel-level visual match with spec.

**Why human:** Pixel-level visual checks not in scope of unit or E2E tests.

---

_Verified: 2026-04-25T18:30:00Z_ _Re-verification: Yes — after 03-09-gap and
03-10-gap closure_ _Verifier: Claude (gsd-verifier, sonnet)_
