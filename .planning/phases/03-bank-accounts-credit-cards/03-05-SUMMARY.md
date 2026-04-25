---
phase: 03-bank-accounts-credit-cards
plan: 05
subsystem: frontend
tags: [frontend, typescript, zod, constants, rename, account-types]

# Dependency graph
requires:
  - phase: 02-dual-database-engine
    provides: PostgreSQL storage layer + Account base shape
provides:
  - Frontend canonical AccountType extension (CHECKING, SAVINGS, CREDIT_CARD,
    LOAN)
  - AccountKind enum + accountKind() classification helper (mirrors crates/core)
  - defaultGroupForAccountType extended with Banking / Credit Cards / Loans
  - Account TS interface with 11 new Phase 3 fields + balance→currentBalance
    rename
  - newAccountSchema with CC-gated superRefine (D-06 + D-11 enforcement)
  - Wave 0 tests:
      constants.test.ts + newAccountSchema describe block in schemas.test.ts
affects:
  - 03-03 (settings/accounts UI consumes extended Account + selector icon maps)
  - 03-04 (backend NewAccount/AccountUpdate mirror — already merged in same
    wave)
  - 03-06 (account edit form uses extended schema)
  - 03-07 (CC-specific UI sections)
  - 03-08 (unified accounts list)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Exhaustiveness check via `const _exhaustive: never = x; void
      _exhaustive;`"
    - "Decimal-as-string TS typing for monetary fields (matches PG NUMERIC DTO)"
    - "zod superRefine for cross-field validation gated on accountType"

key-files:
  created:
    - apps/frontend/src/lib/constants.test.ts
  modified:
    - apps/frontend/src/lib/constants.ts
    - apps/frontend/src/lib/types/account.ts
    - apps/frontend/src/lib/schemas.ts
    - apps/frontend/src/lib/schemas.test.ts
    - apps/frontend/src/components/account-selector.tsx
    - apps/frontend/src/components/account-selector-mobile.tsx
    - apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
    - apps/frontend/src/pages/settings/accounts/components/account-item.tsx
    - apps/frontend/src/pages/account/account-page.tsx
    - apps/frontend/src/components/app-launcher.tsx
    - apps/frontend/src/pages/insights/portfolio-insights.tsx
    - apps/frontend/src/pages/holdings/holdings-page.tsx
    - apps/frontend/src/pages/holdings/components/holdings-mobile-filter-sheet.tsx
    - apps/frontend/src/pages/activity/components/activity-data-grid/activity-utils.test.ts
    - apps/frontend/src/pages/dashboard/accounts-summary.test.tsx
    - packages/addon-sdk/src/data-types.ts

key-decisions:
  - "Account.currentBalance typed as `string` (Decimal-as-string DTO), not
    number"
  - "Account.currentBalance is optional — synthetic Portfolio Account stubs use
    undefined"
  - "Use Icons.Building for LOAN — Icons.Building2 lucide name is exposed under
    Icons.Building in @whaleit/ui (line 280: `Building: Building2`)"
  - "Mirror addon-sdk AccountType + Account shape to keep type-bridge.ts
    assignments structurally compatible"
  - "Use `void _exhaustive;` to satisfy noUnusedLocals while preserving
    exhaustiveness check"

patterns-established:
  - "AccountType const + accountTypeSchema z.enum + AccountKind helper triple —
    single source of truth"
  - "superRefine block with isCC / isBankOrLoan booleans for clean CC-vs-bank
    gating"
  - "Per-Record-AccountType maps must list all 7 keys (TS catches missing
    entries)"

requirements-completed: [ACCT-01, ACCT-02, ACCT-03, ACCT-05]

# Metrics
duration: ~25min
completed: 2026-04-25
---

# Phase 3 Plan 05: Frontend Canonical Sources Summary

**Extended frontend AccountType (4 new variants), Account TS interface (11 new
fields + balance→currentBalance rename), newAccountSchema (CC-gated
superRefine), with full downstream consumer rewiring across 16 files.**

## Performance

- **Duration:** ~25 min (active execution time)
- **Started:** 2026-04-25T03:30Z (approx, after worktree base reset)
- **Completed:** 2026-04-25T03:56Z
- **Tasks:** 4 (all auto-completed, 2 of 4 with TDD RED/GREEN gates)
- **Files modified:** 15 source files + 1 new test file

## Accomplishments

- AccountType extended with CHECKING, SAVINGS, CREDIT_CARD, LOAN —
  accountTypeSchema z.enum and defaultGroupForAccountType updated in lockstep.
- AccountKind enum (ASSET / LIABILITY / INVESTMENT) and accountKind() helper
  added with `_exhaustive: never` guard mirroring the Rust definition.
- Account TS interface gains 11 Phase 3 fields (institution, openingBalance,
  currentBalance, balanceUpdatedAt, creditLimit, statementCycleDay,
  statementBalance, minimumPayment, statementDueDate, rewardPointsBalance,
  cashbackBalance) — all camelCase, monetary fields typed as `string` to match
  Decimal DTO.
- Legacy `balance: number` removed; replaced by optional
  `currentBalance?: string`. 7 consumer sites rewired (3 listed in plan + 4
  additional discovered via type-check).
- newAccountSchema extended with 11 optional fields and a superRefine block
  enforcing D-06 (CC-only fields nullable on non-CC types) and D-11
  (openingBalance required for bank/CC/LOAN); CC accounts also require
  creditLimit and statementCycleDay.
- Selectors gain icon entries for the 4 new types
  (Wallet/Coins/CreditCard/Building); mobile selector gains 4 new label cases.
- 13 new tests (7 in constants.test.ts, 6 in newAccountSchema describe block) —
  all pass alongside the existing 505 frontend tests.
- packages/addon-sdk/src/data-types.ts mirrored for type-bridge structural
  compatibility.

## Task Commits

1. **Task 1 RED: failing accountKind + group helper tests** — `654dbe5b` (test)
2. **Task 1 GREEN: extend AccountType + accountKind helper** — `09aabd58` (feat)
3. **Task 1 fix: extend Record<AccountType, ...> consumers + addon-sdk** —
   `e071e9c1` (fix, Rule 3)
4. **Task 2: extend Account interface + balance→currentBalance rename** —
   `97b9fed2` (feat)
5. **Task 3 RED: failing newAccountSchema tests** — `ec944600` (test)
6. **Task 3 GREEN: newAccountSchema 11 fields + superRefine** — `7ef14415`
   (feat)
7. **Task 4: selector icon maps + mobile label switch** — `75d935ca` (feat)

## Files Created/Modified

### Created

- `apps/frontend/src/lib/constants.test.ts` — 7 tests for accountKind +
  defaultGroupForAccountType.

### Modified — Plan-listed (8)

- `apps/frontend/src/lib/constants.ts` — AccountType, accountTypeSchema,
  defaultGroupForAccountType, AccountKind, accountKind helper.
  createPortfolioAccount stub: `balance: 0` → `currentBalance: undefined`.
- `apps/frontend/src/lib/types/account.ts` — `balance: number` →
  `currentBalance?: string` + 10 new fields.
- `apps/frontend/src/lib/schemas.ts` — newAccountSchema extended with 11
  fields + superRefine.
- `apps/frontend/src/lib/schemas.test.ts` — appended
  `describe("newAccountSchema", …)` with 6 tests; original 6 tests preserved.
- `apps/frontend/src/components/account-selector.tsx` — icon map +4 entries;
  stub `currentBalance` rename.
- `apps/frontend/src/components/account-selector-mobile.tsx` — icon map +4
  entries; getAccountTypeLabel +4 cases; stub rename.
- `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
  — defaultValues `balance` → `currentBalance`; type cast `as AccountType`
  (replaces hard-coded 3-type union); imports `AccountType` from
  `@/lib/constants`.

### Modified — Rule 3 deviations (8)

- `apps/frontend/src/components/app-launcher.tsx` — accountTypeIcons map now
  lists all 7 AccountType keys (Record<AccountType | "TOTAL", Icon>).
- `apps/frontend/src/pages/account/account-page.tsx` — accountTypeIcons
  Record<AccountType, Icon> +4 entries.
- `apps/frontend/src/pages/settings/accounts/components/account-item.tsx` —
  accountTypeConfig Record<AccountType, …> +4 entries with bg/icon classes.
- `apps/frontend/src/pages/insights/portfolio-insights.tsx` — synthetic Account
  stub: `balance: 0` → `currentBalance: undefined`.
- `apps/frontend/src/pages/holdings/holdings-page.tsx` — same stub rename.
- `apps/frontend/src/pages/holdings/components/holdings-mobile-filter-sheet.tsx`
  — same stub rename.
- `apps/frontend/src/pages/activity/components/activity-data-grid/activity-utils.test.ts`
  — mock fixture `balance: 10000` → `currentBalance: "10000"`.
- `apps/frontend/src/pages/dashboard/accounts-summary.test.tsx` — mock fixture
  `balance` field renamed.
- `packages/addon-sdk/src/data-types.ts` — AccountType const +4 keys; Account
  interface mirrors frontend (currentBalance + 10 new fields).

## Decisions Made

- **Use `Icons.Building` for LOAN, not `Icons.Building2`.** Plan referenced
  `Icons.Building2` but the @whaleit/ui Icons namespace exposes the lucide
  `Building2` icon under the name `Building` (icons.tsx:280
  `Building: Building2`). Type-check enforced this discovery.
- **Update `addon-sdk/src/data-types.ts` to mirror frontend AccountType +
  Account shape.** addon-sdk has its own duplicate type definitions;
  type-bridge.ts performs structural assignments between the two. Without
  mirroring, the AccountType extension would have broken the bridge with cryptic
  Promise<Account> incompatibility errors.
- **Use `void _exhaustive;` for the never-guard.** tsconfig has
  `noUnusedLocals: true`. The standard alternatives are `// @ts-expect-error`
  (unsafe — masks any future error) or referencing the binding.
  `void _exhaustive;` is a one-line zero-cost reference that preserves the
  exhaustiveness compile-time check.
- **`currentBalance` typed as `string` (not `number`).** Plan explicitly
  mandated this for parity with the wire DTO (rust_decimal serialized as
  string). On frontend, formatting is delegated to existing currency display
  helpers.
- **Use `currentBalance: undefined` for synthetic Portfolio Account stubs.**
  Field is optional in the Account interface, so dropping the entire key would
  also work — but keeping the explicit `undefined` makes the intent (no balance
  for the synthetic aggregator row) self-documenting.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wrong icon name `Icons.Building2`**

- **Found during:** Task 1 follow-up (consumer fix)
- **Issue:** Plan specified `Icons.Building2`, but the icon is exposed under
  `Icons.Building` (icons.tsx:280 maps `Building` → lucide `Building2`).
- **Fix:** Used `Icons.Building` consistently in app-launcher, account-page,
  account-item, account-selector, account-selector-mobile.
- **Files modified:** Same as Files Created/Modified.
- **Verification:** `pnpm --filter frontend type-check` clean.
- **Committed in:** `e071e9c1` and `75d935ca`.

**2. [Rule 3 - Blocking] Additional Record<AccountType, ...> maps (not in
plan)**

- **Found during:** Task 1 type-check
- **Issue:** AccountType extension required ALL `Record<AccountType, ...>`
  consumers to list 7 keys, not 3. Three additional maps existed beyond the two
  selectors: app-launcher.tsx, account-page.tsx, account-item.tsx.
- **Fix:** Added entries for CHECKING / SAVINGS / CREDIT_CARD / LOAN in each map
  (Wallet / Coins / CreditCard / Building, with sensible bg/icon class colors in
  account-item.tsx — blue/green/purple/red).
- **Verification:** type-check clean after edits.
- **Committed in:** `e071e9c1`.

**3. [Rule 3 - Blocking] addon-sdk AccountType + Account out of sync**

- **Found during:** Task 1 type-check (type-bridge.ts errors)
- **Issue:** packages/addon-sdk/src/data-types.ts had its own duplicate
  AccountType (3 variants) and Account (`balance: number`). Frontend → SDK
  assignments in apps/frontend/src/addons/type-bridge.ts became structurally
  incompatible.
- **Fix:** Extended addon-sdk AccountType const (added 4 variants) and Account
  interface (renamed balance → currentBalance, added 10 new fields). Rebuilt
  addon-sdk dist.
- **Verification:** type-check clean across whole frontend after rebuild.
- **Committed in:** `e071e9c1` (Task 1) and `97b9fed2` (Task 2 currentBalance
  update).

**4. [Rule 3 - Blocking] Four additional `balance: 0` Portfolio Account stubs**

- **Found during:** Task 2 sanity grep + type-check
- **Issue:** Plan listed 3 known consumers of Account.balance, but a grep found
  4 more synthetic Account stubs using `balance: 0` (constants.ts
  createPortfolioAccount, portfolio-insights.tsx, holdings-page.tsx,
  holdings-mobile-filter-sheet.tsx) and 2 mock fixtures in test files
  (activity-utils.test.ts, accounts-summary.test.tsx).
- **Fix:** Renamed all to `currentBalance: undefined` (or
  `currentBalance: "10000"` for the test fixture that wanted a value).
- **Verification:** type-check clean; impacted vitest files (51 tests) all pass.
- **Committed in:** `97b9fed2`.

---

**Total deviations:** 4 auto-fixed (all Rule 3 — blocking issues caused by the
plan's intentional schema change rippling into more consumers than the plan
enumerated). **Impact on plan:** All auto-fixes were mechanically necessary to
satisfy `tsc --noEmit` after the AccountType extension and
balance→currentBalance rename. No scope creep — every change traces to the
plan's stated breaking changes.

## Issues Encountered

- **Stale UI / addon-sdk dist files at session start.** `node_modules` was
  missing → had to run `pnpm install`. After install, frontend type-check failed
  with TS6305 (output file not built from source) errors pointing at
  `packages/ui/dist/` and `packages/addon-sdk/dist/`. Resolved by running
  `pnpm --filter @whaleit/ui build` and `pnpm --filter @whaleit/addon-sdk build`
  once each. After the addon-sdk schema change, ran the addon-sdk build a second
  time to refresh the dist before re-running frontend type-check.
- **Plan's TDD test command syntax glitch.**
  `pnpm --filter frontend test -- --run <path>` doesn't filter to a single file
  — vitest still runs the whole suite. Used
  `pnpm --filter frontend exec vitest --run <path>` instead, which actually
  filters.

## TDD Gate Compliance

Plan flagged Tasks 1 and 3 as `tdd="true"`. Both followed strict RED → GREEN
sequence with separate commits:

- **Task 1:** RED `654dbe5b` (test: 6 failing tests) → GREEN `09aabd58` (feat:
  implementation).
- **Task 3:** RED `ec944600` (test: 4 failing tests in newAccountSchema describe
  block; 2 of the 6 happened to pass before changes because the original schema
  permitted the input) → GREEN `7ef14415` (feat: superRefine).

No REFACTOR commit needed — both implementations were direct transcriptions of
the plan's source listings.

## User Setup Required

None — no external service configuration introduced.

## Next Phase Readiness

- Plan 03-03 (settings/accounts UI) can now import the extended `Account`
  interface and rely on `accountKind` / `defaultGroupForAccountType` for
  grouping decisions.
- Plan 03-06 (account edit form) has the schema with CC-gated validation ready.
- Plan 03-07 / 03-07b (CC-specific UI) have all 11 Phase 3 fields available.
- Plan 03-08 (unified accounts list) has the icon maps + label switch
  primitives.
- Backend mirror (Plan 03-04, also Wave 1) lands the Rust side; this plan
  deliberately adopts the same camelCase + decimal-as-string conventions so
  Phase 3 frontend and backend are wire-compatible.

## Self-Check: PASSED

All claims verified:

**Files created (1/1 found):**

- `apps/frontend/src/lib/constants.test.ts` — FOUND.

**Files modified (16/16 confirmed via git log):** All listed files appear in the
diff between `e80eb753` (base) and HEAD `75d935ca`.

**Commits exist (7/7 found via `git log e80eb753..HEAD`):**

- `654dbe5b` — FOUND
- `09aabd58` — FOUND
- `e071e9c1` — FOUND
- `97b9fed2` — FOUND
- `ec944600` — FOUND
- `7ef14415` — FOUND
- `75d935ca` — FOUND

**Tests:**

- `pnpm --filter frontend exec vitest --run` → 518/518 tests pass across 43
  files.
- `pnpm --filter frontend type-check` → exits 0.

---

_Phase: 03-bank-accounts-credit-cards_ _Plan: 05_ _Completed: 2026-04-25_
