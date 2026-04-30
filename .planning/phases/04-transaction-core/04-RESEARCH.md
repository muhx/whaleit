# Phase 4: Transaction Core — Research

**Researched:** 2026-04-30 **Domain:** Transaction ledger (manual entry, CSV/OFX
import, duplicates, splits, transfers, multi-currency, running balance) on
PostgreSQL. **Confidence:** HIGH for code patterns and schema (verified
in-repo); MEDIUM for OFX library choice (verified via lib.rs/docs.rs) and
running-balance strategy (verified via PG docs).

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Transfer Modeling

- **D-01:** Transfers stored as paired rows linked by nullable
  `transfer_group_id` on `transactions`. NOT NULL only on transfer-direction
  rows. Deleting either side cascades to the sibling.
- **D-02:** Cross-currency transfers store each leg in its native currency, with
  each row capturing its own snapshotted FX rate at transfer time (reuses
  `crates/core/src/fx/`). System rate changes do NOT rewrite history.
- **D-03:** Both legs visible in the global `/transactions` ledger by default.
  "Show transfers" toggle hides BOTH legs simultaneously when off. Each leg
  shows the `ArrowLeftRight` icon plus a subtle pair indicator. Tapping either
  leg opens a detail sheet that surfaces both sides.
- **D-04:** Editing a paired transfer leg syncs date, transfer_group_id, and
  notes to the sibling automatically. Editing the **amount** prompts an
  AlertDialog: "Apply to both legs (preserves transfer pairing) or only this leg
  (breaks the link)?". "This leg only" clears `transfer_group_id` on the edited
  row, leaving the sibling pair-less.
- **D-05:** Per-account ledger queries show only the leg attached to that
  `account_id` — no artificial collapse logic.

#### Duplicate Detection

- **D-06:** Required match keys: same `account_id` + same `amount` (within $0.01
  epsilon) + date within ±3 calendar days. All three must hold.
- **D-07:** Payee similarity is a confidence multiplier on the normalized
  merchant string, not a gate.
- **D-08:** Detector runs at import time only (CSV + OFX). Manual entry
  intentionally skips the check.
- **D-09:** Confidence buckets fixed by UI-SPEC §6: ≥95 → `bg-destructive/10`,
  70-94 → `bg-warning/10`, 50-69 → `bg-muted/50`, <50 → suppressed.
- **D-10:** Per-pair "this is not a duplicate, don't ask again" memory and
  background-scan after-edit detection are deferred.

#### Categorization Rules (TXN-02 minus AI fallback)

- **D-11:** Phase 4 ships implicit memory only. No Settings rules manager UI.
- **D-12:** Memory shape: `(normalized_merchant, account_id) → category_id` with
  a "last seen" timestamp. Storage layout = Claude's discretion.
- **D-13:** Merchant normalization: lowercase + strip leading/trailing
  whitespace + collapse runs of digits and runs of spaces. Pure Rust, no NLP
  deps. `"WHOLEFDS GRP #10403"` → `"wholefds grp #"`.
- **D-14:** Category-edit semantics: last write wins, silently update memory.
  Historical transactions are NOT bulk-recategorized.
- **D-15:** Auto-fill timing — payee autofill on manual entry; CSV/OFX importer
  Review step pre-fills before user sees preview rows.

#### CSV Import Templates

- **D-16:** User-saved templates only — no starter pack.
- **D-17:** When a saved template is selected, validate header positions still
  match. On mismatch, surface inline message in Mapping step.
- **D-18:** Templates are globally scoped (mirrors activity-import).
- **D-19:** OFX imports do NOT use templates. Strict schema. OFX 1.x SGML
  mandatory; OFX 2.x XML preferred.

#### Carried forward (not re-litigated)

- PG-only storage. SQLite is removed.
- Phase 3 D-14 reconciliation hook: first-transaction-insert against an account
  synthesizes "Opening Balance" + (optional) "Balance adjustment"
  system-generated rows.
- Phase 3 D-13: credit-card balances stored positive — spending INCREASES,
  payments DECREASE. Phase 4 transaction direction (Income/Expense/Transfer) is
  independent of liability semantics.
- Category model: system taxonomy `"Transaction Categories"` with 10 seeded
  defaults; inline create via `Autocomplete` footer.

### Claude's Discretion

- Exact `transactions` table schema, indexes, FKs, partial-index strategy.
- Splits storage: separate `transaction_splits` table vs. `splits JSONB` column
  (optimize for Phase 5/6 query complexity).
- Running balance computation strategy (TXN-09): three options to evaluate — (a)
  on-the-fly window function, (b) materialized `running_balance` column, (c)
  per-account snapshot table.
- OFX parsing crate selection: `ofx-rs`, `sgmlish`, hand-roll. OFX 1.x SGML
  MANDATORY.
- Confidence-score formula details under bucket constraint D-09.
- Storage shape for `(normalized_merchant, account_id) → category_id` memory.
- Per-row idempotency key for re-imports.
- Whether to record audit metadata for "category was learned from memory vs
  user-typed".
- Visual treatment of paired-transfer rows beyond the icon.
- Bottom-nav placement of new "Transactions" entry.

### Deferred Ideas (OUT OF SCOPE)

AI categorization (Phase 8), conversational entry (Phase 8), receipt OCR (Phase
8), recurring detection (Phase 7), budget assignment UI (Phase 5),
spending-by-category charts (Phase 6), net-worth ribbon (Phase 6),
per-transaction tag editor (Phase 12), bulk multi-row edit, swipe-row gestures,
date-group collapse/expand, duplicate-detection rule editor, reconciliation
widget UI, Settings → Categorization-rules manager UI, TXN-02 amount-pattern
rules, per-pair "not a duplicate" memory, background-scan after-edit detection,
starter-pack CSV templates, community-contributable templates, token-level fuzzy
merchant matching beyond normalization, CSV transfer auto-pair detection,
apply-category-to-historical prompt. </user_constraints>

<phase_requirements>

## Phase Requirements

| ID     | Description                                                                               | Research Support                                                                                                                            |
| ------ | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| TXN-01 | Quick add manual transaction (amount, payee, category, date, account, notes)              | Schema (§3) + transactions_service.rs create flow + form wired to UI-SPEC §3                                                                |
| TXN-02 | Auto-categorize from rules (merchant patterns, amount patterns) — AI fallback deferred    | Categorization Memory (§9) — implicit `(normalized_merchant, account_id) → category_id` only; amount-pattern rules deferred per D-11        |
| TXN-03 | Search and filter across accounts (merchant, amount range, date range, category, account) | Schema indexes (§3) — `(account_id, transaction_date DESC)` covers common ledger queries; trigram index on payee handles search             |
| TXN-04 | Import from CSV with flexible column mapping                                              | CSV strategy (§5) reuses `activities/csv_parser.rs` verbatim; new compiler-style staged pipeline in `transactions/compiler.rs`              |
| TXN-05 | Import from OFX                                                                           | OFX strategy (§4) — `sgmlish` crate for OFX 1.x SGML, optional XML branch for OFX 2.x                                                       |
| TXN-06 | Detect and flag potential duplicates for review                                           | Duplicate Detection (§6) — three-key gate + Levenshtein-based confidence multiplier on normalized merchant                                  |
| TXN-07 | Multi-currency with FX                                                                    | FX integration (§10) — leg-native storage; `fx_service.get_exchange_rate_for_date()` for snapshots                                          |
| TXN-08 | Split single transaction across multiple categories                                       | Splits storage (§3) — separate `transaction_splits` table                                                                                   |
| TXN-09 | Running balance per account                                                               | Running Balance (§7) — Option A: on-the-fly window function via SQL view, with composite index `(account_id, transaction_date, created_at)` |

</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Surgical changes:** touch only what you must; don't refactor adjacent code.
- **Simplicity first:** minimum code that solves the problem; nothing
  speculative.
- **Match existing style** even when the researcher would prefer a different
  pattern. Phase 4 mirrors `crates/core/src/accounts/` and
  `crates/core/src/activities/` layouts verbatim.
- **Goal-driven execution:** define falsifiable success criteria upfront (drives
  the Validation Architecture section below).
- **Plan mode:** extremely concise plans, sacrifice grammar for concision, end
  with unresolved questions list.
- **Money columns:** NUMERIC(20,8) in PG (per Phase 3 D-10), `Decimal` at Diesel
  boundary, JSON number wire format (`rust_decimal serde-float` enabled per
  Phase 3 fix `7e9eb697`). Frontend Zod uses `z.number()`.

## Summary

- **Architecture is a near-clone of `crates/core/src/activities/`.** Mirror the
  directory layout verbatim: `transactions_model.rs`, `transactions_service.rs`,
  `transactions_traits.rs`, `csv_parser.rs` (re-export from activities),
  `ofx_parser.rs`, `compiler.rs`, `duplicate_detector.rs`, `idempotency.rs`,
  `merchant_normalizer.rs`. The `Repository → Service → DomainEvent` pattern is
  established and proven.
- **Transactions is a single normalized table; splits is a separate child
  table.** A `splits JSONB` column would be cheaper to write but blocks Phase 5
  (Budgeting) and Phase 6 (Reporting) from doing `JOIN-based` aggregation by
  category. A `transaction_splits` table with
  `(transaction_id, category_id, amount, sort_order, notes)` keeps every future
  report a one-line aggregate query and matches existing `taxonomies` reference
  patterns.
- **OFX 1.x SGML uses `sgmlish`.** Verified explicit OFX support; pure-Rust;
  Serde integration; license MIT. No reasonable hand-roll alternative meets the
  OFX 1.x mandate without re-implementing SGML normalization.
- **Running balance uses Option A (on-the-fly window function).** Materialized
  recompute on out-of-order insert (Option B/C) is large-attack-surface
  premature optimization for v1 ledger sizes (<100k rows per account). Composite
  index `(account_id, transaction_date DESC, created_at DESC)` carries the cost.
  See §7 for benchmark expectations and the Phase 6 escape hatch.
- **Frontend: Tauri "IPC" is HTTP**, not real `tauri::command`. The Tauri
  adapter (`apps/frontend/src/adapters/tauri/core.ts`) just `fetch`es the Axum
  server using the same `COMMANDS` map as the web adapter. This means Phase 4
  wiring is a single new file `apps/server/src/api/transactions.rs`
  - COMMANDS map entries; no parallel Tauri command registration.

**Primary recommendation:** Mirror `activities/` layout 1:1 in core and PG. Use
`sgmlish` for OFX 1.x. Splits in their own table. Running balance via window
function with composite index. Categorization memory in a dedicated
`payee_category_memory` table keyed by `(account_id, normalized_merchant)`.
Reconciliation hook fires inside `TransactionService::create_transaction()` not
`accounts_service.rs`, with account state read via the existing
`AccountServiceTrait`.

## Architectural Responsibility Map

| Capability                       | Primary Tier                            | Secondary Tier                      | Rationale                                                                                    |
| -------------------------------- | --------------------------------------- | ----------------------------------- | -------------------------------------------------------------------------------------------- |
| Manual transaction CRUD (TXN-01) | API / Backend                           | —                                   | Service-layer business logic + persistence in PG; frontend is just a form                    |
| CSV parsing (TXN-04)             | API / Backend                           | —                                   | Reuse `activities/csv_parser.rs` server-side; bytes go in, structured rows + errors come out |
| OFX parsing (TXN-05)             | API / Backend                           | —                                   | `sgmlish` is a Rust crate; parsing on the client would be wrong                              |
| Duplicate detection (TXN-06)     | API / Backend                           | —                                   | Pure SQL window over recent rows + Rust score function; client gets a list of candidates     |
| Search / filter (TXN-03)         | API / Backend                           | —                                   | Indexed PG queries; UI sends filter + cursor                                                 |
| Running balance (TXN-09)         | API / Backend (SQL view)                | Frontend (display)                  | Window function in PG; UI just renders the precomputed `running_balance` column              |
| FX conversion display (TXN-07)   | API / Backend (rate snapshot at insert) | Frontend (display only)             | Snapshot at write-time; reuse `crates/core/src/fx/`; UI never queries FX directly            |
| Splits CRUD (TXN-08)             | API / Backend                           | —                                   | Diesel transaction wraps parent + splits; UI sends one payload                               |
| Wizard state machine             | Frontend                                | —                                   | Existing pattern; same as `activity-import-page.tsx`. Pure UI state                          |
| Reconciliation hook              | API / Backend (TransactionService)      | API / Backend (AccountService read) | Fires on first-transaction-insert; reads `accounts.opening_balance` + `current_balance`      |
| Category memory persist + lookup | API / Backend                           | —                                   | New `payee_category_memory` table; service reads on import preview, writes on create/edit    |

## Standard Stack

### Core (verified existing in workspace `Cargo.toml`)

| Library                | Version                                              | Purpose                                   | Why Standard                                                                   |
| ---------------------- | ---------------------------------------------------- | ----------------------------------------- | ------------------------------------------------------------------------------ |
| `diesel`               | 2.3 (`postgres`, `chrono`, `numeric`, `uuid`)        | Schema + DSL                              | Project standard `[VERIFIED: ./Cargo.toml:29]`                                 |
| `diesel-async`         | 0.8 (`postgres`, `deadpool`, `migrations`)           | Async runtime for Diesel                  | Project standard `[VERIFIED: ./Cargo.toml:31]`                                 |
| `rust_decimal`         | 1.39 (`maths`, `serde-float`, `db-diesel2-postgres`) | Money type                                | Project standard `[VERIFIED: ./Cargo.toml:48]`                                 |
| `chrono`               | 0.4 (`serde`)                                        | Date/time                                 | Project standard `[VERIFIED: ./Cargo.toml:35]`                                 |
| `uuid`                 | 1 (`v4`, `v7`, `serde`)                              | Transaction IDs (use v7 for time-ordered) | `accounts/repository.rs:34` already uses `Uuid::now_v7()` `[VERIFIED]`         |
| `serde` / `serde_json` | 1                                                    | Serialization                             | Project standard `[VERIFIED]`                                                  |
| `csv`                  | 1.4.0                                                | CSV parsing                               | Already in `whaleit-core` `Cargo.toml` `[VERIFIED: crates/core/Cargo.toml:43]` |
| `chardetng`            | 0.1                                                  | Encoding detection                        | Already in `whaleit-core` `[VERIFIED]`                                         |
| `encoding_rs`          | 0.8                                                  | UTF-16 / Shift_JIS / Win-1252 decoding    | Already in `whaleit-core` `[VERIFIED]`                                         |
| `sha2`                 | 0.10                                                 | Idempotency hash                          | Already in `whaleit-core` `[VERIFIED]`                                         |
| `hex`                  | 0.4                                                  | Hash encoding                             | Already in `whaleit-core` `[VERIFIED]`                                         |
| `regex`                | 1.10                                                 | Merchant normalization                    | Already in `whaleit-core` `[VERIFIED]`                                         |

### New dependencies for Phase 4 (must add)

| Library   | Version | Purpose                                                                         | Why                                                                                                                                                                   |
| --------- | ------- | ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `sgmlish` | 0.2     | OFX 1.x SGML parsing                                                            | Only mature Rust SGML parser with explicit OFX support `[CITED: lib.rs/crates/sgmlish]` `[ASSUMED: still latest — verify with `cargo search sgmlish` before locking]` |
| `strsim`  | 0.11    | Levenshtein / Jaro-Winkler for duplicate-detection confidence multiplier (D-07) | Most-downloaded string-similarity crate in Rust ecosystem; pure-Rust; no native deps `[CITED: github.com/rapidfuzz/strsim-rs]` `[ASSUMED: latest version]`            |

### Supporting (already in monorepo)

| Library                                                          | Version   | Purpose                              | When to Use                                     |
| ---------------------------------------------------------------- | --------- | ------------------------------------ | ----------------------------------------------- |
| `whaleit-core::activities::csv_parser`                           | (in-repo) | CSV row parsing primitives           | Re-export verbatim; do NOT fork                 |
| `whaleit-core::activities::idempotency::compute_idempotency_key` | (in-repo) | SHA-256 idempotency pattern          | Adapt signature for transaction fields          |
| `whaleit-core::fx::FxServiceTrait`                               | (in-repo) | FX rate snapshot + convert           | Call at transaction-insert time for D-02        |
| `whaleit-core::taxonomies::*`                                    | (in-repo) | System taxonomy CRUD                 | Seed `"Transaction Categories"` + inline create |
| `whaleit-core::events::DomainEventSink`                          | (in-repo) | Publish "transaction inserted" event | Drives invalidation + reconciliation downstream |

### Alternatives Considered

| Instead of                                 | Could Use                                   | Tradeoff                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------------ | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `sgmlish` for OFX 1.x                      | Hand-roll a minimal SGML normalizer         | Hand-roll requires re-implementing OFX-1.x quirks (unclosed tags, `<TAG>value` without `</TAG>`, FIDIR/CHARSET headers). Estimated 200-400 LOC + tests for marginal control. `sgmlish` is 0.2.0, MIT, stable since Sep 2021 (`[CITED: lib.rs/crates/sgmlish]`). Reject hand-roll.                                                                                                                                                                                                                                                                                                    |
| `sgmlish` for OFX 1.x                      | `ofxy` crate                                | `ofxy` is "early stages of development" `[CITED: docs.rs/ofxy]`. Reject — too immature for v1 production use.                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `sgmlish` for OFX 1.x                      | `ofx-rs`                                    | `ofx-rs` is OpenFX (graphics plugin SDK), unrelated to financial OFX `[CITED: github.com/itadinanta/ofx-rs]`. Reject — not the same library.                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `transaction_splits` table                 | `splits JSONB` column on `transactions`     | JSONB requires `jsonb_array_elements()` unnesting on every Phase 5 budget rollup and every Phase 6 spending-by-category report. Indexes are awkward (GIN on JSONB doesn't help category aggregation). A child table with `(transaction_id, category_id)` index makes future joins free. Reject JSONB.                                                                                                                                                                                                                                                                                |
| Window-function running balance            | Materialized `running_balance` column       | Materialized requires recompute on every out-of-order insert (CSV imports of historical data routinely insert mid-history). Trigger logic must lock the affected per-account suffix and rewrite N rows. The savings are only paid back at >100k rows/account, which Phase 4 will not see. Window function with `(account_id, transaction_date DESC, created_at DESC)` index runs <50ms for 10k-row partitions in PG `[CITED: postgresql.org/docs/current/tutorial-window.html]` `[ASSUMED: typical bare-metal PG perf — should be benchmarked in plan]`. Reject materialized for v1. |
| `strsim::levenshtein` for payee similarity | Token-set ratio (split + intersect / union) | Levenshtein is O(m·n) per pair (small for short merchant strings). Token-set is more permissive on word reorder ("Whole Foods Market" vs "Market Foods Whole") but bank statements rarely reorder. Levenshtein ratio aligns with normalized merchant strings already lowercase + collapsed digits per D-13. Pick `strsim::normalized_levenshtein` (returns 0.0..1.0).                                                                                                                                                                                                                |
| New `payee_category_memory` table          | JSON column on settings                     | Single-record JSON would deserialize the full memory map on every lookup. A keyed table with index on `(account_id, normalized_merchant)` is O(1) lookup, supports `last_seen_at` ORDER BY, and aligns with last-write-wins update semantics in D-14.                                                                                                                                                                                                                                                                                                                                |

**Installation (additions to `crates/core/Cargo.toml`):**

```toml
sgmlish = "0.2"
strsim = "0.11"
```

**Version verification (planner MUST run before locking versions):**

```bash
cargo search sgmlish strsim
```

Document the version + publish date next to the table above. Both crates have
last-update dates older than 12 months, so a check is cheap.

## Architecture Patterns

### System Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                           FRONTEND (React)                         │
│                                                                    │
│  /transactions ledger    Form  Detail sheet  Importer   Dup-review │
│         │                  │         │           │           │     │
│         └──────────────────┴─────────┴───────────┴───────────┘     │
│                            │                                       │
│         adapters/shared/transactions.ts (typed wrappers)           │
│                            │                                       │
│           adapters/web/core.ts ── adapters/tauri/core.ts           │
│                  (both fetch the same Axum server)                 │
└─────────────────────────────────┬──────────────────────────────────┘
                                  │ HTTP + JSON
┌─────────────────────────────────▼──────────────────────────────────┐
│                  AXUM SERVER (apps/server/src/api/)                │
│                                                                    │
│       transactions.rs (NEW) ──── /transactions/* routes            │
│                            │                                       │
└────────────────────────────┼───────────────────────────────────────┘
                             │
┌────────────────────────────▼───────────────────────────────────────┐
│             whaleit-core (crates/core/src/transactions/)           │
│                                                                    │
│     transactions_service.rs                                        │
│        ├─ create_transaction()                                     │
│        │     ├─ reconciliation hook (first-insert detection)       │
│        │     ├─ FX snapshot via fx_service                         │
│        │     ├─ memory lookup → suggested category                 │
│        │     ├─ idempotency_key compute                            │
│        │     ├─ repository.create_with_splits()                    │
│        │     ├─ memory write (last-write-wins)                     │
│        │     └─ event_sink.emit(TransactionInserted)               │
│        ├─ update_transaction()                                     │
│        │     ├─ paired-sibling sync (D-04)                         │
│        │     └─ memory write on category change                    │
│        ├─ search_transactions()  ◄── TXN-03                        │
│        ├─ list_running_balance()  ◄── TXN-09 (window function)     │
│        ├─ import_csv()                                              │
│        │     ├─ csv_parser::parse_csv (re-exported from activities)│
│        │     ├─ compiler::compile_rows (transaction-specific)      │
│        │     ├─ duplicate_detector::detect (within-batch + DB)     │
│        │     └─ memory pre-fill                                    │
│        └─ import_ofx()                                              │
│              └─ ofx_parser::parse_ofx (sgmlish-driven)             │
│                                                                    │
│     duplicate_detector.rs    merchant_normalizer.rs                │
│     idempotency.rs           ofx_parser.rs   compiler.rs           │
└──────────────────────────────────┬─────────────────────────────────┘
                                   │ trait
┌──────────────────────────────────▼─────────────────────────────────┐
│        crates/storage-postgres/src/transactions/ (NEW)             │
│                                                                    │
│     model.rs   ── TransactionDB, TransactionSplitDB,               │
│                   PayeeCategoryMemoryDB, From impls                │
│     repository.rs ── PgTransactionRepository                       │
│         (Diesel async + deadpool pool, mirrors                     │
│          PgAccountRepository pattern verbatim)                     │
└──────────────────────────────────┬─────────────────────────────────┘
                                   │ SQL
┌──────────────────────────────────▼─────────────────────────────────┐
│                          PostgreSQL                                │
│                                                                    │
│  transactions  transaction_splits  payee_category_memory           │
│  taxonomies (existing) ──── 1 new system row "Transaction Categories"
│  taxonomy_categories (existing) ──── 10 seeded category rows       │
│                                                                    │
│  v_transactions_with_running_balance (VIEW, window function)       │
└────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
crates/core/src/transactions/                   # NEW
├── mod.rs                                      # Public re-exports
├── transactions_model.rs                       # Transaction, NewTransaction, TransactionUpdate, TransactionSplit, NewSplit, PayeeCategoryMemory
├── transactions_constants.rs                   # TX_DIRECTION_*, TX_STATUS_*, default sort orders
├── transactions_service.rs                     # TransactionService impl (create/update/delete/search/import/running_balance)
├── transactions_service_tests.rs
├── transactions_traits.rs                      # TransactionRepositoryTrait + TransactionServiceTrait
├── transactions_errors.rs                      # phase-specific TransactionError variants
├── compiler.rs                                 # Rows → typed Transaction (mirrors activities/compiler.rs)
├── csv_parser.rs                               # Re-export `crate::activities::csv_parser::parse_csv` + transaction-specific row→fields mapping
├── ofx_parser.rs                               # sgmlish-driven OFX 1.x parser, optional XML branch
├── ofx_parser_tests.rs                         # Real-world OFX fixtures (anonymized)
├── duplicate_detector.rs                       # 3-key gate + Levenshtein-based confidence formula
├── duplicate_detector_tests.rs
├── idempotency.rs                              # compute_transaction_idempotency_key (CSV: hash; OFX: FITID-or-hash fallback)
├── merchant_normalizer.rs                      # Pure D-13 algo: lowercase + strip + collapse digits/spaces
├── merchant_normalizer_tests.rs                # Cover D-13 examples + Unicode + EU thousand separator
└── reconciliation.rs                           # Phase-3-D-14 first-insert hook: synthesize Opening Balance + Balance Adjustment

crates/storage-postgres/src/transactions/       # NEW
├── mod.rs
├── model.rs                                    # TransactionDB, TransactionSplitDB, PayeeCategoryMemoryDB + From<Domain> + Into<Domain>
├── repository.rs                               # PgTransactionRepository
├── repository_tests.rs                         # Integration tests via testcontainers (already used in storage-postgres)
└── migration_tests.rs                          # Migration up/down round-trip

crates/storage-postgres/migrations/20260501000000_transactions_initial/   # NEW
├── up.sql                                      # CREATE TABLE transactions, transaction_splits, payee_category_memory; CREATE VIEW v_transactions_with_running_balance; INSERT system taxonomy + 10 categories
└── down.sql                                    # Reverse, in dependency order

apps/server/src/api/transactions.rs             # NEW Axum router: /transactions, /transactions/search, /transactions/import/csv, /transactions/import/ofx, /transactions/duplicates, /transactions/{id}, /transactions/payee-category-memory

apps/frontend/src/pages/transactions/           # NEW
├── transactions-page.tsx                       # /transactions ledger
├── transaction-detail-sheet.tsx
├── transaction-form.tsx                        # New / Edit form
├── transaction-row.tsx                         # Reused by ledger + per-account "Recent transactions"
├── transaction-list.tsx                        # Date-grouped list with running balance
├── filter-bar/                                 # filter chips, search, popovers
├── duplicate-review-sheet.tsx
├── duplicate-banner.tsx
├── split-editor.tsx
├── recent-transactions.tsx                     # Embedded into account-page.tsx
└── import/
    ├── transaction-import-page.tsx             # FORK from activity-import-page.tsx
    ├── components/
    │   ├── (re-exports of FileDropzone, CSVFileViewer, WizardStepIndicator,
    │   │   StepNavigation, HelpTooltip, CancelConfirmationDialog from activity/import/components)
    │   ├── transaction-mapping-table.tsx       # FORK
    │   └── transaction-template-picker.tsx     # FORK
    ├── context/
    │   └── transaction-import-context.tsx      # FORK ImportProvider — simpler state shape
    ├── steps/
    │   ├── upload-step.tsx                     # FORK or share (file upload + account picker)
    │   ├── transaction-mapping-step.tsx        # FORK from mapping-step-unified.tsx
    │   ├── transaction-review-step.tsx         # FORK from review-step.tsx
    │   └── transaction-confirm-step.tsx        # FORK from confirm-step.tsx
    └── utils/
        ├── transaction-draft-utils.ts          # FORK from draft-utils.ts
        └── transaction-default-template.ts     # FORK from default-activity-template.ts

apps/frontend/src/adapters/shared/transactions.ts                 # NEW adapter shim (mirrors activities.ts)
apps/frontend/src/adapters/web/core.ts                            # EXTEND COMMANDS map
```

### Pattern 1: Service + Repository Trait Mirror

**What:** Define service operations in `crates/core` behind
`TransactionServiceTrait` + `TransactionRepositoryTrait`. Implement
`PgTransactionRepository` in `crates/storage-postgres`. Expose via Axum at
`apps/server/src/api/transactions.rs`.

**When to use:** Every Phase 4 backend feature.

**Example (verbatim pattern from `accounts_service.rs`):**

```rust
// Source: crates/core/src/accounts/accounts_service.rs:24-42 [VERIFIED]
pub struct TransactionService {
    repository: Arc<dyn TransactionRepositoryTrait>,
    account_repository: Arc<dyn AccountRepositoryTrait>,
    fx_service: Arc<dyn FxServiceTrait>,
    base_currency: Arc<RwLock<String>>,
    event_sink: Arc<dyn DomainEventSink>,
    payee_memory_repository: Arc<dyn PayeeCategoryMemoryRepositoryTrait>,
}
```

### Pattern 2: From-Impls At The Diesel Boundary

**What:** Domain types live in `whaleit-core`. Diesel-aware DB types live in
`storage-postgres`. `From<NewX> for XDB`, `From<XUpdate> for XDB`, and
`From<XDB> for X` impls bridge them.

**When to use:** All transaction storage code.

**Example:** see `crates/storage-postgres/src/accounts/model.rs:46-167`
`[VERIFIED]` — same shape exactly for `TransactionDB`.

### Pattern 3: Idempotency-Key Hash

**What:** SHA-256 hash of normalized canonical fields. Activity pattern is
verbatim reusable.

**When to use:** Every CSV-imported row gets a key; OFX rows use FITID when
present, fall back to the same hash.

**Example:**

```rust
// Source: crates/core/src/activities/idempotency.rs:25-91 [VERIFIED]
pub fn compute_transaction_idempotency_key(
    account_id: &str,
    direction: &str,            // "INCOME" | "EXPENSE" | "TRANSFER"
    transaction_date: &NaiveDate,
    amount: Decimal,
    currency: &str,
    payee: Option<&str>,
    external_ref: Option<&str>, // OFX FITID or CSV bank-ref column
) -> String { /* SHA-256 hex */ }
```

For OFX with FITID: `external_ref = Some(fitid)` → bank-stable key. For CSV or
OFX without FITID: `external_ref = None` → key derives entirely from fields,
identical-to-the-byte rows collapse.

### Pattern 4: Compiler (Staged Compile From Parsed Rows To Typed Domain)

**What:** Activity import has `compile_drip` etc. that expand a single stored
row into multiple postings. Phase 4 transactions don't need multi-leg expansion
(transfers are stored as two rows directly), so the compiler is **simpler**:
each `ParsedRow → 1 NewTransaction`. But the indirection layer is the right home
for: payee normalization, direction-from-sign inference, currency defaulting, FX
rate snapshotting, category memory pre-fill.

**Example:** see `crates/core/src/activities/compiler.rs:36-67` `[VERIFIED]`.

### Anti-Patterns to Avoid

- **Manual SQL for running balance via `LAG()`/`COALESCE` chains**: PG's
  `SUM() OVER (PARTITION BY account_id ORDER BY transaction_date, created_at)`
  is the right primitive. Don't recurse.
- **Storing splits as `Vec<Split>` JSON in the parent row**: blocks Phase 5
  category aggregation. Use `transaction_splits` table.
- **Embedding parsing logic in the Axum handler**: handlers stay thin glue. All
  parsing happens in `whaleit-core::transactions::*_parser`.
- **Hand-rolling string similarity for the duplicate detector**: use `strsim`.
  Levenshtein is decades-old, never a custom-implementation priority.
- **Leaking a SQLite-era assumption**: project is PG-only. Reject any plan that
  mentions a `crates/storage-sqlite` path.

## Don't Hand-Roll

| Problem                                   | Don't Build                       | Use Instead                                                                      | Why                                                                                                  |
| ----------------------------------------- | --------------------------------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| OFX 1.x SGML parsing                      | A custom SGML normalizer          | `sgmlish` 0.2                                                                    | Unclosed tags, FIDIR/CHARSET headers, entity edge cases — covered by 4 yrs of bug-fixes in the crate |
| String similarity for dedup confidence    | A custom Levenshtein              | `strsim::normalized_levenshtein`                                                 | Decades-old algorithm; pure-Rust crate is canonical                                                  |
| CSV encoding detection                    | Manually probing UTF-8 / Latin-1  | `chardetng` (already in repo)                                                    | Already used by `activities/csv_parser.rs`                                                           |
| Idempotency hashing                       | Custom hash composition           | Reuse `activities/idempotency.rs` shape                                          | Proven; SHA-256 with normalized fields is the project pattern                                        |
| Decimal arithmetic                        | Float math anywhere               | `rust_decimal::Decimal` everywhere                                               | Already standardized; `serde-float` enabled                                                          |
| UUID generation                           | Random bytes                      | `Uuid::now_v7()`                                                                 | Time-ordered IDs; `accounts/repository.rs:34` already uses this                                      |
| Running balance                           | Recursive CTE or manual LAG chain | PG `SUM() OVER (PARTITION BY account_id ORDER BY transaction_date, created_at)`  | Native window function; one statement; index-friendly                                                |
| Date-format detection on CSV import       | Heuristic Rust regex              | Existing `parse_csv` `detected_config.date_format`                               | Wired through `activities/csv_parser.rs`                                                             |
| Duplicate-detection within-batch ordering | Custom batch-buffering            | Detector queries DB by `(account_id, transaction_date)` window with `±3d` filter | One PG query per import, in-memory cross-check on the rows already in the batch                      |

**Key insight:** Phase 4 has zero genuinely new infrastructure problems — every
primitive needed is already in the workspace. The lift is composing them, not
inventing them.

## Runtime State Inventory

> Phase 4 is a brand-new domain (no `transactions` table exists), not a rename.
> Inventory still applies because the new `payee_category_memory` store +
> reconciliation hook touch existing PG state.

| Category                             | Items Found                                                                                                                            | Action Required                                                                             |
| ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Stored data                          | None — `transactions` table does not exist yet `[VERIFIED: schema.rs lines 12-42 only has accounts]`. New tables created from scratch. | New migration                                                                               |
| Live service config                  | None — no external services depend on a `transactions` table existing                                                                  | None                                                                                        |
| OS-registered state                  | None — no Task Scheduler / pm2 / launchd jobs reference transactions                                                                   | None                                                                                        |
| Secrets and env vars                 | None — no env keys mention transactions; FX provider keys reused via existing `fx_service`                                             | None                                                                                        |
| Build artifacts / installed packages | New crate dependencies (`sgmlish`, `strsim`) require `cargo build` after `Cargo.toml` edit                                             | `cargo build` after dependency add; `cargo update -p sgmlish -p strsim` to confirm versions |

**Indirect runtime state from Phase 3 D-14 reconciliation hook:**

- The `accounts.opening_balance` and `accounts.current_balance` columns are
  populated for every Phase-3-era account
  `[VERIFIED: 20260425000000_accounts_extend_types_and_balances/up.sql]`. Phase
  4's first-transaction-insert reads both. **Action:** the reconciliation
  generator MUST tolerate `opening_balance IS NULL` for legacy pre-Phase-3
  accounts (`SECURITIES`, `CASH`, etc.) by skipping reconciliation.
- The `accounts.balance_updated_at` column is auto-stamped by
  `accounts_service.rs:118` `[VERIFIED]`. Phase 4 uses this timestamp to decide
  whether a "Balance adjustment" row is needed (delta since `created_at` ≠
  sum(transactions) → synthesize the adjustment).
- The existing system taxonomy CRUD
  `[VERIFIED: taxonomies/taxonomy_service.rs:143]` is used to seed
  `"Transaction Categories"`. Idempotent seed (UPSERT on
  `(name, is_system=true)`) so re-running migrations doesn't duplicate.

## Schema Design

### Migration: `20260501000000_transactions_initial`

```sql
-- crates/storage-postgres/migrations/20260501000000_transactions_initial/up.sql

-- ============================================================================
-- transactions: the ledger source of truth
-- ============================================================================
CREATE TABLE transactions (
    id                  TEXT PRIMARY KEY,                     -- UUIDv7 string
    account_id          TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,

    -- Core fields
    direction           TEXT NOT NULL CHECK (direction IN ('INCOME', 'EXPENSE', 'TRANSFER')),
    amount              NUMERIC(20,8) NOT NULL CHECK (amount > 0),  -- always positive; sign comes from direction
    currency            TEXT NOT NULL,
    transaction_date    DATE NOT NULL,                              -- the user-meaningful date
    payee               TEXT,                                       -- NULL for transfers
    notes               TEXT,
    category_id         TEXT REFERENCES taxonomy_categories(id) ON DELETE SET NULL,  -- NULL when split (use transaction_splits)
    has_splits          BOOLEAN NOT NULL DEFAULT FALSE,             -- index hint; mirrors `transaction_splits` row presence

    -- Multi-currency (TXN-07, D-02)
    fx_rate             NUMERIC(20,8),                              -- snapshot at write-time; NULL when currency = account.currency
    fx_rate_source      TEXT CHECK (fx_rate_source IN ('SYSTEM', 'MANUAL_OVERRIDE', NULL)),

    -- Transfer pairing (D-01..D-05)
    transfer_group_id   TEXT,                                       -- non-NULL only on transfer-direction rows; both legs share it
    counterparty_account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,  -- helper denormalization for transfer detail-sheet rendering

    -- Idempotency / import (TXN-04, TXN-05)
    idempotency_key     TEXT UNIQUE,                                -- SHA-256 of normalized fields; NULL only for system-generated reconciliation rows
    import_run_id       TEXT,                                       -- groups rows imported in one wizard run
    source              TEXT NOT NULL CHECK (source IN ('MANUAL', 'CSV', 'OFX', 'SYSTEM')),
    external_ref        TEXT,                                       -- OFX FITID or user-entered "Reference / external ID"

    -- Audit
    is_system_generated BOOLEAN NOT NULL DEFAULT FALSE,             -- Phase-3-D-14 reconciliation rows
    is_user_modified    BOOLEAN NOT NULL DEFAULT FALSE,             -- post-import edit flag (parallels activities.is_user_modified)
    category_source     TEXT CHECK (category_source IN ('USER', 'MEMORY', 'IMPORT', NULL)),  -- audit metadata for Phase 8 AI training; NOT user-visible

    created_at          TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at          TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),

    -- Constraints
    CONSTRAINT transfer_must_have_group_id  CHECK (
        (direction = 'TRANSFER' AND transfer_group_id IS NOT NULL)
        OR (direction != 'TRANSFER' AND transfer_group_id IS NULL)
    ),
    CONSTRAINT non_transfer_must_have_payee CHECK (
        direction = 'TRANSFER' OR payee IS NOT NULL
    ),
    CONSTRAINT counterparty_only_for_transfer CHECK (
        (direction = 'TRANSFER' AND counterparty_account_id IS NOT NULL)
        OR (direction != 'TRANSFER' AND counterparty_account_id IS NULL)
    )
);

-- Hot-path indexes
CREATE INDEX idx_tx_account_date          ON transactions (account_id, transaction_date DESC, created_at DESC);  -- ledger query, running-balance window
CREATE INDEX idx_tx_account_idempotency   ON transactions (account_id, idempotency_key);                          -- re-import dedup
CREATE INDEX idx_tx_transfer_group        ON transactions (transfer_group_id) WHERE transfer_group_id IS NOT NULL;-- partial: only ~5% of rows
CREATE INDEX idx_tx_category              ON transactions (category_id) WHERE category_id IS NOT NULL;            -- partial; supports per-category aggregation in Phase 5/6
CREATE INDEX idx_tx_import_run            ON transactions (import_run_id) WHERE import_run_id IS NOT NULL;
CREATE INDEX idx_tx_payee_trgm            ON transactions USING gin (payee gin_trgm_ops);                         -- TXN-03 search; requires `pg_trgm` extension (already on PG18)
CREATE INDEX idx_tx_date                  ON transactions (transaction_date DESC);                                 -- cross-account global ledger pagination

-- Note on partial index for splits:
CREATE INDEX idx_tx_has_splits            ON transactions (account_id, transaction_date DESC) WHERE has_splits = TRUE;

-- ============================================================================
-- transaction_splits: child of transactions (TXN-08)
-- ============================================================================
CREATE TABLE transaction_splits (
    id              TEXT PRIMARY KEY,
    transaction_id  TEXT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    category_id     TEXT NOT NULL REFERENCES taxonomy_categories(id) ON DELETE RESTRICT,
    amount          NUMERIC(20,8) NOT NULL CHECK (amount > 0),
    notes           TEXT,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    updated_at      TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);

CREATE INDEX idx_tx_splits_tx_id    ON transaction_splits (transaction_id);
CREATE INDEX idx_tx_splits_category ON transaction_splits (category_id);

-- Service layer enforces: SUM(transaction_splits.amount WHERE transaction_id = X) = transactions.amount.
-- NOT enforced as DB constraint to keep batch insert cheap; `transactions_service.rs::create_transaction`
-- wraps parent + splits in one Diesel transaction and validates the sum before commit.

-- ============================================================================
-- payee_category_memory: D-12 lookup
-- ============================================================================
CREATE TABLE payee_category_memory (
    account_id          TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    normalized_merchant TEXT NOT NULL,
    category_id         TEXT NOT NULL REFERENCES taxonomy_categories(id) ON DELETE CASCADE,
    last_seen_at        TIMESTAMP NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
    seen_count          INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (account_id, normalized_merchant)
);

CREATE INDEX idx_payee_mem_last_seen ON payee_category_memory (last_seen_at DESC);

-- ============================================================================
-- v_transactions_with_running_balance: window-function VIEW (TXN-09)
-- ============================================================================
CREATE VIEW v_transactions_with_running_balance AS
SELECT
    t.*,
    SUM(
        CASE
            WHEN t.direction = 'INCOME' THEN t.amount
            WHEN t.direction = 'EXPENSE' THEN -t.amount
            -- For transfers, the leg's effect on this account is signed by whether it's
            -- the source (account_id == this row.account_id, counterparty != this) or
            -- destination (vice versa). We don't know which without examining the pair,
            -- so we inspect: if the leg.amount has been pre-signed at insert (NO — we
            -- store positive amount + direction), the rule is: source leg is debit
            -- (subtract), destination leg is credit (add). The repository writes a
            -- helper column `transfer_leg_role` ('SOURCE' | 'DESTINATION' | NULL)
            -- so the view can sum correctly. (This column is added below.)
            WHEN t.direction = 'TRANSFER' AND t.transfer_leg_role = 'DESTINATION' THEN t.amount
            WHEN t.direction = 'TRANSFER' AND t.transfer_leg_role = 'SOURCE' THEN -t.amount
            ELSE 0
        END
    ) OVER (
        PARTITION BY t.account_id
        ORDER BY t.transaction_date ASC, t.created_at ASC
        ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
    ) AS running_balance
FROM transactions t;
```

> **NOTE on `transfer_leg_role`:** add this column to `transactions` alongside
> `counterparty_account_id`. Values: `'SOURCE'` (this leg debits the account),
> `'DESTINATION'` (this leg credits the account), `NULL` for non-transfers.
> Without it the view would need a self-join — the column is the simplest way to
> keep the view a pure aggregate.

```sql
-- Adds to transactions:
--   transfer_leg_role TEXT CHECK (transfer_leg_role IN ('SOURCE', 'DESTINATION', NULL))
--   AND a constraint: transfer_leg_role IS NOT NULL iff direction = 'TRANSFER'
```

### Seed: `"Transaction Categories"` system taxonomy

```sql
-- 10 default categories per UI-SPEC §Category Model (10:53a)
INSERT INTO taxonomies (id, name, color, description, is_system, is_single_select, sort_order, created_at, updated_at)
VALUES (
    'sys_taxonomy_transaction_categories',
    'Transaction Categories',
    '#8abceb',
    'System-managed transaction categories',
    TRUE,
    TRUE,
    100,
    NOW() AT TIME ZONE 'utc',
    NOW() AT TIME ZONE 'utc'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO taxonomy_categories (id, taxonomy_id, parent_id, name, key, color, description, sort_order, created_at, updated_at)
VALUES
    ('cat_income',        'sys_taxonomy_transaction_categories', NULL, 'Income',        'income',        '#36b81e', NULL,  0, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_dining',        'sys_taxonomy_transaction_categories', NULL, 'Dining',        'dining',        '#f4a06b', NULL,  1, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_entertainment', 'sys_taxonomy_transaction_categories', NULL, 'Entertainment', 'entertainment', '#cba5e1', NULL,  2, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_groceries',     'sys_taxonomy_transaction_categories', NULL, 'Groceries',     'groceries',     '#a29c8a', NULL,  3, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_healthcare',    'sys_taxonomy_transaction_categories', NULL, 'Healthcare',    'healthcare',    '#73d7e6', NULL,  4, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_housing',       'sys_taxonomy_transaction_categories', NULL, 'Housing',       'housing',       '#85b4d6', NULL,  5, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_shopping',      'sys_taxonomy_transaction_categories', NULL, 'Shopping',      'shopping',      '#e6a8c8', NULL,  6, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_transport',     'sys_taxonomy_transaction_categories', NULL, 'Transport',     'transport',     '#f5c873', NULL,  7, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_utilities',     'sys_taxonomy_transaction_categories', NULL, 'Utilities',     'utilities',     '#f5d770', NULL,  8, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc'),
    ('cat_uncategorized', 'sys_taxonomy_transaction_categories', NULL, 'Uncategorized', 'uncategorized', '#b8b3a8', NULL, 99, NOW() AT TIME ZONE 'utc', NOW() AT TIME ZONE 'utc')
ON CONFLICT (id) DO NOTHING;
```

> **Seeding strategy choice:** put the seed in the migration's `up.sql` file
> (atomic, irrevocable in the migration history), not in
> `taxonomy_service::initialize()`. Reasons: (1) re-running migrations on a
> fresh DB always seeds; (2) the IDs are stable strings the frontend can
> hard-reference (e.g., for the "Income" default direction mapping); (3) a
> service-layer seeder is harder to test and harder to drop/restore from backups
> in lockstep.

## Crate and Module Layout

See "Recommended Project Structure" above. The exact responsibilities are:

| File                                 | Responsibility                                                                                                                                                                                                                               | Mirrors                                                                                  |
| ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `transactions_model.rs`              | Pure-domain structs: `Transaction`, `NewTransaction`, `TransactionUpdate`, `TransactionSplit`, `NewSplit`, `SplitUpdate`, `PayeeCategoryMemory`. `validate()` impls. NO Diesel.                                                              | `accounts_model.rs`                                                                      |
| `transactions_constants.rs`          | `direction::INCOME` etc. const strings.                                                                                                                                                                                                      | `accounts_constants.rs`                                                                  |
| `transactions_service.rs`            | All business logic. ~600-1000 LOC. Hooks reconciliation, paired-leg sync, FX snapshot, memory write.                                                                                                                                         | `accounts_service.rs` (smaller — Phase 4 service is bigger because it owns import paths) |
| `transactions_traits.rs`             | `TransactionRepositoryTrait`, `TransactionServiceTrait`.                                                                                                                                                                                     | `accounts_traits.rs`                                                                     |
| `csv_parser.rs`                      | Re-export `crate::activities::csv_parser::parse_csv` and add transaction-specific row→fields mapper.                                                                                                                                         | `activities/csv_parser.rs`                                                               |
| `ofx_parser.rs`                      | OFX 1.x SGML and OFX 2.x XML → `Vec<NewTransaction>`. ~400 LOC.                                                                                                                                                                              | (new)                                                                                    |
| `compiler.rs`                        | `compile_csv_rows(rows: &[ParsedRow], mapping: &Mapping, account_id: &str) -> Vec<NewTransaction>`.                                                                                                                                          | `activities/compiler.rs` (simpler — no leg expansion)                                    |
| `duplicate_detector.rs`              | `detect_duplicates(candidates: &[NewTransaction], existing_in_window: &[Transaction]) -> Vec<DuplicateMatch>`.                                                                                                                               | (new)                                                                                    |
| `idempotency.rs`                     | `compute_transaction_idempotency_key(...) -> String`.                                                                                                                                                                                        | `activities/idempotency.rs`                                                              |
| `merchant_normalizer.rs`             | `normalize_merchant(s: &str) -> String`. Pure-function regex per D-13.                                                                                                                                                                       | (new)                                                                                    |
| `reconciliation.rs`                  | `synthesize_reconciliation_rows(account: &Account, sum_so_far: Decimal) -> Vec<NewTransaction>`. Called only when first non-system row inserts against the account.                                                                          | (new)                                                                                    |
| `storage-postgres/.../model.rs`      | `TransactionDB`, `TransactionSplitDB`, `PayeeCategoryMemoryDB` + From-impls.                                                                                                                                                                 | `storage-postgres/src/accounts/model.rs`                                                 |
| `storage-postgres/.../repository.rs` | `PgTransactionRepository`. Operations: `create_with_splits` (Diesel txn), `update_with_splits`, `delete`, `get_by_id`, `search`, `list_in_window`, `list_with_running_balance`, `lookup_payee_memory`, `upsert_payee_memory`, `delete_pair`. | `storage-postgres/src/accounts/repository.rs`                                            |

## OFX Parsing Strategy

**Decision: `sgmlish` 0.2 for OFX 1.x; same crate handles OFX 2.x XML through
its XML-tolerant config.**

### Why `sgmlish`

- Pure Rust, no native dependencies, MIT license, ~1.7k downloads/month
  `[CITED: lib.rs/crates/sgmlish]`.
- Documentation **explicitly calls out OFX 1.x** as a target use case.
- Serde integration: `sgmlish::from_str::<OfxDocument>(s)` → typed.

### What about OFX 2.x

OFX 2.x is well-formed XML. Two viable options:

1. Use `sgmlish` for both (it's permissive enough for both formats).
2. Branch on the file header: `OFXHEADER:100 ENCODING:USASCII...` → 1.x;
   `<?xml version="1.0"?>` → 2.x via `quick-xml` or `serde-xml-rs`.

**Recommendation: use `sgmlish` for both.** Branching adds complexity for no
real gain — `sgmlish` parses both correctly. Skip `quick-xml` unless a specific
OFX 2.x file fails (then drop into a dedicated Phase 4.5).

### OFX 1.x Quirks the Parser Must Handle

| Quirk                                                             | How `sgmlish` handles it                                                                                                                            |
| ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Unclosed tags (`<TAG>value` without `</TAG>`)                     | `sgmlish` config `omitted_end_tags` covers this — the OFX standard's tag-set is fixed, so configure SGML normalizer with the OFX tag-omission list. |
| `OFXHEADER:100` plain-text header above the SGML body             | Strip header lines before passing to `sgmlish`. The header ends with a blank line. ~10 LOC of pre-parse.                                            |
| `CHARSET:1252` declaration                                        | Apply `encoding_rs` (already a dep) to convert to UTF-8 before parsing. Fall through to `chardetng` for missing CHARSET.                            |
| Self-closing semantics on `<TAG/>` style (rare in 1.x but exists) | `sgmlish` permits this with `xml_compatible(true)` config flag.                                                                                     |
| Mixed-case tag names (`<OfxResponse>` vs `<OFXRESPONSE>`)         | Normalize to upper before parsing — OFX spec is case-insensitive for tags.                                                                          |
| Embedded `&amp;` `&lt;` etc. inside text content                  | Configure `sgmlish` entity table; OFX uses standard entities.                                                                                       |

### Mandatory OFX Fields the Parser Extracts

Per `04-UI-SPEC.md` §5 + D-19:

| OFX Field                  | Use For                                                                                                                                        |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `FITID`                    | `transactions.external_ref` AND fallback FITID-or-hash idempotency key                                                                         |
| `TRNAMT`                   | `amount` (sign determines direction)                                                                                                           |
| `DTPOSTED`                 | `transaction_date`                                                                                                                             |
| `NAME`                     | primary `payee`                                                                                                                                |
| `MEMO`                     | secondary, falls back to `notes`                                                                                                               |
| `TRNTYPE`                  | maps to `direction`: `CREDIT/DEP/INT/DIV → INCOME`; `DEBIT/PAYMENT/CHECK/SRVCHG → EXPENSE`; `XFER → TRANSFER` (rare in single-account exports) |
| `CURDEF` (statement-level) | default `currency` for all transactions in this OFX file when the per-row `CURRENCY` block is missing                                          |

### Edge Cases

- OFX exports without `FITID` (some smaller credit unions). Fall back to the
  same SHA-256 idempotency hash used for CSV.
- Multi-account OFX (`<BANKMSGSRSV1>` + `<CREDITCARDMSGSRSV1>` blocks in one
  file). Phase 4: enforce one-account-per-import in the upload step (file scoped
  to user-chosen account at upload time per UI-SPEC) — if the OFX has multiple
  accounts, surface a parser warning and let the user pick which account.
- OFX 1.x with `\r\n` line endings on Windows exports — `sgmlish` handles
  natively.

## CSV Parsing Strategy

**Decision: re-export `crate::activities::csv_parser::parse_csv` verbatim; add a
transaction-specific row→fields compiler.**

### Reuse Plan

`crates/core/src/activities/csv_parser.rs` already handles `[VERIFIED]`:

- BOM detection (`\xEF\xBB\xBF` UTF-8 BOM, UTF-16 LE/BE).
- Delimiter auto-detection (comma / semicolon / tab) by consistency scoring
  across first 10 lines.
- Quote-character configurability (default `"`, custom `'`).
- Skip-top-rows / skip-bottom-rows for bank-export preambles and totals.
- Empty-row filtering.
- Encoding fallback chain: BOM → UTF-8 → `chardetng` statistical guess.
- Header detection at configurable index.
- Uneven-row padding/truncation with structured errors.

### Edge Cases Specific To Bank Statement CSVs

| Case                                                           | Mitigation                                                                                                                                                                                                           |
| -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Leading BOM                                                    | Already handled                                                                                                                                                                                                      |
| EU exports with `1.234,56` decimal format                      | `csv_parser::ParseConfig.decimal_separator` = `","`, `thousands_separator` = `"."`. Already a config knob.                                                                                                           |
| Date format MM/DD/YYYY vs DD/MM/YYYY ambiguity                 | Mapping step lets the user pick `date_format` per template. Default `auto` reads first 5-10 dates and picks the format that produces no future dates and no day > 31. Falls back to surfacing a warning on mismatch. |
| Combined "Debit" + "Credit" columns instead of single `amount` | UI-SPEC §5 mapping table: `amount` OR (`debit` + `credit`). Compiler reads either pair and computes signed amount + direction.                                                                                       |
| Trailing empty rows from bank summary footer                   | `skip_bottom_rows` config.                                                                                                                                                                                           |
| Currency column missing                                        | Default to `account.currency` (account is locked at upload step per UI-SPEC §5).                                                                                                                                     |
| Multi-line cells (notes with embedded `\n`)                    | Already handled by `csv` crate quoted-field parsing.                                                                                                                                                                 |
| Negative amounts in parentheses `($42.10)`                     | Compiler detects parens-wrap → strip + flip sign. **Note:** this is NOT in `activities/csv_parser.rs` today; the transaction compiler must add it.                                                                   |

### What's New for Phase 4

Per-template `header_signature` validation (D-17). On template re-use, compare
the file's first row against the saved template's expected headers. If mismatch
in any required field's expected position, surface inline message:
`"Your saved 'Chase Checking CSV' template doesn't match this file's columns. Re-map?"`.
Implementation: store `expected_headers: Vec<String>` on each saved template and
pre-validate in the Mapping step before the user sees the table.

## Duplicate Detection

### Algorithm

Per D-06, D-07, D-09. Three-key gate THEN confidence multiplier:

```
For each new candidate row C in the import batch:
  1. Query existing transactions where:
       account_id = C.account_id
       AND amount BETWEEN C.amount - 0.01 AND C.amount + 0.01
       AND transaction_date BETWEEN C.date - 3d AND C.date + 3d
  2. If ZERO existing matches → C is not a duplicate. Continue.
  3. For each existing match E:
       confidence = (
           amount_exactness * 0.4
         + date_closeness    * 0.3
         + payee_similarity  * 0.3
       ) * 100
       where:
         amount_exactness = 1.0 if |E.amount - C.amount| < 0.001 else 0.5  // within $0.01 epsilon: full points if within rounding noise
         date_closeness   = 1.0 - min(|E.date - C.date|, 3) / 3
         payee_similarity = strsim::normalized_levenshtein(
             normalize_merchant(E.payee.unwrap_or("")),
             normalize_merchant(C.payee.unwrap_or(""))
         )                                                                  // 0.0..=1.0
  4. Pick the highest-confidence match across all E.
  5. If confidence < 50 → suppress (D-09).
  6. Else attach DuplicateMatch { existing_id, confidence, bucket } to C.
```

### Confidence Buckets (verbatim from D-09)

| Confidence | Bucket           | UI Treatment        |
| ---------- | ---------------- | ------------------- |
| ≥ 95       | `ALMOST_CERTAIN` | `bg-destructive/10` |
| 70-94      | `LIKELY`         | `bg-warning/10`     |
| 50-69      | `POSSIBLE`       | `bg-muted/50`       |
| < 50       | `SUPPRESSED`     | (not surfaced)      |

### Performance: 1000-Row Import

Naive: 1000 candidates × 1 PG query per candidate = 1000 round trips. Bad.

**Optimization:** ONE query for the whole batch:

```sql
SELECT * FROM transactions
WHERE account_id = $1
  AND transaction_date BETWEEN $2 AND $3
  AND amount BETWEEN $4 AND $5
ORDER BY transaction_date;
```

Where `$2..$5` cover the union of all candidates' date and amount windows.
Result is paginated into an in-memory `Vec<Transaction>`. The detector then does
an in-memory cross-product against all candidates (1000 × ~K, where K = #
existing rows in window — typically <500 for a month-long import on a busy
account). With `idx_tx_account_date`, single-query latency is <50ms for 50k-row
accounts.

For pathological 10k-row imports (e.g., 5 years of Chase CSV), chunk the
candidates into batches of 500 and run one detector pass per batch. The detector
sees both DB-existing rows and within-batch candidates already processed (so a
CSV containing two literally-identical lines flags the second as a duplicate of
the first).

### Levenshtein vs. Token-Set

Picked **Levenshtein** (`strsim::normalized_levenshtein`) because:

- Bank statements rarely reorder merchant tokens (`"Whole Foods Market"` vs
  `"Market Whole Foods"` is essentially never observed).
- Levenshtein on the **already normalized** merchant string (D-13 pre-collapsed
  digit runs) handles the only realistic variation: bank inserts/strips suffix
  codes (`"WHOLEFDS GRP #10403"` vs `"WHOLEFDS GRP"` collapse to
  `"wholefds grp #"` and `"wholefds grp"` — 1-char delta = high similarity).
- O(m·n) per pair; m and n bounded ~50 chars → <1µs. 1000 candidates × 500
  in-memory rows × 1µs ≈ 500ms total — acceptable.

### Boundary Cases on the 3-Day Window

- "Same date" → distance = 0, score 1.0 ⇒ contributes 0.3 to confidence.
- "1 day apart" → distance = 1, score 1 - 1/3 ≈ 0.67 ⇒ contributes 0.20.
- "3 days apart" → distance = 3, score 0.0 ⇒ contributes 0.0.
- "4 days apart" → outside the gate, never reaches confidence calc.
- Amount within $0.005: full 0.4 points. Amount 0.005 < d ≤ 0.01: 0.2 points
  (rounded into the gate but not exact).

### Within-Batch Detection

If the user uploads a CSV that contains the same row twice (literally duplicated
bank export), the detector sees the second occurrence and flags it as a
duplicate of the first. Implementation: as we process the batch, append each
candidate's "if accepted" `Transaction` projection to the in-memory cross-check
set.

## Running Balance Strategy

**Decision: Option A — on-the-fly window function via VIEW.**

### Why Option A

- **Correctness is free**: PG owns ordering, no app-level reconciliation bug
  class. Out-of-order inserts (the pathological case) just re-rank automatically
  on next read.
- **Index cost paid once**: composite
  `(account_id, transaction_date DESC, created_at DESC)` already needed for
  ledger pagination. Window function rides it.
- **Bench expectation**: PG window function on 10k partition rows: <50ms read
  latency on commodity HW
  `[ASSUMED — should be benchmarked in plan; PG docs show window functions are designed for exactly this case [CITED: postgresql.org/docs/current/tutorial-window.html]]`.
- **Insert cost is O(1)**: just an INSERT, no recompute trigger, no snapshot
  table maintenance.

### Why Not Option B (Materialized Column)

- Out-of-order insert (CSV import of 5-year-old transactions) requires rewriting
  `running_balance` on every row chronologically AFTER the inserted row. Worst
  case = recompute the whole partition. Trigger logic is complex; race
  conditions on concurrent imports are real.
- The "win" is read-side perf, but Option A reads are already fast at v1 scale.

### Why Not Option C (Snapshot Table)

- Background-recompute job creates a freshness lag. UI users would see stale
  balances. UI-SPEC requires real-time balances on row insert.
- Adds a separate consistency invariant.

### When To Revisit

If a single account has >100k transactions AND ledger queries exceed 500ms:
introduce a materialized column with a recompute job for that account only. Not
Phase 4. Phase 6 reporting may consume aggregates differently — that phase will
reassess.

### Pathological Case Mitigation

A 10k-row historical import inserts mid-history. Steps:

1. Disable the duplicate-detection cap; accept all (or the user-resolved subset)
   into a single Diesel `transaction { ... }` block.
2. The VIEW recomputes naturally on next SELECT — there's nothing to maintain.
3. The composite index keeps the SELECT cheap.

For 100k-row imports (rare; probably never seen in v1): chunk inserts into
batches of 1000 inside one Diesel transaction. PG batch-insert performance is
acceptable.

## Categorization Memory

**Decision: dedicated `payee_category_memory` table, primary key
`(account_id, normalized_merchant)`.**

### Why Not JSON On Settings

Per-record JSON on `settings` would require deserializing the whole map on every
payee lookup. With even 1000 distinct payees, that's wasteful. A keyed table is
O(1) lookup.

### Why Not Taxonomy-Side Metadata

Taxonomy is shared with assets and other domains. Bolting per-merchant mappings
onto `taxonomy_categories` would overload the table's purpose and make Phase 8
re-skinning harder.

### Read Path

```rust
// During CSV/OFX import preview (D-15) or manual entry:
let normalized = normalize_merchant(&row.payee);
let suggested = repository.lookup_payee_memory(account_id, &normalized).await?;
// If suggested.is_some(), pre-fill category column in preview.
```

### Write Path

```rust
// During create_transaction OR update_transaction with category change (D-14):
// Last-write-wins semantics:
let now = Utc::now().naive_utc();
repository.upsert_payee_memory(PayeeCategoryMemory {
    account_id,
    normalized_merchant,
    category_id,
    last_seen_at: now,
    seen_count: existing.map(|e| e.seen_count + 1).unwrap_or(1),
}).await?;
```

`upsert_payee_memory` uses PG
`ON CONFLICT (account_id, normalized_merchant) DO UPDATE SET category_id = EXCLUDED.category_id, last_seen_at = EXCLUDED.last_seen_at, seen_count = seen_count + 1`.
The `seen_count` is non-user-visible audit-only; useful in Phase 8 for weighting
AI training data.

### Phase 8 Compatibility (look-ahead, not built)

When Phase 8 lands AI fallback for unknown merchants:

- The lookup path becomes: memory-table hit → use it; miss → call AI service →
  return suggestion.
- The `payee_category_memory` table is unchanged; AI suggestions write into it
  just like user choices (with `category_source = 'AI'` audit metadata,
  parallels `category_source` on `transactions`).
- Phase 8 might add an `is_ai_suggested` BOOLEAN column. **Recommendation: add
  it now as `category_source TEXT NULL`** with allowed values
  `'USER' | 'IMPORT' | 'MEMORY'` — Phase 8 just adds `'AI'` to the check
  constraint. Forward-compatible without a migration risk.

### `category_source` On `transactions`

Already added in §3 schema. Audit metadata: `'USER'` (manually picked),
`'MEMORY'` (auto-filled from payee_category_memory at import preview),
`'IMPORT'` (the CSV had a Category column). NOT user-visible.

## Multi-Currency Display

### How Activities Use FX Today

`crates/core/src/fx/fx_traits.rs:53-77` `[VERIFIED]`:

- `get_latest_exchange_rate(from, to) -> Decimal` — most-recent rate.
- `get_exchange_rate_for_date(from, to, date) -> Decimal` — historical snapshot;
  falls back to most-recent if no historical rate stored.
- `convert_currency_for_date(amount, from, to, date) -> Decimal` — full convert
  with rate snapshot.
- `register_currency_pair(from, to)` — ensures the pair exists in the FX service
  before queries.

Account creation already calls `register_currency_pair`
`[VERIFIED: crates/core/src/accounts/accounts_service.rs:57-60]`.

### What Phase 4 Adds

#### Write Side (insert / update)

```rust
// transactions_service.rs::create_transaction
if new.currency != account.currency {
    // Snapshot the rate at transaction_date for D-02 + TXN-07
    let rate = self.fx_service
        .get_exchange_rate_for_date(&new.currency, &account.currency, new.transaction_date)
        .await?;
    new.fx_rate = Some(rate);
    new.fx_rate_source = Some("SYSTEM");
} else {
    new.fx_rate = None;
    new.fx_rate_source = None;
}
```

For D-02 cross-currency transfers (paired rows, two currencies, two FX rates):

```rust
// Source leg snapshots its native currency → base; destination leg
// snapshots its native currency → base. The two rates are stored
// independently. System rate changes do NOT rewrite history.
let src_rate = fx.get_exchange_rate_for_date(src_currency, base, date).await?;
let dst_rate = fx.get_exchange_rate_for_date(dst_currency, base, date).await?;
```

#### Manual override

UI-SPEC §3 form section 9 ("More details" → FX rate override). Frontend posts
`fx_rate_source = "MANUAL_OVERRIDE"` + `fx_rate = <user value>`. Service trusts
the override (no system check). Audit-visible via `fx_rate_source` column.

#### Read Side (display)

```rust
// transactions_service.rs::list_transactions
// Returns rows as-stored; frontend shows native primary, base equivalent
// (~$32.11) below using transactions.fx_rate column. No conversion happens
// on display fetch — pre-snapshotted rate is the truth.
```

#### Per-Account vs Per-Currency Toggle

UI-SPEC §3 displays native currency primary and base-currency equivalent
sub-line when `native ≠ base`. There is no UI toggle in Phase 4 — display rule
is fixed by spec. **No new code path needed beyond writing `fx_rate` at insert
time.**

#### FX Stale-Warning Chip (UI-SPEC §Color)

`bg-warning/10` chip when `transactions.fx_rate` snapshot date is older than the
system's current rate by some threshold. Calculation:

- Stale-threshold: 30 days (configurable; not exposed in v1 UI).
- The chip is rendered server-side via a `is_fx_stale: bool` field on the search
  response — the frontend doesn't compute it. Compare
  `transaction.transaction_date` to the latest rate's `effective_date` for the
  same currency pair.

## Reconciliation Hook (Phase 3 D-14)

### Trigger

When the FIRST non-system-generated row is inserted against an account.
Detection: `repository.has_user_transactions(account_id) == false` immediately
before committing the new row.

### Where It Fires

**`TransactionService::create_transaction()` (NOT `accounts_service.rs`).** Two
reasons:

1. The hook needs to access `TransactionRepositoryTrait` to write reconciliation
   rows. `accounts_service.rs` does not own that trait.
2. The trigger condition is "first transaction insert against this account",
   which is a transactions-domain event.

### Algorithm

```rust
// transactions_service.rs::create_transaction
async fn create_transaction(&self, new: NewTransaction) -> Result<Transaction> {
    // ...validation...

    // Phase-3-D-14 reconciliation hook
    let account = self.account_repository.get_by_id(&new.account_id).await?;
    let needs_reconciliation = !self.repository
        .has_user_transactions(&new.account_id)
        .await?
        && account.opening_balance.is_some();           // Only Phase-3-era accounts have this set

    let mut to_insert: Vec<NewTransaction> = vec![];
    if needs_reconciliation {
        // Synthesize reconciliation rows BEFORE the user's row.
        // 1. Opening Balance row dated at account.created_at
        let opening = NewTransaction {
            account_id: account.id.clone(),
            direction: if account.opening_balance > Decimal::ZERO { "INCOME" } else { "EXPENSE" }.to_string(),
            amount: account.opening_balance.unwrap().abs(),
            transaction_date: account.created_at.date(),
            payee: None,
            currency: account.currency.clone(),
            category_id: Some("cat_uncategorized".to_string()),
            source: "SYSTEM".to_string(),
            is_system_generated: true,
            // ... other fields default
        };
        to_insert.push(opening);

        // 2. Optional Balance Adjustment if account.current_balance disagrees
        // with sum of (opening + sibling-imports between created_at and balance_updated_at).
        // For v1, this simplifies to: if account.current_balance != account.opening_balance,
        // synthesize a single adjustment row dated at balance_updated_at.
        if let (Some(current), Some(opening_amt), Some(updated)) =
            (account.current_balance, account.opening_balance, account.balance_updated_at)
        {
            let delta = current - opening_amt;
            if delta != Decimal::ZERO {
                let adj = NewTransaction {
                    account_id: account.id.clone(),
                    direction: if delta > Decimal::ZERO { "INCOME" } else { "EXPENSE" }.to_string(),
                    amount: delta.abs(),
                    transaction_date: updated.date(),
                    payee: None,
                    currency: account.currency.clone(),
                    category_id: Some("cat_uncategorized".to_string()),
                    source: "SYSTEM".to_string(),
                    is_system_generated: true,
                    notes: Some("Balance adjustment from manual update".to_string()),
                    // ...
                };
                to_insert.push(adj);
            }
        }
    }

    // Append the user's transaction
    to_insert.push(new);

    // Commit all rows in one Diesel transaction
    let result = self.repository.create_many_with_splits(to_insert).await?;

    // Emit events
    self.event_sink.emit(DomainEvent::transaction_inserted(...));

    Ok(result.last().unwrap().clone())
}
```

### What It Reads From `accounts`

- `accounts.opening_balance`: from D-11. NULL for legacy non-Phase-3 accounts →
  skip reconciliation entirely.
- `accounts.current_balance`: D-12. May be NULL (no manual balance update) →
  skip the adjustment row, just synthesize Opening.
- `accounts.balance_updated_at`: D-12. The date for the adjustment row.
- `accounts.created_at`: the date for the Opening Balance row.

### Test Cases

- New account with `opening_balance=$100`, `current_balance=NULL`, no manual
  updates → first user txn synthesizes 1 row (Opening $100). 2 rows total in DB
  after.
- Account with `opening=$100`, `current=$120`, `balance_updated_at=Apr 15` →
  first user txn synthesizes 2 rows (Opening dated `created_at`, +$100;
  Adjustment dated Apr 15, +$20). 3 rows total in DB after.
- Account with `opening=$100`, `current=$80` → Opening +$100, Adjustment -$20
  (direction=EXPENSE).
- Legacy SECURITIES account (`opening_balance=NULL`) → no reconciliation, just
  the user row. 1 row total.

## Frontend Wizard Fork Plan

Verbatim from `04-CONTEXT.md` `<canonical_refs>` shared list:

| Component                                                       | Action                                                                                                                            |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `FileDropzone` (`activity/import/components/file-dropzone.tsx`) | **Share verbatim** — re-export from new module                                                                                    |
| `CSVFileViewer` (`csv-file-viewer.tsx`)                         | **Share verbatim**                                                                                                                |
| `WizardStepIndicator` (`wizard-step-indicator.tsx`)             | **Share verbatim**                                                                                                                |
| `StepNavigation` (`step-navigation.tsx`)                        | **Share verbatim**                                                                                                                |
| `HelpTooltip` (`help-tooltip.tsx`)                              | **Share verbatim**                                                                                                                |
| `CancelConfirmationDialog` (`cancel-confirmation-dialog.tsx`)   | **Share with parameterized copy props** — copy text differs slightly                                                              |
| `MappingTable` (`mapping-table.tsx`)                            | **Fork** — transaction field set differs (no asset resolution; new `direction` inference column)                                  |
| `TemplatePicker` (`template-picker.tsx`)                        | **Fork** — templates are now transaction-mapping templates with `header_signature` for D-17                                       |
| `ImportProvider` context (`context/import-context.tsx`)         | **Fork** — transaction state shape is simpler (no asset resolution; adds duplicate-review state and reconciliation-warning state) |

### File List Under `apps/frontend/src/pages/transactions/import/`

```
transaction-import-page.tsx                          # forked from activity-import-page.tsx
context/
└── transaction-import-context.tsx                   # forked
components/
├── file-dropzone.tsx                                # re-exports activity/import/components/file-dropzone
├── csv-file-viewer.tsx                              # re-exports
├── wizard-step-indicator.tsx                        # re-exports
├── step-navigation.tsx                              # re-exports
├── help-tooltip.tsx                                 # re-exports
├── cancel-confirmation-dialog.tsx                   # re-exports (props parameterized)
├── transaction-mapping-table.tsx                    # forked
├── transaction-template-picker.tsx                  # forked
└── duplicate-review-list.tsx                        # NEW — preview-step companion
steps/
├── upload-step.tsx                                  # forked (CSV+OFX MIME, account picker)
├── transaction-mapping-step.tsx                     # forked from mapping-step-unified.tsx
├── transaction-review-step.tsx                      # forked — surfaces duplicates inline
└── transaction-confirm-step.tsx                     # forked
utils/
├── transaction-draft-utils.ts                       # forked
└── transaction-default-template.ts                  # forked
hooks/
└── use-transaction-import-mapping.ts                # forked from use-import-mapping.ts
```

### Why Fork Instead Of Extract Shared Package

- The activity-import code is 130k+ chars across 32 tsx files. Extracting a
  shared `import-wizard-shell` package would be a multi-day refactor that
  freezes the activity-import owner code path.
- Phase 4 surface differs in ~30% of the wizard logic (no asset resolution, adds
  duplicate review). Forking + selectively re-exporting primitives is the right
  complexity tradeoff for v1.
- Phase 8 (AI categorization) will revisit the wizard surface anyway.
- Project CLAUDE.md says "minimum code that solves the problem; nothing
  speculative."

### Routing Plan

- **Add to react-router config**: `/transactions` → `TransactionsPage`,
  `/transactions/import` → `TransactionImportPage`,
  `/transactions/import?accountId=X` → pre-scoped to account.
- Frontend code reads `?accountId=` query param at upload step (replicates
  `activity-import-page.tsx` pattern of `useSearchParams()`).

### Bottom-Nav Slot

UI-SPEC §Responsive says the bottom-nav adds a "Transactions" entry. Verify the
existing nav config in `apps/frontend/src/components/header.tsx` (or wherever
the nav lives — planner confirms). Slot order:
`Dashboard | Accounts | Transactions | Reports | Settings`. Phase 4 adds the
Transactions entry; Reports waits for Phase 6.

## Frontend Command Adapter Wiring

**KEY FACT:** `apps/tauri/src/lib.rs` does NOT register Tauri
`#[tauri::command]` handlers `[VERIFIED]`. The desktop app uses
`apps/frontend/src/adapters/tauri/core.ts` which `fetch`es the same Axum server
(`apps/server/src/api/`) over HTTP `[VERIFIED: adapters/tauri/core.ts:38-93]`.
Both desktop and web hit the same backend.

This means **the only command-registration changes needed are**:

1. New Axum routes in `apps/server/src/api/transactions.rs`.
2. New `COMMANDS` entries in `apps/frontend/src/adapters/web/core.ts` (also used
   by `tauri/core.ts`).
3. New typed wrapper module `apps/frontend/src/adapters/shared/transactions.ts`.

### `COMMANDS` Map Additions (`apps/frontend/src/adapters/web/core.ts`)

```typescript
// Transactions
search_transactions:           { method: "POST",   path: "/transactions/search" },
get_transaction:               { method: "GET",    path: "/transactions/item" },
create_transaction:            { method: "POST",   path: "/transactions" },
update_transaction:            { method: "PUT",    path: "/transactions" },
delete_transaction:            { method: "DELETE", path: "/transactions" },
list_running_balance:          { method: "POST",   path: "/transactions/running-balance" },
get_account_recent_transactions: { method: "GET",  path: "/transactions/by-account/recent" },

// Transaction import
import_transactions_csv:       { method: "POST",   path: "/transactions/import/csv" },
import_transactions_ofx:       { method: "POST",   path: "/transactions/import/ofx" },
preview_transaction_import:    { method: "POST",   path: "/transactions/import/preview" },
detect_transaction_duplicates: { method: "POST",   path: "/transactions/import/duplicates" },
list_transaction_templates:    { method: "GET",    path: "/transactions/import/templates" },
save_transaction_template:     { method: "POST",   path: "/transactions/import/templates" },
delete_transaction_template:   { method: "DELETE", path: "/transactions/import/templates" },
get_transaction_template:      { method: "GET",    path: "/transactions/import/templates/item" },

// Transfer pair
create_transfer:               { method: "POST",   path: "/transactions/transfer" },
update_transfer_leg:           { method: "PUT",    path: "/transactions/transfer/leg" },
break_transfer_pair:           { method: "POST",   path: "/transactions/transfer/break" },

// Payee category memory
lookup_payee_category:         { method: "POST",   path: "/transactions/payee-category-memory/lookup" },
list_payee_category_memory:    { method: "GET",    path: "/transactions/payee-category-memory" },
```

### `apps/frontend/src/adapters/shared/transactions.ts` (NEW)

Mirrors the shape of `activities.ts`:

```typescript
// Sketch — actual types come from a new lib/types/transaction.ts module
export const searchTransactions = async (
  page: number,
  pageSize: number,
  filters: TransactionFilters,
  searchKeyword: string,
  sort?: TransactionSort,
): Promise<TransactionSearchResponse> => {
  return await invoke<TransactionSearchResponse>("search_transactions", {
    page, pageSize, ...filters, searchKeyword, sort,
  });
};

export const createTransaction = async (
  transaction: TransactionCreate,
): Promise<Transaction> => invoke<Transaction>("create_transaction", { transaction });

export const updateTransaction = async (
  transaction: TransactionUpdate,
): Promise<Transaction> => invoke<Transaction>("update_transaction", { transaction });

export const deleteTransaction = async (
  transactionId: string,
): Promise<Transaction> => invoke<Transaction>("delete_transaction", { transactionId });

export const importTransactionsCsv = async (
  request: TransactionCsvImportRequest,
): Promise<TransactionImportResult> => invoke(...);

export const importTransactionsOfx = async (
  request: TransactionOfxImportRequest,
): Promise<TransactionImportResult> => invoke(...);

export const detectTransactionDuplicates = async (
  candidates: TransactionImport[],
): Promise<DuplicateCandidate[]> => invoke(...);

export const listRunningBalance = async (
  accountId: string,
  fromDate?: string,
  toDate?: string,
): Promise<TransactionWithRunningBalance[]> => invoke(...);

// + transfer ops, template ops, memory ops
```

### Contract-Level Types

```typescript
// New apps/frontend/src/lib/types/transaction.ts
export type TransactionDirection = "INCOME" | "EXPENSE" | "TRANSFER";
export type TransactionSource = "MANUAL" | "CSV" | "OFX" | "SYSTEM";
export type FxRateSource = "SYSTEM" | "MANUAL_OVERRIDE";

export interface Transaction {
  id: string;
  accountId: string;
  direction: TransactionDirection;
  amount: number; // z.number() per Phase 3 fix 7e9eb697
  currency: string;
  transactionDate: string; // YYYY-MM-DD
  payee: string | null;
  notes: string | null;
  categoryId: string | null;
  hasSplits: boolean;
  fxRate: number | null;
  fxRateSource: FxRateSource | null;
  transferGroupId: string | null;
  counterpartyAccountId: string | null;
  transferLegRole: "SOURCE" | "DESTINATION" | null;
  idempotencyKey: string | null;
  importRunId: string | null;
  source: TransactionSource;
  externalRef: string | null;
  isSystemGenerated: boolean;
  isUserModified: boolean;
  categorySource: "USER" | "MEMORY" | "IMPORT" | null;
  createdAt: string; // ISO 8601
  updatedAt: string;
  // Server-computed convenience
  isFxStale?: boolean; // §10
}

export interface TransactionSplit {
  id: string;
  transactionId: string;
  categoryId: string;
  amount: number;
  notes: string | null;
  sortOrder: number;
}

export interface TransactionWithRunningBalance extends Transaction {
  runningBalance: number;
}

export interface DuplicateCandidate {
  candidateRowIndex: number; // Position in import batch
  existingTransactionId: string;
  confidence: number; // 0..100 integer
  bucket: "ALMOST_CERTAIN" | "LIKELY" | "POSSIBLE";
}

export interface TransactionFilters {
  accountIds?: string[];
  categoryIds?: string[];
  directions?: TransactionDirection[];
  amountMin?: number;
  amountMax?: number;
  dateFrom?: string; // YYYY-MM-DD
  dateTo?: string;
  showTransfers?: boolean;
  source?: TransactionSource[];
}
```

## Validation Architecture

### Test Framework

| Property                            | Value                                                                                          |
| ----------------------------------- | ---------------------------------------------------------------------------------------------- |
| Rust unit / integration tests       | `cargo test --workspace` (existing)                                                            |
| Diesel migration tests              | `cargo test -p whaleit-storage-postgres` (existing pattern from `accounts/migration_tests.rs`) |
| Rust integration via testcontainers | `cargo test -p whaleit-storage-postgres --features integration` (existing pattern)             |
| Frontend unit                       | `pnpm test` (vitest) — existing                                                                |
| Frontend type check                 | `pnpm type-check`                                                                              |
| Frontend lint                       | `pnpm lint`                                                                                    |
| E2E                                 | Playwright — invoked through the project's `run-e2e-tests` skill                               |
| Quick run command                   | `cargo test -p whaleit-core transactions` (per-module)                                         |
| Full suite command                  | `cargo test --workspace && pnpm test && pnpm type-check && pnpm lint`                          |

### Phase Requirements → Test Map

| Req ID       | Behavior                                                                                           | Test Type   | Automated Command                                                                                                                        | File Exists?                                                   |
| ------------ | -------------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| TXN-01       | Manual transaction CRUD: create + read returns same fields                                         | unit        | `cargo test -p whaleit-core transactions::transactions_service_tests::create_returns_persisted`                                          | ❌ Wave 0                                                      |
| TXN-01       | Direction-locked sign convention preserved                                                         | unit        | `cargo test -p whaleit-core transactions::transactions_service_tests::direction_governs_sign`                                            | ❌ Wave 0                                                      |
| TXN-01       | Frontend form: invalid amount (<=0) blocks submit                                                  | unit        | `pnpm test transactions/transaction-form.test.tsx`                                                                                       | ❌ Wave 0                                                      |
| TXN-02       | `payee_category_memory` lookup returns prior choice                                                | unit        | `cargo test -p whaleit-core transactions::transactions_service_tests::memory_lookup_returns_last_category`                               | ❌ Wave 0                                                      |
| TXN-02       | `payee_category_memory` upsert updates last_seen_at                                                | unit        | `cargo test -p whaleit-core transactions::transactions_service_tests::memory_upsert_last_write_wins`                                     | ❌ Wave 0                                                      |
| TXN-02       | Merchant normalization examples from D-13                                                          | unit        | `cargo test -p whaleit-core transactions::merchant_normalizer_tests`                                                                     | ❌ Wave 0                                                      |
| TXN-03       | Search by date range returns only matching rows                                                    | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::search_by_date_range --features integration`                     | ❌ Wave 0                                                      |
| TXN-03       | Search by amount range applies BETWEEN inclusively                                                 | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::search_by_amount_range --features integration`                   | ❌ Wave 0                                                      |
| TXN-03       | Search by category filters via FK index                                                            | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::search_by_category --features integration`                       | ❌ Wave 0                                                      |
| TXN-03       | Search by payee uses trgm index (no full table scan)                                               | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::search_payee_uses_trgm --features integration` (inspect EXPLAIN) | ❌ Wave 0                                                      |
| TXN-04       | Import 100-row Chase CSV produces 100 transactions on chosen account                               | integration | `cargo test -p whaleit-core transactions::transactions_service_tests::csv_chase_100_rows` (uses test fixture)                            | ❌ Wave 0 (fixture file `tests/fixtures/chase_100.csv` needed) |
| TXN-04       | Re-importing same CSV produces 0 new transactions (idempotency)                                    | integration | `cargo test -p whaleit-core transactions::transactions_service_tests::csv_reimport_zero_new`                                             | ❌ Wave 0                                                      |
| TXN-04       | EU CSV format (`1.234,56`) parses to correct Decimal                                               | unit        | `cargo test -p whaleit-core transactions::compiler::eu_decimal_format`                                                                   | ❌ Wave 0                                                      |
| TXN-04       | Saved-template header validation surfaces mismatch error                                           | unit        | `pnpm test transactions/transaction-mapping-step.test.tsx::header_mismatch_warning`                                                      | ❌ Wave 0                                                      |
| TXN-05       | OFX 1.x SGML fixture (Chase BankMsgsRsv1) parses to expected rows                                  | unit        | `cargo test -p whaleit-core transactions::ofx_parser_tests::ofx_1x_chase` (fixture `tests/fixtures/chase.ofx`)                           | ❌ Wave 0                                                      |
| TXN-05       | OFX 2.x XML fixture parses to expected rows                                                        | unit        | `cargo test -p whaleit-core transactions::ofx_parser_tests::ofx_2x`                                                                      | ❌ Wave 0                                                      |
| TXN-05       | OFX without FITID falls back to hash idempotency key                                               | unit        | `cargo test -p whaleit-core transactions::ofx_parser_tests::no_fitid_fallback`                                                           | ❌ Wave 0                                                      |
| TXN-06       | Three-key gate: account match + amount match (±$0.01) + date match (±3d) all required              | unit        | `cargo test -p whaleit-core transactions::duplicate_detector_tests::three_key_gate`                                                      | ❌ Wave 0                                                      |
| TXN-06       | Confidence ≥95 → ALMOST_CERTAIN bucket                                                             | unit        | `cargo test -p whaleit-core transactions::duplicate_detector_tests::confidence_buckets`                                                  | ❌ Wave 0                                                      |
| TXN-06       | Confidence <50 suppressed (not surfaced)                                                           | unit        | `cargo test -p whaleit-core transactions::duplicate_detector_tests::below_50_suppressed`                                                 | ❌ Wave 0                                                      |
| TXN-06       | Within-batch duplicate (same row twice in CSV) flags second as duplicate of first                  | integration | `cargo test -p whaleit-core transactions::transactions_service_tests::within_batch_dupe`                                                 | ❌ Wave 0                                                      |
| TXN-07       | Cross-currency insert snapshots fx_rate at transaction_date                                        | integration | `cargo test -p whaleit-core transactions::transactions_service_tests::cross_currency_snapshots_fx`                                       | ❌ Wave 0                                                      |
| TXN-07       | Manual FX override stored with fx_rate_source = MANUAL_OVERRIDE                                    | unit        | `cargo test -p whaleit-core transactions::transactions_service_tests::manual_fx_override`                                                | ❌ Wave 0                                                      |
| TXN-07       | System FX rate change does NOT modify historical transaction.fx_rate                               | integration | `cargo test -p whaleit-core transactions::transactions_service_tests::historic_fx_immutable`                                             | ❌ Wave 0                                                      |
| TXN-08       | Splits sum equal to transaction.amount enforced                                                    | unit        | `cargo test -p whaleit-core transactions::transactions_service_tests::splits_sum_must_match`                                             | ❌ Wave 0                                                      |
| TXN-08       | Atomic insert: if split insert fails, parent rolls back                                            | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::splits_atomic_rollback --features integration`                   | ❌ Wave 0                                                      |
| TXN-08       | Update with new splits replaces all old splits in one transaction                                  | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::update_replaces_splits --features integration`                   | ❌ Wave 0                                                      |
| TXN-09       | Insert one row in middle of account ledger; running_balance for all later rows shifts              | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::running_balance_out_of_order --features integration`             | ❌ Wave 0                                                      |
| TXN-09       | Running balance for transfers: source leg subtracts, destination leg adds                          | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::running_balance_transfer_legs --features integration`            | ❌ Wave 0                                                      |
| TXN-09       | Running balance excludes archived account transactions correctly                                   | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::running_balance_archived_account --features integration`         | ❌ Wave 0                                                      |
| Phase-3 D-14 | First transaction insert against an account with `opening_balance` synthesizes Opening Balance row | integration | `cargo test -p whaleit-core transactions::reconciliation_tests::first_insert_synthesizes_opening`                                        | ❌ Wave 0                                                      |
| Phase-3 D-14 | First transaction insert when current_balance differs synthesizes adjustment row                   | integration | `cargo test -p whaleit-core transactions::reconciliation_tests::current_balance_delta_synthesizes_adjustment`                            | ❌ Wave 0                                                      |
| Phase-3 D-14 | Pre-Phase-3 account (opening_balance NULL) skips reconciliation entirely                           | integration | `cargo test -p whaleit-core transactions::reconciliation_tests::null_opening_skips`                                                      | ❌ Wave 0                                                      |
| D-04         | Edit paired-transfer leg amount with "apply both" syncs sibling                                    | integration | `cargo test -p whaleit-core transactions::transactions_service_tests::transfer_pair_edit_apply_both`                                     | ❌ Wave 0                                                      |
| D-04         | Edit paired-transfer leg amount with "this leg only" clears transfer_group_id                      | integration | `cargo test -p whaleit-core transactions::transactions_service_tests::transfer_pair_break_link`                                          | ❌ Wave 0                                                      |
| D-01         | Delete one leg of a transfer cascades to sibling                                                   | integration | `cargo test -p whaleit-storage-postgres transactions::repository_tests::transfer_delete_cascade --features integration`                  | ❌ Wave 0                                                      |
| Wizard E2E   | Upload CSV → map → review → confirm → row visible on /transactions                                 | E2E         | `pnpm playwright test transaction-import.spec.ts`                                                                                        | ❌ Wave 0                                                      |
| Wizard E2E   | Duplicate-review banner renders, "Discard new" removes the new row                                 | E2E         | `pnpm playwright test transaction-duplicate-review.spec.ts`                                                                              | ❌ Wave 0                                                      |

### Sampling Rate

- **Per task commit:** `cargo test -p whaleit-core transactions` (the per-module
  quick run, ~5s)
- **Per wave merge:**
  `cargo test --workspace && pnpm test && pnpm type-check && pnpm lint` +
  `cargo test -p whaleit-storage-postgres --features integration` (the postgres
  integration suite, ~30s with testcontainers)
- **Phase gate:** Full suite green + Playwright E2E green before
  `/gsd-verify-work`.

### Wave 0 Gaps

- [ ] `crates/core/src/transactions/transactions_service_tests.rs` — covers
      TXN-01..02, TXN-04..09 service-level tests
- [ ] `crates/core/src/transactions/duplicate_detector_tests.rs` — TXN-06
- [ ] `crates/core/src/transactions/ofx_parser_tests.rs` — TXN-05
- [ ] `crates/core/src/transactions/merchant_normalizer_tests.rs` — D-13
- [ ] `crates/core/src/transactions/reconciliation_tests.rs` — Phase-3 D-14
- [ ] `crates/core/src/transactions/compiler_tests.rs` (or inline in
      compiler.rs) — TXN-04 EU decimal, parens-wrap negative
- [ ] `crates/storage-postgres/src/transactions/repository_tests.rs` — TXN-03
      indexes, TXN-08 atomic, TXN-09 running balance, D-01 transfer cascade
- [ ] `crates/storage-postgres/src/transactions/migration_tests.rs` — up/down
      round-trip
- [ ] `crates/core/tests/fixtures/chase_100.csv` — synthetic CSV fixture
      (anonymized)
- [ ] `crates/core/tests/fixtures/chase.ofx` — anonymized OFX 1.x SGML
- [ ] `crates/core/tests/fixtures/sample_v2.ofx` — OFX 2.x XML
- [ ] `apps/frontend/src/pages/transactions/__tests__/transaction-form.test.tsx`
- [ ] `apps/frontend/src/pages/transactions/import/__tests__/transaction-mapping-step.test.tsx`
- [ ] `apps/frontend/tests/e2e/transaction-import.spec.ts`
- [ ] `apps/frontend/tests/e2e/transaction-duplicate-review.spec.ts`
- [ ] No new framework install — `cargo test`, `vitest`, and Playwright already
      present

## Threat Model Notes

> Required for the security gate. STRIDE categories applied to Phase 4 threat
> surface.

### Applicable ASVS Categories

| ASVS Category         | Applies   | Standard Control                                                                                                                                      |
| --------------------- | --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| V2 Authentication     | inherited | Existing JWT/auth — no new auth surface                                                                                                               |
| V3 Session Management | inherited | Same                                                                                                                                                  |
| V4 Access Control     | yes       | New `/transactions/*` routes MUST require authenticated user; account ownership check (user can only see own accounts' transactions)                  |
| V5 Input Validation   | yes       | All new endpoints validate input via `serde` deserialization + service-layer `validate()` impls; CSV/OFX file size cap (10 MB hard limit recommended) |
| V6 Cryptography       | yes       | SHA-256 idempotency hash; never roll custom — already using `sha2` workspace dep                                                                      |
| V8 Data Protection    | yes       | Money amounts handled as `Decimal` (arbitrary precision, no float drift); FX rate snapshots immutable post-write                                      |

### Known Threat Patterns For Phase 4 Stack

| Pattern                                                                                                           | STRIDE                                                           | Standard Mitigation                                                                                                                                                                                                                                                                                                                                                                       |
| ----------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **OFX/CSV file path traversal**                                                                                   | Tampering                                                        | The frontend uploads file BYTES via multipart, not paths. Backend never opens disk paths from request body. Already safe by architecture; Wave-0 plan must NOT introduce a `file_path` JSON field.                                                                                                                                                                                        |
| **Oversized file → DoS**                                                                                          | Denial of Service                                                | Cap multipart body at 10 MB at Axum router level (already standard). Also cap parsed row count per import to 50k (configurable) — reject larger uploads with friendly error.                                                                                                                                                                                                              |
| **Malformed OFX → parser crash / infinite loop**                                                                  | DoS                                                              | `sgmlish::from_str` returns `Result`; never `.unwrap()` in handler path. Wrap parse in `tokio::time::timeout(30s)` defense-in-depth.                                                                                                                                                                                                                                                      |
| **CSV with billion-laughs-style nested quote escape**                                                             | DoS                                                              | The `csv` crate is robust; cap row count + line length pre-parse if input bytes > 10 MB skip parse.                                                                                                                                                                                                                                                                                       |
| **`Decimal::from_str` with extreme magnitude / precision**                                                        | Tampering                                                        | `rust_decimal` clamps internally; service validates `amount > 0` and `amount < 10^12` (no person has $1T transactions; this is sane bound).                                                                                                                                                                                                                                               |
| **Currency manipulation (negative amount via API)**                                                               | Tampering                                                        | Service-level `validate()` rejects `amount <= 0`; SQL CHECK enforces same. Direction governs sign, not amount.                                                                                                                                                                                                                                                                            |
| **Transfer-pair forgery (attacker fabricates a `transfer_group_id` for a sibling row in another user's account)** | Tampering                                                        | Service-level enforcement: BOTH legs of a transfer MUST have `accounts.user_id == current_user.id`. Repository call goes through `account_repository.get_by_id` for each side — non-owned accounts return 404 (no leak of existence). The DB's `counterparty_account_id` FK + CHECK that `transfer_group_id IS NOT NULL FOR BOTH` enforces the integrity, but ownership is service-layer. |
| **Duplicate-detector bypass to mass-insert spam rows**                                                            | DoS                                                              | Already an authenticated, per-user surface. Rate-limit at Axum middleware level (project standard) — reject > 10 imports/min per user. Existing pattern.                                                                                                                                                                                                                                  |
| **Idempotency-key collision (extremely improbable with SHA-256, but)**                                            | Tampering                                                        | UNIQUE constraint on `transactions.idempotency_key`; insert collision returns DB error → service catches and returns `409 Conflict` to client.                                                                                                                                                                                                                                            |
| **Memory-table read leak (SELECT category for arbitrary `account_id`)**                                           | Information Disclosure                                           | Service enforces `account_id ∈ current_user.account_ids` before any memory query.                                                                                                                                                                                                                                                                                                         |
| **Splits sum bypass to embezzle**                                                                                 | Tampering                                                        | Service validates `SUM(splits.amount) == transaction.amount` before commit. NOT a DB constraint (perf), so the validation MUST live in the service-layer create/update path, with a property-test ensuring no edge case slips.                                                                                                                                                            |
| **OFX `<NAME>` field with embedded NULL byte / HTML / SQL**                                                       | Injection                                                        | Diesel parameterizes all queries; payee renders as plain text in React (default escaping). No SSR templates accept payee as raw HTML.                                                                                                                                                                                                                                                     |
| \*\*CSV "row" with `=cmd                                                                                          | "/c calc"!A1` formula injection (CSV injection on re-export)\*\* | Tampering                                                                                                                                                                                                                                                                                                                                                                                 | Phase 4 does not export CSVs (out of scope per UI-SPEC). When export lands in a future phase, sanitize: strip leading `=` `+` `-` `@` from cells. Note for Phase 6 reporting. |

### Crypto Note

The idempotency SHA-256 hash is NOT a security primitive — it's a deterministic
key. Don't use it for authn/authz. The actual auth token is the existing
JWT/API-key surface. **Do not** invent new crypto for Phase 4.

## Risks and Unknowns

### Risks the Planner MUST Address

1. **Atomic parent + splits insert.** `transactions_service.rs` MUST wrap parent
   `transactions` INSERT and child `transaction_splits` INSERTs inside one
   Diesel `transaction { ... }` block (use `diesel-async`'s transaction helper).
   If the splits sum doesn't match, fail BEFORE inserting the parent.

2. **Atomic transfer-pair insert.** Both legs of a transfer MUST be inserted in
   one DB transaction. If the second insert fails (e.g., destination account
   lookup race), roll back the first.

3. **Reconciliation hook race.** Two concurrent first-transaction inserts
   against the same Phase-3-era account could both detect "no user transactions
   yet" and both synthesize Opening Balance rows. Mitigation: use PG row-level
   lock `SELECT ... FROM accounts WHERE id = $1 FOR UPDATE` inside the
   reconciliation block, OR use an advisory lock keyed by `account_id`. Planner
   picks; document the choice.

4. **OFX FITID may be missing in some bank exports.** Fall back to hash
   idempotency. Ensure the OFX parser returns `external_ref: None` when FITID is
   absent and the idempotency key derives from canonical fields alone. Test
   fixture: `chase_no_fitid.ofx`.

5. **CSV import of 10k rows MUST not hold a single PG connection for >10
   seconds.** Chunk inserts into batches of 500 inside one Diesel transaction
   (or break into multiple smaller transactions if the wave needs more
   granularity). Alternatively: stream-insert via PG `COPY`, though this
   complicates idempotency-key uniqueness handling. **Planner decision
   required.**

6. **Frontend 10k-row preview rendering performance.** The Review step shows the
   user ALL rows for confirmation. Use virtualization (e.g., `react-window` if
   not already vendored, or hand-roll list-windowing).
   `apps/frontend/src/pages/activity/import/components/import-preview-table.tsx`
   already exists at 20.5K — read its approach first; reuse if it handles 10k+.

7. **Bottom-nav slot might already be at capacity.** Check
   `apps/frontend/src/components/header.tsx` (or wherever the nav is defined —
   UI-SPEC says "verify"). Phase 4 adds a Transactions entry; if the nav has 5
   fixed slots Reports may need to wait for Phase 6 anyway.

8. **`payee_category_memory.normalized_merchant` length limit.** Postgres TEXT
   has no limit, but unbounded strings invite weirdness. Recommendation:
   VARCHAR(500) with CHECK constraint, or just TEXT and trust normalization to
   keep under ~100 chars in practice.

9. **`fx_rate_source = NULL` ambiguity.** When transaction.currency ==
   account.currency, no FX is applied. Set `fx_rate = NULL` AND
   `fx_rate_source = NULL`. The CHECK constraint should enforce that both are
   NULL or both are non-NULL.

10. **System taxonomy seed migration must be idempotent.** Re-running migrations
    on a freshly-cloned dev DB must NOT duplicate seed rows. The
    `ON CONFLICT (id) DO NOTHING` handles this, but the migration file itself
    MUST use stable IDs (`cat_income`, `cat_dining`, etc.) rather than
    freshly-generated UUIDs.

### Open Questions (RESOLVED)

> All 7 open questions are resolved as of 2026-04-30. The planner has locked these recommendations into the execution plans (04-01..04-10).

1. **Connection-pool sizing for import path.**
   - What we know: deadpool default is small (~10). Concurrent imports + normal
     traffic could starve the pool.
   - What's unclear: whether Phase 4 should bump the pool size or rely on
     bulk-insert batching to reduce concurrent connection demand.
   - **RESOLVED:** keep the default; chunk imports into ≤500-row batches;
     revisit if real-world testing surfaces a wait.

2. **Whether to expose `category_source` audit metadata in the API response.**
   - What we know: D-11 says no Settings rules manager UI; Phase 8 will consume
     `category_source = 'MEMORY'` for AI training.
   - What's unclear: should the field be on the wire today (so Phase 8 can
     backfill against historical data), or wait until Phase 8?
   - **RESOLVED:** include on the wire NOW (it's free; fields are read
     anyway), do NOT surface in UI. Forward-compatible with Phase 8 with zero
     migration risk.

3. **Where to seed the system taxonomy: SQL migration or Rust service
   `initialize()`?**
   - What we know: existing `taxonomy_service.rs` has no analog seed at init
     time — the existing taxonomies were seeded via earlier migrations.
   - **RESOLVED** (also stated in §3): SQL migration. Atomic with schema;
     planner verifies that re-running migrations remains idempotent on a
     development DB with prior state.

4. **OFX 2.x XML detection threshold.**
   - What we know: file header `OFXHEADER:100` (1.x) vs `<?xml ?>` (2.x).
   - What's unclear: do any exports omit both? Some custom bank exports do.
   - **RESOLVED:** detect by sniffing first 100 bytes; if neither header
     recognized, attempt 1.x parse first, fall through to 2.x XML if 1.x fails.
     Surface a warning to the user but don't block.

5. **CSV Date detection: heuristic edge cases.**
   - What we know: `YYYY-MM-DD` is unambiguous. `MM/DD/YYYY` vs `DD/MM/YYYY`
     ambiguous when day ≤ 12.
   - What's unclear: what error to surface when both interpretations produce
     valid dates and the file has all values ≤ 12.
   - **RESOLVED:** when ambiguous, prompt the user via the Mapping step's
     date-format dropdown. Default to `MM/DD/YYYY` (US bias — most common in
     tracked banks).

6. **Reconciliation hook & idempotency.**
   - What we know: hook fires on first user transaction. Synthesized rows have
     `is_system_generated = true`.
   - What's unclear: should synthesized rows ALSO get an idempotency key, or is
     `idempotency_key = NULL` acceptable for SYSTEM source?
   - **RESOLVED:** allow NULL for SYSTEM source. The UNIQUE constraint on
     `idempotency_key` permits multiple NULLs in PG. Document this.

7. **`transfer_leg_role` synthesis.** The view-side running-balance formula
   needs the column. Is there a simpler model where direction alone implies the
   role?
   - **RESOLVED:** not really. A "TRANSFER" with positive amount could be either
     source or destination of the pair without an explicit marker. Stick with
     the explicit column.

## Environment Availability

| Dependency                | Required By                    | Available                                                   | Version           | Fallback                                                                                                                                                                           |
| ------------------------- | ------------------------------ | ----------------------------------------------------------- | ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo`                   | Rust build/test                | ✓                                                           | 1.93.1            | —                                                                                                                                                                                  |
| `psql` (libpq client)     | DB access during testing       | ✓                                                           | 18.3              | —                                                                                                                                                                                  |
| Postgres server (running) | Migrations + integration tests | ✗ at `/tmp:5432`                                            | —                 | Use Docker Compose `postgres` service from `compose.yaml` (verified: docker available via OrbStack). Plan must include "start PG container before running storage-postgres tests". |
| Docker                    | Compose-based local PG         | ✓                                                           | 28.5.2 (OrbStack) | —                                                                                                                                                                                  |
| `node`                    | Frontend tests + build         | ✓                                                           | v22.22.2          | —                                                                                                                                                                                  |
| `pnpm`                    | Frontend package mgmt          | ✓                                                           | 10.33.2           | —                                                                                                                                                                                  |
| Playwright browsers       | E2E                            | ASSUMED ✓ — installed by `run-e2e-tests` skill on first run | —                 | First-run install                                                                                                                                                                  |
| `sgmlish` crate           | OFX parsing                    | ✗ (must add)                                                | 0.2 (target)      | None — required for TXN-05                                                                                                                                                         |
| `strsim` crate            | Duplicate detection confidence | ✗ (must add)                                                | 0.11 (target)     | None — required for TXN-06                                                                                                                                                         |

**Missing dependencies with no fallback:**

- A running Postgres instance for integration tests. Plan must include starting
  `compose.yaml`'s `postgres` service before running
  `cargo test -p whaleit-storage-postgres --features integration`.

**Missing dependencies with fallback:**

- Both `sgmlish` and `strsim` are crate additions; install via `cargo add` in
  their respective `Cargo.toml`. No fallback needed beyond the crate-add step.

## Sources

### Primary (HIGH confidence)

- `./Cargo.toml` — workspace dep versions `[VERIFIED]`
- `./crates/core/Cargo.toml` — core dependencies `[VERIFIED]`
- `./crates/core/src/activities/csv_parser.rs` — CSV parsing reuse plan
  `[VERIFIED]`
- `./crates/core/src/activities/idempotency.rs` — idempotency hashing pattern
  `[VERIFIED]`
- `./crates/core/src/activities/compiler.rs` — staged compile pattern
  `[VERIFIED]`
- `./crates/core/src/accounts/accounts_service.rs` — service pattern
  `[VERIFIED]`
- `./crates/core/src/accounts/accounts_model.rs` — domain model pattern
  `[VERIFIED]`
- `./crates/core/src/accounts/accounts_traits.rs` — trait pattern `[VERIFIED]`
- `./crates/core/src/taxonomies/taxonomy_model.rs` + `taxonomy_service.rs` —
  taxonomy CRUD, `createCategory` mutation exists `[VERIFIED]`
- `./crates/core/src/fx/fx_traits.rs` — FX service surface `[VERIFIED]`
- `./crates/storage-postgres/src/accounts/model.rs` + `repository.rs` — Diesel
  boundary pattern `[VERIFIED]`
- `./crates/storage-postgres/src/schema.rs` — confirmed no `transactions` table
  exists `[VERIFIED]`
- `./crates/storage-postgres/migrations/20260425000000_accounts_extend_types_and_balances/up.sql`
  — Phase 3 columns Phase 4 reads `[VERIFIED]`
- `./apps/frontend/src/adapters/web/core.ts` +
  `./apps/frontend/src/adapters/tauri/core.ts` — adapter wiring (Tauri "IPC" is
  HTTP) `[VERIFIED]`
- `./apps/frontend/src/adapters/shared/activities.ts` — adapter shape pattern
  Phase 4 mirrors `[VERIFIED]`
- `./apps/server/src/api/activities.rs` — Axum route + handler pattern
  `[VERIFIED]`
- `./apps/frontend/src/pages/activity/import/activity-import-page.tsx` — wizard
  scaffold pattern `[VERIFIED]`
- `./apps/frontend/src/pages/activity/import/context/import-context.tsx` —
  ImportProvider state shape `[VERIFIED]`
- `./.planning/phases/04-transaction-core/04-CONTEXT.md` — locked decisions
  D-01..D-19 `[VERIFIED]`
- `./.planning/phases/04-transaction-core/04-UI-SPEC.md` — visual + interaction
  contract `[VERIFIED]`
- `./.planning/phases/03-bank-accounts-credit-cards/03-CONTEXT.md` — Phase 3
  carry-forward decisions `[VERIFIED]`
- `./.planning/REQUIREMENTS.md` — TXN-01..TXN-09 full text `[VERIFIED]`
- `./.planning/ROADMAP.md` — Phase 4 goal + success criteria `[VERIFIED]`

### Secondary (MEDIUM confidence)

- [sgmlish on lib.rs](https://lib.rs/crates/sgmlish) — OFX 1.x explicit support,
  MIT, last update 2021 `[CITED]`
- [strsim on GitHub](https://github.com/rapidfuzz/strsim-rs) — string similarity
  metrics including normalized_levenshtein `[CITED]`
- [PostgreSQL Window Functions docs](https://www.postgresql.org/docs/current/tutorial-window.html)
  — running balance pattern `[CITED]`
- [PostgreSQL Window Functions reference](https://www.postgresql.org/docs/current/functions-window.html)
  — frame specification `[CITED]`

### Tertiary (LOW confidence — flag for validation)

- `[ASSUMED]` `sgmlish` 0.2 is still the latest version as of 2026-04-30 —
  planner runs `cargo search sgmlish` to confirm
- `[ASSUMED]` `strsim` 0.11 is current — planner runs `cargo search strsim` to
  confirm
- `[ASSUMED]` PG window function on 10k partition <50ms — should be benchmarked
  in Wave 0 before committing to Option A; if exceeds, fall back to Option B
- `[ASSUMED]` `ofxy` is too immature for v1 — based on docs.rs description
  "early stages of development"; planner can re-evaluate by reading the crate's
  GitHub README during Wave 0 if curious

## Assumptions Log

| #   | Claim                                                                                                             | Section                         | Risk if Wrong                                                                                                                                                                                                                               |
| --- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A1  | `sgmlish` 0.2 is the latest stable version                                                                        | Standard Stack, OFX Parsing     | Use a newer version with breaking API changes; mitigated by `cargo search` before lock                                                                                                                                                      |
| A2  | `strsim` 0.11 is current                                                                                          | Standard Stack                  | Same — version drift; minor                                                                                                                                                                                                                 |
| A3  | PG window function over 10k-row account partition runs in <50ms on commodity HW                                   | Running Balance                 | If significantly slower at v1 scale, fallback is Option B (materialized column) — but Phase 4 ledgers will be small                                                                                                                         |
| A4  | `ofxy` crate is too immature for production use                                                                   | OFX Parsing alternatives        | If `ofxy` matured rapidly between training-cutoff and now, we're missing an option; mitigation: planner spends 5 min reading `ofxy` GitHub README before locking                                                                            |
| A5  | EU bank exports use `1.234,56` decimal format (comma-as-decimal)                                                  | CSV Parsing                     | If a specific bank uses some unusual format, the user has the manual mapping step; this is a soft assumption                                                                                                                                |
| A6  | Levenshtein on normalized merchant strings is sufficient for confidence multiplier (no token-set-ratio needed)    | Duplicate Detection             | If users complain about false negatives on banks that reorder merchant tokens, swap to token-set or combine; mitigation: detector formula is replaceable behind the trait                                                                   |
| A7  | PG `pg_trgm` extension is already enabled on the project's PG instance                                            | Schema                          | If not enabled, the migration must `CREATE EXTENSION IF NOT EXISTS pg_trgm` first; trivially fixable                                                                                                                                        |
| A8  | The Tauri shell does NOT register `tauri::command` handlers and the desktop adapter is HTTP-only                  | Frontend Command Adapter Wiring | If a real Tauri command surface exists somewhere in the Tauri shell I missed, plans must add command registration; mitigation: planner double-checks `apps/tauri/src/` for `#[tauri::command]` macros (confirmed empty as of this research) |
| A9  | The system taxonomy seed in SQL migration is the cleanest path                                                    | Schema                          | If the team prefers service-layer seeders for any reason, this can be flipped without architectural cost                                                                                                                                    |
| A10 | Bottom-nav config in the existing frontend can accommodate a new "Transactions" entry without breaking the layout | Frontend Wizard Fork Plan       | If nav is at capacity, Reports may need to wait for Phase 6 in any case; UI-SPEC §Responsive already says "planner verifies"                                                                                                                |

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all libraries verified in workspace; new deps verified
  via lib.rs and docs.rs
- Architecture: HIGH — direct mirror of proven `accounts/` and `activities/`
  patterns
- Schema design: HIGH for shape, MEDIUM for index strategy (depends on actual
  query workload — bench in Wave 0 if any single index seems off)
- Pitfalls: HIGH — drawn from existing code patterns (idempotency, transaction
  wrapping) and PG documentation
- OFX strategy: MEDIUM — `sgmlish` is the only mature option but the crate
  hasn't been updated since 2021; planner should sanity-check it still builds
  clean against current Rust
- Running balance: MEDIUM — Option A is the right v1 choice but the bench claim
  is unverified

**Research date:** 2026-04-30 **Valid until:** 2026-05-30 (30 days — schema and
architecture are stable; OFX library landscape can shift)
