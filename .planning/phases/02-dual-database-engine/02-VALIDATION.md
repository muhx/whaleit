---
phase: 02
slug: dual-database-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-21
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `tokio::test` + Vitest (TS) |
| **Config file** | `Cargo.toml` (Rust), `vitest.config.ts` (TS) |
| **Quick run command** | `cargo test -p whaleit-storage-postgres --lib` |
| **Full suite command** | `cargo test --workspace && pnpm test` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p whaleit-storage-postgres --lib`
- **After every plan wave:** Run `cargo test --workspace && cargo check --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 120 seconds

---

## Per-task Verification Map

| task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | DB-01 | T-02-01 | Connection strings not logged | unit | `cargo test -p whaleit-core --lib` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | DB-04 | — | N/A | unit | `cargo test -p whaleit-core --lib` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 2 | DB-02 | T-02-02 | Parameterized queries prevent SQL injection | unit | `cargo test -p whaleit-storage-postgres --lib` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 2 | DB-03 | — | N/A | unit | `cargo test -p whaleit-storage-postgres --lib` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 3 | DB-01 | T-02-03 | PG credentials not in compose.yml | integration | `cargo test -p whaleit-storage-postgres --lib` | ❌ W0 | ⬜ pending |
| 02-03-02 | 03 | 3 | DB-05 | — | N/A | integration | `cargo test -p whaleit-storage-postgres --lib` | ❌ W0 | ⬜ pending |
| 02-04-01 | 04 | 4 | DB-01,DB-02 | T-02-01 | Parity tests cover all repos | parity | `cargo test parity_ --workspace` | ❌ W0 | ⬜ pending |
| 02-04-02 | 04 | 4 | DB-02,DB-03 | — | N/A | parity | `cargo test parity_ --workspace` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/storage-postgres/tests/` — test module for PG repository tests
- [ ] PG test database setup fixture (Docker or embedded pg-embed)
- [ ] `crates/core/src/` — test stubs for async trait conversion

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Desktop app launches with SQLite | DB-04 | Requires running Tauri binary | `pnpm tauri dev` → verify SQLite in logs |
| Web server connects to PG | DB-04 | Requires running PG instance | `pnpm run dev:web` → verify PG connection in logs |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
