---
phase: 03-bank-accounts-credit-cards
plan: 02
subsystem: domain
tags: [rust, core, accounts, validation, account-types, credit-card]

# Dependency graph
requires:
  - phase: 02-dual-database-engine
    provides: rust_decimal + chrono workspace deps already wired into whaleit-core
provides:
  - AccountType `&str` constants for CHECKING, SAVINGS, CREDIT_CARD, LOAN
  - AccountKind enum (Asset/Liability/Investment) + account_kind() classifier
  - Extended default_group_for_account_type covering Banking/Credit Cards/Loans
  - 11 new fields on Account/NewAccount/AccountUpdate (institution, opening_balance,
    current_balance, balance_updated_at, credit_limit, statement_cycle_day,
    statement_balance, minimum_payment, statement_due_date,
    reward_points_balance, cashback_balance)
  - NewAccount::validate() rules: D-06 CC null-rule, D-11 opening_balance required,
    credit_limit > 0 (CC), statement_cycle_day in 1..=31 (CC), opening_balance >= 0
    (bank/LOAN)
  - AccountUpdate::validate() D-06 null-rule
  - 6 Wave 0 unit tests (test_account_kind, test_default_group_for_new_types,
    test_new_account_validate_bank, test_new_account_validate_credit_card,
    test_new_account_validate_credit_card_rejects_invalid,
    test_new_account_validate_non_cc_rejects_cc_fields)
affects: [03-03 storage-postgres, 03-04 http-dtos, 03-05 frontend-constants, 06 net-worth]

# Tech tracking
tech-stack:
  added: []  # All deps were already in workspace (rust_decimal, rust_decimal_macros, chrono)
  patterns:
    - "&str AccountType constants in pub mod account_types (no enum, per D-01)"
    - "Derived classifier helper (account_kind) instead of stored column (per D-03)"
    - "Service-layer validation in NewAccount::validate / AccountUpdate::validate (RESEARCH map)"
    - "..Default::default() shorthand for all in-core Account struct literals (orphan-proof against future field additions)"

key-files:
  created:
    - crates/core/src/accounts/accounts_model_tests.rs (extended — was Wave 0 stub)
    - .planning/phases/03-bank-accounts-credit-cards/deferred-items.md
  modified:
    - crates/core/src/accounts/accounts_constants.rs
    - crates/core/src/accounts/accounts_model.rs
    - crates/core/src/accounts/accounts_model_tests.rs
    - crates/core/src/portfolio/snapshot/snapshot_service.rs
    - crates/core/src/portfolio/snapshot/snapshot_service_tests.rs
    - crates/core/src/portfolio/net_worth/net_worth_service_tests.rs
    - crates/core/src/activities/activities_service_tests.rs
    - .planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md

key-decisions:
  - "AccountKind landed in whaleit-core::accounts via `pub use accounts_constants::*;` so consumers import via `whaleit_core::accounts::AccountKind`"
  - "All in-core Account struct literals converted to ..Default::default() shorthand to make Account future-extensible without scattering edits"
  - "Did NOT touch net_worth_service.rs — Phase 6 scope per CONTEXT.md canonical_refs landmine"

patterns-established:
  - "Phase 6 net-worth code can adopt account_kind(account.account_type) — already exported"
  - "Downstream plans (03-03 PG storage, 03-04 HTTP DTOs) should map the 11 new domain fields 1:1; field types are stable"

requirements-completed: [ACCT-01, ACCT-02, ACCT-03, ACCT-05]

# Metrics
duration: ~25 min
completed: 2026-04-25
---

# Phase 3 Plan 02: Core domain extension for bank accounts + credit cards Summary

**Extended whaleit-core account domain with 4 new AccountType `&str` constants,
AccountKind/account_kind() classifier, 11 new bank/CC fields on
Account/NewAccount/AccountUpdate, and CC-gated validate() rules — landing the
Wave 0 unit-test scaffolding that downstream plans target.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-04-25T03:25:00Z (approx)
- **Completed:** 2026-04-25T03:50:11Z
- **Tasks:** 3 / 3
- **Files modified:** 8

## Accomplishments

- Added `account_types::{CHECKING, SAVINGS, CREDIT_CARD, LOAN}` `&str` constants
  and extended `default_group_for_account_type` with Banking / Credit Cards /
  Loans arms (D-01, D-16).
- Introduced `AccountKind` enum + `account_kind(&str) -> AccountKind` classifier
  (D-03) — exported automatically via the existing
  `pub use accounts_constants::*;` re-export so Phase 6 net-worth can adopt it
  without further plumbing.
- Extended `Account`, `NewAccount`, and `AccountUpdate` with 11 new nullable
  fields covering institution, opening/current balance + timestamp, credit
  limit, statement cycle/balance/min-payment/due-date, and
  reward-points/cashback (D-06, D-09, D-11, D-12, D-18).
- Hardened `NewAccount::validate()` with D-06 (CC fields only on CC), D-11
  (opening_balance required for bank/CC/LOAN), `credit_limit > 0`,
  `statement_cycle_day ∈ 1..=31`, and non-negative opening balance for
  bank/LOAN. Mirrored the D-06 null-rule on `AccountUpdate::validate()`.
- Added 6 sync Wave 0 unit tests; all pass alongside the 7 pre-existing
  TrackingMode/Account tests
  (`cargo test -p whaleit-core accounts:: → 13 passed`).
- Updated `03-VALIDATION.md` Per-Task Verification Map: 5 rows (Bank validate,
  CC validate valid + invalid, Non-CC rejects CC fields, account_kind Rust)
  flipped from `⬜ pending` (W0) to `✅ green`.

## Task Commits

1. **Task 1: Extend AccountType constants + add AccountKind helper + extend
   default_group_for_account_type** — `93473ff9` (feat)
2. **Task 2: Add 11 new fields to Account/NewAccount/AccountUpdate + extend
   validate()** — `83ce7cb9` (feat)
3. **Task 3: Convert test helper to ..Default::default() and add Wave 0 unit
   tests** — `69f1365c` (test)

## Files Created/Modified

- `crates/core/src/accounts/accounts_constants.rs` — Added 4 AccountType
  constants, 3 `default_group_for_account_type` arms, `AccountKind` enum,
  `account_kind()` helper.
- `crates/core/src/accounts/accounts_model.rs` — Imported `chrono::NaiveDate`,
  `rust_decimal::Decimal`, `crate::accounts::account_types`. Added 11 fields
  (with doc comments) to `Account`, `NewAccount`, `AccountUpdate`. Replaced
  `NewAccount::validate()` body with full CC + opening-balance rule set.
  Extended `AccountUpdate::validate()` with D-06 null-rule.
- `crates/core/src/accounts/accounts_model_tests.rs` — Converted
  `create_test_account` to `..Default::default()` shorthand; added
  `new_account_base()` fixture and 6 sync Wave 0 tests.
- `crates/core/src/portfolio/snapshot/snapshot_service.rs` — Appended
  `..Default::default()` to `create_total_virtual_account` Account literal
  (orphan from Task 2).
- `crates/core/src/portfolio/net_worth/net_worth_service_tests.rs` — Appended
  `..Default::default()` to two test Account literals (orphans from Task 2).
- `crates/core/src/portfolio/snapshot/snapshot_service_tests.rs` — Same orphan
  fix in two test helpers.
- `crates/core/src/activities/activities_service_tests.rs` — Same orphan fix in
  one test helper.
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` — 5
  verification-map rows updated.
- `.planning/phases/03-bank-accounts-credit-cards/deferred-items.md` — Logged
  pre-existing tokio-runtime test failure unrelated to this plan.

## Decisions Made

- Used `..Default::default()` shorthand on every in-core Account struct literal
  rather than enumerating the 11 new fields per call-site. `Account` already
  derives `Default`, so this is the surgical, future-proof fix.
- Honored the plan's NOTE that scope is `cargo check -p whaleit-core` only —
  left the storage-postgres/server/connect/ai consumer call-sites for plans
  03-03 and 03-04 to address (their explicit `NewAccount { ... }` literals will
  need the 11 new fields).
- Did NOT modify `net_worth_service.rs` (only `net_worth_service_tests.rs` for
  the orphan fix). Production net-worth signing logic stays Phase 6 scope per
  CONTEXT canonical_refs landmine.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Account field additions broke 5 in-core test helpers
and 1 in-core production helper**

- **Found during:** Task 2 (`cargo check -p whaleit-core` after model edits) and
  Task 3 (`cargo test -p whaleit-core accounts::`).
- **Issue:** Adding 11 fields to `Account` invalidated explicit
  `Account { ... }` struct literals at:
  - `crates/core/src/portfolio/snapshot/snapshot_service.rs:208` (production)
  - `crates/core/src/portfolio/net_worth/net_worth_service_tests.rs:810, 1708`
  - `crates/core/src/portfolio/snapshot/snapshot_service_tests.rs:1015, 4614`
  - `crates/core/src/activities/activities_service_tests.rs:906`
- **Fix:** Appended `..Default::default()` to each literal (Account already has
  `#[derive(Default)]`). Surgical, single-line per site.
- **Files modified:** snapshot_service.rs, snapshot_service_tests.rs,
  net_worth_service_tests.rs, activities_service_tests.rs.
- **Verification:** `cargo check -p whaleit-core` and
  `cargo test -p whaleit-core accounts::` both green.
- **Committed in:** `83ce7cb9` (snapshot_service.rs production helper),
  `69f1365c` (the four test helpers).

---

**Total deviations:** 1 auto-fix (Rule 3 — Blocking) **Impact on plan:**
Necessary to complete Tasks 2 and 3. No scope creep — no behavior changed in any
of the affected helpers; only field-completion semantics.

## Issues Encountered

- **Pre-existing test failure** (not my changes):
  `portfolio::snapshot::holdings_calculator_tests::tests::test_multi_currency_same_asset_buy_activities`
  panics with `Cannot start a runtime from within a runtime` — a tokio runtime
  conflict unrelated to the Account domain. Logged to `deferred-items.md`. Per
  scope-boundary rules, did not attempt to fix.

## TDD Gate Compliance

This plan's frontmatter is `type: execute` (not `type: tdd`), but all 3 tasks
have `tdd="true"`. The plan front-loaded the test design into Task 3 rather than
per-task RED/GREEN cycles, so per-task TDD gates do not strictly apply. The Wave
0 tests in Task 3 confirm the Task 1 + Task 2 implementation behavior
end-to-end:

- `test_account_kind` + `test_default_group_for_new_types` → exercise Task 1
  changes.
- `test_new_account_validate_bank` + `test_new_account_validate_credit_card` +
  `test_new_account_validate_credit_card_rejects_invalid` +
  `test_new_account_validate_non_cc_rejects_cc_fields` → exercise Task 2
  changes.

All 6 new tests passed on first run (no test fix iterations).

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **03-03 storage-postgres:** Domain shape is locked. The 11 fields map 1:1 to
  the migration columns described in `03-CONTEXT.md` D-06 / D-11 / D-12 / D-18.
  `From<AccountDB> for Account` and `From<NewAccount> for AccountDB` need to
  populate all 11 new fields.
- **03-04 HTTP DTOs:** `apps/server/src/models.rs::NewAccount` and
  `apps/server/src/models.rs::AccountUpdate` are unchanged in this plan and
  currently lack the 11 fields — that crate fails to compile until 03-04 mirrors
  the additions and updates the `From` impls. (This is by design and matches the
  plan acceptance scope: "compile may surface failures in the storage-postgres
  or server crates — those are addressed in later plans".)
- **03-05 frontend-constants:** Frontend `AccountKind` mirror still pending.
  Plan 03-05 should add `accountKind()` and the four AccountType strings to
  `apps/frontend/src/lib/constants.ts`.
- **06 net-worth:** Phase 6 can now
  `use whaleit_core::accounts::{account_kind, AccountKind};` to flip the sign on
  liability balances without further core changes.

## Self-Check: PASSED

Created files exist:

- `crates/core/src/accounts/accounts_model_tests.rs` → FOUND (extended)
- `.planning/phases/03-bank-accounts-credit-cards/deferred-items.md` → FOUND
- `.planning/phases/03-bank-accounts-credit-cards/03-02-SUMMARY.md` → FOUND
  (this file)

Modified files exist:

- `crates/core/src/accounts/accounts_constants.rs` → FOUND
- `crates/core/src/accounts/accounts_model.rs` → FOUND
- `crates/core/src/portfolio/snapshot/snapshot_service.rs` → FOUND
- `crates/core/src/portfolio/snapshot/snapshot_service_tests.rs` → FOUND
- `crates/core/src/portfolio/net_worth/net_worth_service_tests.rs` → FOUND
- `crates/core/src/activities/activities_service_tests.rs` → FOUND
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` → FOUND

Commits exist (`git log --oneline | head -10`):

- `93473ff9` (Task 1) → FOUND
- `83ce7cb9` (Task 2) → FOUND
- `69f1365c` (Task 3) → FOUND

Acceptance commands:

- `cargo test -p whaleit-core accounts::` → `13 passed, 796 filtered out` (PASS)
- `cargo check -p whaleit-core` → `Finished dev profile` (PASS)

---

_Phase: 03-bank-accounts-credit-cards_ _Completed: 2026-04-25_
