---
status: testing
phase: 03-bank-accounts-credit-cards
source:
  [03-VERIFICATION.md, 03-08-SUMMARY.md, 03-09-SUMMARY.md, 03-10-SUMMARY.md]
started: 2026-04-25T18:35:00Z
updated: 2026-04-25T18:55:00Z
---

## Current Test

number: 1 name: UAT Golden Path expected: | Run `pnpm run dev:web`. In the
browser, sequentially:

1. Create a CHECKING account — institution "Wells Fargo", opening balance
   $1,234.56.
2. Create a SAVINGS account — institution "Chase", opening balance $5,000.
3. Create a CREDIT_CARD account — institution "Amex", credit limit $10,000,
   statement cycle day 15.
4. Open the edit dialog on the CHECKING account → "Wells Fargo" and $1,234.56
   are pre-filled. Submit without changes — succeeds.
5. Open the edit dialog on the CREDIT_CARD account → institution, creditLimit,
   statementCycleDay all pre-filled. Submit without changes — succeeds.
6. Archive each of the three accounts → they vanish from the default list.
7. Toggle "Show archived" → all three reappear.
8. On the CC row, the "Available credit" chip shows ($10,000 − current_balance).

All steps succeed end-to-end without errors. awaiting: user response

## Tests

### 1. UAT Golden Path

expected: Run `pnpm run dev:web` and confirm: create CHECKING (Wells Fargo,
$1,234.56), SAVINGS (Chase, $5,000), CREDIT_CARD (Amex, limit $10,000, cycle day
15); edit dialog on each pre-fills institution / openingBalance / CC fields;
archive each; toggle "Show archived" reveals all three; CC row shows "Available
credit" chip equal to limit minus current balance. result: [pending — re-test
after fix landed] prior_result: issue prior_reported: "When updating accounts,
Expected string, received number. The expected should be number, since its
financial account" prior_severity: major fix_commit: 7e9eb697 fix_summary: |
Frontend now treats Decimal money fields as JSON numbers end-to-end, matching
the rust_decimal serde-float wire format (Cargo.toml:48). Account interface, Zod
newAccountSchema, account-form MoneyInput handlers, account-page detail card,
update-balance-modal, and credit-helpers all flipped string → number. 539/539
frontend tests pass. diagnosis: | Pre-existing contract mismatch exposed by
03-09 H-01 fix. Root cause: rust_decimal in workspace Cargo.toml is configured
with feature "serde-float", so backend serializes Decimal money fields as JSON
numbers (e.g. `"creditLimit": 10000.00`). But the frontend Account type and Zod
newAccountSchema both declare these fields as `string` (per the comment in
account.ts:12 — "Decimal serialized as string (matches PG NUMERIC DTO)" — which
is aspirational, not actual). The `#[schema(value_type =   Option<String>)]`
annotation on the Rust DTOs is utoipa-only; it does not change the actual serde
wire format.

The pre-existing AccountEditModal pre-filled only `currentBalance`. The H-01 fix
added 4 more decimal fields to defaultValues (openingBalance, creditLimit,
statementBalance, minimumPayment, cashbackBalance), so the schema mismatch now
surfaces on any edit of an existing CHECKING / SAVINGS / CREDIT_CARD / LOAN
account.

### 2. E2E Spec on Clean Host

expected: On a host where port 8088 is free (no OrbStack), run
`node scripts/prep-e2e.mjs && pnpm run dev:web && ./scripts/wait-for-both-servers-to-be-ready.sh && npx playwright test e2e/11-accounts.spec.ts`
→ 6/6 tests pass (login, create CHECKING, create CREDIT_CARD, update balance
modal, archive, show-archived toggle). result: [pending]

### 3. PG Integration Tests

expected: With `DATABASE_URL=postgres://...` set,
`cargo test -p whaleit-storage-postgres accounts` passes 5 round-trip tests +
the migration up/down test against a real PostgreSQL instance. (Recommended add:
`test_update_clears_cc_columns_on_type_change_pg` to assert D-06 at the DB row
level.) result: [pending]

### 4. Visual Fidelity vs UI-SPEC §1+§6

expected: On `/settings/accounts` after creating a CC account, the "Available
credit" chip placement, color tier (green / yellow / red), hover state, and the
Progress bar utilization color all match UI-SPEC §1 + §6 pixel-for-pixel.
result: [pending]

## Summary

total: 4 passed: 0 issues: 0 pending: 4 skipped: 0 blocked: 0 note: "Test 1
originally reported issue (decimal type contract); fix landed in 7e9eb697. Test
1 reset to pending for re-test."

## Gaps

- truth: "Editing an existing CHECKING / SAVINGS / CREDIT_CARD / LOAN account
  submits successfully when no field changes." status: failed reason: 'User
  reported: "When updating accounts, Expected string, received number. The
  expected should be number, since its financial account"' severity: major test:
  1 root_cause: | rust_decimal feature `serde-float` (Cargo.toml:48) makes
  backend serialize Decimal money fields as JSON numbers, but frontend Account
  interface (account.ts:26-35) and Zod newAccountSchema (schemas.ts:99-108)
  declare them as `string`. Latent since project inception; surfaced now because
  H-01 fix forwarded 5 additional Decimal fields into defaultValues. artifacts:
  - apps/frontend/src/lib/types/account.ts
  - apps/frontend/src/lib/schemas.ts
  - apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
  - apps/server/src/models.rs
  - Cargo.toml missing: [] debug_session: ""
