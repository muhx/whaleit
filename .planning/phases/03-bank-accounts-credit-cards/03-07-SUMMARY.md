---
phase: 03-bank-accounts-credit-cards
plan: 07
subsystem: frontend
tags: [frontend, ui, react, settings, credit-card, balance-modal, helpers]

# Dependency graph
requires:
  - phase: 03-bank-accounts-credit-cards
    plan: 04
    provides:
      Server auto-bump of balance_updated_at on current_balance change (D-12)
  - phase: 03-bank-accounts-credit-cards
    plan: 05
    provides:
      Frontend AccountType (7 variants), Account interface (11 Phase 3 fields),
      newAccountSchema (CC-gated superRefine)
  - phase: 03-bank-accounts-credit-cards
    plan: 06
    provides: account-form accountTypes ResponsiveSelectOption[] with 7 variants
provides:
  - credit-helpers.ts named exports availableCredit, utilizationPercent,
    utilizationTier (consumed by Plan 03-07b)
  - UpdateBalanceModal component wrapping updateAccountMutation; triggers server
    auto-bump
  - account-form.tsx dynamic Institution + Opening balance inputs for
    CHECKING/SAVINGS/CREDIT_CARD/LOAN
  - account-form.tsx 7 CC-only FormFields rendered when CREDIT_CARD selected
  - account-form.tsx LOAN scope note (UI-SPEC §2 form section 5)
  - Wave 0 unit tests for derivation helpers (14 assertions)
affects:
  - 03-07b (consumes credit-helpers.ts named exports for utilization chip + CC
    detail page)
  - 03-08 (unified accounts list — UpdateBalanceModal will be wired into the row
    "Update balance" action)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dynamic FormField rendering driven by form.watch('accountType') —
      exhaustive boolean flags (isCreditCard, requiresInstitution,
      requiresOpeningBalance)"
    - "Wire-format-aware MoneyInput integration: parseFloat for displayed value,
      String() for outbound (Decimal-as-string DTO)"
    - "Number.isFinite() guards on every parseFloat result to handle malformed
      wire input (T-3-03 mitigation)"

key-files:
  created:
    - apps/frontend/src/pages/settings/accounts/credit-helpers.ts
    - apps/frontend/src/pages/settings/accounts/credit-helpers.test.ts
    - apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx
  modified:
    - apps/frontend/src/pages/settings/accounts/components/account-form.tsx
    - .planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md

key-decisions:
  - "Used `name` + `accountType` + every Phase 3 field in UpdateBalanceModal
    payload so the server's diff against `existing` sees only `current_balance`
    changed — auto-bump (Plan 03-04) reads cleanly"
  - "Dropped the planner-suggested `as never` cast on the mutate payload; the
    underlying updateAccount adapter accepts NewAccount (z.infer) which now
    includes all 11 Phase 3 fields per Plan 03-05's schema cascade"
  - "Used `<Select><SelectTrigger><SelectContent>` Radix shadcn primitives (not
    bare Radix Select) for statementCycleDay because the planner sketch's
    `<Select><SelectItem>...` would not have rendered a trigger or popover
    content"
  - "utilizationTier returns 'warning' for both 30-69% and 70-89% bands; UI-SPEC
    distinguishes tinted vs solid as a caller render decision, not a tier
    decision (avoids tier explosion)"

patterns-established:
  - "Form-side number/string bridging: `field.value ? parseFloat(field.value) :
    undefined` for input value, `(v) => field.onChange(v != null ? String(v) :
    undefined)` for output. Reused 6 times in CC fields + opening_balance."
  - "Modal payload construction for partial updates: spread the entire account
    into the AccountUpdate shape with only the target field overridden — server
    diff logic stays simple."

requirements-completed: [ACCT-01, ACCT-02, ACCT-05, ACCT-06, ACCT-07]
threats-addressed: [T-3-02, T-3-03]

# Metrics
duration: ~6 min
completed: 2026-04-25
---

# Phase 3 Plan 07: Form + Helpers + Modal Summary

**Landed credit-card derivation helpers (`availableCredit`,
`utilizationPercent`, `utilizationTier`), the focused `UpdateBalanceModal` that
triggers the server's balance auto-bump, and the dynamic CC / bank / loan
FormField sections inside the existing settings/accounts form. After this plan a
user can create a CHECKING / SAVINGS / CREDIT_CARD / LOAN account through the
existing form and update its balance via a focused modal.**

## Performance

- **Duration:** ~6 min (active execution time)
- **Started:** 2026-04-25T04:33:30Z
- **Completed:** 2026-04-25T04:39:20Z
- **Tasks:** 3 / 3
- **Files created:** 3
- **Files modified:** 2

## Accomplishments

- `credit-helpers.ts` exports three pure functions backed by 14 vitest
  assertions: `availableCredit(creditLimit, currentBalance)` returns
  `limit - balance` clamped to 0 (or `undefined` if limit missing/non-numeric);
  `utilizationPercent(creditLimit, currentBalance)` returns `0..100` rounded to
  int (or `undefined` when limit ≤ 0); `utilizationTier(percent)` maps to
  `'success' | 'warning' | 'destructive'` per the UI-SPEC color ramp. All
  numeric parsing is guarded by `Number.isFinite(...)` (T-3-03 mitigation).
- `UpdateBalanceModal` renders a `Dialog` with `MoneyInput` autofocused,
  `PrivacyAmount` showing the prior balance + `balanceUpdatedAt`, and footer
  buttons "Cancel" / "Save balance". The save action calls
  `updateAccountMutation.mutate` with the full account payload and only
  `currentBalance` mutated, so the server's auto-bump logic (Plan 03-04) sees
  the change cleanly and stamps `balance_updated_at`. Note field is omitted —
  Phase 3 service does not persist notes (UI-SPEC §5 dead-UI rule).
- `account-form.tsx` gains four dynamic sections gated on
  `form.watch("accountType")`: Institution
  (`CHECKING/SAVINGS/CREDIT_CARD/LOAN`), Opening balance MoneyInput (same set,
  required per D-11), seven CC-only fields (`creditLimit`, `statementCycleDay`
  1..31 Select, `statementBalance`, `minimumPayment`, `statementDueDate`
  DatePickerInput, `rewardPointsBalance` numeric Input, `cashbackBalance`
  MoneyInput), and a LOAN scope-disclosure muted card. T-3-02 mitigation: non-CC
  users cannot even type into CC fields because they're conditionally not
  rendered.
- 14 new tests added; all 532 frontend tests pass (518 baseline + 14 new).
  Frontend `type-check` exits 0.
- VALIDATION.md row "Available credit derived" flipped to ✅ green with Plan ID
  `03-07`, Task ID `1`.

## Task Commits

| Task    | Name                                                                              | Commit     | Files                                                                         |
| ------- | --------------------------------------------------------------------------------- | ---------- | ----------------------------------------------------------------------------- |
| 1.RED   | Failing tests for credit-helpers                                                  | `31c9a380` | apps/frontend/src/pages/settings/accounts/credit-helpers.test.ts              |
| 1.GREEN | Implement credit-helpers (availableCredit / utilizationPercent / utilizationTier) | `bbbb8cfa` | apps/frontend/src/pages/settings/accounts/credit-helpers.ts                   |
| 1.docs  | Flip VALIDATION.md row green                                                      | `ded39e73` | .planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md               |
| 2       | UpdateBalanceModal component                                                      | `f2c9997c` | apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx |
| 3       | Dynamic CC / bank / loan sections in account-form                                 | `459d208d` | apps/frontend/src/pages/settings/accounts/components/account-form.tsx         |

## Files Created/Modified

### Created

- `apps/frontend/src/pages/settings/accounts/credit-helpers.ts` — 50 lines.
  Three pure functions + `UtilizationTier` type. All inputs are
  `string | undefined` (Decimal-as-string DTO); all outputs are
  `number | undefined` or tier string. No external imports.
- `apps/frontend/src/pages/settings/accounts/credit-helpers.test.ts` — 53 lines.
  14 assertions across 3 `describe` blocks covering: missing limit, missing
  balance, balance > limit clamp, non-numeric input, division-by-zero guard,
  tier classification at boundaries.
- `apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx`
  — 113 lines. Dialog/MoneyInput/PrivacyAmount composition. No `useEffect`
  (state initialized from props at mount; modal remounts on re-open). Props:
  `{ account: Account; open: boolean; onClose: () => void; }`.

### Modified

- `apps/frontend/src/pages/settings/accounts/components/account-form.tsx` —
  extended imports to include `MoneyInput`, `DatePickerInput`,
  `Select`/`SelectTrigger`/`SelectContent`/`SelectItem`/`SelectValue` from
  `@whaleit/ui`. Added `selectedType`, `isCreditCard`, `requiresInstitution`,
  `requiresOpeningBalance` derived booleans. Inserted 4 conditional render
  blocks between the existing accountType FormField and the currency FormField:
  Institution, Opening balance, CC-block (7 fields), LOAN note card. The form's
  `onSubmit` and the existing 7 fields are byte-identical. +222 lines / -0
  lines.
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` — flipped
  the "Available credit derived" row from `TBD / TBD / TBD / ❌ W0` to
  `3 / 03-07 / 1 / ✅ green`.

## Decisions Made

- **No `as never` cast on the mutate payload in UpdateBalanceModal.** The
  planner sketch suggested `} as never,` because it assumed the
  `updateAccountMutation` type might not yet accept the 11 new Phase 3 fields.
  In this worktree, Plan 03-05's schema cascade already extended
  `newAccountSchema`, and the adapter `updateAccount` accepts
  `z.infer<typeof newAccountSchema>` directly (apps/frontend/src/adapters/
  shared/accounts.ts:8). The full payload type-checks cleanly without any cast.
- **Used `<Select><SelectTrigger><SelectContent>...</SelectContent></Select>`
  for statementCycleDay** instead of the planner sketch's bare
  `<Select><SelectItem>` form. The shadcn `Select` from `@whaleit/ui` is the
  Radix root and requires `SelectTrigger` (the actual button) + `SelectContent`
  (the popover) wrapping `SelectItem`s. The planner's sketch would have rendered
  nothing visible. (Verified against
  `packages/ui/src/components/ui/select.tsx`.)
- **Imported `Dialog`/`DialogContent`/`DialogFooter`/`DialogHeader`/
  `DialogTitle` from `@whaleit/ui/components/ui/dialog`** rather than the
  top-level `@whaleit/ui` re-export. This matches the existing
  `account-edit-modal.tsx` and `update-valuation-modal.tsx` patterns in this
  codebase. `MoneyInput` and `PrivacyAmount` come from the top-level package
  index because that's how every other consumer imports them.
- **Used `account-edit-modal.tsx` toast import path
  (`@whaleit/ui/components/ui/use-toast`)** to match the codebase's existing 10+
  consumer sites. This is the same path `useAccountMutations.ts` uses for its
  baseline error/success toasts.
- **`utilizationTier` returns `'warning'` for both 30-69% and 70-89% bands.**
  UI-SPEC distinguishes the bands as `--warning` tinted vs `--warning` solid,
  which is a caller render decision (the consumer can read the percent and pick
  the variant). Pushing it into the tier helper would either require a 4-tier
  enum or a `density` flag — both add complexity without informing the helper.
  The current consumer in Plan 03-07b will pick the variant based on the percent
  directly.
- **MoneyInput `value` accepts a number; we pass parsed strings.**
  `field.value ? Number.parseFloat(field.value) : undefined` parses the
  Decimal-as-string DTO into a number for display. On output, `String(v)`
  converts back to the wire format. The truthy guard (`field.value ? ...`)
  handles the `""` initial case from optional fields.
- **Statement due date stored as ISO date (YYYY-MM-DD), not full ISO datetime.**
  `d.toISOString().slice(0, 10)` truncates the time component. The schema
  expects `string` (no format constraint per Plan 03-05). The server's
  `statement_due_date: chrono::NaiveDate` deserializer accepts this format.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Bare `<Select><SelectItem>` in planner sketch would
render nothing**

- **Found during:** Task 3 implementation (statementCycleDay block)
- **Issue:** The planner sketch wrote
  `<Select value={...} onValueChange={...}>{Array.from({length:31}, ...).map(d => <SelectItem .../>)}</Select>`.
  shadcn's `Select` (Radix) requires `SelectTrigger` (the visible button) and
  `SelectContent` (the popover viewport) wrapping `SelectItem`s. The bare form
  would render no trigger and no popover.
- **Fix:** Wrapped the items in
  `<SelectTrigger><SelectValue placeholder="Select day" /></SelectTrigger><SelectContent>...</SelectContent>`
  — the standard shadcn pattern. Imports updated to include `SelectTrigger`,
  `SelectContent`, `SelectValue` alongside `Select`/`SelectItem`.
- **Files modified:**
  `apps/frontend/src/pages/settings/accounts/components/account-form.tsx` (Task
  3 commit).
- **Verification:** `pnpm --filter frontend type-check` exits 0; the Radix
  primitives now compose correctly.
- **Committed in:** `459d208d` (Task 3, single commit).

**2. [Rule 1 - Bug] Initial
`setNewBalance(account.currentBalance ? ... : undefined)` evaluated
`account.currentBalance` before the parsed number**

- **Found during:** Task 2 self-review prior to commit
- **Issue:** The planner sketch's `useState` initializer parsed inside the
  initializer call, but the reference `currentBalanceNum` was needed twice (once
  for state init, once for the unchanged comparison). Computing it twice would
  risk drift if the parser changed.
- **Fix:** Hoisted `currentBalanceNum` to a top-level `const`, then reused it in
  both `useState(currentBalanceNum)` and the `unchanged` comparison. Single
  source of truth.
- **Files modified:**
  `apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx`
  (Task 2 commit).
- **Verification:** Type-check clean; no behavior regression — same parsed value
  reaches both consumers.
- **Committed in:** `f2c9997c` (Task 2, single commit; not a separate fix commit
  because the issue was caught before the file was first committed).

---

**Total deviations:** 2 auto-fixes (Rule 1 + Rule 3). **Impact on plan:** None —
both changes are internal mechanics that preserve the plan's stated behavior. No
scope creep; every change traces to the plan's Task 2 / Task 3 acceptance
criteria.

## Issues Encountered

- **Worktree `node_modules` empty at session start.** Required `pnpm install`
  (~90s) followed by `pnpm --filter @whaleit/ui build` and
  `pnpm --filter @whaleit/addon-sdk build` to populate the dist outputs that
  frontend type-check depends on. Same recurring environmental quirk noted by
  Plans 03-05 and 03-06 SUMMARYs.

- **Vitest `--run <path>` filter.**
  `pnpm --filter frontend test -- --run <path>` does not filter to a single file
  when pnpm interprets the trailing `<path>` as a directory hint. Used
  `cd apps/frontend && pnpm exec vitest --run src/pages/settings/accounts/credit-helpers.test.ts`
  for the targeted RED/GREEN runs (matches the workaround in Plan 03-05's
  SUMMARY).

## TDD Gate Compliance

Plan 03-07 marks Task 1 as `tdd="true"`. Tasks 2 and 3 are `type="auto"` without
TDD — these are scaffolding-heavy UI components where pre-writing failing tests
against an unbuilt JSX tree adds no design pressure (the JSX shape is dictated
by the plan).

- **Task 1 RED:** `31c9a380` (test commit, 14 failing assertions — file fails to
  import the missing `./credit-helpers` module).
- **Task 1 GREEN:** `bbbb8cfa` (feat commit, 50 lines, all 14 tests pass on
  first run).
- **Task 1 REFACTOR:** Not needed — the implementation was a direct
  transcription from the plan's source listing; no cleanup opportunity surfaced.

The TDD cycle is visible in `git log --oneline 03-07-*`: `test(...)` precedes
`feat(...)` immediately for Task 1.

## Threat Mitigation

- **T-3-02 Tampering of CC fields:** Mitigated by conditional rendering
  (`{isCreditCard && ...}`) — non-CC users cannot type into CC fields because
  they don't render. Schema-level `superRefine` (Plan 03-05) catches direct
  schema-bypass attempts; backend (Plan 03-02) is the authority for stored
  values.
- **T-3-03 Tampering of credit-helpers parseFloat:** Mitigated by
  `Number.isFinite(...)` guards on every parsed number; `utilizationPercent`
  blocks division-by-zero with `limit > 0` check; `availableCredit` clamps to 0
  with `Math.max(...)`.

## User Setup Required

None — no external service configuration introduced.

## Next Phase Readiness

- **Plan 03-07b** can import `availableCredit`, `utilizationPercent`,
  `utilizationTier` from `@/pages/settings/accounts/credit-helpers` (named
  exports stable). The utilization chip on the unified list and the CC sections
  on the per-account detail page can wire directly to the helpers.
- **Plan 03-08 (unified accounts list)** can import `UpdateBalanceModal` from
  `./components/update-balance-modal` and trigger it from the row-level "Update
  balance" action.
- **End-to-end account creation** for the 4 new types now works via the existing
  `AccountEditModal` create flow:
  - CHECKING/SAVINGS/LOAN: Institution + Opening balance render.
  - CREDIT_CARD: Institution + Opening balance + 7 CC fields render. zod
    `superRefine` (Plan 03-05) blocks creation if CC `creditLimit` or
    `statementCycleDay` are missing.

## Self-Check: PASSED

**Files created (3/3 found):**

- `apps/frontend/src/pages/settings/accounts/credit-helpers.ts` → FOUND
- `apps/frontend/src/pages/settings/accounts/credit-helpers.test.ts` → FOUND
- `apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx`
  → FOUND

**Files modified (2/2 confirmed via git diff):**

- `apps/frontend/src/pages/settings/accounts/components/account-form.tsx` →
  FOUND (+222 lines)
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` → FOUND (1
  row updated)

**Commits exist (5/5 found via `git log f5811ccb..HEAD`):**

- `31c9a380` (Task 1 RED) → FOUND
- `bbbb8cfa` (Task 1 GREEN) → FOUND
- `ded39e73` (Task 1 docs) → FOUND
- `f2c9997c` (Task 2) → FOUND
- `459d208d` (Task 3) → FOUND

**Acceptance commands:**

- `pnpm --filter frontend type-check` → exits 0 (PASS)
- `cd apps/frontend && pnpm exec vitest --run src/pages/settings/accounts/credit-helpers.test.ts`
  → 14/14 pass (PASS)
- `pnpm --filter frontend exec vitest --run` → 532/532 tests pass across 44
  files (PASS — 518 baseline + 14 new)

**Acceptance criteria literals (all met):**

- [x] `credit-helpers.ts` exports `availableCredit`, `utilizationPercent`,
      `utilizationTier`
- [x] `credit-helpers.test.ts` has 14 assertions (5 + 5 + 4) across the 3
      functions
- [x] `update-balance-modal.tsx` contains literal
      `<DialogTitle>Update balance</DialogTitle>`
- [x] `update-balance-modal.tsx` contains literal `"Balance updated just now"`
- [x] `update-balance-modal.tsx` contains literal
      `currentBalance: String(newBalance),`
- [x] `update-balance-modal.tsx` contains no `<Textarea` and no `Note` field
- [x] `update-balance-modal.tsx` contains no `useEffect`
- [x] `account-form.tsx` contains literal
      `const isCreditCard = selectedType === "CREDIT_CARD";`
- [x] `account-form.tsx` contains literal `name="creditLimit"`
- [x] `account-form.tsx` contains literal `name="statementCycleDay"`
- [x] `account-form.tsx` contains literal `name="institution"`
- [x] `account-form.tsx` contains literal `name="openingBalance"`
- [x] `account-form.tsx` contains literal
      `Loans in v1 track name, institution, currency`
- [x] `account-form.tsx` contains literal `Day the statement closes each month.`
- [x] `account-form.tsx` contains literal
      `Today's balance becomes your starting point.`

---

_Phase: 03-bank-accounts-credit-cards_ _Plan: 07_ _Completed: 2026-04-25_
