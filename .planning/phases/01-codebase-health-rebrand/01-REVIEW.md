---
phase: 01-codebase-health-rebrand
reviewed: 2026-04-20T15:30:00Z
depth: standard
files_reviewed: 38
files_reviewed_list:
  - apps/frontend/src/globals.css
  - apps/tauri/tauri.conf.json
  - apps/tauri/src/menu.rs
  - apps/server/src/main_lib.rs
  - crates/ai/src/system_prompt.txt
  - Dockerfile
  - compose.yml
  - compose.dev.yml
  - README.md
  - packages/ui/package.json
  - apps/frontend/vite.config.ts
  - apps/frontend/src/lib/types.ts
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
  - apps/frontend/src/adapters/web/core.ts
  - apps/frontend/src/adapters/web/modules/accounts.ts
  - apps/frontend/src/adapters/web/modules/activities.ts
  - apps/frontend/src/adapters/web/modules/portfolio.ts
  - apps/frontend/src/routes.tsx
  - apps/frontend/src/pages/onboarding/onboarding-page.tsx
  - apps/frontend/src/pages/onboarding/onboarding-step1.tsx
  - apps/frontend/src/pages/onboarding/onboarding-step2.tsx
  - apps/frontend/src/pages/onboarding/onboarding-connect.tsx
  - apps/frontend/src/pages/onboarding/onboarding-appearance.tsx
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-04-20T15:30:00Z
**Depth:** standard
**Files Reviewed:** 38
**Status:** issues_found

## Summary

Reviewed 38 source files changed across 4 sub-plans in phase 01-codebase-health-rebrand: visual identity (color palette + icons), backend/config rebrand, frontend package scope rename + onboarding rebrand, and modular splits (web adapter + types).

The rebrand is thorough and consistent. Backend strings, Tauri config, Docker/compose, AI prompt, npm scope, and onboarding screens all reflect "WhaleIt" branding. The modular splits preserve backward compatibility via barrel re-exports. Build and type-check pass per phase summaries.

Two warnings found: a Docker image reference that mixes old org with new name (potential deployment breakage), and `support@whaleit.app` pointing to a domain that likely doesn't have email configured yet. Three info items cover redundant CSS, known deferred references, and pre-existing CSP disable.

## Warnings

### WR-01: Docker image reference uses old org name with new app name

**File:** `compose.yml:11`
**Issue:** The image reference `afadil/whaleit:latest` combines the old Docker Hub organization (`afadil`) with the new app name (`whaleit`). The README now points to GitHub org `muhx/whaleit`. If this image is pushed to Docker Hub under `afadil/whaleit`, it will work — but the org reference is inconsistent with all other `muhx/whaleit` URLs in the codebase. If someone expects the image at `muhx/whaleit:latest`, the compose file will fail to pull.
**Fix:**
```yaml
# Either update to match the new GitHub org:
image: muhx/whaleit:latest
# Or document the Docker Hub org explicitly in compose.yml comments
```

### WR-02: support@whaleit.app email in error dialog may be non-functional

**File:** `apps/tauri/src/menu.rs:63`
**Issue:** The "Report Issue" dialog tells users to email `support@whaleit.app`. Unlike the deferred `support@wealthfolio.app` references in the Connect feature (which point to an existing domain), `whaleit.app` is the new domain and may not have email configured yet. Users clicking "Report Issue" will see a non-functional email address.
**Fix:**
```rust
// If whaleit.app email isn't set up yet, keep the old functional address:
.message("If you encounter any issues, please email us at support@wealthfolio.app")
// Or add a comment documenting the migration dependency:
// TODO: Update to support@whaleit.app once email is configured on whaleit.app domain
```

## Info

### IN-01: Duplicate color definitions in globals.css

**File:** `apps/frontend/src/globals.css:107-211` and `apps/frontend/src/globals.css:273-357`
**Issue:** The same color scales (blue, cyan, green, red, orange, purple) are defined identically in both the `@theme` block (lines 107-211) and the `@theme inline` block (lines 273-357). In Tailwind v4, `@theme` generates CSS custom properties and utility classes, while `@theme inline` generates utility classes without separate CSS custom properties. The `@theme inline` values override `@theme` when both define the same token. Since the values are identical, there's no visual bug, but ~100 lines of redundant CSS definitions.
**Fix:** Remove the duplicate color scales from one of the two `@theme` blocks. If utility classes are needed without CSS variables, keep them in `@theme inline` only. If CSS variables are also needed, keep them in `@theme` only.

### IN-02: Known deferred rebrand references remain in codebase

**File:** Multiple files across `apps/frontend/src/features/connect/` and `apps/frontend/src/features/devices-sync/`
**Issue:** Several infrastructure-bound references remain unchanged per documented decisions (D-03, D-06):
- `wealthfolio.app` URLs (connect, auth, docs) — 15+ occurrences
- `support@wealthfolio.app` email — 5 occurrences
- `__wealthfolio_query_client__` and `__wealthfolio_navigate__` globals — addon SDK contract
- `localStorage` key `wealthfolio-theme`
- `wealthfolio://` deep link protocol
These are documented as deferred in the phase summaries. No action needed now, but they should be tracked for the infrastructure migration phase.

### IN-03: Content Security Policy disabled in Tauri config (pre-existing)

**File:** `apps/tauri/tauri.conf.json:72`
**Issue:** `"csp": null` disables Content Security Policy entirely for the Tauri webview. This is a pre-existing condition (not introduced by this phase), but worth noting since security is a project constraint. With CSP disabled, any XSS vulnerability could load external resources. This was not introduced in this phase — flagging for awareness only.

---

_Reviewed: 2026-04-20T15:30:00Z_
_Reviewer: OpenCode (gsd-code-reviewer)_
_Depth: standard_
