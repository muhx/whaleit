---
phase: 01
slug: codebase-health-rebrand
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-20
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 3.2 (TS) + Cargo test (Rust) |
| **Config file** | `apps/frontend/vitest.config.ts` / `Cargo.toml` |
| **Quick run command** | `pnpm type-check` |
| **Full suite command** | `pnpm check && cargo check` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `pnpm type-check`
- **After every plan wave:** Run `pnpm check && cargo check`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-task Verification Map

| task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| T1-01 | 01 | 1 | BRAND-01 | - | No user-facing "Wealthfolio" strings | build | `pnpm build && cargo check` | ✓ | complete |
| T1-02 | 01 | 1 | BRAND-06 | - | Internal crate names unchanged | grep | `grep -c 'name = "wealthfolio' Cargo.toml` | ✓ | complete |
| T2-01 | 02 | 1 | BRAND-02,03 | - | New color palette in globals.css | file | `test -f globals.css` | ✓ | complete |
| T2-02 | 02 | 1 | BRAND-02,04 | - | Icon assets exist in all formats | file | `ls apps/tauri/icons/icon.icns` | ✓ | complete |
| T3-01 | 03 | 2 | BRAND-01 | - | Package scope renamed | build | `pnpm build` | ✓ | complete |
| T3-02 | 03 | 2 | BRAND-01 | - | Feature dir renamed | file | `test -d features/connect` | ✓ | complete |
| T4-01 | 04 | 2 | BRAND-01 | - | Web adapter modularized | build | `pnpm type-check` | ✓ | complete |
| T4-02 | 04 | 2 | BRAND-01 | - | Types.ts split with barrel | build | `pnpm type-check` | ✓ | complete |

---

## Critical Success Tests

These MUST pass before phase is considered complete:

1. **Zero user-facing "Wealthfolio"**: `grep -ri "Wealthfolio" apps/frontend/src/ apps/tauri/tauri.conf.json` returns 0 results (excluding internal crate refs)
2. **Build integrity**: `pnpm build && cargo check` both pass
3. **Type safety**: `pnpm type-check` passes (validates types.ts barrel re-exports)
4. **Internal crates unchanged**: `grep 'name = "wealthfolio' Cargo.toml` returns all original crate names
5. **Icon assets present**: All required icon formats exist in `apps/tauri/icons/`
