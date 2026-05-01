---
phase: 03-bank-accounts-credit-cards
plan: 07b
type: execute
wave: 3
depends_on: ["03-07"]
files_modified:
  - apps/frontend/src/pages/settings/accounts/accounts-page.tsx
  - apps/frontend/src/pages/settings/accounts/components/account-item.tsx
  - apps/frontend/src/pages/account/account-page.tsx
  - apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx
autonomous: true
requirements: [ACCT-03, ACCT-04, ACCT-05]
requirements_addressed: [ACCT-03, ACCT-04, ACCT-05]
threats_addressed: [T-3-05]
tags:
  [frontend, ui, react, settings, list, account-page, available-credit, archive]

must_haves:
  truths:
    - "/settings/accounts shows a group-by axis (Banking / Credit Cards / Loans
      / Investments / Cash / Crypto / Uncategorized) using account.group ??
      defaultGroupForAccountType"
    - "Show-archived Switch toggles archived rows on/off; archived hidden by
      default"
    - "Account row for CREDIT_CARD shows an 'Available credit' chip computed
      from creditLimit - currentBalance"
    - "Account detail page renders CC-specific Credit overview / Statement
      snapshot / Rewards sections when accountType === CREDIT_CARD"
    - "Account detail page hides investment-only modules (HistoryChart,
      AccountHoldings, AccountMetrics, AccountContributionLimit) for CHECKING /
      SAVINGS / LOAN, showing only a Balance card"
    - "accounts-page.test.tsx covers group-by ordering, archived toggle
      behavior, and CC available-credit chip rendering"
  artifacts:
    - path: "apps/frontend/src/pages/settings/accounts/accounts-page.tsx"
      provides: "Extended host route with group-by + show-archived Switch"
      contains: "defaultGroupForAccountType"
    - path: "apps/frontend/src/pages/settings/accounts/components/account-item.tsx"
      provides: "Available credit chip on CC rows"
      contains: "Available"
    - path: "apps/frontend/src/pages/account/account-page.tsx"
      provides:
        "CC sections + bank Balance card + render-gating by account_kind"
      contains: "Credit overview"
    - path: "apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx"
      provides:
        "Vitest + React Testing Library coverage of group-by, show-archived, and
        available-credit chip behavior"
      contains: "Show archived"
  key_links:
    - from: "accounts-page.tsx group-by reduction"
      to: "apps/frontend/src/lib/constants.ts defaultGroupForAccountType"
      via: "account.group ?? defaultGroupForAccountType(account.accountType)"
      pattern: "defaultGroupForAccountType"
    - from: "account-item.tsx Available credit chip"
      to: "apps/frontend/src/pages/settings/accounts/credit-helpers.ts"
      via: "import { availableCredit }"
      pattern: "availableCredit"
    - from: "account-page.tsx CC sections"
      to: "apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx"
      via: "import + onClick={() => setBalanceModalOpen(true)}"
      pattern: "UpdateBalanceModal"
---

<objective>
Land the consumer half of the Phase 3 frontend: extend the unified
`/settings/accounts` host page with group-by + Show-archived Switch, surface
the Available credit chip on CC rows, extend the per-account detail page with
CC-specific sections (Credit overview / Statement snapshot / Rewards) and a
simple Balance card for CHECKING / SAVINGS / LOAN, and create the
accounts-page.test.tsx unit-test file that VALIDATION.md and PATTERNS.md
already reference as a Wave 0 deliverable.

Purpose: This plan consumes Plan 03-07's credit-helpers.ts,
update-balance-modal, and account-form extensions. It also closes the ACCT-03
frontend test gap by adding the accounts-page.test.tsx coverage that
VALIDATION.md row "renders all types" and "archive toggles" already point to.

Output: Updated accounts-page.tsx + account-item.tsx + account-page.tsx, NEW
accounts-page.test.tsx with Vitest + React Testing Library coverage.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/03-bank-accounts-credit-cards/03-CONTEXT.md
@.planning/phases/03-bank-accounts-credit-cards/03-RESEARCH.md
@.planning/phases/03-bank-accounts-credit-cards/03-PATTERNS.md
@.planning/phases/03-bank-accounts-credit-cards/03-UI-SPEC.md
@.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md
@.planning/phases/03-bank-accounts-credit-cards/03-05-PLAN.md
@.planning/phases/03-bank-accounts-credit-cards/03-06-PLAN.md
@.planning/phases/03-bank-accounts-credit-cards/03-07-PLAN.md
@apps/frontend/src/pages/settings/accounts/accounts-page.tsx
@apps/frontend/src/pages/settings/accounts/components/account-item.tsx
@apps/frontend/src/pages/account/account-page.tsx
@apps/frontend/src/pages/dashboard/accounts-summary.test.tsx
@apps/frontend/src/lib/constants.ts
@.claude/skills/react-useeffect/SKILL.md
@.agents/skills/frontend-design/SKILL.md

<interfaces>
<!-- Existing host page structure, accounts-page.tsx lines 24-296 -->

```typescript
const SettingsAccountsPage = () => {
  const { accounts, isLoading } = useAccounts({ filterActive: false, includeArchived: true });
  // ...
  const [filter, setFilter] = useState<FilterType>("all");
  // search + filter chain produces filteredAccounts
  // useMemo yields { activeAccounts, inactiveAccounts }
  // ...
  return ( <SettingsHeader /> + filter ToggleGroup + AccountItem rows + AccountEditModal );
};
```

<!-- Existing account-page render structure, account-page.tsx ~lines 80-707 -->
<!-- (large file — gates investment-vs-other rendering by accountType already, in places) -->

<!-- AccountKind helper from Plan 03-05 + helpers from Plan 03-07 -->

```typescript
import {
  accountKind,
  AccountKind,
  AccountType,
  defaultGroupForAccountType,
} from "@/lib/constants";
import {
  availableCredit,
  utilizationPercent,
  utilizationTier,
} from "@/pages/settings/accounts/credit-helpers";
```

<!-- Test analog from accounts-summary.test.tsx (lines 1-180): mock pattern with vi.mock for hooks + ui modules -->

</interfaces>

<constraints>
- D-15 (amended): unified list lives inside /settings/accounts, NOT a new /accounts route.
  No new routes added in routes.tsx. Existing search + active/archived/hidden
  filter UI stays.
- D-19 (amended): archived hidden by default. The existing settings page calls
  `useAccounts({ filterActive: false, includeArchived: true })` and applies a
  local filter — keep this. Add a Switch UI labeled "Show archived" that sets
  the local filter to include archived rows. Default state = OFF.
- UI-SPEC §3 CC detail sections must follow the contract: Credit overview card
  (Balance + Available credit + Utilization with color ramp + Limit + Update
  balance button), Statement snapshot card with Edit button, Rewards card.
- UI-SPEC §4 Bank/LOAN detail page: hide investment-only modules
  (HistoryChart / AccountHoldings / AccountMetrics / AccountContributionLimit)
  and show only a Balance card with Update balance action.
- UI-SPEC color discipline: only existing semantic tokens (primary,
  destructive, success, warning, muted, etc). No new tokens.
- DO NOT introduce new shadcn primitives — every component is already in @whaleit/ui.
- DO NOT install or pull blocks from @animate-ui or @diceui (UI-SPEC §Registry Safety).
- For useEffect: if any new useEffect is needed, follow .claude/skills/react-useeffect/SKILL.md.
- Plan 03-07 lands credit-helpers.ts; this plan IMPORTS from it.
- VALIDATION.md and PATTERNS.md already reference
  `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` as a Wave 0
  deliverable — Task 3 below is the file that closes the gap.
</constraints>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Extend account-item.tsx with Available credit chip + extend accounts-page.tsx with group-by + Show archived Switch</name>
  <files>
    apps/frontend/src/pages/settings/accounts/components/account-item.tsx,
    apps/frontend/src/pages/settings/accounts/accounts-page.tsx
  </files>
  <read_first>
    - apps/frontend/src/pages/settings/accounts/components/account-item.tsx (full file — esp. lines 127-138 archived chip pattern)
    - apps/frontend/src/pages/settings/accounts/accounts-page.tsx (full file)
    - apps/frontend/src/pages/settings/accounts/credit-helpers.ts (Plan 03-07 Task 1)
    - .planning/phases/03-bank-accounts-credit-cards/03-UI-SPEC.md §1 "Unified Account List" + §6 "Archive / unarchive interaction"
    - .planning/phases/03-bank-accounts-credit-cards/03-PATTERNS.md §"apps/frontend/src/pages/settings/accounts/accounts-page.tsx (page, HOST)" + §"...account-item.tsx"
  </read_first>
  <action>
    1. `account-item.tsx`: Add an "Available credit" chip rendered next to the
       existing archived chip (around line 127-138 — the existing
       `account.isArchived &&` block). Add AFTER it:

       ```tsx
       {account.accountType === "CREDIT_CARD" && account.creditLimit && (() => {
         const avail = availableCredit(account.creditLimit, account.currentBalance);
         return avail !== undefined ? (
           <span className="inline-flex items-center gap-1 rounded-md border border-border bg-success/10 px-2 py-1 text-xs text-success">
             <Icons.CreditCard className="h-3 w-3" />
             Available {new Intl.NumberFormat(undefined, {
               style: "currency",
               currency: account.currency,
             }).format(avail)}
           </span>
         ) : null;
       })()}
       ```

       Add the import at the top:
       `import { availableCredit } from "../credit-helpers";`

       Color discipline: per UI-SPEC §Color, use `bg-success/10 text-success`
       semantic tokens (NOT `emerald-*`).

    2. `accounts-page.tsx`: Add group-by behavior + show-archived Switch.

       a. Add to imports (verify exact module paths against existing imports):

          ```typescript
          import { Switch } from "@whaleit/ui";
          import { defaultGroupForAccountType } from "@/lib/constants";
          ```

       b. Add a new state hook near the existing `useState<FilterType>("all")`:

          ```typescript
          const [showArchived, setShowArchived] = useState(false);
          ```

       c. Modify the existing filter chain so that when the local FilterType
          is "all" AND `showArchived` is false, archived rows are hidden.
          Locate the existing `filteredAccounts` derivation (search for
          `filteredAccounts` or for the existing `filter` ToggleGroup logic)
          and add a final `.filter` step:

          ```typescript
          const visibleAccounts = useMemo(
            () => (showArchived ? filteredAccounts : filteredAccounts.filter((a) => !a.isArchived)),
            [filteredAccounts, showArchived],
          );
          ```

          Replace any downstream usage of `filteredAccounts` for rendering
          rows with `visibleAccounts`. (KEEP the existing useMemo
          `{ activeAccounts, inactiveAccounts }` derivation — this plan layers
          group-by on top of that, see step (e).)

       d. Add a `<Switch>` UI element near the existing search/filter row
          (place it inline with the existing ToggleGroup):

          ```tsx
          <div className="flex items-center gap-2">
            <Switch
              id="show-archived"
              checked={showArchived}
              onCheckedChange={setShowArchived}
              aria-describedby="show-archived-desc"
            />
            <label htmlFor="show-archived" className="text-sm">
              Show archived
            </label>
            <span id="show-archived-desc" className="sr-only">
              Reveal accounts you've set aside
            </span>
          </div>
          ```

       e. Add a group-by reduction over `visibleAccounts`. Below the existing
          `activeAccounts / inactiveAccounts` memo, add:

          ```typescript
          const groups = useMemo(() => {
            const groupOrder = ["Banking", "Credit Cards", "Loans", "Investments", "Cash", "Crypto", "Uncategorized"];
            const buckets = new Map<string, typeof visibleAccounts>();
            for (const acc of visibleAccounts) {
              const groupName = acc.group ?? defaultGroupForAccountType(acc.accountType);
              const key = groupName ?? "Uncategorized";
              const list = buckets.get(key) ?? [];
              list.push(acc);
              buckets.set(key, list);
            }
            // Sort buckets by groupOrder, then alpha for any custom group names.
            return groupOrder
              .filter((g) => buckets.has(g))
              .map((g) => ({ name: g, accounts: buckets.get(g)! }))
              .concat(
                [...buckets.keys()]
                  .filter((g) => !groupOrder.includes(g))
                  .sort()
                  .map((g) => ({ name: g, accounts: buckets.get(g)! })),
              );
          }, [visibleAccounts]);
          ```

       f. In the JSX, render the rows grouped by `groups` (replace the existing
          flat `activeAccounts.map(...)` block with a per-group iteration):

          ```tsx
          {groups.map((g) => (
            <div key={g.name} className="space-y-2">
              <h3 className="text-sm text-muted-foreground">
                {g.name} · {g.accounts.length} {g.accounts.length === 1 ? "account" : "accounts"}
              </h3>
              {g.accounts.map((acc) => (
                <AccountItem
                  key={acc.id}
                  account={acc}
                  platform={acc.platformId ? platformMap.get(acc.platformId) : null}
                  onEdit={() => handleEditAccount(acc)}
                  onDelete={() => deleteAccountMutation.mutate(acc.id)}
                />
              ))}
            </div>
          ))}
          ```

          IF the existing JSX has a more elaborate row-rendering shape (e.g.,
          inactiveAccounts section), preserve it — fold the group rendering
          inside the existing active/inactive layers as appropriate. The
          minimal acceptance criterion is that group headers and ordering work
          for active accounts.

    Per `.claude/skills/react-useeffect/SKILL.md`: derived state (groups,
    visibleAccounts) is computed via useMemo, NOT useEffect. The Switch is a
    controlled input bound to setShowArchived directly.

  </action>
  <verify>
    <automated>grep -c "showArchived\|defaultGroupForAccountType\|availableCredit" apps/frontend/src/pages/settings/accounts/accounts-page.tsx apps/frontend/src/pages/settings/accounts/components/account-item.tsx | grep -q "[1-9]" && pnpm --filter frontend type-check 2>&1 | tail -3 | grep -E "successfully|error"</automated>
  </verify>
  <acceptance_criteria>
    - `account-item.tsx` imports `availableCredit` from `../credit-helpers`
    - `account-item.tsx` contains the literal `Available ` (the chip prefix)
    - `account-item.tsx` chip uses `bg-success/10` and `text-success` (semantic tokens)
    - `accounts-page.tsx` contains `const [showArchived, setShowArchived] = useState(false);`
    - `accounts-page.tsx` contains the literal `Show archived`
    - `accounts-page.tsx` imports `defaultGroupForAccountType` from `@/lib/constants`
    - `accounts-page.tsx` group order array contains the literal `"Banking", "Credit Cards", "Loans", "Investments", "Cash", "Crypto", "Uncategorized"`
    - `accounts-page.tsx` does NOT add a new useEffect (derived state via useMemo only)
    - `pnpm --filter frontend type-check` exits 0
  </acceptance_criteria>
  <done>Settings/accounts page groups by account.group / defaultGroupForAccountType, archived hidden by default with Show archived Switch, CC rows display Available credit chip. type-check passes.</done>
</task>

<task type="auto">
  <name>Task 2: Extend account-page.tsx with CC sections + bank/loan Balance card + Update balance trigger</name>
  <files>apps/frontend/src/pages/account/account-page.tsx</files>
  <read_first>
    - apps/frontend/src/pages/account/account-page.tsx (full file — large, read once)
    - apps/frontend/src/pages/settings/accounts/components/update-balance-modal.tsx (Plan 03-07 Task 2)
    - apps/frontend/src/pages/settings/accounts/credit-helpers.ts (Plan 03-07 Task 1)
    - apps/frontend/src/lib/constants.ts (after Plan 03-05 — accountKind + AccountType)
    - .planning/phases/03-bank-accounts-credit-cards/03-UI-SPEC.md §3 "Account detail page — CC extensions" (full layouts) + §4 "Bank / LOAN detail page" + §5 "Update balance modal"
    - .planning/phases/03-bank-accounts-credit-cards/03-PATTERNS.md §"apps/frontend/src/pages/account/account-page.tsx"
    - .claude/skills/react-useeffect/SKILL.md (this file is large; verify no new useEffect additions)
  </read_first>
  <action>
    Open `apps/frontend/src/pages/account/account-page.tsx`. The file is ~707 lines.
    Key edits:

    1. Add imports:

       ```typescript
       import { accountKind, AccountKind, AccountType } from "@/lib/constants";
       import { UpdateBalanceModal } from "@/pages/settings/accounts/components/update-balance-modal";
       import { availableCredit, utilizationPercent, utilizationTier } from "@/pages/settings/accounts/credit-helpers";
       ```

    2. Add state for the Update balance modal near the top of the component:

       ```typescript
       const [balanceModalOpen, setBalanceModalOpen] = useState(false);
       ```

    3. Add a render-gating helper near the component body:

       ```typescript
       const isCreditCard = account?.accountType === AccountType.CREDIT_CARD;
       const isInvestment = account ? accountKind(account.accountType) === AccountKind.INVESTMENT : false;
       const isLiabilityOrAsset = account ? !isInvestment : false; // CHECKING/SAVINGS/CASH/CC/LOAN
       ```

    4. For each existing investment-only module render block (HistoryChart,
       AccountHoldings, AccountMetrics, AccountContributionLimit), wrap in
       `{isInvestment && ...}`. Search the file for these component names —
       there are existing render sites that need gating. (If this file already
       has gating logic for some modules, preserve it; layer the new check.)

    5. Add a new Balance card rendered when `isLiabilityOrAsset && !isCreditCard`
       (i.e., CHECKING / SAVINGS / CASH / LOAN). Place it inside the same section
       container that previously held investment modules:

       ```tsx
       {account && isLiabilityOrAsset && !isCreditCard && (
         <Card>
           <CardHeader>
             <CardTitle>Balance</CardTitle>
           </CardHeader>
           <CardContent className="space-y-3">
             <div>
               <p className="text-xs text-muted-foreground">Current balance</p>
               <PrivacyAmount value={account.currentBalance ? Number.parseFloat(account.currentBalance) : 0} currency={account.currency} />
             </div>
             {account.balanceUpdatedAt && (
               <p className="text-xs text-muted-foreground">
                 Last updated: {new Date(account.balanceUpdatedAt).toLocaleDateString()}
               </p>
             )}
             <Button onClick={() => setBalanceModalOpen(true)}>Update balance</Button>
           </CardContent>
         </Card>
       )}
       ```

    6. Add the CC sections rendered when `isCreditCard`. UI-SPEC §3 specifies
       three cards in order: Credit overview, Statement snapshot, Rewards.
       Insert all three inside `{account && isCreditCard && (<> ... </>)}`:

       ```tsx
       {account && isCreditCard && (
         <>
           {/* Credit overview */}
           <Card>
             <CardHeader>
               <CardTitle>Credit overview</CardTitle>
             </CardHeader>
             <CardContent className="space-y-3">
               <div className="grid grid-cols-2 gap-4">
                 <div>
                   <p className="text-xs text-muted-foreground">Balance</p>
                   <PrivacyAmount value={account.currentBalance ? Number.parseFloat(account.currentBalance) : 0} currency={account.currency} />
                 </div>
                 <div>
                   <p className="text-xs text-muted-foreground">Available credit</p>
                   {(() => {
                     const avail = availableCredit(account.creditLimit, account.currentBalance);
                     return avail !== undefined ? (
                       <PrivacyAmount value={avail} currency={account.currency} />
                     ) : (
                       <span className="text-xs text-muted-foreground">—</span>
                     );
                   })()}
                 </div>
               </div>
               {(() => {
                 const pct = utilizationPercent(account.creditLimit, account.currentBalance);
                 const tier = utilizationTier(pct);
                 return pct !== undefined ? (
                   <div>
                     <p className="text-xs text-muted-foreground">Utilization {pct}%</p>
                     <Progress value={pct} className={tier === "destructive" ? "[&>div]:bg-destructive" : tier === "warning" ? "[&>div]:bg-warning" : "[&>div]:bg-success"} aria-label="Credit utilization" aria-valuenow={pct} />
                   </div>
                 ) : null;
               })()}
               <div className="flex items-center justify-between">
                 {account.creditLimit && (
                   <p className="text-xs text-muted-foreground">
                     Limit <PrivacyAmount value={Number.parseFloat(account.creditLimit)} currency={account.currency} />
                   </p>
                 )}
                 <Button onClick={() => setBalanceModalOpen(true)}>Update balance</Button>
               </div>
             </CardContent>
           </Card>

           {/* Statement snapshot */}
           <Card>
             <CardHeader>
               <CardTitle>Statement snapshot</CardTitle>
             </CardHeader>
             <CardContent className="space-y-3">
               {account.statementBalance || account.minimumPayment || account.statementDueDate ? (
                 <>
                   <div className="grid grid-cols-2 gap-4">
                     <div>
                       <p className="text-xs text-muted-foreground">Statement balance</p>
                       {account.statementBalance ? (
                         <PrivacyAmount value={Number.parseFloat(account.statementBalance)} currency={account.currency} />
                       ) : "—"}
                     </div>
                     <div>
                       <p className="text-xs text-muted-foreground">Minimum payment</p>
                       {account.minimumPayment ? (
                         <PrivacyAmount value={Number.parseFloat(account.minimumPayment)} currency={account.currency} />
                       ) : "—"}
                     </div>
                   </div>
                   {account.statementDueDate && (
                     <div>
                       <p className="text-xs text-muted-foreground">Due date</p>
                       <p>{new Date(account.statementDueDate).toLocaleDateString()}</p>
                     </div>
                   )}
                 </>
               ) : (
                 <p className="text-sm text-muted-foreground">No statement recorded yet.</p>
               )}
             </CardContent>
           </Card>

           {/* Rewards */}
           <Card>
             <CardHeader>
               <CardTitle>Rewards</CardTitle>
             </CardHeader>
             <CardContent className="space-y-3">
               {account.rewardPointsBalance !== undefined || account.cashbackBalance ? (
                 <div className="grid grid-cols-2 gap-4">
                   <div>
                     <p className="text-xs text-muted-foreground">Points balance</p>
                     <p>{account.rewardPointsBalance !== undefined ? `${account.rewardPointsBalance.toLocaleString()} pts` : "—"}</p>
                   </div>
                   <div>
                     <p className="text-xs text-muted-foreground">Cashback balance</p>
                     {account.cashbackBalance ? (
                       <PrivacyAmount value={Number.parseFloat(account.cashbackBalance)} currency={account.currency} />
                     ) : "—"}
                   </div>
                 </div>
               ) : (
                 <p className="text-sm text-muted-foreground">No rewards balance tracked.</p>
               )}
             </CardContent>
           </Card>
         </>
       )}
       ```

    7. Render the modal once near the end of the component:

       ```tsx
       {account && balanceModalOpen && (
         <UpdateBalanceModal
           account={account}
           open={balanceModalOpen}
           onClose={() => setBalanceModalOpen(false)}
         />
       )}
       ```

    8. Verify imports of `Card`, `CardHeader`, `CardTitle`, `CardContent`,
       `Button`, `PrivacyAmount`, `Progress` already exist or add them from
       `@whaleit/ui`.

    DO NOT remove any existing functionality (FIRE planner integration,
    investment-mode rendering, etc). The new rendering is layered on top with
    `isInvestment` / `isCreditCard` / `isLiabilityOrAsset` gates.
    DO NOT add useEffect. All derivation is via the helpers in credit-helpers.ts.

  </action>
  <verify>
    <automated>grep -c "isCreditCard\|UpdateBalanceModal\|Credit overview\|Statement snapshot\|Rewards" apps/frontend/src/pages/account/account-page.tsx | grep -q "^[5-9]\\|^[1-9][0-9]" && pnpm --filter frontend type-check 2>&1 | tail -3 | grep -E "successfully|error"</automated>
  </verify>
  <acceptance_criteria>
    - File contains the literal `accountKind(account.accountType) === AccountKind.INVESTMENT`
    - File contains `<CardTitle>Credit overview</CardTitle>`
    - File contains `<CardTitle>Statement snapshot</CardTitle>`
    - File contains `<CardTitle>Rewards</CardTitle>`
    - File contains `<UpdateBalanceModal`
    - File contains `availableCredit(account.creditLimit, account.currentBalance)`
    - File contains `utilizationPercent(account.creditLimit, account.currentBalance)`
    - File contains `aria-label="Credit utilization"`
    - File does NOT add new `useEffect` calls compared to its pre-task state (verify via `git diff` showing no new `useEffect(` lines)
    - `pnpm --filter frontend type-check` exits 0
  </acceptance_criteria>
  <done>Account detail page renders CC-specific sections + bank Balance card + Update balance modal trigger, gated by accountKind / accountType. type-check passes.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Create accounts-page.test.tsx (Vitest + React Testing Library) covering group-by, archive toggle, and CC chip</name>
  <files>apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx</files>
  <read_first>
    - apps/frontend/src/pages/dashboard/accounts-summary.test.tsx (full file — mock pattern reference for vi.mock + RTL render + screen queries)
    - apps/frontend/src/pages/settings/accounts/accounts-page.tsx (after Task 1 of THIS plan)
    - apps/frontend/src/pages/settings/accounts/credit-helpers.ts (Plan 03-07 Task 1)
    - apps/frontend/src/lib/types/account.ts (after Plan 03-05)
    - .planning/phases/03-bank-accounts-credit-cards/03-PATTERNS.md §"apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx (test, NEW)"
    - .planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md row "/settings/accounts renders all account types with current_balance"
  </read_first>
  <behavior>
    - Test 1: Renders all six AccountType groups when accounts of each type exist (Banking, Credit Cards, Loans, Investments, Cash, Crypto). Group headers appear in canonical order.
    - Test 2: Group-by ordering — Banking precedes Credit Cards precedes Loans precedes Investments precedes Cash precedes Crypto.
    - Test 3: Archived accounts are hidden by default — an archived row's name is NOT in the document.
    - Test 4: "Show archived" Switch reveals archived rows — after toggling, the archived row's name IS in the document.
    - Test 5: Available-credit chip appears only on CREDIT_CARD rows — chip text "Available" is found near the CC row but NOT near a CHECKING row.
  </behavior>
  <action>
    Create `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`.

    1. Use the same mock pattern as `apps/frontend/src/pages/dashboard/accounts-summary.test.tsx`:
       - `vi.mock("@/hooks/use-accounts", ...)` to inject deterministic accounts.
       - `vi.mock("@whaleit/ui", ...)` to stub PrivacyAmount, Switch, Button, etc.
       - `vi.mock("@whaleit/ui/components/ui/icons", ...)` to stub Icons.
       - `vi.mock("@tanstack/react-query", ...)` to stub useQuery for platforms.
       - `vi.mock("./components/use-account-mutations", ...)` to stub mutations.

    2. Skeleton:

       ```tsx
       import { render, screen, within } from "@testing-library/react";
       import userEvent from "@testing-library/user-event";
       import { MemoryRouter } from "react-router-dom";
       import { beforeEach, describe, expect, it, vi } from "vitest";
       import { useAccounts } from "@/hooks/use-accounts";
       import type { Account, TrackingMode } from "@/lib/types";
       import { AccountType } from "@/lib/constants";
       import { useQuery } from "@tanstack/react-query";
       import SettingsAccountsPage from "./accounts-page";

       vi.mock("@/hooks/use-accounts", () => ({
         useAccounts: vi.fn(),
       }));

       vi.mock("@tanstack/react-query", async (importActual) => {
         const actual = await importActual<typeof import("@tanstack/react-query")>();
         return {
           ...actual,
           useQuery: vi.fn(),
         };
       });

       vi.mock("./components/use-account-mutations", () => ({
         useAccountMutations: () => ({
           deleteAccountMutation: { mutate: vi.fn() },
           updateAccountMutation: { mutate: vi.fn() },
         }),
       }));

       vi.mock("@whaleit/ui", () => ({
         Button: ({ children, ...rest }: React.ButtonHTMLAttributes<HTMLButtonElement>) => (
           <button {...rest}>{children}</button>
         ),
         EmptyPlaceholder: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
         Icons: new Proxy(
           {},
           { get: () => () => <span /> },
         ),
         Separator: () => <hr />,
         Skeleton: () => <div>loading</div>,
         Switch: ({ checked, onCheckedChange, id, ...rest }: { checked: boolean; onCheckedChange: (v: boolean) => void; id?: string }) => (
           <input
             type="checkbox"
             role="switch"
             id={id}
             checked={checked}
             onChange={(e) => onCheckedChange(e.target.checked)}
             {...rest}
           />
         ),
         ToggleGroup: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
         ToggleGroupItem: ({ children, value }: { children?: React.ReactNode; value?: string }) => (
           <button data-value={value}>{children}</button>
         ),
         PrivacyAmount: ({ value, currency }: { value: number; currency: string }) => (
           <span>{`${currency}:${value}`}</span>
         ),
       }));

       vi.mock("@whaleit/ui/components/ui/input", () => ({
         Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
       }));

       const mockUseAccounts = vi.mocked(useAccounts);
       const mockUseQuery = vi.mocked(useQuery);

       function makeAccount(overrides: Partial<Account>): Account {
         return {
           id: overrides.id ?? "acc-1",
           name: overrides.name ?? "Acc 1",
           accountType: overrides.accountType ?? AccountType.SECURITIES,
           group: overrides.group,
           currency: overrides.currency ?? "USD",
           isDefault: false,
           isActive: true,
           isArchived: overrides.isArchived ?? false,
           trackingMode: ("TRANSACTIONS" as TrackingMode),
           createdAt: new Date("2026-01-01T00:00:00Z"),
           updatedAt: new Date("2026-01-01T00:00:00Z"),
           creditLimit: overrides.creditLimit,
           currentBalance: overrides.currentBalance,
           ...overrides,
         } as Account;
       }

       beforeEach(() => {
         mockUseQuery.mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<typeof useQuery>);
       });

       function renderPage() {
         return render(
           <MemoryRouter>
             <SettingsAccountsPage />
           </MemoryRouter>,
         );
       }

       describe("SettingsAccountsPage", () => {
         it("renders all six AccountType groups when accounts of each type exist", () => {
           mockUseAccounts.mockReturnValue({
             accounts: [
               makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
               makeAccount({ id: "2", name: "Amex Gold", accountType: AccountType.CREDIT_CARD, creditLimit: "5000", currentBalance: "1000" }),
               makeAccount({ id: "3", name: "Mortgage", accountType: AccountType.LOAN }),
               makeAccount({ id: "4", name: "Brokerage", accountType: AccountType.SECURITIES }),
               makeAccount({ id: "5", name: "Wallet Cash", accountType: AccountType.CASH }),
               makeAccount({ id: "6", name: "BTC Wallet", accountType: AccountType.CRYPTOCURRENCY }),
             ],
             isLoading: false,
           } as ReturnType<typeof useAccounts>);

           renderPage();

           expect(screen.getByText(/Banking/)).toBeInTheDocument();
           expect(screen.getByText(/Credit Cards/)).toBeInTheDocument();
           expect(screen.getByText(/Loans/)).toBeInTheDocument();
           expect(screen.getByText(/Investments/)).toBeInTheDocument();
           expect(screen.getByText(/Cash/)).toBeInTheDocument();
           expect(screen.getByText(/Crypto/)).toBeInTheDocument();
         });

         it("orders groups Banking -> Credit Cards -> Loans -> Investments -> Cash -> Crypto", () => {
           mockUseAccounts.mockReturnValue({
             accounts: [
               makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
               makeAccount({ id: "2", name: "Amex Gold", accountType: AccountType.CREDIT_CARD, creditLimit: "5000" }),
               makeAccount({ id: "3", name: "Mortgage", accountType: AccountType.LOAN }),
               makeAccount({ id: "4", name: "Brokerage", accountType: AccountType.SECURITIES }),
               makeAccount({ id: "5", name: "Wallet Cash", accountType: AccountType.CASH }),
               makeAccount({ id: "6", name: "BTC Wallet", accountType: AccountType.CRYPTOCURRENCY }),
             ],
             isLoading: false,
           } as ReturnType<typeof useAccounts>);

           const { container } = renderPage();
           const text = container.textContent ?? "";
           const idxBanking = text.indexOf("Banking");
           const idxCC = text.indexOf("Credit Cards");
           const idxLoans = text.indexOf("Loans");
           const idxInv = text.indexOf("Investments");
           const idxCash = text.indexOf("Cash");
           const idxCrypto = text.indexOf("Crypto");

           expect(idxBanking).toBeGreaterThan(-1);
           expect(idxBanking).toBeLessThan(idxCC);
           expect(idxCC).toBeLessThan(idxLoans);
           expect(idxLoans).toBeLessThan(idxInv);
           expect(idxInv).toBeLessThan(idxCash);
           expect(idxCash).toBeLessThan(idxCrypto);
         });

         it("hides archived accounts by default", () => {
           mockUseAccounts.mockReturnValue({
             accounts: [
               makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
               makeAccount({ id: "2", name: "Old Card", accountType: AccountType.CREDIT_CARD, isArchived: true, creditLimit: "1000" }),
             ],
             isLoading: false,
           } as ReturnType<typeof useAccounts>);

           renderPage();

           expect(screen.getByText("Chase Checking")).toBeInTheDocument();
           expect(screen.queryByText("Old Card")).not.toBeInTheDocument();
         });

         it("reveals archived accounts when Show archived toggle is on", async () => {
           mockUseAccounts.mockReturnValue({
             accounts: [
               makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
               makeAccount({ id: "2", name: "Old Card", accountType: AccountType.CREDIT_CARD, isArchived: true, creditLimit: "1000" }),
             ],
             isLoading: false,
           } as ReturnType<typeof useAccounts>);

           renderPage();

           const user = userEvent.setup();
           const showArchived = screen.getByRole("switch", { name: /show archived/i });
           await user.click(showArchived);

           expect(screen.getByText("Old Card")).toBeInTheDocument();
         });

         it("shows Available credit chip on CREDIT_CARD rows but not on CHECKING rows", () => {
           mockUseAccounts.mockReturnValue({
             accounts: [
               makeAccount({ id: "1", name: "Chase Checking", accountType: AccountType.CHECKING }),
               makeAccount({ id: "2", name: "Amex Gold", accountType: AccountType.CREDIT_CARD, creditLimit: "5000", currentBalance: "1000" }),
             ],
             isLoading: false,
           } as ReturnType<typeof useAccounts>);

           const { container } = renderPage();
           const txt = container.textContent ?? "";

           // Chip prefix appears (matches the "Available {amount}" pattern).
           expect(txt).toMatch(/Available/);
           // The chip text should be near the CC row name, not the checking row.
           const ccIdx = txt.indexOf("Amex Gold");
           const chkIdx = txt.indexOf("Chase Checking");
           const availIdx = txt.indexOf("Available");
           expect(ccIdx).toBeGreaterThan(-1);
           expect(availIdx).toBeGreaterThan(-1);
           // Chip is in the same group as the CC, so its index sits between Credit Cards
           // header and the next group header. Sanity: not adjacent to the CHECKING row.
           expect(Math.abs(availIdx - ccIdx)).toBeLessThan(Math.abs(availIdx - chkIdx));
         });
       });
       ```

    3. Tolerance / adaptation rules:
       - If `SettingsAccountsPage` is exported as a named export instead of
         default, update the import accordingly.
       - If a mock module path differs (e.g., `@/lib/types` vs `@/lib/constants`
         for `AccountType`), align with the actual project layout.
       - The test file MUST exit 0 with `pnpm --filter frontend test -- --run`.

    4. Update VALIDATION.md row "/settings/accounts renders all account types"
       with Plan ID `03-07b`, Task ID `3`, Status `✅` after the test goes green.

  </action>
  <verify>
    <automated>test -f apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx && grep -c "describe\|it(" apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx | grep -q "^[5-9]\\|^[1-9][0-9]" && pnpm --filter frontend test -- --run apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx 2>&1 | tail -10 | grep -E "passed|failed"</automated>
  </verify>
  <acceptance_criteria>
    - File `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` exists
    - File contains 5 `it(...)` blocks under a `describe("SettingsAccountsPage", ...)` block
    - File contains the literal `Show archived` (matched in a getByRole switch query)
    - File contains the literal `Available` (matched in a chip query)
    - File contains literal group names: `Banking`, `Credit Cards`, `Loans`, `Investments`, `Cash`, `Crypto`
    - File imports `userEvent` from `@testing-library/user-event`
    - File mocks `@/hooks/use-accounts` via `vi.mock`
    - `pnpm --filter frontend test -- --run apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` exits 0 with all 5 tests passing
    - Update `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` Per-Task Verification Map row "/settings/accounts renders all account types" with Plan ID `03-07b`, Task ID `3`, Status `✅`
  </acceptance_criteria>
  <done>accounts-page.test.tsx covers group-by ordering, archive default-hide, archive reveal via Switch, and CC chip rendering. All 5 tests pass.</done>
</task>

</tasks>

<threat_model>

## Trust Boundaries

| Boundary                            | Description                                      |
| ----------------------------------- | ------------------------------------------------ |
| Render gate by accountKind          | Pure compute, no external trust                  |
| accounts-page Switch + filter chain | Local state; useAccounts hook is source of truth |

## STRIDE Threat Register

| Threat ID | Category                                   | Component                           | Disposition | Mitigation Plan                                                                                                                                                                        |
| --------- | ------------------------------------------ | ----------------------------------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-3-05    | Information Disclosure (TOCTOU on archive) | accounts-page Switch + filter chain | mitigate    | Show-archived defaults to `false`. The hook (`useAccounts`) is source of truth; the local filter just narrows what is shown. No client-side caching of "archived" state across mounts. |

</threat_model>

<verification>
- `pnpm --filter frontend type-check` exits 0
- `pnpm --filter frontend test -- --run apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` exits 0
- All UI-SPEC §1, §3, §4, §6 layouts render in dev (manual smoke per Plan 03-08)
- Account list groups by canonical order with Show-archived Switch
- Account detail page shows CC sections for CREDIT_CARD, Balance card for CHECKING/SAVINGS/LOAN
</verification>

<success_criteria>

1. accounts-page.tsx groups rows by account.group ?? defaultGroupForAccountType,
   in the canonical group order, with a "Show archived" Switch defaulting to
   off.
2. account-item.tsx shows Available credit chip on CC rows using semantic
   success token.
3. account-page.tsx renders Credit overview / Statement snapshot / Rewards for
   CC accounts and a single Balance card for CHECKING / SAVINGS / LOAN, gated by
   accountKind.
4. accounts-page.test.tsx has 5 passing tests covering group-by ordering,
   archive default-hide, archive Switch reveal, and CC chip rendering.
5. `pnpm --filter frontend type-check` exits 0.
6. ACCT-03 frontend test gap is closed — VALIDATION.md row "/settings/accounts
   renders all account types" points to a real test file. </success_criteria>

<output>
After completion, create `.planning/phases/03-bank-accounts-credit-cards/03-07b-SUMMARY.md`
</output>
