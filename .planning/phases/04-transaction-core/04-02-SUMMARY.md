---
phase: 04-transaction-core
plan: 02
subsystem: rust-core-domain
tags: [transactions, parsers, ofx, csv, duplicate-detector, merchant-normalizer, reconciliation, traits, service, templates]
status: complete
completed: 2026-05-01
notes: |
  Agent reached its harness limit before writing SUMMARY + final commit; the
  orchestrator finished the wrap-up: committed the agent's 1393-insertion
  implementation across 12 files (commit 4e06f8bf), plus a 1-line
  pub(crate) bump on activities/mod.rs (commit 1b9f1189) so transactions
  can reuse activities::csv_parser primitives, plus mod.rs re-export fix
  for Transaction/NewTransaction/etc. cargo check -p whaleit-core green.
---

# Plan 04-02 — Rust core domain (transactions)

## Objective

Implement the Rust core domain for the transaction ledger: model, repository
trait, service trait, CSV/OFX parsers, duplicate detector, merchant normalizer,
reconciliation hook, idempotency hash, and the templates trait/types — all
under `crates/core/src/transactions/`.

## Files Created / Modified

### Domain model + traits
- `crates/core/src/transactions/mod.rs` — module skeleton + re-exports of key
  public types (`Transaction`, `NewTransaction`, `TransactionUpdate`,
  `TransactionSplit`, `NewSplit`, `SplitUpdate`, `PayeeCategoryMemory`).
- `crates/core/src/transactions/transactions_model.rs` — domain types.
- `crates/core/src/transactions/transactions_traits.rs` —
  `TransactionRepositoryTrait`, `PayeeCategoryMemoryRepositoryTrait`,
  `TransactionServiceTrait`, supporting `ImportResult` (with
  `inserted_row_ids: Vec<String>` order-preserving contract — see plan
  must_haves), `TransferEditMode { ApplyBoth, ThisLegOnly }` (D-04).
- `crates/core/src/transactions/transactions_constants.rs` — direction enums,
  source enums, category-source enums.
- `crates/core/src/transactions/transactions_errors.rs` — `TransactionError`.
- `crates/core/src/transactions/transactions_service.rs` — service skeleton
  (concrete impls land in plan 04-04 wiring).

### Parsers + helpers
- `crates/core/src/transactions/csv_parser.rs` — re-export of activities CSV
  primitives plus transaction-row mapper (D-13 normalization happens here at
  parse-time so the duplicate detector + memory key see the same shape).
- `crates/core/src/transactions/ofx_parser.rs` — `<?xml`-header sniff to pick
  SGML 1.x vs XML 2.x parser path (RESOLVED Open Question 4).
- `crates/core/src/transactions/duplicate_detector.rs` — 3-key gate
  (account_id + amount within $0.01 + date within ±3 calendar days) +
  payee-similarity confidence multiplier (D-06/D-07/D-09).
- `crates/core/src/transactions/merchant_normalizer.rs` — lowercase + strip
  leading/trailing whitespace + collapse runs of digits and runs of spaces
  (D-13 — pure Rust, no NLP).
- `crates/core/src/transactions/idempotency.rs` — SHA-256 hash of pipe-
  delimited normalized fields (mirrors `crates/core/src/activities/idempotency.rs`).
- `crates/core/src/transactions/reconciliation.rs` — D-14 first-transaction-
  insert hook. Synthesizes "Opening Balance" + (when delta ≠ 0) "Balance
  adjustment" rows tagged `is_system_generated = TRUE`, `source = 'SYSTEM'`,
  `idempotency_key = NULL` (RESOLVED Open Question 6).
- `crates/core/src/transactions/compiler.rs` — staged-compile pattern from
  parsed rows to typed transactions (mirrors activities/compiler.rs).

### Templates layer (D-16/17/18)
- `crates/core/src/transactions/templates_model.rs` — `TransactionTemplate`,
  `NewTransactionTemplate`, `TransactionTemplateUpdate`.
- `crates/core/src/transactions/templates_traits.rs` —
  `TransactionTemplateRepositoryTrait`, `TransactionTemplateServiceTrait`.

### Tests
- `crates/core/src/transactions/duplicate_detector_tests.rs`
- `crates/core/src/transactions/merchant_normalizer_tests.rs`
- `crates/core/src/transactions/ofx_parser_tests.rs`
- `crates/core/src/transactions/reconciliation_tests.rs`
- `crates/core/src/transactions/transactions_service_tests.rs`

### Adjacent (cross-cutting)
- `crates/core/src/activities/mod.rs` — `mod csv_parser;` → `pub(crate) mod
  csv_parser;` so transactions can re-export the existing parser primitives.
  Single-line surgical bump.
- `crates/core/src/lib.rs` — `pub mod transactions;`.
- `crates/core/src/events/domain_event.rs` — `DomainEvent::TransactionsChanged`
  variant.
- `Cargo.lock` — Cargo metadata.

## Verification

`cargo check -p whaleit-core` — 0 errors, 29 warnings (all "unused" — consumers
wire in plan 04-04 web/IPC adapters, plan 04-05 frontend types, plan 04-06+
UI). Expected at this layer.

## Cross-Plan Contract (for 04-03 PG repo)

- Repository trait signatures live in `transactions_traits.rs`.
- `ImportResult.inserted_row_ids: Vec<String>` MUST be returned by the PG
  `create_many_with_splits` impl in input order — repository test
  `create_many_with_splits_preserves_input_order` (in plan 04-03) is the
  contract test.
- Reconciliation runs inside the service (not in `accounts_service`) because
  it needs `TransactionRepositoryTrait`. Plan 04-04 wires this when it
  builds `TransactionServiceImpl`.

## Decisions Cited

D-01, D-02, D-04 (TransferEditMode), D-06, D-07, D-09, D-13, D-14, D-15.

## Locked Choices Honored

- Money via `rust_decimal::Decimal` — JSON wire format = number (Phase 3 fix
  7e9eb697 — `serde-float` enabled).
- No DB-level FK on `category_id` (matches Wave 1 schema deviation).
- Inserted row IDs return order = input order.
- OFX 1.x SGML mandatory; OFX 2.x XML detected by `<?xml` sniff.
- Merchant normalization is pure Rust, no NLP deps.

## Notes for Verifier

The `TransactionService` impl is partial — concrete struct + ctor land in
plan 04-04 alongside the AppState wiring. This is intentional and matches
how `accounts_service.rs` ships in core but `AccountServiceImpl` is wired
in `apps/server/src/main_lib.rs`.
