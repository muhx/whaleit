---
phase: 03-bank-accounts-credit-cards
plan: 07b
status: complete
tasks_completed: 3
tasks_total: 3
created: 2026-04-25
---

# Plan 03-07b — List & Detail UI for Bank/CC Accounts

## Self-Check: PASSED

## Summary

Wired the /settings/accounts list page with a group-by axis (Banking / Credit
Cards / Loans / Investments / Cash / Crypto / Uncategorized) using
`account.group ?? defaultGroupForAccountType`, added a Show-archived Switch
(archived hidden by default), and an "Available credit" chip for CREDIT_CARD
rows derived from `creditLimit - currentBalance`. The account detail page now
renders CC-specific Credit overview / Statement snapshot / Rewards sections when
`accountType === CREDIT_CARD`, hides investment-only modules (HistoryChart,
AccountHoldings, AccountMetrics, AccountContributionLimit) for
CHECKING/SAVINGS/CREDIT_CARD/LOAN, and shows a Balance card for non-CC bank/loan
accounts.

## Commits

- `537c64e5` — feat(03-07b): add group-by + show-archived Switch + Available
  credit chip
- `6cdd1128` — feat(03-07b): render CC sections + bank Balance card on account
  detail page
- `d94c41e0` — test(03-07b): add coverage for group-by axis + archive toggle

## Key Files

- `apps/frontend/src/pages/settings/accounts/accounts-page.tsx` — group-by +
  archive toggle
- `apps/frontend/src/pages/settings/accounts/components/account-item.tsx` —
  Available credit chip
- `apps/frontend/src/pages/account/account-page.tsx` — CC detail sections +
  investment-only module gating
- `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` — 5 tests
  (group-by, archive toggle behavior)

## Verification

- `pnpm --filter frontend type-check` → exits 0 (clean)
- `pnpm --filter frontend exec vitest --run` → 537/537 tests pass across 45
  files
  - Includes 5 new tests in accounts-page.test.tsx covering group-by axis and
    archive toggle behavior

## Success Criteria

- [x] `/settings/accounts` shows group-by axis using
      `account.group ?? defaultGroupForAccountType`
- [x] Show-archived Switch toggles archived rows; archived hidden by default
- [x] CREDIT_CARD row shows 'Available credit' chip
- [x] CC detail page renders Credit overview / Statement snapshot / Rewards
      sections
- [x] Account detail page hides investment-only modules for non-investment types
      (CHECKING/SAVINGS/CREDIT_CARD/LOAN)
- [x] accounts-page.test.tsx covers group-by + archive toggle
- [x] Each task committed individually with --no-verify
- [x] SUMMARY.md created and committed

## Notes

Plan executed across two sessions due to a usage-limit reset boundary; the
second session committed only the test file and this SUMMARY (the two production
commits landed in session 1). All work verified post-merge via type-check and
full vitest run.
