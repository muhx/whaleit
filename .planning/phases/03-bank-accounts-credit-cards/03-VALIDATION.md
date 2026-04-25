---
phase: 3
slug: bank-accounts-credit-cards
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-25
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Source:
> 03-RESEARCH.md → ## Validation Architecture. Note on routes: Per CONTEXT.md
> D-15 amendment, the unified list lives under `/settings/accounts` (not
> `/accounts`). Test paths for "accounts page" target
> `apps/frontend/src/pages/settings/accounts/`. Adjust file refs in the Per-Task
> Verification Map when executors finalize file paths in plans.

---

## Test Infrastructure

| Property                      | Value                                                                       |
| ----------------------------- | --------------------------------------------------------------------------- |
| **Framework (Rust)**          | `cargo test` (default harness + `tokio::test` for async)                    |
| **Framework (Frontend unit)** | Vitest `3.2.4` + React Testing Library `16.3.2` + jest-dom `6.9.1`          |
| **Framework (E2E)**           | Playwright `^1.58.2` via `node scripts/run-e2e.mjs`                         |
| **Config (frontend)**         | `apps/frontend/vite.config.ts` (vitest reads from vite config)              |
| **Config (E2E)**              | `playwright.config.ts` (repo root)                                          |
| **Quick run (Rust)**          | `cargo test -p whaleit-core accounts::`                                     |
| **Quick run (Frontend)**      | `pnpm --filter frontend test -- --run apps/frontend/src/lib/`               |
| **PG integration**            | `cargo test -p whaleit-storage-postgres accounts` (requires `DATABASE_URL`) |
| **Full suite**                | `cargo test --workspace && pnpm test`                                       |
| **E2E run**                   | `pnpm test:e2e` (use `run-e2e-tests` skill before invoking)                 |
| **Estimated runtime**         | ~30s quick / ~3-5 min full / ~5-8 min E2E                                   |

---

## Sampling Rate

- **After every task commit:** `cargo test -p whaleit-core accounts::` (when the
  task touches Rust core) OR
  `pnpm --filter frontend test -- --run <changed-file>` (when the task touches
  frontend). Combined ≤ 30s.
- **After every plan wave:** `cargo test --workspace` + `pnpm test` + one scoped
  Playwright spec.
- **Before `/gsd-verify-work`:** Full suite green.
  `cargo test --workspace && pnpm test && pnpm test:e2e`.
- **Max feedback latency:** 30 seconds for per-task commits.

---

## Per-Task Verification Map

> Filled by gsd-planner during PLAN.md generation. Each task in each PLAN.md
> must register here with a Test Type, automated command, and pass/fail status.
> Wave 0 stubs that don't yet exist are marked `❌ W0`. Existing tests are
> marked `✅`.

| Task ID | Plan  | Wave | Requirement    | Threat Ref | Secure Behavior                                                           | Test Type          | Automated Command                                                                                                         | File Exists                      | Status     |
| ------- | ----- | ---- | -------------- | ---------- | ------------------------------------------------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------- | -------------------------------- | ---------- |
| 3       | 03-01 | 1    | (migration)    | T-3-01     | Schema migration applies cleanly; no orphan indices                       | Rust integration   | `cargo test -p whaleit-storage-postgres accounts::migration_tests::test_migration_up_down`                                | ❌ W0                            | ✅         |
| 3       | 03-02 | 1    | ACCT-01        | —          | Bank-account creation persists all fields                                 | Rust unit          | `cargo test -p whaleit-core accounts::accounts_model_tests::tests::test_new_account_validate_bank`                        | ✅                               | ✅ green   |
| TBD     | TBD   | TBD  | ACCT-01        | —          | Repository round-trip for CHECKING/SAVINGS                                | Rust integration   | `cargo test -p whaleit-storage-postgres accounts::repository_tests`                                                       | ❌ W0                            | ⬜ pending |
| 3       | 03-02 | 1    | ACCT-02        | T-3-02     | CC create rejects missing `credit_limit` / invalid `statement_cycle_day`  | Rust unit          | `cargo test -p whaleit-core accounts::accounts_model_tests::tests::test_new_account_validate_credit_card_rejects_invalid` | ✅                               | ✅ green   |
| 3       | 03-02 | 1    | ACCT-02        | —          | CC create accepts valid credit_limit + cycle_day                          | Rust unit          | `cargo test -p whaleit-core accounts::accounts_model_tests::tests::test_new_account_validate_credit_card`                 | ✅                               | ✅ green   |
| 3       | 03-02 | 1    | ACCT-02        | T-3-02     | Non-CC create rejects CC fields present                                   | Rust unit          | `cargo test -p whaleit-core accounts::accounts_model_tests::tests::test_new_account_validate_non_cc_rejects_cc_fields`    | ✅                               | ✅ green   |
| TBD     | TBD   | TBD  | ACCT-03        | —          | `/settings/accounts` renders all account types with current_balance       | Frontend component | `pnpm --filter frontend test -- --run apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx`                   | ❌ W0 (create test, page exists) | ⬜ pending |
| TBD     | TBD   | TBD  | ACCT-04        | —          | Archive toggles `is_archived`; archived hidden by default in selectors    | Frontend component | `pnpm --filter frontend test -- --run apps/frontend/src/pages/dashboard/accounts-summary.test.tsx`                        | ✅ partial (extend)              | ⬜ pending |
| TBD     | TBD   | TBD  | ACCT-04        | —          | Edit CC + bank preserves unrelated fields                                 | Rust integration   | `cargo test -p whaleit-storage-postgres accounts::repository_tests::test_update_preserves_fields`                         | ❌ W0                            | ⬜ pending |
| 3       | 03-02 | 1    | ACCT-05 / D-03 | —          | `account_kind()` maps types correctly (Rust)                              | Rust unit          | `cargo test -p whaleit-core accounts::accounts_model_tests::tests::test_account_kind`                                     | ✅                               | ✅ green   |
| TBD     | TBD   | TBD  | ACCT-05 / D-03 | —          | `accountKind()` maps types correctly (TypeScript)                         | Frontend unit      | `pnpm --filter frontend test -- --run apps/frontend/src/lib/constants.test.ts`                                            | ❌ W0                            | ⬜ pending |
| TBD     | TBD   | TBD  | ACCT-05 / D-08 | —          | Available credit derived (`credit_limit - current_balance`)               | Frontend unit      | `pnpm --filter frontend test -- --run apps/frontend/src/pages/settings/accounts/credit-helpers.test.ts`                   | ❌ W0                            | ⬜ pending |
| TBD     | TBD   | TBD  | ACCT-06        | —          | Statement fields optional on CC; NULL on non-CC; round-trip               | Rust integration   | `cargo test -p whaleit-storage-postgres accounts::repository_tests::test_cc_statement_roundtrip`                          | ❌ W0                            | ⬜ pending |
| TBD     | TBD   | TBD  | ACCT-07        | —          | Reward points / cashback round-trip                                       | Rust integration   | `cargo test -p whaleit-storage-postgres accounts::repository_tests::test_cc_rewards_roundtrip`                            | ❌ W0                            | ⬜ pending |
| 3       | 03-04 | 2    | D-12           | —          | Updating `current_balance` bumps `balance_updated_at`                     | Rust unit + integ. | `cargo test -p whaleit-core accounts::accounts_service_tests::tests::test_update_bumps_balance_timestamp`                 | ✅                               | ✅ green   |
| TBD     | TBD   | TBD  | D-19 / ACCT-04 | —          | `/settings/accounts` archive filter reveals archived; selectors hide them | E2E                | Extend `e2e/` with `e2e/11-accounts.spec.ts` (or extend `01-happy-path.spec.ts`)                                          | ❌ W0                            | ⬜ pending |
| TBD     | TBD   | TBD  | ACCT-01..07    | T-3-03     | Full user flow: create bank → CC → archive → update balance               | E2E                | `pnpm test:e2e -- e2e/11-accounts.spec.ts`                                                                                | ❌ W0                            | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave 0 Requirements

Wave 0 of Phase 3 must land these test scaffolds BEFORE other waves run, so
later tasks have a target to flip green. No new framework installs required —
Vitest, RTL, jest-dom, Playwright are all wired.

- [ ] `crates/core/src/accounts/accounts_model_tests.rs` — extend with
      `test_new_account_validate_bank`, `test_new_account_validate_credit_card`,
      `test_new_account_validate_credit_card_rejects_invalid`,
      `test_new_account_validate_non_cc_rejects_cc_fields`
- [ ] `crates/core/src/accounts/accounts_service_tests.rs` — NEW — covers
      balance-timestamp auto-bump (D-12) and CC-field nullability service rules
      (D-06)
- [ ] `crates/core/src/accounts/tests.rs` (or in-module) — `test_account_kind`
      asserting D-03 mapping for every AccountType
- [ ] `crates/storage-postgres/src/accounts/repository_tests.rs` — NEW — PG
      round-trip per AccountType, `test_update_preserves_fields`,
      `test_cc_statement_roundtrip`, `test_cc_rewards_roundtrip`. Requires
      `DATABASE_URL` (use existing test fixture from Phase 2)
- [ ] `crates/storage-postgres/src/accounts/migration_tests.rs` — NEW —
      `test_migration_up_down` for the new Phase 3 migration
- [ ] `apps/frontend/src/lib/constants.test.ts` — NEW — `accountKind()` mapping
      and `defaultGroupForAccountType` extensions
- [ ] `apps/frontend/src/lib/schemas.test.ts` — extend with CC-gated zod cases
      (CC requires credit_limit + cycle_day; bank rejects CC fields)
- [ ] `apps/frontend/src/pages/settings/accounts/accounts-page.test.tsx` — NEW —
      unified list rendering, group-by behavior, archive toggle
- [ ] `apps/frontend/src/pages/settings/accounts/credit-helpers.test.ts` — NEW —
      available-credit + utilization derivation
- [ ] `e2e/11-accounts.spec.ts` — NEW — full user flow (or extend
      `e2e/01-happy-path.spec.ts`)

Framework install: **none** — all infra already present.

---

## Manual-Only Verifications

| Behavior                                                                   | Requirement | Why Manual                                                                        | Test Instructions                                                                                                                                                    |
| -------------------------------------------------------------------------- | ----------- | --------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Visual fidelity of "Available credit" chip and CC sections vs UI-SPEC      | ACCT-05     | Pixel-level visual checks not in scope of unit/E2E here                           | Open `/settings/accounts` after creating a CC → verify chip placement, color, hover state match `03-UI-SPEC.md` §1 + §6                                              |
| Group-by control reorders rows correctly across all six AccountType groups | ACCT-03     | Existing UI test infra does not enforce group-ordering invariants beyond presence | Manually create one account of each type → toggle group-by control → verify Banking → Credit Cards → Loans → Investments → Cash → Crypto order                       |
| Multi-currency FX equivalent renders accurately for each new account type  | ACCT-03     | FX provider rates fluctuate; not deterministic                                    | Create a USD CHECKING + EUR CREDIT_CARD with base = USD → verify base-currency equivalent column shows non-zero, correctly-converted values; cross-check rate source |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
