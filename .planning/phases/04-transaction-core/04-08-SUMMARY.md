---
phase: 04-transaction-core
plan: 08
status: complete
started: 2026-04-30
completed: 2026-05-01
subsystem: frontend / transactions / import-wizard
tags: [import, csv, ofx, wizard, templates, duplicate-detection]
requires:
  - 04-04 # Axum routes + frontend command adapter
  - 04-05 # Frontend types, Zod schemas, TanStack hooks
provides:
  - Working /transactions/import wizard route (CSV + OFX paths)
  - TransactionImportProvider context (format/step/draft/duplicate state)
  - Forked components (TransactionMappingTable, TransactionTemplatePicker)
  - Forked steps (UploadStep, TransactionMappingStep, TransactionReviewStep,
    TransactionConfirmStep)
  - Multipart fetch handlers for CSV/OFX upload (web adapter)
affects:
  - apps/frontend/src/routes.tsx (added /transactions/import route)
  - apps/frontend/src/adapters/web/index.ts (overrode JSON-only import wrappers
    with multipart versions)
tech-stack:
  added: []
  patterns:
    - FormData multipart upload (mirrors web/activities.ts:parseCsv)
    - Wizard step reducer + CSV_STEPS / OFX_STEPS constants for D-19 fork
    - Header-signature validation for D-17 saved-template re-use
key-files:
  created:
    - apps/frontend/src/adapters/web/transactions.ts
    - apps/frontend/src/pages/transactions/import/steps/upload-step.tsx
    - apps/frontend/src/pages/transactions/import/steps/transaction-mapping-step.tsx
    - apps/frontend/src/pages/transactions/import/steps/transaction-review-step.tsx
    - apps/frontend/src/pages/transactions/import/steps/transaction-confirm-step.tsx
    - apps/frontend/src/pages/transactions/import/transaction-import-page.tsx
    - apps/frontend/src/pages/transactions/import/__tests__/transaction-import-page.test.tsx
  modified:
    - apps/frontend/src/adapters/web/index.ts
    - apps/frontend/src/routes.tsx
decisions:
  - D-16 enforced — no system/built-in templates; user-saved only
  - D-17 enforced — saved-template header mismatch shows verbatim banner,
    "Re-map" button clears template
  - D-18 enforced — templates queried via useTransactionTemplates() with no
    per-account filter (global per user)
  - D-19 enforced — OFX format skips Mapping step entirely (UploadStep
    dispatches SET_STEP "review" directly)
metrics:
  duration: ~3 hours
  completed: 2026-05-01
---

# Phase 4 Plan 08: Import Wizard Fork Summary

**One-liner:** Forked the activity-import wizard into
`pages/transactions/import/` with format-aware step list (CSV: 4 steps; OFX: 3
steps per D-19), saved-template header-signature validation (D-17), and
multipart upload handlers in the web adapter.

## Goal

Deliver a working `/transactions/import` wizard route that accepts CSV or OFX
bank files, lets the user map columns (CSV only) using saved templates with
mismatch detection, surfaces duplicates inline using the 04-05 detect hook, and
confirms the import via `useImportTransactionsCsv()` /
`useImportTransactionsOfx()`.

## Inherited from main (prior 04-08 work)

These commits landed before this continuation began:

| Hash       | Type | Subject                                                                              |
| ---------- | ---- | ------------------------------------------------------------------------------------ |
| `77f06810` | feat | Task 1 — re-exports + TransactionImportProvider context + draft utils + mapping hook |
| `1542214d` | feat | Task 2 — TransactionMappingTable + TransactionTemplatePicker                         |
| `34bc18d8` | test | Task 3 RED — add failing transaction-mapping-step tests                              |

That work delivered: 6 component re-export shims (file-dropzone,
csv-file-viewer, wizard-step-indicator, step-navigation, help-tooltip,
cancel-confirmation-dialog), `transaction-import-context.tsx` with
reducer+provider, `transaction-draft-utils.ts` with
`createDraftTransactions`/`validateDraft`/`applyDuplicateMatches`,
`transaction-default-template.ts`, `use-transaction-import-mapping.ts` (with
D-17's `validateHeaderSignature`), `transaction-mapping-table.tsx`,
`transaction-template-picker.tsx`, and a RED test for the mapping step.

## Continuation commits (this worktree)

| Hash          | Type  | Subject                                                                                  |
| ------------- | ----- | ---------------------------------------------------------------------------------------- |
| `31d52744`    | chore | pin worktree continuation with in-progress SUMMARY                                       |
| `86c4abb0`    | feat  | multipart import handlers in adapters/web/transactions.ts                                |
| `50f56819`    | feat  | wizard step files (upload, mapping, review, confirm) — also makes RED mapping test GREEN |
| `ab905fd1`    | feat  | transaction-import-page root with CSV_STEPS / OFX_STEPS constants                        |
| `b9172b50`    | test  | D-19 OFX-skips-Mapping vitest cases                                                      |
| `7f5ee357`    | feat  | /transactions/import route registration                                                  |
| (this commit) | docs  | complete plan summary                                                                    |

## File inventory

### New files

- `apps/frontend/src/adapters/web/transactions.ts` — multipart
  `importTransactionsCsv` / `importTransactionsOfx` handlers (FormData + fetch).
  Mirrors web/activities.ts:parseCsv pattern. Replaces the JSON-only `invoke<T>`
  wrappers in shared/transactions.ts for the web adapter.
- `apps/frontend/src/pages/transactions/import/steps/upload-step.tsx` — file
  dropzone, format detection (sniffs first 500 bytes for `OFXHEADER:` / `<OFX>`
  markers), CSV preview parsing (capped at 500 rows for T-04-29). On OFX:
  dispatches `SET_STEP` "review" directly (D-19).
- `apps/frontend/src/pages/transactions/import/steps/transaction-mapping-step.tsx`
  — D-17 inline mismatch banner with verbatim copy
  `"Your saved '{name}' template doesn't match this file's columns. Re-map?"`.
  Continue button disabled until date + payee + (amount OR debit+credit) are
  mapped.
- `apps/frontend/src/pages/transactions/import/steps/transaction-review-step.tsx`
  — calls `useDetectTransactionDuplicates()` on mount; renders rows with
  confidence-bucket tinting per UI-SPEC §6 (>=95 -> `bg-destructive/10`; 70-94
  -> `bg-warning/10`; 50-69 -> `bg-muted/50`; <50 hidden). Inline
  `[Discard new]` / `[Keep both]` actions.
- `apps/frontend/src/pages/transactions/import/steps/transaction-confirm-step.tsx`
  — summary card + import CTA. Calls `useImportTransactionsCsv()` or
  `useImportTransactionsOfx()` per `state.format`. Stores
  `pendingDuplicateCount` in localStorage for plan 04-09's banner.
  Cancel-confirmation copy verbatim per CONTEXT.md.
- `apps/frontend/src/pages/transactions/import/transaction-import-page.tsx` —
  wizard root. Exports `CSV_STEPS` (4) and `OFX_STEPS` (3) module-level
  constants. `WizardContent` is exported (named) for testability.
- `apps/frontend/src/pages/transactions/import/__tests__/transaction-import-page.test.tsx`
  — 2 vitest cases proving D-19: OFX wizard does NOT render "Mapping" step; CSV
  wizard DOES render all four.

### Modified files

- `apps/frontend/src/adapters/web/index.ts` — replaced two import names from
  `../shared/transactions` with re-exports from `./transactions` (the new
  multipart handlers).
- `apps/frontend/src/routes.tsx` — surgical 2-line addition:
  `import TransactionImportPage` +
  `<Route path="transactions/import" element={<TransactionImportPage />} />`.

## Hooks consumed

- `useTransactionTemplates()` (from `@/hooks/use-transaction-templates`) — D-18
  global-scope template list.
- `useSaveTransactionTemplate()`, `useDeleteTransactionTemplate()` — template
  mutations.
- `useDetectTransactionDuplicates()` (from `@/hooks/use-transactions`) —
  duplicate detection on Review step mount.
- `useImportTransactionsCsv()`, `useImportTransactionsOfx()` — final import on
  Confirm step.

## Decisions (re-verified)

- **D-16 (user-saved only):** `transaction-template-picker.tsx` renders only the
  user templates from `useTransactionTemplates()` — no system grouping. Empty
  state shows "Save a template after mapping to reuse it next time."
- **D-17 (header-signature mismatch):** `validateHeaderSignature()` in
  `use-transaction-import-mapping.ts` produces a positional diff. The mapping
  step renders the banner with verbatim copy
  `"Your saved '{name}' template doesn't match this file's columns. Re-map?"`
  and a `[Re-map]` button that dispatches `CLEAR_TEMPLATE`.
- **D-18 (global per-user):** No `account_id` on the template payload;
  `useTransactionTemplates()` returns the user's templates without filter.
- **D-19 (OFX skips Mapping):** Two enforcement points. (1) `OFX_STEPS` constant
  in the page omits Mapping. (2) `UploadStep.processFile` on OFX detection
  dispatches `SET_STEP "review"` directly. (3) Reducer's
  `getNextStep`/`getPrevStep` use `OFX_STEP_ORDER` for navigation when
  `format === "OFX"`. (4) D-19 test asserts both: OFX wizard renders no
  "Mapping" text; CSV wizard does.

## Verification commands run

```bash
# Test the mapping step (previously RED, now GREEN)
node_modules/.bin/vitest run src/pages/transactions/import/__tests__/transaction-mapping-step.test.tsx
# Result: 4 tests passed

# Test the page (D-19)
node_modules/.bin/vitest run src/pages/transactions/import/__tests__/transaction-import-page.test.tsx
# Result: 2 tests passed

# All transaction import tests
node_modules/.bin/vitest run src/pages/transactions/import
# Result: 6 tests passed (2 test files)

# Type-check
node_modules/.bin/tsc --noEmit --project apps/frontend/tsconfig.json
# Result: only pre-existing baseline errors remain (account-form.tsx:224, type-bridge.ts).
# No new type errors introduced by 04-08 continuation work.
```

## Visual contract held (UI-SPEC §6)

- Step indicator current step uses `bg-primary text-primary-foreground` (one of
  the strict accent locations).
- Header-mismatch banner uses `bg-warning/10`.
- Duplicate-confidence row tinting follows the bucket boundaries exactly.
- No new shadcn/diceui/animate-ui blocks introduced.

## Friendly-companion copy (CONTEXT.md verbatim)

- Duplicate review buttons: `[Discard new]` / `[Keep both]` — never "Reject" /
  "Accept".
- Header mismatch:
  `Your saved '{name}' template doesn't match this file's columns. Re-map?`
- Cancel-confirmation: title `Discard this import?`, body
  `Your file and column choices will be cleared. You can start over anytime.`,
  confirm `Discard`, cancel `Keep editing`.

## Threat-model coverage (re-iteration)

- T-04-29 (DoS via huge CSV): client-side preview capped at 500 rows in
  `parseCsvPreview` and in `createDraftTransactions`.
- T-04-30 (template name SQL injection): server-side concern; backend's Diesel
  queries parameterize.
- T-04-31 (template list cross-tenant): server enforces user_id; UI's "global"
  means global-per-user.

## What's next (plan 04-09)

The Confirm step writes `localStorage.whaleit:pendingDuplicates`
`{count, accountId}` on success when `pendingDuplicateCount > 0`. Plan 04-09's
banner picks that up and surfaces a sticky duplicate-resolution affordance.

## Deviations from Plan

**1. [Rule 3 - Blocking] Exported `WizardContent` as a named export**

- **Found during:** Task 4 (page root + test)
- **Issue:** The plan's pseudocode wrapped `<TransactionImportPage />` in an
  outer `<TransactionImportProvider>` with `initialState={{ format }}` — but
  `TransactionImportPage` creates its own inner provider, so the outer wrap had
  no effect. Tests could not control the `format` field of context.
- **Fix:** Exported `WizardContent` (named) so the test can mount it directly
  inside a custom provider that controls `format`. The default
  `TransactionImportPage` export is unchanged.
- **Files modified:**
  `apps/frontend/src/pages/transactions/import/transaction-import-page.tsx`
- **Commit:** `b9172b50`

**2. [Rule 3 - Blocking] Mapping step now self-initializes a default mapping**

- **Found during:** Task 3 (mapping step + test)
- **Issue:** The RED test renders the mapping step with `state.mapping` unset.
  Original conditional render `{state.mapping && <TransactionMappingTable />}`
  meant the table never appeared in the test, so `screen.getByText("Date")`
  failed.
- **Fix:** Mapping step now `useEffect`-dispatches a default empty
  `CsvFieldMapping` on mount when `state.mapping` is null, and renders the table
  with an `activeMapping` fallback. This is also more correct UX — the empty
  mapping table shows up immediately, rather than waiting for upstream
  initialization.
- **Files modified:**
  `apps/frontend/src/pages/transactions/import/steps/transaction-mapping-step.tsx`
- **Commit:** `50f56819`

**3. [Rule 1 - Bug] Header-mismatch text rendered in single string node for
testability**

- **Found during:** Task 3 (mapping step test)
- **Issue:** Original JSX
  `Your saved &lsquo;{state.selectedTemplateName}&rsquo; template doesn't match...`
  produced multiple text nodes (literal + interpolation + literal), and
  `screen.getByText(/regex/)` does not match across element boundaries.
- **Fix:** Wrapped the entire banner copy in a single template-literal text
  expression `{`Your saved '${name}' template doesn't match this file's columns.
  Re-map?`}` so the visible text is one continuous text node. The visible
  characters match D-17's verbatim copy exactly (straight ASCII apostrophes).
- **Files modified:**
  `apps/frontend/src/pages/transactions/import/steps/transaction-mapping-step.tsx`
- **Commit:** `50f56819`

## Self-Check: PASSED

- All created files exist on disk (verified via `test -f`):
  - `apps/frontend/src/adapters/web/transactions.ts` — FOUND
  - `apps/frontend/src/pages/transactions/import/steps/upload-step.tsx` — FOUND
  - `apps/frontend/src/pages/transactions/import/steps/transaction-mapping-step.tsx`
    — FOUND
  - `apps/frontend/src/pages/transactions/import/steps/transaction-review-step.tsx`
    — FOUND
  - `apps/frontend/src/pages/transactions/import/steps/transaction-confirm-step.tsx`
    — FOUND
  - `apps/frontend/src/pages/transactions/import/transaction-import-page.tsx` —
    FOUND
  - `apps/frontend/src/pages/transactions/import/__tests__/transaction-import-page.test.tsx`
    — FOUND
- All commits exist in git log:
  - `31d52744` — FOUND
  - `86c4abb0` — FOUND
  - `50f56819` — FOUND
  - `ab905fd1` — FOUND
  - `b9172b50` — FOUND
  - `7f5ee357` — FOUND
- Inherited base commits also present: `77f06810`, `1542214d`, `34bc18d8` —
  FOUND.
- All 6 vitest cases pass (4 mapping-step + 2 page).
- Type-check shows no new errors from this work.
