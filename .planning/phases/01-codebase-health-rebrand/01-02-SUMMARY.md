---
phase: 01-codebase-health-rebrand
plan: 02
subsystem: branding
tags: [rebrand, config, documentation, rust]
dependency_graph:
  requires: []
  provides: [BRAND-01, BRAND-05, BRAND-06]
  affects: [tauri-bundle, docker-deployment, ai-assistant, documentation]
tech_stack:
  added: []
  patterns: [string-rename, config-update]
key_files:
  created: []
  modified:
    - apps/tauri/tauri.conf.json
    - apps/tauri/gen/apple/project.yml
    - apps/tauri/src/menu.rs
    - apps/tauri/src/lib.rs
    - apps/server/src/main_lib.rs
    - apps/server/src/api.rs
    - apps/server/src/api/settings.rs
    - crates/ai/src/system_prompt.txt
    - apps/tauri/src/services/connect_service.rs
    - Dockerfile
    - compose.yml
    - compose.dev.yml
    - README.md
decisions:
  - Auth salt strings in auth.rs left unchanged to avoid invalidating existing tokens/secrets (per threat model T-02-03)
  - Internal crate names (whaleit-*) left unchanged per D-06/D-07
  - Twitter handle @WhaleitApp kept as-is (external service URL, not under our control)
  - Docker binary renamed from whaleit-server to whaleit-server (user-facing in Docker)
  - Database filename whaleit.db kept as-is in Docker/compose to avoid breaking existing deployments
  - GitHub URLs updated from muhx/whaleit to muhx/whaleit
metrics:
  duration: 27m
  completed: 2026-04-20
  tasks: 2
  files_modified: 13
---

# Phase 01 Plan 02: Backend & Config Rebrand Summary

All user-facing "Whaleit" strings in backend (Rust), configuration, and documentation renamed to "WhaleIt". Internal crate references remain whaleit-* unchanged.

## Changes Made

### Task 1: Tauri Config & Rust User-Facing Strings
- **apps/tauri/tauri.conf.json**: productName, mainBinaryName → "WhaleIt"; identifier → "com.whaleit.app"; deep-link schemes → "whaleit"; updater endpoint → whaleit.app; window title → "WhaleIt"
- **apps/tauri/gen/apple/project.yml**: bundleIdPrefix → "com.whaleit"; PRODUCT_NAME → "WhaleIt"; PRODUCT_BUNDLE_IDENTIFIER → "com.whaleit.app"
- **apps/tauri/src/menu.rs**: App menu label, about dialog, update message, support email → WhaleIt
- **apps/tauri/src/lib.rs**: Build error message → "WhaleIt application"
- **apps/server/src/main_lib.rs**: Device display name → "WhaleIt Server"
- **apps/server/src/api.rs**: OpenAPI tag → "whaleit"
- **apps/server/src/api/settings.rs**: Backup filename prefix → "whaleit_backup_"

### Task 2: Documentation, Docker, AI Prompt
- **crates/ai/src/system_prompt.txt**: AI identity → "WhaleIt AI"
- **apps/tauri/src/services/connect_service.rs**: Comments → "WhaleIt Connect"
- **Dockerfile**: Binary name → whaleit-server; comments → WhaleIt Connect
- **compose.yml**: Service/container/image/volume → whaleit; comments → WhaleIt
- **compose.dev.yml**: Service name → whaleit; comments → WhaleIt
- **README.md**: Full rebrand — title, tagline ("Your Friendly Finance Companion"), GitHub URLs (muhx/whaleit), website (whaleit.app), Docker examples, folder structure
- **apps/server/src/api/settings.rs**: Update check URL → whaleit.app

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Updated support email in menu.rs**
- **Found during:** task 1
- **Issue:** Plan missed support@whaleit.app email in report_issue handler (line 63)
- **Fix:** Changed to support@whaleit.app
- **Files modified:** apps/tauri/src/menu.rs
- **Commit:** 7ae51160

**2. [Rule 2 - Missing] Updated update check URL in settings.rs**
- **Found during:** task 2
- **Issue:** Plan missed whaleit.app update check URL in settings.rs (line 217)
- **Fix:** Changed to whaleit.app
- **Files modified:** apps/server/src/api/settings.rs
- **Commit:** 33437d1e

None other — plan executed as written.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check` (tauri) | ✅ Passed |
| `grep -c "Whaleit" tauri.conf.json` = 0 | ✅ 0 |
| `grep -c "Whaleit" README.md` = 0 (or 1 external URL) | ✅ 1 (Twitter handle only) |
| Internal crate names unchanged (8 entries) | ✅ All 8 present |
| `grep "WhaleIt AI" system_prompt.txt` | ✅ Found |
| `grep "whaleit-server" Dockerfile` | ✅ Found |
| `grep "whaleit:" compose.yml` (service name) | ✅ Found |

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: deep-link-break | apps/tauri/tauri.conf.json | Existing deep links (whaleit://) will break — expected during rebrand (T-02-02, accept) |

## Self-Check: PASSED

All 13 modified files exist. Both task commits (7ae51160, 33437d1e) verified in git log.
