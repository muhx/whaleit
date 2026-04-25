---
phase: 03-bank-accounts-credit-cards
plan: 04
subsystem: server-dto + service
tags: [axum, dto, service, balance-update, openapi]

# Dependency graph
requires:
  - phase: 03-bank-accounts-credit-cards
    plan: 02
    provides:
      11 new fields on core Account/NewAccount/AccountUpdate;
      AccountUpdate.balance_updated_at slot
provides:
  - HTTP Account/NewAccount/AccountUpdate DTOs carry 11 new fields (camelCase
    wire format)
  - 3 From impls between server DTOs and core accounts copy 11 fields each
    direction
  - 18 #[schema(value_type = Option<String>)] annotations make Decimals string-shaped in OpenAPI
  - AccountService::update_account auto-stamps balance_updated_at when
    current_balance changes (D-12)
  - 3 service-level tests prove auto-bump behavior with mock repository (no #[ignore], no Box<dyn> skipping)
  - accounts_service_tests module wired under #[cfg(test)] mod registration
affects:
  [
    03-03 storage-postgres (peer wave; AccountDB needs same 11 fields),
    03-07 frontend update-balance modal (consumes new endpoint shape),
  ]

# Tech tracking
tech-stack:
  added: [] # rust_decimal, chrono::NaiveDate, async_trait, tokio::test, NoOpDomainEventSink already in workspace
  patterns:
    - "utoipa v4 #[schema(value_type = Option<String>)] for Decimal fields (no
      decimal feature wired)"
    - "Hand-rolled in-test mocks for AccountRepositoryTrait + unimplemented!()
      stubs for FxServiceTrait, AssetRepositoryTrait, SyncStateStore (no
      existing in-tree mocks per RESEARCH.md)"
    - "NoOpDomainEventSink reused from crate::events for sinks not under test"
    - "Server-side mutation overrides client-supplied balance_updated_at (let
      mut shadowed parameter pattern)"

key-files:
  created:
    - crates/core/src/accounts/accounts_service_tests.rs
    - .planning/phases/03-bank-accounts-credit-cards/03-04-SUMMARY.md
  modified:
    - apps/server/src/models.rs
    - crates/core/src/accounts/accounts_service.rs
    - crates/core/src/accounts/mod.rs
    - .planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md

key-decisions:
  - "Used pub re-exports (crate::assets::{Asset, AssetRepositoryTrait, ...},
    crate::fx::{ExchangeRate, ...}) instead of private inner module paths the
    planner sketch suggested"
  - "Reused NoOpDomainEventSink for the test event sink rather than writing a
    4th hand-rolled stub — DomainEventSink::emit is sync, the no-op impl already
    discards events"
  - "Schema annotation for Option<String> institution omitted (only required for
    Decimal types per plan note)"
  - "let mut account_update = account_update; shadow pattern keeps trait
    signature unchanged while allowing service-side mutation of
    balance_updated_at"

patterns-established:
  - "Service-level tests in crates/core can stub the full 6-dependency
    AccountService surface with hand-rolled mocks + unimplemented!() panicking
    stubs for unexercised paths"
  - "OpenAPI Decimal serialization: utoipa v4 + serde rust_decimal default →
    JSON string; #[schema(value_type = Option<String>)] aligns the schema to
    that wire format"

requirements-completed: [ACCT-01, ACCT-02, ACCT-04, ACCT-05, ACCT-06, ACCT-07]

# Metrics
duration: ~10 min
completed: 2026-04-25
---

# Phase 3 Plan 04: Server DTO + Service Auto-Bump Summary

**Surfaced the 11 new account fields through the Axum HTTP DTO layer (Account /
NewAccount / AccountUpdate + 3 From impls + 18 OpenAPI Decimal-as-string
annotations) and wired D-12 balance-auto-bump into
AccountService::update_account behind 3 service-level tests using hand-rolled
trait mocks.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-04-25T04:09:29Z
- **Completed:** 2026-04-25T04:18:35Z
- **Tasks:** 3 / 3
- **Files modified/created:** 5

## Accomplishments

- Extended `apps/server/src/models.rs` with the 11 new fields on `Account`,
  `NewAccount`, `AccountUpdate`. All Decimal-typed fields carry
  `#[schema(value_type = Option<String>)]` (18 annotations total — 6 Decimal
  fields × 3 structs) so utoipa v4 emits string-shaped OpenAPI types matching
  the actual `serde_json` wire format for `rust_decimal::Decimal`.
- Added 11 field copies to each of the 3 From impls (DTO→core for NewAccount /
  AccountUpdate, core→DTO for Account). `apps/server/src/api/accounts.rs`
  remains byte-identical (`git diff` returns empty) — handlers consume the new
  fields transparently through the same `Json<NewAccount>` /
  `Json<AccountUpdate>` extractors.
- Implemented D-12 auto-bump in `AccountService::update_account`: server-side
  `let mut account_update = account_update;` shadow + comparison against the
  fetched `existing.current_balance` writes
  `balance_updated_at = Some(Utc::now().naive_utc())` only when
  `current_balance.is_some()` AND the value differs. Trait signature is
  unchanged; no new methods on the trait or impl.
- Added `crates/core/src/accounts/accounts_service_tests.rs` (NEW) with 3
  `#[tokio::test]` cases proving the bump / no-bump-when-equal /
  no-bump-when-absent behavior. Mock `AccountRepositoryTrait` captures the
  `AccountUpdate` payload that hits the repository so the test can assert on the
  server-mutated value, not the client's input.
- Wired stubs for `FxServiceTrait`, `AssetRepositoryTrait`, `SyncStateStore`
  with `unimplemented!("...")` for every method (none of these dependencies are
  exercised by `update_account`'s auto-bump path; any accidental call panics
  loudly). Reused `NoOpDomainEventSink` from `crate::events::sink` for the event
  sink. No `#[ignore]`, no `Box<dyn>` skipping.
- Updated `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md`
  Per-Task Verification Map: D-12 row flipped from `TBD / W0 / pending` to
  `Plan 03-04 / Task 3 / ✅ green` with the corrected nested-tests::tests module
  path.

## Task Commits

1. **Task 1: Extend HTTP DTOs (Account / NewAccount / AccountUpdate) + From
   impls with 11 new fields + schema annotations** — `63c76770` (feat)
2. **Task 2: Auto-stamp balance_updated_at in AccountService::update_account
   when current_balance changes (D-12)** — `87b98fbf` (feat)
3. **Task 3: Add accounts_service_tests.rs with 3 tests covering D-12
   auto-bump + register module in mod.rs + flip VALIDATION.md row green** —
   `bfaa7fb5` (test)

## Files Created/Modified

- `apps/server/src/models.rs` — Added `use chrono::NaiveDate;` and
  `use rust_decimal::Decimal;`. Appended 11 fields to `Account`, `NewAccount`,
  `AccountUpdate`. 18 `#[schema(value_type = Option<String>)]` attributes on
  Decimal fields. 33 field copies (11 × 3) across the 3 From impls.
- `crates/core/src/accounts/accounts_service.rs` — Added a 10-line block to
  `update_account` (D-12 auto-bump). No other changes; the surrounding
  currency-change detection, FX pair registration, and event emission are
  untouched.
- `crates/core/src/accounts/accounts_service_tests.rs` — NEW — 386-line file
  with mock `MockAccountRepo`, stubs `StubFx`, `StubAssets`, `StubSyncState`,
  and 3 `#[tokio::test]` cases.
- `crates/core/src/accounts/mod.rs` — Added
  `#[cfg(test)] mod accounts_service_tests;` after the existing
  `accounts_model_tests` registration.
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` — Updated
  D-12 row.

## Decisions Made

- **Public re-export paths over private module paths:** the planner sketch
  pointed to `crate::assets::assets_model::*` and `crate::fx::fx_model::*` for
  the imports, but those inner modules are `mod` (private). Switched to the
  parent re-exports (`crate::assets::{Asset, AssetRepositoryTrait, ...}`,
  `crate::fx::{ExchangeRate, FxServiceTrait, ...}`). One-line fix during Task 3
  RED→GREEN cycle.
- **NoOpDomainEventSink reuse:** `DomainEventSink::emit` is sync (not async).
  Rather than hand-roll a 4th stub trait impl, used the existing
  `crate::events::NoOpDomainEventSink` which discards events. update_account
  emits 1-2 events per call (accounts_changed always; tracking_mode_changed
  conditionally) — no test cares about the event payload.
- **`institution` field schema annotation omitted:** plan said the rule applies
  to Decimal-typed fields only; `Option<String>` doesn't need it. Kept the field
  unannotated to match the plan's "if executor finds the annotation unnecessary
  for `Option<String>` it can be omitted" guidance.
- **`let mut account_update = account_update;` shadow pattern:** keeps the trait
  signature `async fn update_account(&self, account_update: AccountUpdate)`
  exactly as defined in `AccountServiceTrait`. The mutation is invisible to the
  trait surface — only the call to `repository.update(account_update)` sees the
  modified value.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Planner-sketched module paths used private inner
modules**

- **Found during:** Task 3, first
  `cargo test -p whaleit-core accounts::accounts_service_tests` run after
  writing the test file.
- **Issue:** The plan example imported via `crate::assets::assets_model::*` and
  `crate::fx::fx_model::*`, but both inner modules are declared `mod` (private).
  `error[E0603]: module 'assets_model' is private`.
- **Fix:** Replaced both with
  `crate::assets::{Asset, AssetRepositoryTrait, AssetSpec, EnsureAssetsResult, NewAsset, UpdateAssetProfile}`
  and `crate::fx::{ExchangeRate, FxServiceTrait, NewExchangeRate}` (the publicly
  re-exported paths).
- **Files modified:** `crates/core/src/accounts/accounts_service_tests.rs`.
- **Verification:** `cargo test -p whaleit-core accounts::` → 16 passed.
- **Committed in:** `bfaa7fb5` (Task 3, single commit).

---

**Total deviations:** 1 auto-fix (Rule 3 — Blocking). **Impact on plan:** None —
the planner sketch was a hint, not a contract; the behavior, test names, and
acceptance criteria are all met.

## Issues Encountered

- **Storage-postgres compile failure (out-of-scope, addressed by parallel plan
  03-03):** `cargo check -p whaleit-server` does not pass in this worktree
  because `crates/storage-postgres/src/accounts/model.rs::AccountDB` lacks the
  11 new fields the core `Account` requires (since 03-02). This is the identical
  situation 03-02-SUMMARY flagged as "by design and matches the plan acceptance
  scope: 'compile may surface failures in the storage-postgres or server crates
  — those are addressed in later plans'". Plans 03-03 (storage) and 03-04
  (DTO/service) are both in Wave 2; the orchestrator merges both worktrees
  together at wave end, at which point the compile will be green. The plan's
  `<verify><automated>cargo check -p whaleit-server</automated>` command was
  authored assuming serial 03-03→03-04 ordering; in parallel Wave 2 it is not
  achievable per-worktree. **Verified the changes in this plan are individually
  correct via:**
  - `cargo check -p whaleit-core --all-targets` → green.
  - `cargo test -p whaleit-core accounts::` → 16 passed (13 pre-existing + 3 new
    D-12 tests).
  - `git diff apps/server/src/api/accounts.rs` → empty (untouched per plan
    requirement).

## TDD Gate Compliance

This plan's frontmatter is `type: execute`, with Tasks 2 and 3 carrying
`tdd="true"`. Per-task TDD discipline followed:

- **Task 2 (auto-bump implementation):** Wrote the implementation first; the
  proving tests live in Task 3 per the plan's structure.
  `cargo check -p whaleit-core` was green between Task 2 and Task 3 commits.
- **Task 3 (tests):** Wrote 3 `#[tokio::test]` cases that exercise the Task 2
  implementation. All pass on first run after the import-path fix; no iteration
  on the test logic itself.

The plan deliberately separates implementation (Task 2) from test (Task 3) so
the trait-mock scaffolding can land as a single coherent commit, rather than
splitting RED/GREEN per-test.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- **03-03 storage-postgres (parallel Wave 2):** Once it lands the 11 new
  AccountDB fields + From impls + migration, `cargo check -p whaleit-server`
  will compile cleanly with this plan's DTO additions.
- **03-07 / 03-07b frontend update-balance modal (Wave 3):** The HTTP wire shape
  now carries `currentBalance: string | null` (camelCase Decimal-as- string) and
  the server auto-stamps `balanceUpdatedAt`. The "Update balance" modal can send
  a partial AccountUpdate with just `id` + `currentBalance` and rely on the
  server to time-stamp.
- **OpenAPI / Swagger consumers:** All Decimal fields now declare
  `Option<String>` in the generated schema, matching the actual JSON wire
  format. Swagger UI will render input boxes typed as strings (no incorrect
  "number" coercion) and downstream OpenAPI codegen will produce
  `Option<String>`-typed fields.

## Self-Check: PASSED

Created files exist:

- `crates/core/src/accounts/accounts_service_tests.rs` → FOUND
- `.planning/phases/03-bank-accounts-credit-cards/03-04-SUMMARY.md` → FOUND
  (this file)

Modified files exist:

- `apps/server/src/models.rs` → FOUND
- `crates/core/src/accounts/accounts_service.rs` → FOUND
- `crates/core/src/accounts/mod.rs` → FOUND
- `.planning/phases/03-bank-accounts-credit-cards/03-VALIDATION.md` → FOUND

Commits exist (`git log --oneline -5`):

- `63c76770` (Task 1) → FOUND
- `87b98fbf` (Task 2) → FOUND
- `bfaa7fb5` (Task 3) → FOUND

Acceptance commands:

- `cargo check -p whaleit-core --all-targets` → `Finished dev profile` (PASS)
- `cargo test -p whaleit-core accounts::` → `16 passed, 796 filtered out` (PASS
  — 13 pre-existing + 3 new D-12 tests)
- `cargo check -p whaleit-server` → BLOCKED on parallel-wave 03-03 (see Issues
  Encountered; storage-postgres AccountDB needs 11 new fields)
- `git diff apps/server/src/api/accounts.rs` → empty (PASS — handler file
  byte-identical)
- `grep -c "pub credit_limit: Option<Decimal>" apps/server/src/models.rs` → 3
  (PASS — Account, NewAccount, AccountUpdate)
- `grep -c "institution: a.institution," apps/server/src/models.rs` → 3 (PASS —
  3 From impls)
- `grep -F 'schema(value_type' apps/server/src/models.rs | wc -l` → 18 (PASS — 6
  Decimal fields × 3 structs)
- `grep -c "#\[ignore\]" crates/core/src/accounts/accounts_service_tests.rs` → 0
  (PASS — no test skipping)

---

_Phase: 03-bank-accounts-credit-cards_ _Completed: 2026-04-25_
