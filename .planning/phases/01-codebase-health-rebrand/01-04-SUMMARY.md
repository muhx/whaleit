---
phase: 01-codebase-health-rebrand
plan: 04
subsystem: frontend-adapters, frontend-types
tags: [refactor, codebase-health, modular-split]
dependency_graph:
  requires: []
  provides: [modular-web-adapter, domain-type-files, removed-ActivityLegacy]
  affects: [apps/frontend/src/adapters/web/, apps/frontend/src/lib/types/]
tech_stack:
  added: []
  patterns: [barrel-re-export, domain-module-delegation]
key_files:
  created:
    - apps/frontend/src/adapters/web/modules/accounts.ts
    - apps/frontend/src/adapters/web/modules/activities.ts
    - apps/frontend/src/adapters/web/modules/holdings.ts
    - apps/frontend/src/adapters/web/modules/portfolio.ts
    - apps/frontend/src/adapters/web/modules/goals.ts
    - apps/frontend/src/adapters/web/modules/exchange-rates.ts
    - apps/frontend/src/adapters/web/modules/ai.ts
    - apps/frontend/src/adapters/web/modules/connect.ts
    - apps/frontend/src/adapters/web/modules/market-data.ts
    - apps/frontend/src/adapters/web/modules/taxonomies.ts
    - apps/frontend/src/adapters/web/modules/health.ts
    - apps/frontend/src/adapters/web/modules/device-sync.ts
    - apps/frontend/src/adapters/web/modules/settings.ts
    - apps/frontend/src/adapters/web/modules/secrets.ts
    - apps/frontend/src/adapters/web/modules/assets.ts
    - apps/frontend/src/adapters/web/modules/addons.ts
    - apps/frontend/src/adapters/web/modules/utilities.ts
    - apps/frontend/src/adapters/web/modules/alternative-assets.ts
    - apps/frontend/src/lib/types/account.ts
    - apps/frontend/src/lib/types/activity.ts
    - apps/frontend/src/lib/types/asset.ts
    - apps/frontend/src/lib/types/holding.ts
    - apps/frontend/src/lib/types/portfolio.ts
    - apps/frontend/src/lib/types/quote.ts
    - apps/frontend/src/lib/types/goal.ts
    - apps/frontend/src/lib/types/settings.ts
    - apps/frontend/src/lib/types/taxonomy.ts
    - apps/frontend/src/lib/types/health.ts
    - apps/frontend/src/lib/types/ai.ts
    - apps/frontend/src/lib/types/sync.ts
    - apps/frontend/src/lib/types/device.ts
    - apps/frontend/src/lib/types/alternative-assets.ts
    - apps/frontend/src/lib/types/contributions.ts
    - apps/frontend/src/lib/types/liabilities.ts
    - apps/frontend/src/lib/types/net-worth.ts
    - apps/frontend/src/lib/types/fx.ts
    - apps/frontend/src/lib/types/common.ts
    - apps/frontend/src/lib/types/tag.ts
  modified:
    - apps/frontend/src/adapters/web/core.ts
    - apps/frontend/src/lib/types.ts
decisions:
  - D-12:
      Web adapter split into 18 domain modules following shared adapter grouping
  - D-13:
      Types.ts split into 22 domain files with barrel re-export for backward
      compatibility
  - D-14: ActivityLegacy deprecated type removed (zero external usages verified)
metrics:
  duration: 30m
  completed: "2026-04-20"
---

# Phase 01 Plan 04: Modular Split Summary

Web adapter split into 18 domain modules + types.ts split into 22 domain files
with barrel re-exports. ActivityLegacy removed.

## Changes Made

### Task 1: Split web adapter into domain modules

- Extracted all 184 switch cases from the monolithic 1,394-line `core.ts` into
  18 domain-specific handler modules
- `core.ts` reduced from 1,394 to 844 lines — now a slim dispatcher with
  `handleCommand()` function delegating to modules
- Module files created: `accounts.ts`, `activities.ts`, `holdings.ts`,
  `portfolio.ts`, `goals.ts`, `exchange-rates.ts`, `ai.ts`, `connect.ts`,
  `market-data.ts`, `taxonomies.ts`, `health.ts`, `device-sync.ts`,
  `settings.ts`, `secrets.ts`, `assets.ts`, `addons.ts`, `utilities.ts`,
  `alternative-assets.ts`
- Each module exports pure functions that take `(url, payload)` and return
  `{ url, body }`
- All existing exports preserved: `invoke`, `COMMANDS`, `API_PREFIX`,
  `EVENTS_ENDPOINT`, `AI_CHAT_STREAM_ENDPOINT`, `isDesktop`, `isWeb`, `logger`,
  `toBase64`, `fromBase64`

### Task 2: Split types.ts into domain files with barrel re-export

- Split the 1,929-line `types.ts` into 22 domain-specific type files in
  `apps/frontend/src/lib/types/`
- `types.ts` reduced from 1,929 lines to 58 lines (barrel re-export)
- Domain files: `account.ts`, `activity.ts`, `asset.ts`, `holding.ts`,
  `portfolio.ts`, `quote.ts`, `goal.ts`, `settings.ts`, `taxonomy.ts`,
  `health.ts`, `ai.ts`, `sync.ts`, `device.ts`, `alternative-assets.ts`,
  `contributions.ts`, `liabilities.ts`, `net-worth.ts`, `fx.ts`, `common.ts`,
  `tag.ts`
- Removed `ActivityLegacy` deprecated type (per D-14)
- All existing imports from `@/lib/types` continue to work unchanged via barrel
  re-export

## Verification Results

| Check                  | Result                |
| ---------------------- | --------------------- |
| `pnpm type-check`      | ✅ Pass               |
| `pnpm build`           | ✅ Pass               |
| `pnpm test`            | ✅ 505/505 tests pass |
| Web adapter modules    | ✅ 18 files           |
| Domain type files      | ✅ 22 files           |
| ActivityLegacy removed | ✅ 0 occurrences      |
| core.ts lines          | 844 (was 1,394)       |
| types.ts lines         | 58 (was 1,929)        |

## Deviations from Plan

None - plan executed exactly as written.

## Commits

| Commit     | Description                                                                |
| ---------- | -------------------------------------------------------------------------- |
| `eddf554b` | refactor(01-04): split web adapter into 18 domain modules                  |
| `ac867552` | refactor(01-04): split types.ts into 22 domain files with barrel re-export |

## Self-Check: PASSED

All 39 created files verified present. Both commit hashes verified in git log.
