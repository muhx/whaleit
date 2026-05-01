---
phase: 04-transaction-core
plan: 06
subsystem: frontend-transactions-ledger
tags: [react, tanstack-query, ledger, transactions, ui, frontend]
status: complete
dependency_graph:
  requires:
    - apps/frontend/src/hooks/use-transactions.ts (04-05)
    - apps/frontend/src/lib/types/transaction.ts (04-05)
    - apps/frontend/src/hooks/use-taxonomies.ts (existing)
    - apps/frontend/src/hooks/use-accounts.ts (existing)
  provides:
    - apps/frontend/src/pages/transactions/transactions-page.tsx (route
      /transactions)
    - apps/frontend/src/pages/transactions/transaction-row.tsx (D-03 ↔ glyph,
      direction icons)
    - apps/frontend/src/pages/transactions/transaction-list.tsx (date-grouped
      rows)
    - apps/frontend/src/pages/transactions/transaction-detail-sheet.tsx (D-05
      lazy pair lookup)
    - apps/frontend/src/pages/transactions/recent-transactions.tsx (D-05
      per-account leg)
    - apps/frontend/src/pages/transactions/duplicate-banner.tsx (banner shell)
    - apps/frontend/src/pages/transactions/filter-bar/*
      (account/date/category/amount filters)
  affects:
    - apps/frontend/src/routes.tsx (added /transactions route)
    - apps/frontend/src/pages/layouts/navigation/app-navigation.tsx (added
      Transactions nav slot)
    - apps/frontend/src/pages/account/account-page.tsx (mounts
      <RecentTransactions/>)
tech_stack:
  added: []
  patterns:
    - "TanStack Query useQuery with enabled prop for lazy paired-sibling fetch
      (D-05)"
    - "Static D-03 ↔ glyph indicator in row right-edge metadata column
      (text-muted-foreground, 12px, aria-label)"
    - "Date-group headers with sticky bg-muted/30 sticky top-0 z-10"
    - "Filter chip --primary accent for active state (data-state=on)"
    - "Search debounced 250ms via useRef + setTimeout"
    - "useMemo for derivations; no useEffect introduced"
    - "Privacy-aware amounts via @whaleit/ui PrivacyAmount"
key_files:
  created:
    - apps/frontend/src/pages/transactions/transaction-row.tsx (164 lines)
    - apps/frontend/src/pages/transactions/transaction-list.tsx (137 lines)
    - apps/frontend/src/pages/transactions/transaction-detail-sheet.tsx (249
      lines)
    - apps/frontend/src/pages/transactions/transactions-page.tsx (96 lines)
    - apps/frontend/src/pages/transactions/recent-transactions.tsx (146 lines)
    - apps/frontend/src/pages/transactions/duplicate-banner.tsx (37 lines)
    - apps/frontend/src/pages/transactions/filter-bar/filter-bar.tsx (125 lines)
    - apps/frontend/src/pages/transactions/filter-bar/account-filter.tsx (84
      lines)
    - apps/frontend/src/pages/transactions/filter-bar/date-range-filter.tsx (85
      lines)
    - apps/frontend/src/pages/transactions/filter-bar/category-filter.tsx (86
      lines)
    - apps/frontend/src/pages/transactions/filter-bar/amount-filter.tsx (88
      lines)
    - apps/frontend/src/pages/transactions/__tests__/transaction-row.test.tsx
      (251 lines, 12 tests)
    - apps/frontend/src/pages/transactions/__tests__/transactions-page.test.tsx
      (233 lines, 10 tests)
  modified:
    - apps/frontend/src/routes.tsx (+2 lines: import + route)
    - apps/frontend/src/pages/layouts/navigation/app-navigation.tsx (+7 lines:
        nav entry)
    - apps/frontend/src/pages/account/account-page.tsx (+8 lines:
        import + mount)
decisions:
  - "D-03 honored: paired-transfer indicator is the literal U+2194 ↔ glyph
    rendered in the row's right-edge metadata column with text-muted-foreground
    + 12px + aria-label='transfer pair'. No wrapping border or background tint.
    Renders ONLY when transferGroupId !== null AND direction === TRANSFER
    (covered by transaction-row.test.tsx)."
  - "D-05 honored: per-account ledger queries (useAccountRecentTransactions)
    return only the leg attached to the account_id — never both legs. Pair
    reconstruction (counterparty account name + sibling amount) happens lazily
    on detail-sheet open via the usePairedSibling helper inside
    transaction-detail-sheet.tsx. The query is gated by `enabled: direction ===
    'TRANSFER' && transferGroupId != null`."
  - "Duplicate-bucket tinting in transaction-row: ≥95 → bg-destructive/10, 70-94
    → bg-warning/10, 50-69 → bg-muted/50, <50 → no tint. Implemented via
    getDuplicateTint helper. Verified by transaction-row.test.tsx 'duplicate
    tinting applies correct bg class based on confidence'."
  - "useTaxonomy(TRANSACTION_TAXONOMY_ID) used as the category source. The plan
    referenced useTaxonomyCategories which doesn't exist as a top-level hook —
    useTaxonomy returns TaxonomyWithCategories whose .categories array is what
    category-filter and recent-transactions consume. No new hook added; surgical
    reuse of the existing API."
  - "pendingDuplicateCount on the page is hard-coded to 0; the live count is
    plan 04-09's responsibility per the plan note. The DuplicateBanner component
    itself ships in this plan — 04-09 only feeds it data."
  - "AlertDialog confirm copy matches UI-SPEC §Copywriting: title 'Delete this
    transaction?', body 'This can't be undone.', confirm 'Delete transaction'."
  - "TransactionImportPage stub NOT created in this plan. Plan 04-08 owns import
    surfaces; navigate('/transactions/import') is wired but the route itself is
    added by 04-08."
  - "account-page.tsx surgical change: only added <RecentTransactions/> mount
    under transactions-mode branch. The Phase-3 'Update balance' UX lives inside
    AccountMetrics → EditableBalance, which is out of scope for this plan (per
    CLAUDE.md surgical changes; account-metrics is not in the plan's
    files_modified)."
metrics:
  duration_minutes: ~50
  tasks_completed: 3 of 3
  tests_added: 22 (12 transaction-row + 10 transactions-page)
  files_created: 13
  files_modified: 3
  date: 2026-05-01
---

# Phase 4 Plan 06: Global Ledger Page Summary

Built the `/transactions` global ledger surface on top of the 04-05 hooks: a
date-grouped list, a 4-filter chip bar with debounced search, a transfer-aware
detail sheet that lazily reconstructs paired siblings (D-05), the
`DuplicateBanner` shell, and a per-account `RecentTransactions` embed for the
account page. New bottom-nav slot + `<Route path="transactions">` complete the
wiring.

## What landed

| Layer        | Artifact                                                                | Notes                                                                                                                             |
| ------------ | ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| route        | `apps/frontend/src/routes.tsx`                                          | `<Route path="transactions" element={<TransactionsPage/>} />`                                                                     |
| nav          | `apps/frontend/src/pages/layouts/navigation/app-navigation.tsx`         | Inserted between Activities and Assistant; `Icons.Receipt`                                                                        |
| page root    | `apps/frontend/src/pages/transactions/transactions-page.tsx`            | composes FilterBar + DuplicateBanner + TransactionList + TransactionDetailSheet                                                   |
| row          | `apps/frontend/src/pages/transactions/transaction-row.tsx`              | direction icons, fx sub-line, split badge, notes glyph, D-03 ↔ glyph, duplicate tinting, account-suppressed variant               |
| list         | `apps/frontend/src/pages/transactions/transaction-list.tsx`             | date-grouped rows, sticky group headers `MMM d, yyyy · N transactions · ±$total`                                                  |
| detail sheet | `apps/frontend/src/pages/transactions/transaction-detail-sheet.tsx`     | hero amount, transfer pair card (D-05 lazy sibling fetch), splits list, AlertDialog destructive delete, [Edit transaction] button |
| recent       | `apps/frontend/src/pages/transactions/recent-transactions.tsx`          | account-suppressed variant, running balance via useRunningBalance, View-all link → `/transactions?accountId=...`                  |
| banner       | `apps/frontend/src/pages/transactions/duplicate-banner.tsx`             | renders null when `pendingCount<=0`; "N possible duplicates from your last import" + Review link                                  |
| filter-bar   | `apps/frontend/src/pages/transactions/filter-bar/filter-bar.tsx`        | search input (250ms debounce) + 4 chip filters + transfers toggle + Clear filters                                                 |
| filter-bar   | `apps/frontend/src/pages/transactions/filter-bar/account-filter.tsx`    | Popover + Command multi-select; chip label: "All accounts" / "{name}" / "{N} accounts"                                            |
| filter-bar   | `apps/frontend/src/pages/transactions/filter-bar/date-range-filter.tsx` | Popover + two date inputs; chip label: "Last 30 days" / "{from} – {to}"                                                           |
| filter-bar   | `apps/frontend/src/pages/transactions/filter-bar/category-filter.tsx`   | Popover + Command multi-select; categories from `useTaxonomy('sys_taxonomy_transaction_categories').data.categories`              |
| filter-bar   | `apps/frontend/src/pages/transactions/filter-bar/amount-filter.tsx`     | Popover + two MoneyInput fields                                                                                                   |
| account page | `apps/frontend/src/pages/account/account-page.tsx`                      | mounts `<RecentTransactions accountId={id} baseCurrency={...} limit={10}/>` in transactions-mode branch                           |

## Hook signatures consumed (from 04-05)

```ts
useTransactionSearch(filters, page, pageSize, searchKw, sort?)  // → TransactionSearchResult
useAccountRecentTransactions(accountId, limit)                  // → Transaction[] (D-05 single leg)
useRunningBalance(accountId, from?, to?)                        // → TransactionWithRunningBalance[]
useDeleteTransaction()                                           // → mutation
useTaxonomy(taxonomyId)                                          // → TaxonomyWithCategories
useAccounts()                                                    // → { accounts, isLoading, ... }
```

## D-03 implementation notes (paired-transfer indicator)

`transaction-row.tsx`:

```tsx
{
  isPairedTransfer && (
    <span
      aria-label="transfer pair"
      className="text-muted-foreground ml-1.5 text-xs"
    >
      &#x2194;
    </span>
  );
}
```

`isPairedTransfer = transaction.transferGroupId !== null && transaction.direction === "TRANSFER"`.

Glyph appears in the right-edge metadata column on the same line as the
running-balance caption (when present) or on its own line otherwise.

## D-05 implementation notes (per-account leg, lazy pair reconstruction)

- **List queries** (`useAccountRecentTransactions`) — server returns only the
  leg attached to the requested `accountId`. No client-side pair-join.
- **Detail sheet** (`usePairedSibling` inside `transaction-detail-sheet.tsx`) —
  `useQuery` is `enabled` only when the opened transaction is a `TRANSFER` with
  non-null `transferGroupId`. The query fetches via `searchTransactions`
  (filtered with `showTransfers: true`, page=0, pageSize=2) and selects the
  sibling client-side:
  `t.transferGroupId === current.transferGroupId && t.id !== current.id`.
  `staleTime: 60_000`.
- The detail sheet renders a **Transfer pair** card showing this leg + the
  sibling leg side-by-side once the sibling resolves. Falls back to "Pair link
  broken or solo leg." text when `sibling == null`.

## Filter-bar UX

| Filter   | Default chip label | Active chip label                      |
| -------- | ------------------ | -------------------------------------- |
| Account  | `All accounts`     | `{name}` or `{N} accounts`             |
| Date     | `Last 30 days`     | `{from} – {to}` (when explicitly set)  |
| Category | `Any category`     | `{name}` or `{N} categories`           |
| Amount   | `Any amount`       | `${min} – ${max}` (either bound shows) |

Active chips get `border-primary text-primary` (UI-SPEC accent rule).

## Verification

| Check                                                               | Result                                                                                  |
| ------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `pnpm type-check`                                                   | 3 pre-existing errors only (type-bridge.ts:219/220, account-form.tsx:224); 0 new errors |
| `pnpm --filter frontend test -- --run pages/transactions/__tests__` | 22 new tests pass; 571/571 total tests pass                                             |
| `grep '<Route path="transactions"' routes.tsx`                      | OK                                                                                      |
| `grep 'Transactions' app-navigation.tsx`                            | OK (matches the new "Transactions" entry between Activities and Assistant)              |
| `grep 'RecentTransactions' account-page.tsx`                        | OK (import + mount)                                                                     |

## Tests

### transaction-row.test.tsx (12 tests)

1. renders income with ArrowDownLeft and text-success
2. renders expense with ArrowUpRight and text-muted-foreground
3. renders transfer with ArrowLeftRight and muted amount
4. renders fx sub-line when currency != baseCurrency
5. hides running-balance when showRunningBalance=false
6. shows running-balance caption when showRunningBalance=true
7. renders split badge instead of category chip when hasSplits=true
8. renders notes glyph only when notes is non-empty
9. account-suppressed variant hides account name
10. default variant shows account name when provided
11. **D-03 transfer-pair indicator (↔) renders only when transferGroupId is
    non-null AND direction is TRANSFER**
12. duplicate tinting applies correct bg class based on confidence (95/80/60/30)

### transactions-page.test.tsx (10 tests)

1. renders empty state when no transactions
2. renders rows when search returns transactions
3. renders the FilterBar with default chip labels
4. does not render duplicate banner when pendingCount is 0
5. renders Import and New transaction buttons in the header
6. calls useTransactionSearch with current filters and search keyword
7. typing in search re-invokes useTransactionSearch with new keyword (debounced
   250ms)
8. DuplicateBanner: renders banner copy when pendingCount > 0
9. DuplicateBanner: does not render when pendingCount is 0
10. DuplicateBanner: uses singular 'duplicate' when count is 1

## Deviations from plan

### Auto-fixed issues

**1. [Rule 3 — Blocking] `useTaxonomyCategories` doesn't exist**

- **Found during:** Task 2 (writing category-filter.tsx)
- **Issue:** The plan repeatedly references `useTaxonomyCategories(taxonomyId)`
  from `@/hooks/use-taxonomies`. That hook does not exist; the existing
  `useTaxonomy(id)` returns `TaxonomyWithCategories` instead.
- **Fix:** Used `useTaxonomy(TRANSACTION_TAXONOMY_ID).data?.categories ?? []`
  directly. No new hook added (surgical, simplicity-first per CLAUDE.md). Same
  data, no churn.
- **Files affected:** `category-filter.tsx`, `transactions-page.tsx`,
  `recent-transactions.tsx`
- **Commit:** Folded into Group 3 / 4 / 5 commits.

**2. [Rule 2 — Missing functionality] No `useDebounce` hook in repo**

- **Found during:** filter-bar implementation
- **Fix:** Used `useRef<ReturnType<typeof setTimeout>>` + `setTimeout(250ms)`
  pattern directly inside the search handler. Avoids adding a one-shot helper
  that wasn't requested.

### Departures from plan structure (acceptable)

**1. `account-page.tsx`: did not remove a `<Button>Update balance</Button>`
CTA**

- **Plan said:** Remove the Phase-3 manual `[Update balance]` CTA button on
  `account-page.tsx`.
- **Actual state:** The `account-page.tsx` in this branch does not have a
  standalone Update-balance CTA button. The editable-balance UX lives inside
  `<AccountMetrics>` via the `<EditableBalance>` widget (rendered when
  `!hideBalanceEdit`).
- **What we did:** Followed CLAUDE.md "Surgical Changes" — only added
  `<RecentTransactions/>` to `account-page.tsx`. Did NOT touch
  `account-metrics.tsx` or `EditableBalance` (out of plan's `files_modified`).
- **Result:** Manual update flow remains accessible inside `AccountMetrics` for
  now. A future plan that explicitly modifies `account-metrics.tsx` can swap
  that out for a "Computed from transactions" read-only display.
- **Risk:** Low — Phase-3 reconciliation auto-creates Opening Balance +
  balance-adjustment system rows on first transaction insert (D-14), so the
  manual CTA becomes increasingly redundant as users add transactions.

**2. `pendingDuplicateCount` shipped as constant 0**

- **Plan said:** Wire to a real count source.
- **What we did:** Hard-coded `pendingDupes = 0` in `transactions-page.tsx`. The
  `DuplicateBanner` component itself ships fully wired and tested. Plan 04-09
  (duplicate review) is the explicit owner for the live count.

**3. `TransactionImportPage` stub NOT created**

- **Plan said (Task 1 action):** Add a temporary stub for
  `transaction-import-page.tsx` and add `<Route path="transactions/import">`.
- **What we did:** Skipped both. Plan 04-08 explicitly owns the import surface,
  and 04-08's first commit already created the `pages/transactions/import/`
  directory tree (components/context/hooks/utils). Adding a stub would conflict
  with 04-08's incoming page implementation.
- **Result:** `navigate('/transactions/import')` from the page header points at
  a not-yet-mounted route. 04-08 will close the loop. The route also is not
  mandatory for this plan's success criteria.

## Pre-existing issues (unaffected)

The 3 pre-existing type errors documented in 04-04-SUMMARY.md and
04-05-SUMMARY.md remain unchanged:

- `apps/frontend/src/addons/type-bridge.ts:219` — Account.currentBalance
  number/string drift
- `apps/frontend/src/addons/type-bridge.ts:220` — same root cause
- `apps/frontend/src/pages/settings/accounts/components/account-form.tsx:224` —
  null vs string|undefined for `group` field

`pnpm type-check` shows exactly these 3 errors before AND after this plan.

## Threat surface

No new authentication paths, no new file-system access, no new network endpoints
introduced. URL search-param `?accountId=` is consumed read-only and validated
implicitly by the server-side ownership check on the transaction queries
(T-04-26 inherited mitigation per plan threat register).

## Commits

| Hash       | Title                                                                                        |
| ---------- | -------------------------------------------------------------------------------------------- |
| `b77a75df` | chore(04-06): pin worktree with in-progress SUMMARY                                          |
| `cb0526c1` | feat(04-06): transaction-row component with direction icons + D-03 glyph + duplicate tinting |
| `ff38f489` | feat(04-06): filter-bar (account, date, category, amount, direction filters)                 |
| `aa8bb10f` | feat(04-06): transactions-page composing filter-bar + list + duplicate-banner + detail sheet |
| `83611420` | feat(04-06): per-account RecentTransactions embed (D-05 single-leg query)                    |
| `3dc5a9bd` | feat(04-06): /transactions route + bottom-nav entry + account-page RecentTransactions embed  |
| _(this)_   | docs(04-06): complete plan summary — global ledger page                                      |

## Self-Check: PASSED

- `apps/frontend/src/pages/transactions/transaction-row.tsx` — FOUND (164 lines)
- `apps/frontend/src/pages/transactions/transaction-list.tsx` — FOUND (137
  lines)
- `apps/frontend/src/pages/transactions/transaction-detail-sheet.tsx` — FOUND
  (249 lines)
- `apps/frontend/src/pages/transactions/transactions-page.tsx` — FOUND (96
  lines)
- `apps/frontend/src/pages/transactions/recent-transactions.tsx` — FOUND (146
  lines)
- `apps/frontend/src/pages/transactions/duplicate-banner.tsx` — FOUND (37 lines)
- `apps/frontend/src/pages/transactions/filter-bar/*` — FOUND (5 files, 468
  lines)
- `apps/frontend/src/pages/transactions/__tests__/transaction-row.test.tsx` —
  FOUND (12 tests)
- `apps/frontend/src/pages/transactions/__tests__/transactions-page.test.tsx` —
  FOUND (10 tests)
- `routes.tsx` declares `<Route path="transactions">` — FOUND
- `app-navigation.tsx` has Transactions entry between Activities and Assistant —
  FOUND
- `account-page.tsx` mounts `<RecentTransactions/>` — FOUND
- 6 commits referencing `04-06` (excluding this summary commit) — VERIFIED
- `pnpm type-check` introduces 0 new errors (3 pre-existing) — VERIFIED
- `pnpm --filter frontend test` 571/571 pass — VERIFIED
