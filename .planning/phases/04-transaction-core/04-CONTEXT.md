# Phase 4: Transaction Core - Context

**Gathered:** 2026-04-30 **Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the transaction ledger that backs every account: manual add, edit,
delete; CSV + OFX import wizards forked from the existing activity importer;
search + filter; system-driven duplicate detection at import time;
multi-currency display with FX conversion; transaction splits across multiple
categories; running balance per account; transfers between user accounts as
paired rows. Transactions become the source of truth for `current_balance`
(replacing the manual "Update balance" flow from Phase 3).

**Out of scope for Phase 4** (deferred to later phases or explicitly excluded):

- AI-powered category / payee suggestions, conversational entry, receipt OCR
  (Phase 8)
- Recurring transaction / subscription detection (Phase 7)
- Budget assignment UI inside the transaction form (Phase 5)
- Reports / charts / spending-by-category visuals (Phase 6)
- Bank API sync (Plaid, etc.) — out of scope per PROJECT.md
- Bulk multi-row edit, swipe-row gestures, date-group collapse — explicit
  UI-SPEC non-goals
- Settings → Categorization-rules manager UI (Phase 8 will likely re-skin this
  alongside AI fallback)
- Reconciliation widget (Phase 4 ships the data foundation per Phase 3 D-14;
  user-facing reconcile UI is deferred)

</domain>

<spec_lock>

## Locked by `04-UI-SPEC.md` (approved 2026-04-30)

The UI design contract is approved and final. Researcher / planner MUST treat it
as a spec, not a suggestion. Highlights — read the full spec for detail:

- Visual / color / typography / spacing — inherits Phase 3 token stack verbatim.
  Two font weights (400, 600), four sizes (12 / 14 / 16 / 18-24px).
- Direction icons + colors: `ArrowDownLeft text-success` (income),
  `ArrowUpRight text-muted-foreground` (expense),
  `ArrowLeftRight text-muted-foreground` (transfer).
- Accent (`--primary`) reserved exclusively for: single primary CTA per screen,
  direction `ToggleGroup` active state, active filter-chip state, importer
  current-step indicator. Nowhere else.
- Category model = system taxonomy `"Transaction Categories"` in the existing
  `taxonomies` system, seeded with 10 default categories. Inline create via
  `Autocomplete` footer affordance.
- Importer wizard forks `apps/frontend/src/pages/activity/import/*` (shares
  `FileDropzone`, `CSVFileViewer`, `WizardStepIndicator`, `StepNavigation`,
  `HelpTooltip`, `CancelConfirmationDialog` verbatim; forks `MappingTable`,
  `TemplatePicker`, `ImportProvider`).
- Duplicate-confidence display buckets: ≥95 → `bg-destructive/10`; 70-94 →
  `bg-warning/10`; 50-69 → `bg-muted/50`; <50 → not surfaced.
- New route `/transactions` (global ledger) plus per-account "Recent
  transactions" abridged section on `account-page.tsx`. Bottom-nav adds a
  Transactions entry.
- No new third-party registry blocks (`@animate-ui` / `@diceui` net-zero).
- All AI / OCR / conversational / budget / chart UI deliberately excluded.

</spec_lock>

<decisions>
## Implementation Decisions

### Transfer Modeling

- **D-01:** Transfers between two of the user's own accounts are stored as
  **paired rows** — one outflow on the source account + one inflow on the
  destination account, both linked by a nullable `transfer_group_id` column on
  the `transactions` table. `transfer_group_id` is `NOT NULL` only on
  transfer-direction rows. Deleting either side cascades to the sibling.
- **D-02:** Cross-currency transfers store each leg in the leg's native
  currency. Each row in the pair captures its own snapshotted FX rate at
  transfer time (reuses `crates/core/src/fx/`). Source:
  `-$100.00 USD @ rate 1.0870`. Destination: `+€92.00 EUR @ rate 1.0870`. The
  pair stays accurate forever — system rate changes do NOT rewrite historic
  transfers.
- **D-03:** Both legs of a paired transfer are visible in the global
  `/transactions` ledger by default. The existing UI-SPEC "Show transfers"
  toggle hides BOTH legs simultaneously when off. Each leg shows the
  `ArrowLeftRight` Transfer-direction icon + a subtle pair indicator (planner
  picks the visual treatment within the existing chip / icon vocabulary;
  candidates: a small `↔` glyph or grouped border on the row). Tapping either
  leg opens a detail sheet that surfaces both sides.
- **D-04:** Editing a paired transfer leg syncs date, transfer_group_id, and
  notes to the sibling automatically. Editing the **amount** prompts an
  AlertDialog: "Apply to both legs (preserves transfer pairing) or only this leg
  (breaks the link)?". Confirming "this leg only" clears `transfer_group_id` on
  the edited row, leaving the sibling pair-less. This lets the user record
  real-world wire fees / rate-spread asymmetries without silent drift.
- **D-05:** Per-account ledger queries (screen 2 — "Recent transactions" on
  account detail page) show only the leg attached to that `account_id` — no
  artificial collapse logic. The pair is reconstructed in detail-sheet view via
  `transfer_group_id` lookup.

### Duplicate Detection

- **D-06:** Required match keys for a duplicate candidate: same `account_id` +
  same `amount` (within $0.01 epsilon to absorb FX rounding) + date within ±3
  calendar days. All three must hold to even consider the row a candidate.
- **D-07:** Payee similarity is a **confidence multiplier**, not a gate. Compute
  Levenshtein / token-set ratio on the **normalized merchant string** (see D-12
  for the normalization rule) and bump the confidence score proportionally. The
  exact formula = Claude's discretion under the bucket constraint (D-09).
- **D-08:** Detector runs at **import time only** (CSV + OFX wizards). Manual
  entry intentionally skips the check — if a user types a duplicate by hand,
  that's their decision. Surface points are exactly the two locations UI-SPEC §6
  already designs: the dedicated "Review duplicates" wizard step between Mapping
  and Confirm, and the post-import `⚠ N possible duplicates` banner at the top
  of `/transactions`.
- **D-09:** Confidence buckets are fixed by UI-SPEC §6:
  `≥95% → bg-destructive/10` (almost-certain), `70-94 → bg-warning/10` (likely),
  `50-69 → bg-muted/50` (possible), `<50 → suppressed`. Confidence below 50%
  MUST NOT be surfaced to the user. Planner writes the score-to-bucket mapping;
  researcher recommends the formula (likely a weighted sum: amount-exactness ×
  0.4 + date-closeness × 0.3 + payee-similarity × 0.3, normalized to 0-100).
- **D-10:** Per-pair "this is not a duplicate, don't ask again" memory and
  background-scan after-edit detection are **deferred** — Phase 4 ships the
  one-shot import-time check only.

### Categorization Rules (TXN-02 minus AI fallback)

- **D-11:** Phase 4 ships **implicit memory only**. No Settings →
  Categorization-rules manager page. The "amount patterns" half of TXN-02 and
  the AI fallback are explicitly deferred to Phase 8. v1 just remembers the
  user's last category for each merchant.
- **D-12:** Memory shape: `(normalized_merchant, account_id) → category_id` with
  a "last seen" timestamp. Storage layout (separate `payee_categories` table vs.
  JSON column on settings vs. taxonomy-side metadata) = Claude's discretion.
  Researcher recommends.
- **D-13:** Merchant normalization for the memory key: lowercase + strip
  leading/trailing whitespace + collapse runs of digits and runs of spaces. Pure
  Rust, no NLP dependencies. Examples: `"WHOLEFDS GRP #10403"` →
  `"wholefds grp #"`; `"STARBUCKS  STORE 12345"` → `"starbucks store #"`. Both
  variants share one memory entry.
- **D-14:** Category-edit semantics: **last write wins, silently update
  memory**. When a user changes a transaction's category (manual edit or
  importer Review-step override), the
  `(normalized_merchant, account_id) → category_id` row updates immediately.
  Future new transactions for that merchant pre-fill with the new choice.
  Historical transactions are NOT bulk-recategorized — the user's prior data
  stays exactly as it was. No "apply to past 47 transactions?" prompt.
- **D-15:** Auto-fill timing — when the user types a payee in the manual entry
  form, the matched category pre-fills the Category Autocomplete (still
  editable). On CSV / OFX import Review step, learned categories pre-fill the
  category column for matched merchant strings before the user sees the preview
  rows. Whether the lookup is client-side cached or per-request = Claude's
  discretion.

### CSV Import Templates

- **D-16:** **User-saved templates only — no starter pack** of bank-specific
  presets. Mirrors how `activity-import-page.tsx` already works. User maps
  columns once, names + saves the mapping, picks it from a dropdown on
  subsequent imports. No bundled `templates/chase.json` etc. Lower delivery
  cost, zero ongoing maintenance burden when banks tweak their export columns.
- **D-17:** When a previously-saved template is selected for an import, validate
  that the file's headers + positions still match what the template expects. On
  mismatch, surface an inline message in the Mapping step:
  `"Your saved 'Chase Checking CSV' template doesn't match this file's columns. Re-map?"`.
  The user remaps and saves over the existing template (or saves under a new
  name). Extends the existing header-detection logic in
  `mapping-step-unified.tsx`.
- **D-18:** Templates are **globally scoped**, not per-account. The user saves a
  single `"Chase Checking CSV"` template; it appears in the picker on any
  account import. Same Chase format reused across Chase Checking and Chase
  Savings without re-saving. Mirrors the global scope used in the
  activity-import flow today.
- **D-19:** OFX imports do NOT use templates. OFX has a strict schema (FITID,
  TRNAMT, DTPOSTED, NAME, MEMO) — the importer reads it directly without a user
  mapping step. Researcher confirms OFX 1.x SGML vs 2.x XML coverage scope.

### Pre-existing decisions carried forward (NOT re-litigated)

- **PG-only storage** (Phase 02 post-pivot per
  `memory/project_storage_pivot_pg_only.md`). All Phase 4 work targets
  `crates/storage-postgres/`. SQLite is removed from the workspace.
- **Account integration via Phase 3 D-14** — when the first real transaction is
  inserted against a Phase-3-era account, Phase 4 auto-generates an "Opening
  Balance" transaction dated at `account.created_at` with
  `amount = opening_balance`. If the user has manually updated `current_balance`
  since account creation (delta != sum(transactions) between created_at and
  now), Phase 4 also materializes that delta as a "Balance adjustment"
  transaction so totals reconcile exactly. These two reconciliation rows MUST be
  tagged with a marker (e.g., `system_generated: true` or a dedicated category)
  so reports can distinguish them from user-entered rows.
- **CC liability semantics (Phase 3 D-13)** — credit-card balances are stored as
  positive values. Spending on a CC `INCREASES` the balance; payments `DECREASE`
  it. Net-worth math handled via the `account_kind` helper
  (`crates/core/src/accounts/`). Phase 4 transaction direction
  (Income/Expense/Transfer) is independent of liability semantics — an "Expense"
  on a CREDIT_CARD account adds to the card balance.
- **Category model (UI-SPEC)** — categories live in a system taxonomy
  `"Transaction Categories"` with the 10 seeded defaults listed in
  `04-UI-SPEC.md` §Category Model. Seed migration runs at first launch / via a
  Diesel migration. Inline category creation via `Autocomplete` footer
  affordance hits the existing taxonomy `createCategory` mutation (planner
  verifies; adds the mutation if missing).

### Claude's Discretion

Downstream agents (researcher, planner) decide these within the decisions above:

- Exact `transactions` table schema: columns, indexes, foreign keys,
  partial-index strategy for fast per-account ledger queries.
- Whether splits live in a separate `transaction_splits` table (one row per
  split) or as a `splits JSONB` column. The UI-SPEC requires split rows with
  per-row category + amount + optional notes; pick the storage shape that
  minimizes query complexity for Phase 5 (Budgeting) and Phase 6 (Reporting).
- **Running balance computation strategy** (TXN-09 — "running balance per
  account"). Options to evaluate: (a) compute on the fly via window function
  - `account_id` partition + date ordering; (b) materialized `running_balance`
    column on `transactions` with a recompute trigger when rows insert
    out-of-order; (c) per-account snapshot table updated by a background
    recompute job. The choice affects CSV import performance — out-of-date CSV
    imports against an account with thousands of transactions will be the
    pathological case.
- OFX parsing crate selection: evaluate `ofx-rs`, `sgmlish`, or hand-roll a
  minimal parser. Constraint: OFX 1.x SGML coverage is mandatory (US bank
  exports); OFX 2.x XML preferred but not blocking.
- Confidence-score formula details under the bucket constraint (D-09).
- Storage shape for the `(normalized_merchant, account_id) → category_id` memory
  (D-12).
- Per-row idempotency key for re-imports of the same file. Candidates: hash of
  (account_id + raw_csv_row_text) at import; OFX FITID when present (preferred —
  bank-stable). Re-importing the same file MUST not insert duplicate rows.
- Whether to record audit metadata for "this transaction's category was learned
  from merchant memory vs. user-typed" (useful for Phase 8 AI training data; not
  user-visible).
- Visual treatment of paired-transfer rows beyond the icon (D-03 candidates).
- Bottom-nav placement of the new "Transactions" entry — UI-SPEC says it exists;
  planner verifies the existing `header.tsx` / nav config slots.

### Folded Todos

None — `gsd-tools list-todos` returned 0 pending items.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning docs

- `.planning/ROADMAP.md` §Phase 4 — goal, success criteria, dependency on Phase
  3, requirement IDs (TXN-01..09)
- `.planning/REQUIREMENTS.md` §Transactions — full text of TXN-01..09
- `.planning/PROJECT.md` §Constraints, §Out of Scope — local-first / PG-only /
  no bank API guidance
- `.planning/phases/04-transaction-core/04-UI-SPEC.md` — **APPROVED visual +
  interaction contract; treat as spec**
- `.planning/phases/03-bank-accounts-credit-cards/03-CONTEXT.md` — Phase 3
  account-domain decisions (D-04 tracking_mode, D-13 CC positive balances, D-14
  reconciliation contract, D-15..19 unified account list UX)
- `.planning/phases/02-dual-database-engine/02-CONTEXT.md` — storage
  architecture (PG-only post-pivot), repository trait conventions, diesel-async
  strategy

### Existing core domain (extend / reuse — do not duplicate)

- `crates/core/src/accounts/accounts_model.rs` — `Account`, `NewAccount`,
  `AccountUpdate`, `TrackingMode`; balance fields land here for Phase 3
  reconciliation hooks
- `crates/core/src/accounts/accounts_service.rs` — Phase 4 hooks the
  reconciliation generator on first-transaction-insert
- `crates/core/src/accounts/accounts_constants.rs` — `AccountType`,
  `AccountKind`, `default_group_for_account_type`
- `crates/core/src/activities/csv_parser.rs` — CSV parsing helpers (delimiter
  detection, encoding, column extraction)
- `crates/core/src/activities/idempotency.rs` — gold-standard idempotency
  pattern; Phase 4 re-import dedup follows this shape
- `crates/core/src/activities/compiler.rs` — staged-compile pattern from parsed
  rows → typed activities; transactions follow the same flow
- `crates/core/src/fx/` — FX rate snapshotting for D-02 cross-currency transfers
  and TXN-07 multi-currency display
- `crates/core/src/taxonomies/` — taxonomy + category CRUD; seed the
  "Transaction Categories" system taxonomy here

### PostgreSQL storage (extend)

- `crates/storage-postgres/src/schema.rs` — regenerate after Phase 4 migration
- `crates/storage-postgres/migrations/` — add a new numbered migration
  (`20260501000000_transactions_initial` or similar) for the `transactions`
  table + indexes + the "Transaction Categories" seed
- `crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/`
  — Phase 3 schema additions (opening_balance, current_balance,
  balance_updated_at) that Phase 4 reads from
- `crates/storage-postgres/src/accounts/` — Phase 3 PG repository and DTOs

### Frontend (extend / fork)

- `apps/frontend/src/pages/activity/import/activity-import-page.tsx` — wizard
  scaffolding, step orchestration, ImportProvider context shape — fork as the
  basis for `transaction-import-page.tsx`
- `apps/frontend/src/pages/activity/import/components/file-dropzone.tsx` — share
  verbatim
- `apps/frontend/src/pages/activity/import/components/csv-file-viewer.tsx` —
  share verbatim
- `apps/frontend/src/pages/activity/import/components/wizard-step-indicator.tsx`
  — share verbatim
- `apps/frontend/src/pages/activity/import/components/step-navigation.tsx` —
  share verbatim
- `apps/frontend/src/pages/activity/import/components/help-tooltip.tsx` — share
  verbatim
- `apps/frontend/src/pages/activity/import/components/cancel-confirmation-dialog.tsx`
  — share verbatim with parameterized copy
- `apps/frontend/src/pages/activity/import/components/mapping-table.tsx` — fork;
  transaction field set differs (no asset resolution)
- `apps/frontend/src/pages/activity/import/components/template-picker.tsx` —
  fork; templates are now transaction-mapping templates
- `apps/frontend/src/pages/activity/import/steps/mapping-step-unified.tsx` —
  reference for header-mismatch detection (D-17)
- `apps/frontend/src/pages/activity/import/context/import-provider.tsx` — fork
  ImportProvider; transaction state shape is simpler (no asset resolution; adds
  duplicate-review state)
- `apps/frontend/src/pages/settings/taxonomies/taxonomies-page.tsx` — system
  taxonomy "Transaction Categories" management lives here
- `apps/frontend/src/lib/types/taxonomy.ts` — `Taxonomy`, `TaxonomyCategory`,
  `isSystem`, `isSingleSelect`, hierarchy support
- `apps/frontend/src/lib/types/account.ts` — `Account` shape Phase 4 reads
- `apps/frontend/src/hooks/use-accounts.ts` — accounts hook Phase 4 calls for
  the account picker
- `apps/frontend/src/adapters/shared/accounts.ts` — account commands
- `apps/frontend/src/adapters/shared/activities.ts` — pattern reference for the
  new transactions adapter

### Memory context (non-obvious, must apply)

- `memory/project_storage_pivot_pg_only.md` — Phase 02's "dual-engine" framing
  is historical. Only `crates/storage-postgres` exists. No SQLite migrations, no
  parity tests.

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- **`crates/core/src/activities/csv_parser.rs`** — proven CSV parsing for
  investment activities. Phase 4 transactions reuse the same encoding-detection
  / delimiter-sniffing / row-extraction primitives.
- **`crates/core/src/activities/idempotency.rs`** — gold-standard idempotency
  pattern for re-import dedup. Use the same approach (per-row hash) keyed by
  `account_id + normalized_csv_row_payload` for transaction CSV imports. OFX
  uses FITID instead.
- **`crates/core/src/fx/`** — already snapshots quotes by date; Phase 4 calls
  this for D-02 transfer FX snapshots and for TXN-07 multi-currency display.
- **`crates/core/src/accounts/accounts_service.rs`** — Phase 4 hooks here for
  the D-14 reconciliation flow. The "first-transaction-insert" trigger needs to
  read `accounts.opening_balance`, `current_balance`, and `balance_updated_at`
  and synthesize the two backfill rows before the user's transaction is
  committed.
- **`crates/core/src/taxonomies/`** — used to seed the system "Transaction
  Categories" taxonomy. `isSystem: true`, `isSingleSelect: true`, 10 default
  categories per UI-SPEC. Inline-create flow uses the existing `createCategory`
  mutation (verify the adapter exposes it; add if missing).
- **`activity-import-page.tsx` and its `components/`** — full multi-step wizard
  with `ImportProvider` context, step orchestration, header detection, template
  picker, file dropzone, CSV preview. Phase 4 forks the page-level component and
  steps; shares the primitives verbatim.
- **`@whaleit/ui` financial primitives** — `PrivacyAmount`, `MoneyInput`,
  `CurrencyInput`, `AmountDisplay`. UI-SPEC adds NO new financial primitives —
  only compositions.
- **`apps/frontend/src/pages/dashboard/accounts-summary.tsx`** — row shape /
  density / hover / skeleton reference for the transaction-list row visuals.

### Established Patterns

- **Repository trait + concrete storage crate**: new fields require updates in
  `crates/core/src/transactions/transactions_model.rs` (new),
  `crates/storage-postgres/src/transactions/` (new), `schema.rs` (regenerate),
  and the web/Tauri adapter layers.
- **Money columns** use `NUMERIC` in PG, mapped to `Decimal` at the Diesel
  boundary. Wire format is JSON number (per Phase 3 fix `7e9eb697` —
  `rust_decimal` `serde-float` feature is enabled). Frontend Zod schemas use
  `z.number()` for money fields, NOT `z.string()`.
- **Domain events** (`crates/core/src/events/`) are the right place for
  "transaction inserted" / "balance recomputed" side-effect hooks Phase 4
  introduces.
- **Frontend command adapter pattern** (`adapters/shared/`,
  `adapters/web/core.ts`, IPC registry) — every new transaction command surfaces
  in both Tauri IPC and the Axum web adapter following this pattern.
- **Multi-step wizard pattern** is `ImportProvider` context +
  `WizardStepIndicator`
  - numbered `*-step.tsx` files driven by a step machine. Phase 4 imports follow
    this exactly.

### Integration Points

- New migration under `crates/storage-postgres/migrations/` adds the
  `transactions` table, the `transaction_splits` table (or splits JSONB, per
  Claude's discretion), `transfer_group_id` column, indexes for per-account /
  per-date queries, and the system "Transaction Categories" taxonomy seed.
- New Rust crate module `crates/core/src/transactions/` (mirrors `accounts/` and
  `activities/` layout — `transactions_model.rs`, `transactions_service.rs`,
  `transactions_traits.rs`, `csv_parser.rs`, `ofx_parser.rs`,
  `duplicate_detector.rs`, `idempotency.rs`).
- New PG storage module `crates/storage-postgres/src/transactions/` — `model.rs`
  (TransactionDB Diesel mapping), `repository.rs` (PgTransactionRepository).
- Web adapter (`apps/frontend/src/adapters/web/core.ts`) and IPC command
  registry surface new commands: `create_transaction`, `update_transaction`,
  `delete_transaction`, `list_transactions`, `import_transactions_csv`,
  `import_transactions_ofx`, `detect_duplicates`, `get_payee_category` (or
  whatever shape the memory takes).
- New routes: `/transactions` (global ledger), `/transactions/import` (wizard).
  Per-account "Recent transactions" mounts inside existing `account-page.tsx`.
- Bottom-nav config (`apps/frontend/src/components/header.tsx` or equivalent)
  accepts a new "Transactions" entry per UI-SPEC §Responsive.
- `accounts_service.rs` gains a Phase-3-D-14 reconciliation hook that fires on
  first transaction insert against an account with `created_at` predating Phase
  4 launch.

</code_context>

<specifics>
## Specific Ideas

- "Last write wins, silently update memory" on category edits — the user wants
  the system to learn quietly, never nag with "apply to past 47 transactions?"
  prompts.
- The friendly-companion brand voice carries into copy: duplicate review uses
  "Discard new" / "Keep both" (not "Reject" / "Accept"); category-mismatch
  prompts use plain language ("Your saved template doesn't match this file's
  columns. Re-map?").
- Bank wire fees and FX rate spreads should be representable by intentionally
  breaking a transfer pair (D-04). The system never silently absorbs the
  asymmetry — it asks the user.
- Transfers feel like one user action but live as two rows; the detail sheet is
  the single visual focal point for the pair.

</specifics>

<deferred>
## Deferred Ideas

Captured so the roadmap doesn't lose them. None is in Phase 4 scope.

- **AI-powered category / payee suggestion UI** (Phase 8 — AI-05, AI-06).
- **AI-powered conversational transaction entry** (Phase 8 — AI-03).
- **Receipt OCR upload + extraction UI** (Phase 8 — AI-04).
- **Recurring transaction / subscription detection UI** (Phase 7 — SUBS-01).
- **Budget assignment UI inside transaction form** (Phase 5 — BUDG-05).
- **Spending-by-category charts on `/transactions`** (Phase 6 — RPT-01).
- **Income vs expense bars** (Phase 6 — RPT-03).
- **Net worth ribbon at top of `/transactions`** (Phase 6 — RPT-04).
- **Per-transaction tag editor** (Phase 12 — freelancer mode handles
  business/personal toggle).
- **Bulk multi-row edit UI** (multi-select rows + bulk recategorize).
- **Swipe-row gestures** (deferred — UI-SPEC explicit non-goal).
- **Date-group collapse/expand** (deferred — running-balance scroll math).
- **Duplicate-detection rule editor** (Phase 4 ships system-driven detection
  only; surfacing thresholds to the user is deferred).
- **Reconciliation widget** (Phase 4 ships the data foundation per Phase 3 D-14;
  user-facing reconcile UI deferred).
- **Settings → Categorization-rules manager UI** — Phase 8 will likely re-skin
  this alongside AI fallback.
- **TXN-02 amount-pattern rules** (e.g., "amount > $1000 on Chase → Housing") —
  deferred to Phase 8 alongside AI fallback. Phase 4 ships only
  merchant-string-based memory.
- **Per-pair "this isn't a duplicate, don't ask again" memory** — D-10 defers
  the dismiss-history table.
- **Background-scan duplicate detection on edits** — D-10 defers; Phase 4 is
  one-shot at import time only.
- **Starter-pack CSV templates** for Chase, Wells Fargo, Amex, etc. — D-16
  defers to user-saved-only.
- **Community-contributable templates** — D-16 defers; out of scope per
  PROJECT.md privacy constraints in any case (would introduce remote-fetch
  path).
- **Token-level fuzzy merchant matching** (Levenshtein/Jaccard on the memory key
  beyond simple normalization) — D-13 defers; ship simple normalization in Phase
  4, revisit when AI categorization lands in Phase 8.
- **CSV transfer auto-pair detection** across two account imports — punted until
  users complain; for now transfers are entered manually.
- **Apply-category-to-historical-transactions** prompt on edit — D-14 says no;
  defer to a future phase if user feedback demands it.

### Reviewed Todos (not folded)

None — `gsd-tools list-todos` returned 0 pending items.

</deferred>

---

_Phase: 04-transaction-core_ _Context gathered: 2026-04-30_
