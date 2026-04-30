---
phase: 04
slug: transaction-core
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-30
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Detailed
> acceptance tests live in `04-RESEARCH.md` §Validation Architecture. The
> planner fills the per-task verification map below as plans are written.

---

## Test Infrastructure

| Property               | Value                                                                                                                             |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| **Frameworks**         | Rust: `cargo test` (workspace) · Frontend: vitest 1.x (apps/frontend, packages/ui) · E2E: Playwright (per `run-e2e-tests` skill)  |
| **Config files**       | `Cargo.toml` workspace · `apps/frontend/vitest.config.ts` · `packages/ui/vitest.config.ts` · `apps/frontend/playwright.config.ts` |
| **Quick run command**  | `cargo test -p whaleit-core transactions::` (per-module) · `pnpm --filter @whaleit/frontend test -- --run <pattern>`              |
| **Full suite command** | `cargo test --workspace && pnpm test`                                                                                             |
| **Estimated runtime**  | Quick: ~5–15s · Full: ~3–6 min cold (cargo build)                                                                                 |

---

## Sampling Rate

- **After every task commit:** Run quick command scoped to the touched
  module/package.
- **After every plan wave:** Run full suite for the affected layer
  (`cargo test --workspace` for Rust waves; `pnpm test` for frontend waves).
- **Before `/gsd-verify-work`:** Full suite must be green AND Playwright E2E for
  transaction flows must pass.
- **Max feedback latency:** 30 seconds for unit tests, 60 seconds for full Rust
  suite.

---

## Per-Task Verification Map

| Task ID             | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status     |
| ------------------- | ---- | ---- | ----------- | ---------- | --------------- | --------- | ----------------- | ----------- | ---------- |
| _filled by planner_ | —    | —    | —           | —          | —               | —         | —                 | —           | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

> Planner: each PLAN.md task with `<automated>` verify must add a row here, or
> declare a Wave 0 stub fixture. See §Validation Architecture in
> `04-RESEARCH.md` for the required test list per TXN-01..TXN-09.

---

## Wave 0 Requirements

Test infrastructure additions Phase 4 needs before substantive tasks can verify
themselves:

- [ ] `crates/core/src/transactions/tests/fixtures.rs` — shared in-memory
      account + taxonomy fixtures for unit tests.
- [ ] `crates/storage-postgres/tests/transactions_repo.rs` — PG-backed
      repository tests using a test container (mirrors existing `accounts/` test
      pattern; reuse the workspace's testcontainer harness if present).
- [ ] `crates/core/tests/csv/transactions/` — golden CSV samples (Chase, Wells
      Fargo, Amex layouts; minimum 10 rows each, sourced from anonymized
      synthetic data).
- [ ] `crates/core/tests/ofx/transactions/` — golden OFX 1.x SGML and OFX 2.x
      XML samples.
- [ ] `apps/frontend/tests/import-wizard.spec.ts` — Playwright fixtures for the
      new `/transactions/import` route (refer to existing
      `activity-import.spec.ts` if present).
- [ ] Add `sgmlish = "0.2"` to `crates/core/Cargo.toml` (per RESEARCH §OFX
      Parsing Strategy).
- [ ] Add `strsim = "0.11"` (or chosen alternative) for Levenshtein/token-set
      similarity in duplicate detector.

If any of these tests/dependencies already exist for the activities domain and
can be parameterized rather than copied, prefer parameterization.

---

## Manual-Only Verifications

| Behavior                                                                                             | Requirement | Why Manual                                                      | Test Instructions                                                                                                                                                                                 |
| ---------------------------------------------------------------------------------------------------- | ----------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Friendly-companion copy on duplicate-review step matches UI-SPEC voice ("Discard new" / "Keep both") | TXN-06      | Subjective tone judgement; not auto-checkable                   | Run `pnpm tauri dev`, import a CSV with one duplicate, confirm wizard buttons read "Discard new" and "Keep both" (not "Reject"/"Accept")                                                          |
| Direction icons + colors match UI-SPEC                                                               | TXN-01      | Visual contract — color + icon — not asserted by snapshot tests | Manual check on `/transactions`: income rows show `ArrowDownLeft text-success`, expense rows show `ArrowUpRight text-muted-foreground`, transfer rows show `ArrowLeftRight text-muted-foreground` |
| Multi-currency display reads naturally for the user's locale                                         | TXN-07      | Locale-dependent rendering (decimal/thousands separators)       | Manual check with `en-US`, `de-DE`, `ja-JP` locales — values render with the right separators and currency placement                                                                              |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags (`--watch`, `--ui`, `pnpm dev`) in test commands
- [ ] Feedback latency < 30s for quick run, < 60s for full Rust suite
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
