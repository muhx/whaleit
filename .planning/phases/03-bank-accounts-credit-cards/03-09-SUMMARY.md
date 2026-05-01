---
phase: 03-bank-accounts-credit-cards
plan: "09"
type: gap-closure
subsystem: frontend
tags: [gap-closure, frontend, regression, h-01, accounts-edit, tdd]
dependency_graph:
  requires: [03-07, 03-07b]
  provides: [h-01-closed, acct-04-edit-restored]
  affects: [account-edit-modal, accounts-page-tests]
tech_stack:
  added: []
  patterns: [tdd-red-green, component-level-regression-test]
key_files:
  created: []
  modified:
    - apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
    - apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx
decisions:
  - "Deleted blanket vi.mock('account-edit-modal') from test file and added
    global mocks for useSettingsContext + useIsMobileViewport to allow direct
    AccountEditModal rendering"
  - "Mocked @whaleit/ui/components/ui/dialog to render children when open=true
    (Radix portals don't work in JSDOM)"
  - "Extended @whaleit/ui mock with MoneyInput, ResponsiveSelect, Select*,
    RadioGroup, CurrencyInput, DatePickerInput — each rendering an <input> that
    passes value through so getByLabelText assertions work"
  - "balanceUpdatedAt intentionally excluded from defaultValues (server-only per
    D-12, per plan instructions)"
metrics:
  duration: "~15 minutes"
  completed: "2026-04-25T16:59:44Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
requirements_addressed: [ACCT-04, ACCT-06, ACCT-07]
---

# Phase 3 Plan 09: AccountEditModal H-01 Gap Closure Summary

**One-liner:** Forwarded all 9 Phase 3 fields into
`AccountEditModal.defaultValues` from the account prop, closing H-01 and
restoring ACCT-04 edit flow for CHECKING / SAVINGS / CREDIT_CARD / LOAN account
types.

## What Was Built

Gap H-01 from `03-VERIFICATION.md` identified that `AccountEditModal` omitted
all 9 Phase 3 fields (`institution`, `openingBalance`, `creditLimit`,
`statementCycleDay`, `statementBalance`, `minimumPayment`, `statementDueDate`,
`rewardPointsBalance`, `cashbackBalance`) from its `defaultValues` object.
Combined with `newAccountSchema.superRefine` requiring `openingBalance` for
bank/CC/LOAN types, this blocked edit submission for every existing account of
those types.

This plan delivers a two-commit TDD fix:

1. **RED commit** (`d216f7c3`): Added component-level regression tests asserting
   the modal pre-fills CC and CHECKING fields from the account prop — both fail
   against the buggy modal.
2. **GREEN commit** (`5b2d5d26`): Added the 9 Phase 3 fields to `defaultValues`
   in `AccountEditModal` — both regression tests turn green.

## TDD Cycle

| Gate  | Commit                                                                                   | Status                            |
| ----- | ---------------------------------------------------------------------------------------- | --------------------------------- |
| RED   | `d216f7c3` test(03-09): add H-01 regression tests for AccountEditModal pre-fill          | 2 new tests FAIL, 5 existing PASS |
| GREEN | `5b2d5d26` fix(03-09): forward Phase 3 fields into AccountEditModal defaultValues (H-01) | All 7 tests PASS                  |

## Test Count Delta

| Scope                  | Before    | After                                         |
| ---------------------- | --------- | --------------------------------------------- |
| accounts-page.test.tsx | 5 passing | 7 passing (+2 new H-01 regression)            |
| New describe block     | —         | `AccountEditModal pre-fill regression (H-01)` |

## defaultValues Shape That Landed

```typescript
const defaultValues = {
  id: account?.id ?? undefined,
  name: account?.name ?? "",
  currentBalance: account?.currentBalance,
  accountType: (account?.accountType ?? "SECURITIES") as AccountType,
  group: account?.group ?? undefined,
  currency: account?.currency ?? settings?.baseCurrency ?? "USD",
  isDefault: account?.isDefault ?? false,
  isActive: account?.id ? account?.isActive : true,
  isArchived: account?.isArchived ?? false,
  trackingMode: account?.trackingMode,
  meta: account?.meta,
  // Phase 3 additions (D-06, D-11, D-18) — closes gap H-01:
  institution: account?.institution,
  openingBalance: account?.openingBalance,
  creditLimit: account?.creditLimit,
  statementCycleDay: account?.statementCycleDay,
  statementBalance: account?.statementBalance,
  minimumPayment: account?.minimumPayment,
  statementDueDate: account?.statementDueDate,
  rewardPointsBalance: account?.rewardPointsBalance,
  cashbackBalance: account?.cashbackBalance,
};
```

`balanceUpdatedAt` intentionally omitted — server-only field per D-12.

## Gap Closure

- **H-01:** `account-edit-modal.tsx defaultValues` omits Phase 3 fields —
  **CLOSED** by this plan.
- **ACCT-04** edit flow restored for CHECKING / SAVINGS / CREDIT_CARD / LOAN.
- **ROADMAP SC-4** ("User can edit and archive accounts without losing
  historical transaction data") moves from `partial` to `verified` at the source
  level.

## Verification Results

```
Tests  7 passed (7)
# grep audits:
grep -cE "(institution|openingBalance|creditLimit|...): account\?\." account-edit-modal.tsx → 9
grep -c "balanceUpdatedAt" account-edit-modal.tsx → 0
grep -c "AccountEditModal pre-fill regression" accounts-page.test.tsx → 1
grep -cE "^\s*it\(" accounts-page.test.tsx → 7
```

Type-check: Our files (`account-edit-modal.tsx`, `accounts-page.test.tsx`) have
0 TS errors. Remaining errors in the environment are pre-existing TS6305
`addon-sdk` dist-not-built issues unrelated to this plan's changes.

## Deviations from Plan

### Mock infrastructure additions (Rule 2 — required for correct test behavior)

The plan's preferred implementation (render `AccountEditModal` directly)
required additional mocks beyond what the existing test file provided. The plan
explicitly anticipated this in its action step ("If the @whaleit/ui mock for
ResponsiveSelect or MoneyInput swallows the value prop..."):

1. **Removed**
   `vi.mock("./components/account-edit-modal", () => ({ AccountEditModal: () => null }))`
   — mock was blocking direct import; existing 5 tests confirmed unaffected
   (modal is mounted but closed, and `useSettingsContext` is now mocked
   globally).

2. **Added** `vi.mock("@/lib/settings-provider", ...)` — `AccountEditModal`
   calls `useSettingsContext()` which throws without a SettingsProvider. Global
   mock prevents throw.

3. **Added** `vi.mock("@/hooks/use-platform", ...)` — `AccountEditModal` passes
   `useIsMobileViewport` to `Dialog`.

4. **Added** `vi.mock("@whaleit/ui/components/ui/dialog", ...)` — Radix UI
   Dialog uses portals; JSDOM doesn't render portal children. Replaced with
   simple `open ? <div>{children}</div> : null`.

5. **Added** additional mocks: `@whaleit/ui/components/ui/button`,
   `@whaleit/ui/components/ui/checkbox`, `@whaleit/ui/components/ui/alert`,
   `@whaleit/ui/components/ui/alert-dialog` — all used by `AccountForm`.

6. **Extended** `@whaleit/ui` mock to include `MoneyInput`, `ResponsiveSelect`,
   `Select*`, `RadioGroup`, `RadioGroupItem`, `CurrencyInput`, `DatePickerInput`
   — each renders an `<input>` that forwards `value` so `getByLabelText`
   assertions work.

All additions are minimal (template: existing Input mock pattern). No
refactoring of unrelated mocks.

## Human Verification Items (Remain Open)

The following items from `03-VERIFICATION.md` `human_verification` section are
NOT closed by this plan:

1. **E2E spec run on a clean host** —
   `npx playwright test e2e/11-accounts.spec.ts` against a fresh PG database.
   Blocked by port 8088 / OrbStack on the executor host. Run:
   `node scripts/prep-e2e.mjs && pnpm run dev:web && npx playwright test e2e/11-accounts.spec.ts`.

2. **PG integration tests** — `cargo test -p whaleit-storage-postgres accounts`
   with `DATABASE_URL` set. No PG instance available in this environment.

3. **Visual fidelity check** — Available credit chip, Progress bar utilization
   color tier vs UI-SPEC §1 + §6. Pixel-level visual checks out of scope.

4. **Manual edit-flow smoke test** — Open edit dialog on existing CHECKING and
   CREDIT_CARD accounts in the running app and confirm fields pre-fill + submit
   succeeds. Unit regression (Task 1) covers structural assertion; live UX
   confirmation remains human-only.

## Known Stubs

None. The `account-edit-modal.tsx` fix is fully wired: all 9 Phase 3 fields flow
from the `account` prop through `defaultValues` to `AccountForm` which renders
them in the appropriate form inputs.

## Threat Flags

None. No new network endpoints, auth paths, or file access patterns introduced.
The change is purely a defaultValues object extension in a client-side form
component.

## Self-Check: PASSED

| Item                                         | Result |
| -------------------------------------------- | ------ |
| `account-edit-modal.tsx` exists              | FOUND  |
| `accounts-page.test.tsx` exists              | FOUND  |
| `03-09-SUMMARY.md` exists                    | FOUND  |
| Commit `d216f7c3` (RED) exists               | FOUND  |
| Commit `5b2d5d26` (GREEN) exists             | FOUND  |
| No unexpected file deletions in GREEN commit | PASSED |
