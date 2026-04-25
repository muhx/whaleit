---
status: partial
phase: 03-bank-accounts-credit-cards
source: [03-VERIFICATION.md]
started: 2026-04-25T18:35:00Z
updated: 2026-04-25T18:35:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. UAT golden path (live browser, confirms H-01 closure)

expected: In `pnpm run dev:web`, sequentially: create CHECKING (Wells Fargo,
$1,234.56), SAVINGS (Chase, $5,000), CREDIT_CARD (Amex, limit $10,000, cycle day
15). Open edit dialog on each — institution / openingBalance / CC fields
pre-fill, submit succeeds without re-entry. Archive each; they vanish from the
default list. Toggle "Show archived"; all three reappear. CC row shows
"Available credit" chip = $10,000 − current_balance. result: [pending]

### 2. E2E Playwright spec on a clean host

expected:
`node scripts/prep-e2e.mjs && pnpm run dev:web && ./scripts/wait-for-both-servers-to-be-ready.sh && npx playwright test e2e/11-accounts.spec.ts`
→ 6/6 pass (login, create CHECKING, create CREDIT_CARD, update balance modal,
archive, show-archived toggle). Initial run on the verification host was blocked
by OrbStack holding port 8088 — run on a clean host. result: [pending]

### 3. PG integration tests against real Postgres

expected: With `DATABASE_URL=postgres://...` set,
`cargo test -p whaleit-storage-postgres accounts` passes 5 round-trip tests +
the migration up/down test. Recommended enhancement: add
`test_update_clears_cc_columns_on_type_change_pg` to
`crates/storage-postgres/src/accounts/repository_tests.rs` so D-06 is asserted
at the DB row level, not only the service contract. result: [pending]

### 4. Visual fidelity vs UI-SPEC §1 + §6

expected: On `/settings/accounts` after creating a CC account, the Available
credit chip placement, color tier (green / yellow / red), hover state, and the
Progress bar utilization color all match UI-SPEC §1 + §6 pixel-for-pixel.
result: [pending]

## Summary

total: 4 passed: 0 issues: 0 pending: 4 skipped: 0 blocked: 0

## Gaps
