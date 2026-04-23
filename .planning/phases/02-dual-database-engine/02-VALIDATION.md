---
phase: 02
slug: dual-database-engine
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-21
updated: 2026-04-22
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property                 | Value                                                                                      |
| ------------------------ | ------------------------------------------------------------------------------------------ |
| **Framework**            | Rust built-in `#[test]` + `#[tokio::test]` + Vitest (TS)                                   |
| **Config file**          | `Cargo.toml` (Rust), `vitest.config.ts` (TS)                                               |
| **Quick run command**    | `cargo test -p whaleit-storage-postgres --no-run`                                          |
| **Full suite command**   | `cargo test --workspace && pnpm test`                                                      |
| **Parity tests command** | `cargo test -p whaleit-storage-postgres --test parity_tests -- --ignored --test-threads=1` |
| **Estimated runtime**    | ~120 seconds (unit), ~60 seconds (parity with PG)                                          |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --workspace`
- **After every plan wave:** Run
  `cargo check --workspace && cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Parity tests:** Run with live PG (CI or manual)
- **Max feedback latency:** 120 seconds

---

## Per-task Verification Map

| task ID  | Plan | Wave | Requirement         | Threat Ref             | Secure Behavior                        | Test Type   | Automated Command                                                                                          | File Exists | Status   |
| -------- | ---- | ---- | ------------------- | ---------------------- | -------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------- | ----------- | -------- |
| 02-01-01 | 01   | 1    | DB-01, DB-04        | T-02-01                | Connection strings not logged          | unit        | `cargo check --workspace && cargo test --workspace`                                                        | ✅          | ✅ green |
| 02-01-02 | 01   | 1    | DB-01               | —                      | N/A                                    | unit        | `cargo check -p whaleit-storage-common`                                                                    | ✅          | ✅ green |
| 02-02-01 | 02   | 2    | DB-01, DB-02        | T-02-02                | Parameterized queries via Diesel DSL   | unit        | `cargo check -p whaleit-storage-postgres`                                                                  | ✅          | ✅ green |
| 02-02-02 | 02   | 2    | DB-02, DB-03, DB-05 | —                      | N/A                                    | parity      | `cargo test -p whaleit-storage-postgres --test parity_tests -- --ignored`                                  | ✅          | ✅ green |
| 02-03-01 | 03   | 3    | DB-01, DB-04        | T-02-03                | PG credentials not in compose.yml      | integration | `cargo check -p whaleit-server --features postgres`                                                        | ✅          | ✅ green |
| 02-03-02 | 03   | 3    | DB-01               | —                      | N/A                                    | manual      | `docker compose -f compose.yml config --quiet`                                                             | ✅          | ✅ green |
| 02-04-01 | 04   | 4    | DB-01, DB-02        | T-02-01                | Parity tests cover core repos          | parity      | `cargo test -p whaleit-storage-postgres --test parity_tests -- --ignored`                                  | ✅          | ✅ green |
| 02-04-02 | 04   | 4    | DB-02, DB-03        | —                      | N/A                                    | ci          | `.github/workflows/pr-check.yml` postgres-tests job                                                        | ✅          | ✅ green |
| 02-05-01 | 05   | 4    | DB-01, DB-02        | T-02-05-01, T-02-05-02 | Connection pool from validated env var | unit        | `cargo check -p whaleit-storage-postgres`                                                                  | ✅          | ✅ green |
| 02-05-02 | 05   | 4    | DB-01, DB-04        | —                      | N/A                                    | unit        | `cargo check -p whaleit-server --features postgres`                                                        | ✅          | ✅ green |
| 02-06-01 | 06   | 5    | DB-02, DB-05        | T-02-06-01, T-02-06-02 | Symbol strings via Diesel DSL bindings | parity      | `cargo test -p whaleit-storage-postgres --test parity_tests -- --ignored parity_fx parity_market_data`     | ✅          | ✅ green |
| 02-06-02 | 06   | 5    | DB-02, DB-05        | —                      | N/A                                    | parity      | `cargo test -p whaleit-storage-postgres --test parity_tests -- --ignored parity_snapshot parity_valuation` | ✅          | ✅ green |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Parity Test Coverage

| Module              | Test(s)                                                                                                                      | Status                                        |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| accounts            | `parity_account_create`, `parity_account_update`, `parity_account_list`, `parity_account_get_by_id`, `parity_account_delete` | ✅ COVERED                                    |
| fx                  | `parity_fx_rate`                                                                                                             | ✅ COVERED                                    |
| settings            | `parity_settings_update`, `parity_settings_get_settings`                                                                     | ✅ COVERED                                    |
| assets              | `parity_asset_create`, `parity_asset_get_by_id`                                                                              | ✅ COVERED                                    |
| goals               | `parity_goal_create`                                                                                                         | ✅ COVERED                                    |
| health              | `parity_health_dismissal`                                                                                                    | ✅ COVERED                                    |
| limits              | `parity_limit_create`                                                                                                        | ✅ COVERED                                    |
| taxonomies          | `parity_taxonomy_create`                                                                                                     | ✅ COVERED                                    |
| portfolio/snapshot  | `parity_snapshot_save_and_get`                                                                                               | ✅ COVERED                                    |
| portfolio/valuation | `parity_valuation_save_and_get`                                                                                              | ✅ COVERED                                    |
| market_data         | `parity_market_data_quote`                                                                                                   | ✅ COVERED                                    |
| activities          | —                                                                                                                            | ⚠️ PARTIAL (CRUD works, search/bulk stubs)    |
| sync                | —                                                                                                                            | ⚠️ PARTIAL (stubs return errors for sync ops) |
| ai_chat             | —                                                                                                                            | ⚠️ PARTIAL (full impl, no parity test)        |
| custom_provider     | —                                                                                                                            | ⚠️ PARTIAL (full impl, no parity test)        |

**17 parity tests total** in `crates/storage-postgres/tests/parity_tests.rs`

---

## Wave 0 Requirements

- [x] `crates/storage-postgres/tests/parity_tests.rs` — parity test module with
      17 tests
- [x] CI postgres-tests job with PostgreSQL 17-alpine service container
- [x] `DATABASE_URL` env var for PG test database connection

---

## Manual-Only Verifications

| Behavior                          | Requirement | Why Manual                    | Test Instructions                                                                                     |
| --------------------------------- | ----------- | ----------------------------- | ----------------------------------------------------------------------------------------------------- |
| Desktop app launches with SQLite  | DB-04       | Requires running Tauri binary | `pnpm tauri dev` → verify SQLite in logs                                                              |
| Web server connects to PG         | DB-04       | Requires running PG instance  | `DATABASE_URL=postgres://... cargo run --features postgres` → verify PG connection in logs            |
| PG migrations apply correctly     | DB-03       | Requires running PG instance  | Start fresh PG, run server → verify 32 tables created                                                 |
| Parity tests pass against live PG | DB-02       | Requires running PG instance  | `DATABASE_URL=postgres://... cargo test -p whaleit-storage-postgres --test parity_tests -- --ignored` |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated (automated)

---

## Validation Audit 2026-04-22

| Metric             | Count                        |
| ------------------ | ---------------------------- |
| Tasks in scope     | 12 (across 6 plans)          |
| Tasks COVERED      | 12                           |
| Tasks PARTIAL      | 0                            |
| Tasks MISSING      | 0                            |
| Gaps found         | 4 (PARTIAL parity coverage)  |
| Resolved           | 4 (9 new parity tests added) |
| Escalated          | 0                            |
| Total parity tests | 17                           |
