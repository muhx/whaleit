---
phase: 01-codebase-health-rebrand
plan: 03
subsystem: ui, branding, packages
tags: [npm-scope, vite, typescript, react, rebrand, whaleit]

# Dependency graph
requires:
  - phase: 01-codebase-health-rebrand
    plan: 02
    provides: Theme and color token updates from Plan 02
provides:
  - npm package scope renamed from @wealthfolio/* to @whaleit/*
  - Feature directory renamed from wealthfolio-connect to connect
  - All frontend onboarding screens rebranded to WhaleIt
  - Zero user-facing Wealthfolio text in frontend TS/TSX files
affects: [02-database-engine, 03-transaction-core, ui, addons, ai-assistant]

# Tech tracking
tech-stack:
  added: []
  patterns: ["@whaleit/* npm scope for internal packages", "features/connect/ for broker sync feature"]

key-files:
  created:
    - apps/frontend/src/features/connect/ (renamed from wealthfolio-connect/)
    - apps/frontend/src/pages/settings/connect/ (renamed from wealthfolio-connect/)
  modified:
    - packages/ui/package.json
    - packages/addon-sdk/package.json
    - packages/addon-dev-tools/package.json
    - apps/frontend/vite.config.ts
    - apps/frontend/tsconfig.json
    - apps/frontend/src/routes.tsx
    - apps/frontend/src/App.tsx
    - apps/frontend/src/pages/onboarding/ (all 5 files)
    - packages/addon-sdk/src/manifest.ts
    - packages/addon-sdk/src/index.ts
    - packages/ui/tsup.config.ts

key-decisions:
  - "Renamed internal variable names (WealthfolioConnectProvider, useWealthfolioConnect) to WhaleIt equivalents for consistency, not strictly required per D-03"
  - "Left lowercase wealthfolio references in infrastructure URLs (wealthfolio.app, connect.wealthfolio.app), deep links (wealthfolio://), localStorage keys, and email addresses unchanged — these are service endpoints that require backend/infrastructure changes"
  - "Renamed addon-sdk property minWealthfolioVersion to minWhaleItVersion to match rebrand"

patterns-established:
  - "All internal packages use @whaleit/* scope: @whaleit/ui, @whaleit/addon-sdk, @whaleit/addon-dev-tools"
  - "Connect feature directory is features/connect/ (not features/wealthfolio-connect/)"
  - "Provider file is connect-provider.tsx (not wealthfolio-connect-provider.tsx)"

requirements-completed: [BRAND-01, BRAND-04]

# Metrics
duration: 24min
completed: 2026-04-20
---

# Phase 01 Plan 03: Package Scope Rename & Frontend Rebrand Summary

**Renamed npm scope from @wealthfolio/* to @whaleit/*, renamed connect feature directory, rebranded all onboarding screens, and swept all user-facing "Wealthfolio" text from frontend**

## Performance

- **Duration:** 24 min
- **Started:** 2026-04-20T14:22:00Z
- **Completed:** 2026-04-20T14:46:00Z
- **Tasks:** 2
- **Files modified:** 479

## Accomplishments
- Renamed 3 npm packages from @wealthfolio/* to @whaleit/* scope with all config updates
- Updated 1074 frontend import references across all TS/TSX files
- Renamed features/wealthfolio-connect/ → features/connect/ directory with all import path updates
- Rebranded all 5 onboarding screen files with WhaleIt branding
- Swept all user-facing "Wealthfolio" text across the entire frontend (89 occurrences in 30+ files)
- Updated all 3 addon packages (goal-progress-tracker, investment-fees-tracker, swingfolio-addon)
- Updated addon-sdk type property minWealthfolioVersion → minWhaleItVersion

## Task Commits

Each task was committed atomically:

1. **task 1: Rename package scope, update vite aliases, and sweep all @wealthfolio imports** - `a2a023cc` (feat)
2. **task 2: Rename feature directory, rebrand onboarding, sweep remaining frontend text** - `ebfec4c0` (feat)

## Files Created/Modified
- `packages/ui/package.json` - Renamed to @whaleit/ui, updated URLs to muhx/whaleit
- `packages/addon-sdk/package.json` - Renamed to @whaleit/addon-sdk, updated URLs
- `packages/addon-dev-tools/package.json` - Renamed to @whaleit/addon-dev-tools, CLI bin to whaleit
- `packages/ui/tsup.config.ts` - Updated alias from @wealthfolio/ui to @whaleit/ui
- `packages/addon-sdk/src/index.ts` - Updated JSDoc from @wealthfolio to @whaleit
- `packages/addon-sdk/src/manifest.ts` - minWealthfolioVersion → minWhaleItVersion
- `apps/frontend/vite.config.ts` - Updated aliases to @whaleit/*
- `apps/frontend/tsconfig.json` - Updated paths to @whaleit/*
- `apps/frontend/package.json` - Updated dependency references
- `apps/frontend/src/routes.tsx` - Updated import paths from wealthfolio-connect to connect
- `apps/frontend/src/App.tsx` - Updated import from features/wealthfolio-connect to features/connect
- `apps/frontend/src/features/connect/` - Renamed from features/wealthfolio-connect/ (29 files)
- `apps/frontend/src/pages/settings/connect/` - Renamed from settings/wealthfolio-connect/
- `apps/frontend/src/pages/onboarding/` - All 5 files rebranded (WhaleIt text, URLs)
- `addons/*/package.json` - Updated @whaleit/* dependency references
- `addons/*/tsconfig.json` - Updated @whaleit/* path aliases

## Decisions Made
- Renamed internal code identifiers (WealthfolioConnectProvider → WhaleItConnectProvider, useWealthfolioConnect → useWhaleItConnect) for consistency even though D-03 says internal names aren't strictly required
- Left infrastructure URLs unchanged: wealthfolio.app, connect.wealthfolio.app, auth.wealthfolio.app — these are actual service endpoints that require backend infrastructure changes
- Left deep link protocol `wealthfolio://` unchanged — requires OS-level registration changes
- Left localStorage key `wealthfolio-theme` unchanged — changing would reset user preferences
- Left email `support@wealthfolio.app` unchanged — requires actual email account to exist

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated tsconfig.json paths**
- **Found during:** task 1 (pnpm build failed with TS2307 module not found errors)
- **Issue:** Plan didn't mention updating apps/frontend/tsconfig.json paths, but TypeScript uses these for type resolution
- **Fix:** Updated tsconfig.json paths from @wealthfolio/* to @whaleit/*
- **Files modified:** apps/frontend/tsconfig.json
- **Verification:** pnpm type-check and pnpm build pass
- **Committed in:** a2a023cc (task 1 commit)

**2. [Rule 3 - Blocking] Updated addon tsconfig.json paths and package.json deps**
- **Found during:** task 1 (pnpm install failed with ERR_PNPM_WORKSPACE_PKG_NOT_FOUND)
- **Issue:** Addon packages (swingfolio-addon, etc.) also referenced @wealthfolio/* in dependencies and tsconfig paths
- **Fix:** Updated all 3 addon packages' package.json and tsconfig.json
- **Files modified:** addons/goal-progress-tracker/package.json, addons/swingfolio-addon/package.json, addons/investment-fees-tracker/package.json, and their tsconfig.json files
- **Verification:** pnpm install succeeds
- **Committed in:** a2a023cc (task 1 commit)

**3. [Rule 1 - Bug] Fixed minWealthfolioVersion type property rename**
- **Found during:** task 2 (type-check failed with TS2339 Property 'minWhaleItVersion' does not exist)
- **Issue:** Bulk sed replaced user-facing "Wealthfolio" to "WhaleIt" including `minWealthfolioVersion` in consumer code, but the type definition in addon-sdk still had the old name
- **Fix:** Updated packages/addon-sdk/src/manifest.ts to rename minWealthfolioVersion → minWhaleItVersion
- **Files modified:** packages/addon-sdk/src/manifest.ts
- **Verification:** pnpm type-check passes
- **Committed in:** ebfec4c0 (task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 blocking, 1 bug)
**Impact on plan:** All auto-fixes necessary for build correctness. No scope creep.

## Issues Encountered
- Initial package.json edits didn't persist for packages/ui and packages/addon-sdk — had to use sed for reliable replacement
- pnpm workspace resolution requires all workspace packages to have matching names — addon packages blocked install until their references were also updated

## Known Stubs
- `wealthfolio.app` URLs remain in about-page, connect feature — require infrastructure/backend setup before changing
- `support@wealthfolio.app` email remains — requires actual email account
- `wealthfolio://` deep link protocol remains — requires OS registration
- `__wealthfolio_query_client__` and `__wealthfolio_navigate__` globals remain — internal addon SDK contract
- `localStorage wealthfolio-theme` key remains — changing would reset user prefs

## Next Phase Readiness
- All frontend imports use @whaleit/* scope — ready for new feature development
- Connect feature directory renamed to features/connect/ — consistent naming
- Zero user-facing "Wealthfolio" text in frontend TS/TSX files
- Build and type-check pass cleanly
- Infrastructure URLs (wealthfolio.app domain, deep links) deferred to backend/infra phases

---
*Phase: 01-codebase-health-rebrand*
*Completed: 2026-04-20*

## Self-Check: PASSED

All key files verified present. Both commits (a2a023cc, ebfec4c0) exist in git log. SUMMARY.md created in correct location.
