---
phase: 03-bank-accounts-credit-cards
plan: 10
subsystem: backend-rust
tags:
  [
    gap-closure,
    backend,
    rust,
    invariants,
    h-02,
    h-03,
    d-06,
    d-12,
    accounts-update,
  ]
dependency_graph:
  requires: [03-02, 03-03, 03-04]
  provides: [D-06-invariant-on-type-transition, D-12-server-only-timestamp]
  affects: [accounts_service, server-dto-seam]
tech_stack:
  added: []
  patterns: [service-layer-sanitization, dto-seam-discard, tdd-red-green]
key_files:
  created: []
  modified:
    - crates/core/src/accounts/accounts_service_tests.rs
    - crates/core/src/accounts/accounts_service.rs
    - apps/server/src/models.rs
decisions:
  - "D-06 fix placed in service layer (before validate()), not in validate() or
    repository — consistent with the fix-at-correct-seam principle"
  - "D-12 fix uses dual-seam defense: DTO From impl discards at HTTP boundary;
    service else-branch discards for all callers (Tauri IPC, future MCP)"
  - "Did not remove balance_updated_at field from DTO structs — kept for
    response-side consumers, only discarded on inbound conversion"
metrics:
  duration_minutes: 7
  completed_date: "2026-04-25T16:57:29Z"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 3
---

# Phase 3 Plan 10: Gap Closure — D-06 + D-12 Invariants Summary

**One-liner:** Service-layer type-transition CC-field clearing (D-06) and
unconditional client-timestamp discard at DTO seam + service else-branch (D-12),
pinned by 2 new RED→GREEN TDD tests.

## What Was Built

Closed 2 carry-forward High-severity issues from 03-REVIEW.md:

### H-02 closed: D-06 invariant on type transitions

**Root cause:** `AccountUpdate::validate()` rejected CC fields being present on
non-CC types, but Diesel's `AsChangeset` treats `Option::None` as "skip column"
— so a client submitting a CREDIT_CARD→CHECKING transition with all 7 CC fields
as `None` passed validation but left stale CC data in the database row.

**Fix:** In `update_account` (service layer), detect
`existing.account_type == CREDIT_CARD && account_update.account_type != CREDIT_CARD`
and force all 7 CC fields to `None` on the `AccountUpdate` before forwarding to
the repository. This happens BEFORE validate(), so a buggy client sending CC
fields with a non-CC type also gets sanitized rather than rejected.

```rust
let type_transition_out_of_cc =
    existing.account_type == super::accounts_constants::account_types::CREDIT_CARD
        && account_update.account_type != super::accounts_constants::account_types::CREDIT_CARD;
if type_transition_out_of_cc {
    account_update.credit_limit = None;
    account_update.statement_cycle_day = None;
    account_update.statement_balance = None;
    account_update.minimum_payment = None;
    account_update.statement_due_date = None;
    account_update.reward_points_balance = None;
    account_update.cashback_balance = None;
}
```

### H-03 closed: D-12 server-only balance_updated_at

**Root cause:** HTTP `AccountUpdate` DTO accepted `balance_updated_at` from
clients. The service only stamped the field when `current_balance` changed — so
a client could backdate/future-date the field by sending it without a balance
change.

**Fix (dual seam):**

1. `From<NewAccount> for core_accounts::NewAccount` and
   `From<AccountUpdate> for core_accounts::AccountUpdate` in
   `apps/server/src/models.rs` now discard the client value:

   ```rust
   balance_updated_at: None, // D-12: server-only field, client value discarded
   ```

2. Service else-branch unconditionally discards any value that bypasses the DTO
   seam (Tauri IPC, future MCP callers):
   ```rust
   } else {
       // Belt and suspenders for D-12: discard any inbound value that
       // bypassed the DTO sanitation.
       account_update.balance_updated_at = None;
   }
   ```

## TDD Cycle

### RED commit: `d85f1aec`

`test(03-10): add D-06 + D-12 invariant tests for update_account`

- `test_update_ignores_client_supplied_balance_updated_at_when_balance_unchanged`:
  balance unchanged + client sends backdated timestamp → asserts captured update
  has `balance_updated_at == None`. Failed because service passed client value
  through.
- `test_update_clears_cc_fields_on_type_transition_out_of_cc`: existing CC
  account, update with CHECKING type + all 7 CC fields still `Some(...)` →
  asserts `result.is_ok()` AND all 7 CC fields `None`. Failed because service
  forwarded to validate() which rejected CC fields on non-CC type.

Runner output at RED: `3 passed; 2 failed (expected RED for H-02/H-03)`

### GREEN commit: `8528d463`

`fix(03-10): enforce D-06 + D-12 invariants on update_account (H-02, H-03)`

All 5 service tests pass. Full accounts suite: **16 → 18 passed**.

## Test Count Delta

| Suite                                                                | Before | After |
| -------------------------------------------------------------------- | ------ | ----- |
| `cargo test -p whaleit-core accounts::accounts_service_tests::tests` | 3      | 5     |
| `cargo test -p whaleit-core accounts::`                              | 16     | 18    |

## Verification Results

| Check                                                                | Result                                  |
| -------------------------------------------------------------------- | --------------------------------------- |
| `cargo test -p whaleit-core accounts::accounts_service_tests::tests` | 5 passed; 0 failed                      |
| `cargo test -p whaleit-core accounts::`                              | 18 passed; 0 failed                     |
| `cargo check -p whaleit-server`                                      | Exit 0 (clean)                          |
| `cargo check -p whaleit-storage-postgres --tests`                    | Exit 0 (clean)                          |
| `git diff crates/core/src/accounts/accounts_model.rs`                | Empty (unchanged)                       |
| `git diff crates/storage-postgres/src/accounts/repository.rs`        | Empty (unchanged)                       |
| `type_transition_out_of_cc` in accounts_service.rs                   | 2 occurrences (decl + if)               |
| `balance_updated_at: None` in From impls (models.rs)                 | 2 occurrences                           |
| `pub balance_updated_at: Option<NaiveDateTime>` in models.rs         | 3 (struct fields kept for response use) |

## Deviations from Plan

None — plan executed exactly as written. The plan correctly identified that Test
2's RED signal required a buggy-client payload (CC fields as `Some(...)` with
non-CC type) to trigger the validate() rejection path, making `result.is_ok()`
FALSE today.

## Known Stubs

None. All production code is fully wired.

## Human Verification Remaining

Per plan `human_verification` section, these items remain pending after this
plan AND Plan 03-09:

1. **PG integration tests with DATABASE_URL.**
   `cargo test -p whaleit-storage-postgres accounts` against a real PG instance.
   This plan's H-02 fix produces correct `AccountUpdate` values at the
   service-layer boundary; the round-trip assertion that PG ends up with NULL
   columns for the 7 CC fields requires DATABASE_URL.

   **Recommended next CI step:** Add
   `test_update_clears_cc_columns_on_type_change_pg` to
   `crates/storage-postgres/src/accounts/repository_tests.rs` — an integration
   test that creates a CC account, runs update with CHECKING type, then queries
   the row and asserts all 7 CC columns are NULL. This closes the assertion gap
   between service-layer contract (verified here) and actual DB state.

2. **E2E spec on a clean host.** Port 8088 conflict on the verifier host. Recipe
   in 03-08-SUMMARY.md.

3. **Manual smoke test.** Edit a CC account, change type to CHECKING, submit,
   verify in DB that `credit_limit` / `statement_cycle_day` / etc. are NULL.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes
introduced. Both fixes are purely service-layer and DTO-seam logic within
existing trust boundaries.

| Threat ID                                                  | Disposition | Result                                               |
| ---------------------------------------------------------- | ----------- | ---------------------------------------------------- |
| T-3-02 (Tampering — client-supplied balance_updated_at)    | mitigated   | From-impl discards + service else-branch             |
| T-3-02 (Tampering — CC-field leak via type transition)     | mitigated   | Service-layer type-transition detection + force-NULL |
| T-3-04 (Info Disclosure — stale CC fields on CHECKING row) | mitigated   | Same H-02 fix clears columns on type transition      |

## Self-Check: PASSED

| Item                                                                                     | Status |
| ---------------------------------------------------------------------------------------- | ------ |
| accounts_service_tests.rs exists                                                         | FOUND  |
| accounts_service.rs exists                                                               | FOUND  |
| models.rs exists                                                                         | FOUND  |
| 03-10-SUMMARY.md exists                                                                  | FOUND  |
| Commit d85f1aec (RED) exists                                                             | FOUND  |
| Commit 8528d463 (GREEN) exists                                                           | FOUND  |
| test_update_ignores_client_supplied_balance_updated_at_when_balance_unchanged count == 1 | PASS   |
| test_update_clears_cc_fields_on_type_transition_out_of_cc count == 1                     | PASS   |
