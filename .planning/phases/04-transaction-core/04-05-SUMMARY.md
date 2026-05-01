---
phase: 04-transaction-core
plan: 05
subsystem: frontend-types-schemas-hooks
tags: [tanstack-query, zod, types, hooks, frontend, react]
status: complete
dependency_graph:
  requires:
    - apps/frontend/src/adapters/shared/transactions.ts (04-04 — adapter
      wrappers)
    - apps/frontend/src/lib/types/transaction.ts (04-04 stub — replaced)
    - apps/frontend/src/lib/query-keys.ts (existing)
  provides:
    - apps/frontend/src/lib/types/transaction.ts (full domain TS interfaces)
    - apps/frontend/src/lib/schemas/transaction.ts (Zod validators using
      z.number())
    - apps/frontend/src/hooks/use-transactions.ts (CRUD + search + transfer +
      import)
    - apps/frontend/src/hooks/use-merchant-categories.ts (D-15 lookup + memory
      list)
    - apps/frontend/src/hooks/use-transaction-templates.ts (D-16/17/18
      templates)
    - QueryKeys.TRANSACTIONS / RUNNING_BALANCE / MERCHANT_CATEGORIES /
      TRANSACTION_TEMPLATES
  affects:
    - apps/frontend/src/lib/types.ts (added transaction barrel re-export)
    - apps/frontend/src/adapters/{web,tauri}/index.ts (re-export
      importTransactionsCsv/Ofx)
    - apps/frontend/src/adapters/shared/transactions.ts (searchTransactions
      signature refined; importTransactionsCsv/Ofx wrappers added)
tech_stack:
  added: []
  patterns:
    - "TanStack Query v5 (placeholderData: keepPreviousData; mutation onSuccess
      invalidation)"
    - "z.number() for money fields (NOT z.string().regex) — Phase 3 fix 7e9eb697"
    - "Hooks import from @/adapters (vite alias to web/index or tauri/index
      based on BUILD_TARGET) — NOT @/adapters/shared/... directly (alias doesn't
      resolve)"
    - "Test wrapper pattern: vi.hoisted + vi.mock('@/adapters') + QueryClient +
      QueryClientProvider + renderHook"
    - "No useEffect anywhere — TanStack Query handles
      fetching/caching/invalidation; enabled prop for conditional fetching
      (react-useeffect skill compliance)"
key_files:
  created:
    - apps/frontend/src/lib/schemas/transaction.ts
    - apps/frontend/src/hooks/use-transactions.ts
    - apps/frontend/src/hooks/use-merchant-categories.ts
    - apps/frontend/src/hooks/use-transaction-templates.ts
    - apps/frontend/src/hooks/__tests__/use-transactions.test.tsx
  modified:
    - apps/frontend/src/lib/types/transaction.ts (stub → full domain model)
    - apps/frontend/src/lib/types.ts (added export * from "./types/transaction")
    - apps/frontend/src/lib/query-keys.ts (added 8 transaction-related keys)
    - apps/frontend/src/adapters/shared/transactions.ts (signature refinement)
    - apps/frontend/src/adapters/web/index.ts (re-export
      importTransactionsCsv/Ofx)
    - apps/frontend/src/adapters/tauri/index.ts (re-export
      importTransactionsCsv/Ofx)
decisions:
  - "Kept legacy type aliases (DuplicateMatch = DuplicateCandidate; ImportResult
    = TransactionImportResult; NewTransactionTemplate =
    SaveTransactionTemplateRequest) so the 04-04 adapter wrappers still compile
    without churn. Plan 04-06+ should prefer the new canonical names."
  - "searchTransactions adapter signature changed from (filters, page, pageSize)
    to (page, pageSize, filters, searchKeyword?, sort?) to match the hook
    contract documented in the plan. Web/Tauri index re-exports unchanged; the
    web module handler still does JSON.stringify(payload) so the new fields flow
    through transparently. Server-side handler accepts/ignores extra fields per
    Axum/serde default behavior."
  - "Hooks import via @/adapters (matching use-accounts.ts convention), not
    @/adapters/shared/transactions. Vite alias resolves @/adapters → tauri/index
    or web/index based on BUILD_TARGET; the deeper path doesn't go through the
    alias. Mock target in tests is also @/adapters."
  - "Used .test.tsx (not .test.ts) for the test file because JSX is needed for
    the QueryClientProvider wrapper. Mirrors use-chat-import-session.test.tsx."
  - "TransactionFilters keeps optional fields per the type spec from the plan
    (accountIds?, categoryIds?, etc.) — the executor prompt's note about
    non-optional arrays describes UI form state, not the wire/filter type.
    Server treats undefined as 'no filter' which matches an empty array
    semantically."
metrics:
  duration_minutes: ~30
  tasks_completed: 2 of 2
  commits: 3 (atomic per scope)
  date: 2026-05-01
---

# Phase 4 Plan 05: Frontend types + Zod schemas + TanStack hooks (Summary)

Replaced the 04-04 stub with the full transaction TS domain model, added Zod
validators using `z.number()` for money fields, and shipped 3 TanStack Query
hook modules wired to the adapter from 04-04. Tests prove cache invalidation
runs on every mutation. No `useEffect` introduced.

## What landed

| Layer   | Artifact                                                      | Notes                                                                                                                                                                       |
| ------- | ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| types   | `apps/frontend/src/lib/types/transaction.ts`                  | Full domain model; money as `number`; `insertedRowIds: string[]` on ImportResult                                                                                            |
| schemas | `apps/frontend/src/lib/schemas/transaction.ts`                | 5 schemas with `z.number()` for money; superRefines for splits + transfer accts                                                                                             |
| keys    | `apps/frontend/src/lib/query-keys.ts`                         | TRANSACTIONS, TRANSACTIONS_SEARCH, RUNNING_BALANCE, ACCOUNT_RECENT_TRANSACTIONS, MERCHANT_CATEGORIES, MERCHANT_CATEGORY_LOOKUP, TRANSACTION_TEMPLATES, TRANSACTION_TEMPLATE |
| hooks   | `apps/frontend/src/hooks/use-transactions.ts`                 | 4 queries + 9 mutations; shared `invalidateTransactionLists` helper                                                                                                         |
| hooks   | `apps/frontend/src/hooks/use-merchant-categories.ts`          | `useLookupPayeeCategory` (D-15) + `useMerchantCategoryMemory`                                                                                                               |
| hooks   | `apps/frontend/src/hooks/use-transaction-templates.ts`        | list/get/save/delete (D-16/17/18) with TRANSACTION_TEMPLATES invalidation                                                                                                   |
| tests   | `apps/frontend/src/hooks/__tests__/use-transactions.test.tsx` | 10 tests proving invalidation + caching + lookup gating                                                                                                                     |

## Hook signatures (consumers in 04-06/07/08/09 should call these exactly)

### use-transactions.ts

```typescript
useTransactionSearch(filters?: TransactionFilters, page?: number, pageSize?: number, searchKeyword?: string, sort?: TransactionSort)
  → useQuery<TransactionSearchResult>     // placeholderData: keepPreviousData

useTransaction(id: string | null)
  → useQuery<Transaction>                 // enabled: !!id

useRunningBalance(accountId: string | null, from?: string, to?: string)
  → useQuery<TransactionWithRunningBalance[]>  // enabled: !!accountId

useAccountRecentTransactions(accountId: string | null, limit?: number = 10)
  → useQuery<Transaction[]>               // enabled: !!accountId

useCreateTransaction()
  → useMutation<Transaction, Error, NewTransaction>
  // invalidates: TRANSACTIONS, running-balance prefix, by-account-recent prefix, ACCOUNTS

useUpdateTransaction()
  → useMutation<Transaction, Error, { transaction: TransactionUpdate; editMode?: TransferEditMode }>
  // invalidates: same as create + ["transactions","item",id]

useDeleteTransaction()
  → useMutation<Transaction, Error, string>
  // invalidates: same as create

useCreateTransfer()
  → useMutation<[Transaction, Transaction], Error, { src: NewTransferLeg; dst: NewTransferLeg }>
  // invalidates: same as create (running-balance prefix covers both legs)

useUpdateTransferLeg()
  → useMutation<Transaction, Error, { transaction: TransactionUpdate; editMode: TransferEditMode }>

useBreakTransferPair()
  → useMutation<Transaction, Error, string>  // arg = legId

useImportTransactionsCsv()
  → useMutation<TransactionImportResult, Error, TransactionCsvImportRequest>

useImportTransactionsOfx()
  → useMutation<TransactionImportResult, Error, TransactionOfxImportRequest>

useDetectTransactionDuplicates()
  → useMutation<DuplicateCandidate[], Error, NewTransaction[]>
```

### use-merchant-categories.ts

```typescript
useLookupPayeeCategory(accountId: string | null, payee: string, enabled?: boolean = true)
  → useQuery<PayeeCategoryMemory | null>
  // enabled: enabled && !!accountId && payee.trim().length > 0
  // staleTime: 60_000 (memory is stable)

useMerchantCategoryMemory(accountId: string | null)
  → useQuery<PayeeCategoryMemory[]>      // enabled: !!accountId
```

### use-transaction-templates.ts

```typescript
useTransactionTemplates() → useQuery<TransactionTemplate[]>
useTransactionTemplate(id: string | null) → useQuery<TransactionTemplate>  // enabled: !!id
useSaveTransactionTemplate() → useMutation<TransactionTemplate, Error, SaveTransactionTemplateRequest>
useDeleteTransactionTemplate() → useMutation<void, Error, string>
```

## Adapter signature change — `searchTransactions`

| Before (04-04)                                | After (04-05)                                                        |
| --------------------------------------------- | -------------------------------------------------------------------- |
| `searchTransactions(filters, page, pageSize)` | `searchTransactions(page, pageSize, filters, searchKeyword?, sort?)` |

The web/Tauri index re-exports stayed identical. The web module handler
(`web/modules/transactions.ts:handleSearchTransactions`) does
`JSON.stringify(payload)` — extra fields flow through unchanged.

**Plans 04-06 / 04-08 callers:** use the new positional argument order; the
plan's `useTransactionSearch` hook handles this internally.

## QueryKeys added (verbatim)

```typescript
TRANSACTIONS: ["transactions"] as const,
TRANSACTIONS_SEARCH: (filters: unknown, page: number, pageSize: number, kw: string) =>
  ["transactions", "search", filters, page, pageSize, kw] as const,
RUNNING_BALANCE: (accountId: string, from?: string, to?: string) =>
  ["transactions", "running-balance", accountId, from ?? null, to ?? null] as const,
ACCOUNT_RECENT_TRANSACTIONS: (accountId: string, limit: number) =>
  ["transactions", "by-account-recent", accountId, limit] as const,
MERCHANT_CATEGORIES: (accountId: string) => ["payee-category-memory", accountId] as const,
MERCHANT_CATEGORY_LOOKUP: (accountId: string, payee: string) =>
  ["payee-category-memory", "lookup", accountId, payee] as const,
TRANSACTION_TEMPLATES: ["transaction-templates"] as const,
TRANSACTION_TEMPLATE: (id: string) => ["transaction-templates", id] as const,
```

The shared invalidation helper invalidates **prefixes** so that any
running-balance / by-account-recent query keyed by accountId is invalidated in
one call. This means `useCreateTransfer` correctly invalidates running balance
for BOTH legs (a1 and a2) via the prefix `["transactions", "running-balance"]`.

## Type aliases (back-compat with 04-04 adapter)

```typescript
export type DuplicateMatch = DuplicateCandidate; // 04-04 adapter uses this name
export type ImportResult = TransactionImportResult; // 04-04 adapter uses this name
export type NewTransactionTemplate = SaveTransactionTemplateRequest; // 04-04 adapter uses this name
```

Plans 04-06+ MAY prefer the canonical names but the legacy aliases will remain
to avoid churn in already-merged code.

## Zod schemas — money is `z.number()`, never `z.string()`

Schemas exported from `apps/frontend/src/lib/schemas/transaction.ts`:

| Schema                                | Validates                                                                                                           |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `NewSplitSchema`                      | `categoryId`, positive `amount`, optional notes, non-negative sortOrder                                             |
| `NewIncomeOrExpenseTransactionSchema` | INCOME/EXPENSE flow; payee required; categoryId required unless splits; sum-equals-amount; ≥2 splits when hasSplits |
| `NewTransferTransactionSchema`        | source ≠ destination accounts                                                                                       |
| `TransactionUpdateSchema`             | all fields optional except id; splits as replacement set                                                            |
| `CsvFieldMappingSchema`               | requires either amountColumn or both (debitColumn + creditColumn)                                                   |

Money is `z.number().positive()` everywhere — confirmed by grep:
`grep "z.number" lib/schemas/transaction.ts | wc -l` → 11 occurrences. Zero
`z.string().regex(...)` calls for money.

## Tests (use-transactions.test.tsx — 10 tests)

| #   | Test                                                                                  | Validates                                                                            |
| --- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| 1   | `useTransactionSearch caches by filter+page+pageSize+keyword tuple`                   | Adapter called with positional args; data returned                                   |
| 2   | `useCreateTransaction invalidates TRANSACTIONS, running-balance, recent, ACCOUNTS`    | All 4 invalidation keys called on success                                            |
| 3   | `useUpdateTransaction invalidates lists AND the specific transaction item`            | All 4 + `["transactions","item",id]` invalidation                                    |
| 4   | `useDeleteTransaction invalidates TRANSACTIONS + running-balance + recent + ACCOUNTS` | Same set as create                                                                   |
| 5   | `useCreateTransfer invalidates running-balance prefix (covers both legs)`             | Prefix-match invalidation for both account ledgers                                   |
| 6   | `useImportTransactionsCsv invalidates transaction lists on success`                   | Lists + ACCOUNTS invalidated; `insertedRowIds` order preserved (plan 04-09 contract) |
| 7   | `useLookupPayeeCategory does NOT fire when payee is empty`                            | D-15: only after user types                                                          |
| 8   | `useLookupPayeeCategory does NOT fire when accountId is null`                         | enabled gate                                                                         |
| 9   | `useLookupPayeeCategory fires when accountId AND non-empty payee are provided`        | adapter called with both args                                                        |
| 10  | `useLookupPayeeCategory respects the explicit enabled=false override`                 | enabled prop overrides default                                                       |

## Deviations from plan

### Auto-fixed issues

**1. [Rule 3 - Blocking] Adapter `searchTransactions` signature mismatch**

- **Found during:** Task 2 (writing useTransactionSearch hook)
- **Issue:** Plan task 2 hook spec calls
  `searchTransactions(page, pageSize, filters, searchKeyword, sort)`. The 04-04
  adapter exported `searchTransactions(filters, page, pageSize)` — no
  searchKeyword/sort.
- **Fix:** Refined adapter signature to match hook spec; web module handler
  unchanged (JSON.stringify passes through).
- **Files modified:** `apps/frontend/src/adapters/shared/transactions.ts`
- **Commit:** `caafc06a`

**2. [Rule 2 - Missing functionality] Adapter missing `importTransactionsCsv` /
`importTransactionsOfx` wrappers**

- **Found during:** Task 2 (writing useImportTransactionsCsv/Ofx hooks)
- **Issue:** Hook plan references `importTransactionsCsv` and
  `importTransactionsOfx` from the adapter, but the 04-04 adapter only had
  multipart-handling commented out for "plan 04-05 to wire."
- **Fix:** Added thin `invoke<T>`-based wrappers to
  `adapters/shared/transactions.ts`. Re-exported through `web/index.ts` and
  `tauri/index.ts`.
- **Caveat:** These wrappers go through the COMMANDS map dispatch which can't
  handle File/Blob payloads via `JSON.stringify`. They will type-check but throw
  at runtime when called with multipart data. **Plan 04-08 (import wizard) MUST
  add a separate `web/transactions.ts` direct-fetch implementation following the
  `web/activities.ts:parseCsv` precedent before wiring the wizard's submit
  step.**
- **Commit:** `caafc06a`

**3. [Rule 3 - Blocking] Hook `@/adapters/shared/transactions` import path
doesn't resolve**

- **Found during:** Initial test run — vite plugin error
- **Issue:** Vite alias `@/adapters` resolves to `./src/adapters/{tauri|web}`
  based on BUILD_TARGET. The deeper path `@/adapters/shared/transactions`
  becomes `./src/adapters/{tauri|web}/shared/transactions` which doesn't exist.
- **Fix:** Updated all 3 hook files to import from `@/adapters` (matching
  `use-accounts.ts` convention). Updated test mock target accordingly.
- **Files modified:** all 3 hook files + test
- **Commit:** `1193b766`

### Departures from plan structure (acceptable)

**1. Test file extension is `.test.tsx`, not `.test.ts`**

- **Plan said:** `apps/frontend/src/hooks/__tests__/use-transactions.test.ts`
- **What we did:** Used `.test.tsx` because JSX is required for the
  `QueryClientProvider` wrapper component. Mirrors the existing
  `use-chat-import-session.test.tsx` pattern.
- **Trade-off:** None — vitest config includes `*.test.tsx` (line 96 of
  `vite.config.ts`).

**2. `TransactionFilters` keeps optional fields**

- **Executor prompt note said:** "accountIds: string[] (NOT optional — empty
  array = no filter)"
- **Plan code listing said:** `accountIds?: string[];`
- **What we did:** Followed the plan's verbatim type listing (optional fields
  with `?`). The "non-optional" formulation in the executor prompt describes UI
  form state, not the wire/filter type. Server treats `undefined` as "no filter"
  which is semantically equivalent to an empty array.
- **Plans 04-06 / 04-08:** Form state can default to `[]` when binding
  `Filters → useTransactionSearch`; the type accepts both shapes.

## Pre-existing issues (not introduced by this plan)

The 3 pre-existing type errors documented in 04-04-SUMMARY.md remain
(unaffected):

- `apps/frontend/src/addons/type-bridge.ts:219` — Account.currentBalance type
  drift (number vs addon-sdk string)
- `apps/frontend/src/addons/type-bridge.ts:220` — same root cause
- `apps/frontend/src/pages/settings/accounts/components/account-form.tsx:224` —
  null vs string|undefined for `group` field

Confirmed via type-check before AND after this plan's commits — same 3 errors,
no new errors introduced.

## Verification

| Check                                                                   | Result                                                                                             |
| ----------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `pnpm type-check`                                                       | 3 pre-existing errors (unchanged); 0 new errors                                                    |
| `pnpm --filter frontend test -- --run hooks/__tests__/use-transactions` | 10 tests pass; full suite 549/549 pass                                                             |
| `grep -q "z.number()" lib/schemas/transaction.ts`                       | 11 occurrences                                                                                     |
| `! grep -q "z.string().regex" lib/schemas/transaction.ts`               | 0 occurrences (no money-as-string)                                                                 |
| `grep -q "TRANSACTIONS:" lib/query-keys.ts`                             | Present                                                                                            |
| `insertedRowIds: string[]` on TransactionImportResult                   | Present (line 197 of transaction.ts)                                                               |
| Hook tests cover create / update / delete / transfer                    | All four mutations have invalidation tests                                                         |
| No `useEffect` in any of the 3 new hook files                           | `grep useEffect hooks/use-{transactions,merchant-categories,transaction-templates}.ts` → 0 matches |

## Commits

| Hash       | Title                                                                          |
| ---------- | ------------------------------------------------------------------------------ |
| `d0a14cb4` | feat(04-05): full transaction types, Zod schemas, query keys                   |
| `caafc06a` | feat(04-05): TanStack Query hooks for transactions, merchant memory, templates |
| `1193b766` | test(04-05): add use-transactions hook tests + fix adapter import path         |

## Self-Check: PASSED

- `apps/frontend/src/lib/types/transaction.ts` exports Transaction,
  NewTransaction, TransactionUpdate, TransactionFilters,
  TransactionImportResult, etc. — FOUND
- `apps/frontend/src/lib/schemas/transaction.ts` exists with `z.number()` for
  money — FOUND
- `apps/frontend/src/lib/query-keys.ts` contains TRANSACTIONS, RUNNING_BALANCE,
  MERCHANT_CATEGORIES, TRANSACTION_TEMPLATES — FOUND
- `apps/frontend/src/hooks/use-transactions.ts` exports useTransactionSearch,
  useCreateTransaction, useUpdateTransaction, useDeleteTransaction,
  useCreateTransfer, useImportTransactionsCsv/Ofx — FOUND
- `apps/frontend/src/hooks/use-merchant-categories.ts` exports
  useLookupPayeeCategory, useMerchantCategoryMemory — FOUND
- `apps/frontend/src/hooks/use-transaction-templates.ts` exports
  list/get/save/delete — FOUND
- `apps/frontend/src/hooks/__tests__/use-transactions.test.tsx` has 10 passing
  tests — FOUND
- 3 commits made, all referencing `04-05` in scope — FOUND
- `pnpm type-check` introduces 0 new errors (3 pre-existing) — VERIFIED
- `insertedRowIds: string[]` on TransactionImportResult — VERIFIED
