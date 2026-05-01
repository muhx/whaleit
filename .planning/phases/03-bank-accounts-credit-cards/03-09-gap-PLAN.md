---
phase: 03-bank-accounts-credit-cards
plan: 09
type: execute
wave: 5
depends_on: ["03-07", "03-07b"]
gap_closure: true
files_modified:
  - apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
  - apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx
autonomous: true
requirements: [ACCT-04]
requirements_addressed: [ACCT-04, ACCT-06, ACCT-07]
threats_addressed: []
tags: [gap-closure, frontend, regression, h-01, accounts-edit]

must_haves:
  truths:
    - "Opening the edit dialog on an existing CHECKING/SAVINGS/LOAN account
      pre-fills institution + openingBalance from the account prop"
    - "Opening the edit dialog on an existing CREDIT_CARD account pre-fills
      institution, openingBalance, creditLimit, statementCycleDay, and any of
      statementBalance / minimumPayment / statementDueDate / rewardPointsBalance
      / cashbackBalance that are set on the account"
    - "AccountEditModal default-values seed flows from the `account` prop
      directly — no scavenger-hunt through hooks or context"
    - "A regression vitest case (component-level) asserts that the edit modal
      renders with creditLimit and openingBalance values pre-filled from a CC
      account fixture"
  artifacts:
    - path: "apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx"
      provides:
        "Edit modal that pre-fills all 9 Phase 3 fields from the account prop,
        unblocking ACCT-04 edit flow for CHECKING / SAVINGS / CREDIT_CARD / LOAN"
      contains: "openingBalance: account?.openingBalance"
    - path: "apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx"
      provides:
        "Vitest regression test asserting edit modal pre-fills CC fields from
        account prop (negative test for H-01)"
      contains: "edit dialog pre-fills"
  key_links:
    - from: "apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx"
      to: "apps/frontend/src/pages/settings/accounts/components/account-form.tsx"
      via: "<AccountForm defaultValues={defaultValues} />"
      pattern: "defaultValues=\\{defaultValues\\}"
    - from: "apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx"
      to: "apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx"
      via:
        "rendering the real AccountEditModal (not the mocked stub) for the
        regression case"
      pattern: "AccountEditModal"
---

<objective>
Close gap H-01 from 03-VERIFICATION.md: AccountEditModal omits all 9 Phase 3
fields from `defaultValues`, blocking submit on every existing
CHECKING / SAVINGS / CREDIT_CARD / LOAN account because
`newAccountSchema.superRefine` requires `openingBalance` for those types.

Purpose: Restore ACCT-04 edit flow to "fully verified". Database-level
historical preservation already works; this plan fixes the UX layer that
prevents users from submitting edits at all.

Output: One file modified (account-edit-modal.tsx) and one component-level
regression test added to accounts-page.test.tsx that asserts the modal pre-
fills CC fields from the account prop. Both changes are surgical (no refactoring
of unrelated code, per CLAUDE.md "Surgical Changes"). </objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/03-bank-accounts-credit-cards/03-CONTEXT.md
@.planning/phases/03-bank-accounts-credit-cards/03-VERIFICATION.md
@.planning/phases/03-bank-accounts-credit-cards/03-REVIEW.md
@.planning/phases/03-bank-accounts-credit-cards/03-07-SUMMARY.md
@.planning/phases/03-bank-accounts-credit-cards/03-07b-SUMMARY.md

@apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
@apps/frontend/src/pages/settings/accounts/components/account-form.tsx
@apps/frontend/src/lib/types/account.ts @apps/frontend/src/lib/schemas.ts
@apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx

<interfaces>
<!-- Key contracts the executor needs. Pre-extracted from the codebase so
     the executor does not have to scavenger-hunt. -->

From apps/frontend/src/lib/types/account.ts (lines 7-36) — the Account
interface, including the 9 Phase 3 fields the modal must forward:

```typescript
export interface Account {
  id: string;
  name: string;
  accountType: AccountType;
  group?: string;
  currentBalance?: string; // Decimal-as-string (NUMERIC DTO)
  currency: string;
  isDefault: boolean;
  isActive: boolean;
  isArchived: boolean;
  trackingMode: TrackingMode;
  createdAt: Date;
  updatedAt: Date;
  platformId?: string;
  accountNumber?: string;
  meta?: string;
  provider?: string;
  providerAccountId?: string;
  // Phase 3 additions (D-06, D-11, D-12, D-18):
  institution?: string;
  openingBalance?: string;
  balanceUpdatedAt?: Date;
  creditLimit?: string;
  statementCycleDay?: number;
  statementBalance?: string;
  minimumPayment?: string;
  statementDueDate?: string; // ISO date
  rewardPointsBalance?: number;
  cashbackBalance?: string;
}
```

From apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
(current state, lines 14-37) — the buggy defaultValues object that omits 9 Phase
3 fields:

```typescript
export function AccountEditModal({ account, open, onClose }: AccountEditModalProps) {
  const { settings } = useSettingsContext();

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
  };

  return (
    <Dialog open={open} onOpenChange={onClose} useIsMobile={useIsMobileViewport}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-[625px]">
        <AccountForm defaultValues={defaultValues} onSuccess={onClose} />
      </DialogContent>
    </Dialog>
  );
}
```

From apps/frontend/src/lib/schemas.ts (lines 100-164) — the superRefine that
REQUIRES openingBalance for bank/CC/LOAN. This is what blocks submit when the
modal omits the field; the fix is to seed the value, not to soften the schema:

```typescript
.superRefine((data, ctx) => {
  const isCC = data.accountType === "CREDIT_CARD";
  const isBankOrLoan =
    data.accountType === "CHECKING" ||
    data.accountType === "SAVINGS" ||
    data.accountType === "LOAN";
  // ... (D-06 CC-only-fields rule on non-CC types) ...
  if ((isBankOrLoan || isCC) && !data.openingBalance) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      path: ["openingBalance"],
      message: "Opening balance is required for this account type.",
    });
  }
});
```

From apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx (current
state, lines 32-34) — the existing test mock of AccountEditModal that has to be
REMOVED (or scoped to specific tests) so the regression test can render the real
modal:

```typescript
vi.mock("./components/account-edit-modal", () => ({
  AccountEditModal: () => null,
}));
```

</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add regression vitest case asserting edit modal pre-fills CC fields (RED)</name>

<read_first> - apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx
(full file — understand existing mocks, fixtures, and test layout) -
apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
(current buggy state) -
apps/frontend/src/pages/settings/accounts/components/account-form.tsx (lines
1-120 — to confirm what label / input contracts the form exposes for
openingBalance and creditLimit) - apps/frontend/src/lib/types/account.ts
(Account interface) - apps/frontend/src/lib/schemas.ts (lines 78-164 —
newAccountSchema, to understand how field input types map to form values)
</read_first>

  <files>
    apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx
  </files>

  <behavior>
    - Test 1 (RED initially, GREEN after Task 2 lands):
      * Render AccountEditModal directly (not via the mocked stub) with a CC
        Account fixture where:
          - accountType: AccountType.CREDIT_CARD
          - institution: "Chase Bank"
          - openingBalance: "0"
          - creditLimit: "5000"
          - statementCycleDay: 15
          - currentBalance: "1000"
      * Assert that the rendered form contains an Opening balance input whose
        value reflects "0" (or visible numeric "0" / "0.00" — accept any
        zero-equivalent string the MoneyInput renders).
      * Assert that the rendered form contains a Credit limit input whose
        value reflects "5000".
      * Assert that the rendered form contains a Statement cycle day input/
        select whose value reflects 15.
      * Assert that the rendered form contains an Institution input whose
        value reflects "Chase Bank".
    - Test 2 (RED initially, GREEN after Task 2 lands):
      * Render AccountEditModal with a CHECKING Account fixture where:
          - accountType: AccountType.CHECKING
          - institution: "Wells Fargo"
          - openingBalance: "1234.56"
      * Assert Opening balance input reflects "1234.56".
      * Assert Institution input reflects "Wells Fargo".
  </behavior>

  <action>
    Add a new `describe("AccountEditModal pre-fill regression (H-01)", ...)`
    block to apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx.

    Mechanics — DO NOT touch the existing top-level mocks. They must keep
    working for the existing `SettingsAccountsPage` describe block. Instead,
    inside the new describe block:

    1. At the top of the file, REMOVE the blanket
       `vi.mock("./components/account-edit-modal", ...)` mock and replace it
       with a scoped re-mock pattern: import the real `AccountEditModal`
       inside the new describe block AFTER calling
       `vi.unmock("./components/account-edit-modal")`. If `vi.unmock()` is
       not viable from inside a describe (vitest hoists `vi.mock`), instead:
         (a) Delete the blanket `vi.mock("./components/account-edit-modal", ...)`
             from the top of the file (it is a noop today — the existing 5
             tests do not assert anything about the modal being rendered).
         (b) Confirm the existing 5 tests still pass (they should — they
             never depended on the modal being null; they test grouping and
             archive toggle, which only render row links).
         (c) If any existing test newly fails due to the modal mounting in
             the page, scope the unmock by importing AccountEditModal directly
             in the new describe block and rendering it standalone (not via
             SettingsAccountsPage).

    2. PREFERRED IMPLEMENTATION (recommended): render `AccountEditModal`
       directly in the regression block. This avoids any interaction with
       the existing page-level mocks and is the cleanest scope:

       ```tsx
       describe("AccountEditModal pre-fill regression (H-01)", () => {
         it("pre-fills CC fields from the account prop", () => {
           const ccAccount = makeAccount({
             id: "cc-1",
             name: "Amex Gold",
             accountType: AccountType.CREDIT_CARD,
             institution: "Chase Bank",
             openingBalance: "0",
             creditLimit: "5000",
             statementCycleDay: 15,
             currentBalance: "1000",
           });
           render(
             <MemoryRouter>
               <AccountEditModal account={ccAccount} open onClose={() => undefined} />
             </MemoryRouter>,
           );
           // Assert pre-filled fields. Use display-value queries (accessible
           // labels) so the assertions are robust against MoneyInput's
           // internal Number.parseFloat normalization (per L-07 in REVIEW).
           expect(screen.getByLabelText(/Institution/i)).toHaveValue("Chase Bank");
           // openingBalance flows through MoneyInput → Number.parseFloat → number.
           // Accept either "0" or "0.00" or 0; the safe assertion is that the
           // input is NOT empty / NOT undefined.
           const openingInput = screen.getByLabelText(/Opening balance/i) as HTMLInputElement;
           expect(openingInput.value).not.toBe("");
           const creditLimitInput = screen.getByLabelText(/Credit limit/i) as HTMLInputElement;
           // creditLimit: "5000" → MoneyInput renders 5000
           expect(Number.parseFloat(creditLimitInput.value)).toBe(5000);
           const cycleDayInput = screen.getByLabelText(/Statement cycle day/i) as HTMLInputElement;
           // statementCycleDay: 15 → renders 15
           expect(Number.parseInt(cycleDayInput.value, 10)).toBe(15);
         });

         it("pre-fills CHECKING institution + openingBalance from the account prop", () => {
           const checkingAccount = makeAccount({
             id: "chk-1",
             name: "Daily Spending",
             accountType: AccountType.CHECKING,
             institution: "Wells Fargo",
             openingBalance: "1234.56",
           });
           render(
             <MemoryRouter>
               <AccountEditModal account={checkingAccount} open onClose={() => undefined} />
             </MemoryRouter>,
           );
           expect(screen.getByLabelText(/Institution/i)).toHaveValue("Wells Fargo");
           const openingInput = screen.getByLabelText(/Opening balance/i) as HTMLInputElement;
           expect(Number.parseFloat(openingInput.value)).toBeCloseTo(1234.56, 2);
         });
       });
       ```

    3. Imports the new block needs (add at top of file alongside existing
       imports — `AccountEditModal` import is the only NEW import; the rest
       already exist):
       ```ts
       import { AccountEditModal } from "./components/account-edit-modal";
       ```
       (Note: the existing `vi.mock("./components/account-edit-modal", ...)`
       at the top of the file MUST be deleted or this import will return the
       mock. Delete it; the existing 5 tests do not depend on the mock.)

    4. The `makeAccount` helper at line 127 already accepts overrides for
       creditLimit, currentBalance, and (per the existing `...overrides`
       spread on line 142) any extra Account fields, including the Phase 3
       fields institution, openingBalance, statementCycleDay, etc. No helper
       changes needed.

    5. After writing the tests, run them once and CONFIRM they FAIL with the
       expected RED signal: "expected '' to equal 'Chase Bank'" or similar
       (because the buggy modal omits these fields → form receives undefined
       → inputs are empty). DO NOT proceed to Task 2 until RED is observed.
       Commit the failing tests as
       `test(03-09): add H-01 regression tests for AccountEditModal pre-fill`.

    Do NOT modify @whaleit/ui mocks unless you confirm via test failure that
    they are blocking the assertions (they should not be — the existing
    Input mock at line 120 forwards all props, and the FormLabel + label
    relationship is preserved by the form library).

    If the @whaleit/ui mock for ResponsiveSelect or MoneyInput swallows the
    `value` prop in a way that breaks the assertion, augment ONLY the
    affected mock to forward `value` to a real `<input>` (the existing Input
    mock pattern at line 120 is the template). Do not refactor the broader
    mock surface.

    Reference existing code: `accounts-page.test.tsx:127-144` (makeAccount
    helper), `account-form.tsx:97-308` (which fields render under what type
    gate). Gap reason: H-01 — defaultValues object in account-edit-modal.tsx
    omits all 9 Phase 3 fields.

  </action>

  <verify>
    <automated>
      pnpm --filter frontend test -- --run accounts-page.test.tsx 2>&1 | grep -E "(FAIL|✗|H-01 regression)"
    </automated>
  </verify>

<acceptance_criteria> - [ ]
`apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` contains a
`describe("AccountEditModal pre-fill regression (H-01)", ...)` block. Verify
with
`grep -c "AccountEditModal pre-fill regression" apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`
→ output must be 1. - [ ] The describe block contains at least 2 `it(...)` cases
(one for CREDIT_CARD, one for CHECKING). Verify with
`grep -cE "^\s*it\(" apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`
→ must be ≥ 7 (5 existing + 2 new). - [ ] Running
`pnpm --filter frontend test -- --run accounts-page.test.tsx` produces FAILING
tests for the new H-01 regression block (RED signal). Capture the failure output
in commit message body. - [ ] The pre-existing 5 `SettingsAccountsPage` tests
STILL PASS (the test ledger went from 5/5 green to 5/5 green + 0/2 new red).
Verify by counting passing tests in stdout. - [ ] No changes to any file other
than `accounts-page.test.tsx`. - [ ] Commit message:
`test(03-09): add H-01 regression tests for AccountEditModal pre-fill` (use
`cargo`-equivalent of conventional-commits format; this is the RED commit per
TDD pattern). </acceptance_criteria>

  <done>
    Two new vitest cases exist and FAIL with assertion errors that name the
    Phase 3 fields (institution / openingBalance / creditLimit /
    statementCycleDay). The 5 pre-existing tests continue to pass. RED
    commit landed; ready for GREEN in Task 2.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Forward 9 Phase 3 fields into AccountEditModal defaultValues (GREEN)</name>

<read_first> -
apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
(current buggy state — lines 14-29) - apps/frontend/src/lib/types/account.ts
(Account interface, lines 26-35) - apps/frontend/src/lib/schemas.ts (lines
78-164 — newAccountSchema input/output types — confirm the form expects each
field with the same name and type as the Account interface) -
apps/frontend/src/pages/settings/accounts/components/account-form.tsx (lines
64-92 — defaultValues consumption shape) </read_first>

  <files>
    apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx
  </files>

  <action>
    Edit `defaultValues` object (currently lines 17-29) to add the 9 Phase 3
    fields, sourced directly from the `account` prop. Final shape:

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

    Per CLAUDE.md "Surgical Changes":
    - DO NOT add `balanceUpdatedAt` to defaultValues (it is a server-only
      field per D-12 and is being explicitly removed from the inbound DTO
      by Plan 03-10; it is not user-editable).
    - DO NOT touch any other code in the file. The Dialog wrapper, prop
      destructuring, and import block stay as-is.
    - DO NOT add `// TODO` markers, defensive fallbacks (`?? ""`,
      `?? undefined`, etc.) beyond what is shown above. The Account
      interface declares each new field as `T | undefined`; the form's
      schema accepts undefined for each (zod `.optional()`); RHF
      `defaultValues: undefined` for an optional field is the correct
      "unset" state. If the existing line uses `?? ""`, mirror that style
      only where the field's display widget cannot tolerate `undefined`
      (none of the 9 Phase 3 fields require this — verified against the
      MoneyInput / Input / DatePickerInput / ResponsiveSelect call sites in
      account-form.tsx lines 225-308).

    After the edit, re-run the regression tests from Task 1. They MUST
    transition from RED to GREEN. The pre-existing 5 tests must continue
    to pass.

    Reference existing code: REVIEW.md H-01 §"Suggested fix" provides the
    exact spread pattern. The `account-form.tsx` consumes
    `defaultValues?.openingBalance` at line 88 via the spread
    `{ ...defaultValues, trackingMode: ... }` so no form-side change is
    needed.

    Gap reason: H-01 — closing it restores ACCT-04 (edit) for
    CHECKING / SAVINGS / CREDIT_CARD / LOAN.

  </action>

  <verify>
    <automated>
      pnpm --filter frontend test -- --run accounts-page.test.tsx 2>&1 | tail -20 &&
      pnpm --filter frontend type-check
    </automated>
  </verify>

<acceptance_criteria> - [ ]
`apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
contains all 9 of the literal lines: `institution: account?.institution,`,
`openingBalance: account?.openingBalance,`,
`creditLimit: account?.creditLimit,`,
`statementCycleDay: account?.statementCycleDay,`,
`statementBalance: account?.statementBalance,`,
`minimumPayment: account?.minimumPayment,`,
`statementDueDate: account?.statementDueDate,`,
`rewardPointsBalance: account?.rewardPointsBalance,`,
`cashbackBalance: account?.cashbackBalance,`. Verify with:
`grep -cE "(institution|openingBalance|creditLimit|statementCycleDay|statementBalance|minimumPayment|statementDueDate|rewardPointsBalance|cashbackBalance): account\?\." apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
→ output ≥ 9. - [ ] The file does NOT add `balanceUpdatedAt:` to defaultValues.
Verify:
`grep -c "balanceUpdatedAt" apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
→ must be 0. - [ ] No imports added or removed. Verify:
`git diff apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
shows only inserted lines inside the `defaultValues` object literal, no new
`import` lines. - [ ]
`pnpm --filter frontend test -- --run accounts-page.test.tsx` passes for ALL
tests (existing 5 + new 2). Capture green test count. - [ ]
`pnpm --filter frontend type-check` exits 0. - [ ] Commit message:
`fix(03-09): forward Phase 3 fields into AccountEditModal defaultValues (H-01)`
— body cites the gap and the test transition RED → GREEN. </acceptance_criteria>

  <done>
    Edit dialog on existing CHECKING / SAVINGS / CREDIT_CARD / LOAN accounts
    now pre-fills all 9 Phase 3 fields from the account prop. The H-01
    regression tests from Task 1 transition RED → GREEN. ACCT-04 edit flow
    is restored. Type-check is clean. ROADMAP SC-4 is fully satisfied.
  </done>
</task>

</tasks>

<verification>
After both tasks land, the gap closure is verified by:

1. **Test count delta:**
   `pnpm --filter frontend test -- --run accounts-page.test.tsx` → ≥ 7 tests
   pass (5 existing + 2 new H-01 regression). RED → GREEN transition documented
   in Task 2's commit body.

2. **Type-check:** `pnpm --filter frontend type-check` → exit 0. No new TS
   errors from the defaultValues additions.

3. **Grep audits:**
   - `grep -cE "(institution|openingBalance|creditLimit|statementCycleDay|statementBalance|minimumPayment|statementDueDate|rewardPointsBalance|cashbackBalance): account\?\." apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
     ≥ 9.
   - `grep -c "balanceUpdatedAt" apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
     == 0 (server-only per D-12).
   - `grep -c "AccountEditModal pre-fill regression" apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`
     == 1.

4. **No collateral damage:** `git diff --stat HEAD~2` (or however many commits
   this plan added) shows ONLY 2 files modified:
   - `apps/frontend/src/pages/settings/accounts/components/account-edit-modal.tsx`
   - `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`
     </verification>

<human_verification> The following verification items from 03-VERIFICATION.md
`human_verification` section CANNOT be automated by this plan and remain pending
after completion:

1. **E2E spec run on a clean host.**
   `npx playwright test e2e/11-accounts.spec.ts` against a fresh PG database.
   Blocked by port 8088 / OrbStack on the verifier's host. Recipe: see
   03-08-SUMMARY.md "Run command for the next CI cycle".

2. **PG integration tests.** `cargo test -p whaleit-storage-postgres accounts`
   with `DATABASE_URL` set. Blocked by missing PG instance in the verifier
   environment.

3. **Visual fidelity check.** Compare `/settings/accounts` against UI-SPEC §1 +
   §6 (Available credit chip, Progress bar utilization color tier). Pixel-level
   visual checks are out of unit/E2E scope.

4. **Manual edit-flow smoke test.** Open the edit dialog on an existing CHECKING
   account, then on an existing CREDIT_CARD account, and confirm:
   - Institution + Opening balance pre-fill
   - Credit limit + Statement cycle day pre-fill on the CC dialog
   - Submit succeeds without re-entering data

   This task closes the source-level fix; the unit-level regression test from
   Task 1 covers the structural assertion. The user-visible smoke test remains a
   human verification item. </human_verification>

<success_criteria>

- ACCT-04 edit flow is unblocked for all 4 new account types (CHECKING, SAVINGS,
  CREDIT_CARD, LOAN). Verified via Task 1 + Task 2 regression tests
  transitioning RED → GREEN.
- 03-VERIFICATION.md gap H-01 status moves from `partial` to `verified` on next
  `/gsd-verify-work` run (separate concern; this plan delivers the fix that the
  next verification reads).
- No regression in pre-existing 5 vitest cases on accounts-page.test.tsx.
- No new TS errors. Type-check clean.
- Two atomic commits in git log (TDD RED + GREEN). </success_criteria>

<output>
After completion, create
`.planning/phases/03-bank-accounts-credit-cards/03-09-SUMMARY.md` following
the standard summary template. Include:

- TDD cycle commits (test commit + fix commit)
- Test count delta (5 → 7 in accounts-page.test.tsx)
- The exact `defaultValues` shape that landed
- Which gap (H-01) was closed and the ACCT-\* requirement restored
- Confirmation that human-verification items 1-4 above remain open and routed to
  the next verifier run / human smoke test </output>
