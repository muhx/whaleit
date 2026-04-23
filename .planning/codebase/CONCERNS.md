# Codebase Concerns

**Analysis Date:** 2026-04-20

Whaleit is a 158k-LoC Rust + 161k-LoC TS/TSX monorepo (Cargo workspace + pnpm
workspace). It has three deployment surfaces (Tauri desktop, Axum web server,
iOS/Android via Tauri mobile) that share a `crates/core` business-logic layer.
The issues below are grouped by theme; file paths and line numbers point to the
actual offending sites.

## Tech Debt

**Health auto-fix actions are stubs:**

- Issue: Three `execute_fix` branches in the health service log
  `"fix action not yet implemented"` and return `Ok(())`. The UI can offer "Fix
  this" buttons that silently do nothing.
- Files:
  - `crates/core/src/health/service.rs:584` (`sync_prices`, `retry_sync`)
  - `crates/core/src/health/service.rs:594` (`fetch_fx`)
  - `crates/core/src/health/service.rs:603` (`migrate_classifications`)
- Impact: Users click a remediation action, get no error, and assume the issue
  is resolved. Undermines trust in the Health panel.
- Fix approach: Wire the fix actions to `QuoteService::sync_assets`,
  `FxService::refresh_rates`, and
  `TaxonomyService::migrate_legacy_classifications`. The payload parsing is
  already in place; just replace the `warn!` + `Ok(())` with the actual service
  call.

**AI chat tag API is a no-op on both backends:**

- Issue: `add_tag`, `remove_tag` endpoints accept requests and return success
  without persisting anything. The `ChatService` has no tag storage.
- Files:
  - `apps/server/src/api/ai_chat.rs:233` and
    `apps/server/src/api/ai_chat.rs:244`
  - `apps/tauri/src/commands/ai_chat.rs:154` and
    `apps/tauri/src/commands/ai_chat.rs:165`
- Impact: Frontend thinks tags are saved; reloading the thread reveals they
  vanished.
- Fix approach: Add a `thread_tags` table to `crates/storage-sqlite`, implement
  `add_tag/remove_tag` on `ChatService`, and replace the TODO bodies with calls
  through `state.ai_chat_service`.

**Addon store ratings endpoint is empty (web mode):**

- Issue: `get_addon_ratings_web` returns an empty `Vec` to "avoid UI errors".
  Desktop/Tauri mode reaches the real store; web users silently see no ratings.
- Files: `apps/server/src/api/addons.rs:187`
- Impact: Feature parity gap between desktop and web. Docker users see a
  less-useful addon catalog.
- Fix approach: Proxy the web handler to `ADDON_STORE_API_BASE_URL`
  (`crates/core/src/addons/service.rs:11`) the same way the Tauri command does.

**Symbol-resolution "manual" defaults duplicated:**

- Issue: `createManualSymbol` hardcodes fallback values that also live in
  `create-custom-asset-dialog.tsx`. Drift guaranteed.
- Files:
  `apps/frontend/src/pages/activity/import/components/symbol-resolution-panel.tsx:17`
- Impact: When one copy is updated (e.g., new `dataSource` value), the other
  produces assets that look manual but route differently.
- Fix approach: Extract to `apps/frontend/src/lib/manual-symbol.ts` and import
  from both sites.

**Permission "detection" for addons is substring matching:**

- Issue: `detect_addon_permissions` does `file.content.contains(pattern)`
  against bundled JS. A minifier renaming `getHoldings` to `e.gH()` evades
  detection. The SDK merges declared + detected, so a malicious addon can just
  omit declarations and evade detection simultaneously.
- Files: `crates/core/src/addons/service.rs:148-475`, called from
  `extract_addon_zip_internal` (`crates/core/src/addons/service.rs:570`) and
  `install_addon_zip` (`crates/core/src/addons/service.rs:1418`).
- Impact: The "Permissions required" consent screen shown to users can be
  misleading.
- Fix approach: Treat permission declarations as a manifest-only contract
  (require `permissions[]`); enforce at the host API boundary by rejecting calls
  from addons whose manifest didn't declare that category.

**Large files with mixed responsibilities:**

- Files (all >1500 lines, indicating missing splits):
  - `crates/core/src/activities/activities_service.rs` (4279 lines)
  - `crates/storage-sqlite/src/sync/app_sync/repository.rs` (3673 lines)
  - `crates/core/src/quotes/service.rs` (2740 lines)
  - `crates/core/src/quotes/sync.rs` (2446 lines)
  - `apps/server/src/api/device_sync_engine.rs` (2001 lines)
  - `crates/ai/src/chat.rs` (1973 lines)
  - `crates/core/src/portfolio/snapshot/snapshot_service.rs` (1933 lines)
  - `apps/tauri/src/commands/device_sync/mod.rs` (1885 lines)
  - `packages/ui/src/components/data-grid/use-data-grid.ts` (3286 lines)
  - `apps/frontend/src/lib/types.ts` (1929 lines — single file with all
    cross-cutting types)
  - `apps/frontend/src/adapters/web/core.ts` (1394 lines — giant `COMMANDS` map)
- Impact: Slow compile, hard to review, merge conflicts, IDE sluggishness.
- Fix approach: Split `activities_service.rs` into `activities_crud.rs` /
  `activities_import.rs` / `activities_validation.rs`. Shard `lib/types.ts` by
  domain.

## Known Bugs / Recent Bug-Fix Signals

Recent commits show a high concentration of fixes in two areas — hot fragility
zones:

**AI-assistant + CSV import flow (>=20 fix commits since last release):**

- Symptoms: tool-call storms, language drift post-tool-call, stale CSV cards,
  incorrect validation behavior when account unselected, empty `csvContent`
  accepted silently.
- Files:
  `apps/frontend/src/features/ai-assistant/hooks/use-chat-import-session.ts`
  (1025 lines),
  `apps/frontend/src/features/ai-assistant/components/tool-uis/import-csv-tool-ui.tsx`,
  `crates/ai/src/chat.rs`.
- Workaround: N/A — fixes merged, but cluster suggests the feature is still
  settling.
- Indicator: >=15 consecutive `fix(ai-import): ...` commits on `main` within a
  short window (e.g., `1c124078`, `d2f4d3d8`, `3a1f535c`, `6bd0c861`,
  `431f4c84`, `fb06c517`, `417a83be`, `322d847a`, `cbd2881b`, `59478d36`,
  `7b6578ac`, `5506d85f`, `3b4a3024`).

**Symbol mapping validation churn:**

- Symptoms: revalidation running after stale state; "Validate" button leaving UI
  inconsistent.
- Files:
  `apps/frontend/src/pages/activity/import/steps/review-step.tsx:209-348`.
- Indicator: commits `68676ddb`, `4722c50a`, `7d562698`, `d9bb9dbd`, `eea9f2da`
  all touching the same flow.

**Rust 1.95 clippy compatibility:**

- Files: `f48f1a44` and `748531ca` show hand-fixing `sort_by` -> `sort_by_key`
  across the codebase for a toolchain bump.
- Indicator: No `rust-toolchain.toml` pinning means CI-breaking clippy surprises
  on every minor Rust release.
- Fix approach: Add `rust-toolchain.toml` with a known-good channel.

## Security Considerations

**No Content Security Policy on Tauri WebView:**

- Risk: `"security": { "csp": null }` disables CSP entirely. The frontend loads
  user-installed addon bundles as Blob URLs
  (`apps/frontend/src/addons/addons-core.ts:127` creates a `Blob`, then
  `URL.createObjectURL(blob)`, then `await import(blobUrl)`). With no CSP, a
  malicious addon can inject a remote `<script>` element or fetch user data to
  arbitrary hosts.
- Files:
  - `apps/tauri/tauri.conf.json:71` (`"csp": null`)
  - `apps/frontend/src/addons/addons-core.ts:126-133` (blob-URL dynamic import)
- Current mitigation: Permission declaration/detection (bypassable — see
  "permission detection" above).
- Recommendations:
  1. Set a strict `csp` in `tauri.conf.json`
     (`default-src 'self'; script-src 'self' blob:; connect-src 'self' https://whaleit.app https://api.whaleit.app ...`).
  2. Enforce declared permissions at the host-API boundary, not just UI consent.
  3. Audit `packages/addon-sdk/src/host-api.ts` (781 lines) to ensure every
     function checks the calling addon's declared categories.

**Custom quote-provider SSRF surface:**

- Risk: `validate_url` explicitly allows arbitrary hosts ("self-hosted providers
  on private networks are supported"). Any user with write access to
  custom-provider settings can direct the backend to fetch
  `http://169.254.169.254/...` (AWS metadata), internal databases, or rebind DNS
  to exfiltrate.
- Files: `crates/core/src/custom_provider/model.rs:60` (scheme check only),
  fetched by `crates/core/src/quotes/custom_scraper_provider.rs:40` (reqwest
  client with `redirect::Policy::none()` — good — but no host allow/deny list,
  no DNS pinning).
- Current mitigation: Redirects disabled, 15s timeout, response size cap
  (`MAX_RESPONSE_BYTES`). Feature is explicitly documented as allowing private
  networks (commit `b404c93a`).
- Recommendations (desktop mode — single user — lower risk; web/docker mode —
  multi-user — meaningful risk):
  1. In web/server mode, add a configurable
     `WF_CUSTOM_PROVIDER_ALLOW_PRIVATE=false` default.
  2. Block RFC1918 / loopback / link-local ranges unless explicitly opted in.
  3. Log every custom-provider fetch URL at INFO.

**Addon dev server permissive CORS to hardcoded origins:**

- Risk: `packages/addon-dev-tools/dev-server.js:37` whitelists
  `http://localhost:1420` and `http://localhost:3000` with `credentials: true`.
  Any process on the dev machine binding those ports can request addon source.
  Lower risk (dev-only).
- Files: `packages/addon-dev-tools/dev-server.js:37-41`
- Current mitigation: Dev-only (not shipped), opt-in via
  `VITE_ENABLE_ADDON_DEV_MODE`.

**Rust `.unwrap()` density in crypto and sync:**

- Risk: 1359 occurrences across 99 files. Concentrations in hot paths:
  - `crates/device-sync/src/crypto.rs` — 15 unwraps in E2EE primitives.
  - `crates/device-sync/src/client.rs` — 14 unwraps.
  - `crates/device-sync/src/engine/mod.rs` — 5 unwraps in the sync engine.
  - `crates/connect/src/client.rs` — 1 in HTTP client.
  - `crates/core/src/quotes/sync.rs` — 16 unwraps including
    `SYNC_LOCKS.lock().unwrap()` (panic if lock poisoned).
  - `crates/core/src/portfolio/snapshot/holdings_calculator.rs` — 9 unwraps in
    math.
  - `crates/market-data/src/provider/alpha_vantage/mod.rs` — 28 unwraps in
    response parsing.
- Impact: Any unhandled poison or parse failure crashes the Tauri backend
  (taking the whole app) or terminates an Axum request task. On mobile
  (iOS/Android) this surfaces as app kills.
- Current mitigation: 391 `expect()` calls add slightly more context, but don't
  prevent panics.
- Recommendations: Start with the cryptographic/sync paths — replace `unwrap()`
  in `crates/device-sync/src/engine/runtime.rs:126-149` (5
  `pairing_flows.lock().unwrap()`) with `.map_err(|_| ...Poisoned...)?`.

**Secrets storage key derivation order:**

- Observation: `apps/server/src/auth.rs:219` derives JWT + secrets keys from a
  single `WF_SECRET_KEY` via HKDF (good). Rotating the master key requires
  migrating all stored secrets — `FileSecretStore::persist_migrated`
  (`apps/server/src/secrets/mod.rs:63`) exists for this. Verify it is actually
  invoked on rotation; current code paths aren't obvious.
- Files: `apps/server/src/auth.rs:218-231`, `apps/server/src/secrets/mod.rs:63`

**Inline HTML injection sites (reviewed, currently safe):**

- `apps/frontend/src/pages/settings/market-data/response-preview.tsx:88` — safe:
  input is `escapeHtml`'d first (lines 60, 93-99), only numeric `jsonpath` spans
  are re-injected. Must stay that way on future edits.
- `packages/ui/src/components/ui/chart.tsx:78` — shadcn-generated CSS template,
  safe.
- No actively exploitable cases found. Flag in code review: any new inline HTML
  injection site without prior escaping.

## Performance Bottlenecks

**N-assets loop in quote sync with per-provider batching:**

- Problem: `QuoteSyncService::build_sync_plans` loops over every asset to group
  by provider (`crates/core/src/quotes/sync.rs:431`), then calls
  `get_quote_bounds_for_assets` once per provider. Good batching inside
  providers, but for 1000+ holdings, `build_asset_sync_plan` still allocates per
  asset.
- Files: `crates/core/src/quotes/sync.rs:425-450`,
  `crates/core/src/quotes/sync.rs:1265`, `:1318`, `:1359`, `:1374` (five
  separate passes over `assets`).
- Improvement path: Single fused pass building all three structures
  (`assets_by_provider`, `quote_bounds`, plans) in one iteration.

**Portfolio snapshot service operates on entire history:**

- Problem: `SnapshotService` (1933 lines) recalculates account state snapshots.
  `save_snapshots` does `diesel::replace_into` row-by-row
  (`crates/storage-sqlite/src/portfolio/snapshot/repository.rs:51`), which in
  SQLite triggers WAL writes per row rather than batched transactions.
- Files: `crates/storage-sqlite/src/portfolio/snapshot/repository.rs:33-59`
- Improvement path: Wrap the insert in a single transaction (already inside
  `writer.exec` — verify all replacements share one commit).

**`.unwrap()` on `scraper::Selector::parse`:**

- Problem: Hot-path HTML parsing in `custom_scraper_provider` does
  `Selector::parse("*").expect(...)` every invocation
  (`crates/core/src/quotes/custom_scraper_provider.rs:787`, `:811`, `:812`).
  Selector parsing is not free.
- Improvement path: `LazyLock<Selector>` for the three static selectors.

**AI chat service — 1973-line monolith:**

- Problem: `crates/ai/src/chat.rs` holds streaming, tool-dispatch, history
  persistence, provider tuning, and rig hooks in one struct. Each incoming
  message walks through the whole file. Recent commits (`fa13d23d`, `43c2d893`,
  `33286314`, `deba54c6`) all patched performance/correctness here.
- Improvement path: Extract streaming into `crates/ai/src/streaming.rs`;
  tool-call dispatch into `crates/ai/src/dispatch.rs`.

**Mobile form with 34 `any` casts in one file:**

- Problem:
  `apps/frontend/src/pages/activity/components/mobile-forms/mobile-details-step.tsx`
  contains 34 `any` occurrences (most as `any` casts on `setValue/watch`). Each
  cast defeats memoization hints and increases re-render cost on mobile devices.
- Files:
  `apps/frontend/src/pages/activity/components/mobile-forms/mobile-details-step.tsx`
- Improvement path: Type the form values properly (`NewActivityFormValues`
  already exists); replace casts with `Path<NewActivityFormValues>`
  discriminated unions.

## Fragile Areas

**Device sync engine state machine:**

- Files: `crates/device-sync/src/engine/mod.rs` (1815 lines),
  `crates/device-sync/src/client.rs` (1735 lines),
  `apps/server/src/api/device_sync_engine.rs` (2001 lines),
  `apps/tauri/src/commands/device_sync/mod.rs` (1885 lines).
- Why fragile: E2EE + pairing + reconcile + outbox replay + two-phase init, all
  touching `Mutex`/`RwLock` and calling `.unwrap()` on poisoned locks
  (`crates/device-sync/src/engine/runtime.rs:126-149`).
- Safe modification: Do not modify without running the full device-sync
  integration test suite. Touch one phase (pair, push, pull, reconcile) at a
  time.
- Test coverage: Unknown — `crates/device-sync/src/engine/runtime.rs` uses
  unwraps on lock guards; lock-poisoning paths are untested.

**AI chat streaming + tool-call loops:**

- Files: `crates/ai/src/chat.rs:1973`,
  `apps/frontend/src/features/ai-assistant/hooks/use-chat-runtime.ts:995`,
  `apps/frontend/src/features/ai-assistant/hooks/use-chat-import-session.ts:1025`.
- Why fragile: 6 `rig` hook callbacks, streaming deltas, tool-call detection
  state machine with module-level `Set`s (`1c124078`). Recent commit `fa13d23d`
  literally named "prevent tool-call storms and text loops via rig hook".
- Safe modification: Any change requires end-to-end test with at least two LLM
  providers (Ollama + OpenAI) and a tool-heavy conversation (CSV import).
- Test coverage: `types.test.ts` (898 lines) covers types, not runtime behavior.

**Frontend single-file type registry:**

- Files: `apps/frontend/src/lib/types.ts` (1929 lines).
- Why fragile: Every feature imports from this file; circular-import risk; a
  `git blame` hotspot; PR diffs frequently span 10+ sections.
- Safe modification: Add new types in feature-local files; only promote here
  when shared across 3+ features.

**Snapshot + Holdings + Valuation + Performance coupling:**

- Files: `crates/core/src/portfolio/snapshot/snapshot_service.rs`,
  `holdings/holdings_service.rs`, `valuation/valuation_service.rs`,
  `performance/performance_service.rs`.
- Why fragile: All four services must agree on timezone handling
  (`new_with_timezone` variants). Cross-service event wiring in
  `apps/server/src/main_lib.rs:231-290`.
- Safe modification: Change `timezone` handling in all four services together,
  with regression tests.

**Activities service import + validation:**

- Files: `crates/core/src/activities/activities_service.rs` (4279 lines).
- Why fragile: Handles CSV parsing, FX normalization, symbol resolution, asset
  creation, idempotency keys — all in one service. Recent fix `d2f4d3d8` made
  per-account validation failures handled "gracefully" after regressions.

## Scaling Limits

**SQLite single-writer:**

- Current capacity: Single `write_actor` task
  (`crates/storage-sqlite/src/db/write_actor.rs`). All writes serialize through
  one mpsc channel.
- Limit: Write throughput around 100-500 tx/s depending on WAL checkpointing.
- Scaling path: Acceptable for desktop + single-user web. For multi-tenant
  hosted deployments, would need per-user databases or migration to Postgres.

**Market-data provider rate limits:**

- Alpha Vantage free tier: 5 calls/minute (documented in
  `crates/market-data/src/provider/alpha_vantage/mod.rs:9`).
- Finnhub: rate-limited in `crates/market-data/src/registry/circuit_breaker.rs`.
- Limit: A user with 1000 symbols syncing initial history will hit provider
  limits and the circuit breaker opens.
- Scaling path: Already implemented — circuit breaker
  (`HALF_OPEN_SUCCESS_THRESHOLD` in
  `crates/market-data/src/registry/circuit_breaker.rs:60`). Verify it is tuned.

**Frontend bundle size risk:**

- `packages/ui/src/components/data-grid/use-data-grid.ts` (3286 lines) +
  `packages/ui/src/components/data-grid/data-grid-cell-variants.tsx` (2970
  lines) all imported by default. Tree-shaking may not eliminate unused
  variants.
- Scaling path: Split variants into separate files; lazy-load the grid on routes
  that use it.

## Dependencies at Risk

**Rig (AI framework):**

- Risk: `crates/ai/src/chat.rs` comments reference "rig hook" (`fa13d23d`). Rig
  is a newer crate with a smaller ecosystem; breaking changes more likely.
- Impact: AI assistant feature breaks on upgrade.
- Migration plan: Abstraction via `AiProviderService` trait exists
  (`crates/ai/src/providers.rs`); swapping implementations is possible but
  requires rewriting tool-call plumbing.

**Recharts typing:**

- Three `@ts-expect-error` comments on recharts interactions:
  - `apps/frontend/src/components/history-chart-symbol.tsx:60`
  - `addons/investment-fees-tracker/src/components/donut-chart.tsx:146`
- Risk: Recharts major-version bump silently breaks charts; the
  `@ts-expect-error` comments become stale.
- Migration plan: Pin recharts major version; add visual regression tests for
  charts.

**Diesel + SQLite 3.35+ feature:**

- Workspace Cargo.toml enables `returning_clauses_for_sqlite_3_35`. Target
  systems on Debian Stable (older libsqlite) may fail.
- Current mitigation: `rusqlite = { version = "0.34", features = ["bundled"] }`
  ships its own SQLite.

**Tauri v2 updater plugin:**

- Endpoint:
  `https://whaleit.app/releases/{{target}}/{{arch}}/{{current_version}}`
  (`apps/tauri/tauri.conf.json:41`). Single point of failure — outage blocks
  auto-updates for all desktop users.
- Public key is pinned (good).

## Missing Critical Features

**From ROADMAP.md (checked, not yet implemented):**

- Options trading support (Phase 4, explicit `- [ ]` in `ROADMAP.md:65`).
- Portfolio analysis: sector allocation, concentration risk, dividend yield
  (Phase 5, `ROADMAP.md:71`).
- Monte Carlo projection (Phase 5, `ROADMAP.md:72`).
- Retirement/FIRE planner withdrawal strategies (Phase 5, `ROADMAP.md:73`).
  Note: `apps/frontend/src/pages/fire-planner/` and
  `crates/core/src/portfolio/fire/calculator.rs` exist — basic planner is there,
  but withdrawal strategies missing.
- Addons marketplace (Phase 6, `ROADMAP.md:77`).
- Budgeting and spend tracking module (`ROADMAP.md:81` — noted with typo:
  "Sprend").

**Operational gaps observed:**

- No `rust-toolchain.toml` — toolchain drift causes CI breakage (see commits
  `748531ca`, `f4403a7e`, `f48f1a44`, `644dcb6e` all fixing Rust 1.95 issues).
- No Dependabot / renovate config found in `.github/` for Rust crate updates.
- No `.github/CODEOWNERS` observed — review routing is implicit.

## Test Coverage Gaps

**Frontend tests: 42 test files for 579 source files (~7% file coverage):**

- What's not tested:
  - AI assistant streaming flows
    (`apps/frontend/src/features/ai-assistant/hooks/use-chat-runtime.ts`,
    `use-chat-import-session.ts`).
  - Addon loading + context creation (`apps/frontend/src/addons/addons-core.ts`,
    `addons-runtime-context.ts`).
  - Device sync UI (`apps/frontend/src/features/devices-sync/`).
  - Whaleit Connect auth flow (`apps/frontend/src/features/whaleit-connect/`).
- Risk: UI regressions in security-critical flows (OAuth callback, device
  pairing) ship undetected.
- Priority: **High** for addon loading and auth flows; **Medium** elsewhere.

**Rust test concentration is uneven:**

- Heavy: `crates/core/src/activities/activities_service_tests.rs` (5147),
  `crates/core/src/portfolio/snapshot/snapshot_service_tests.rs` (5314),
  `crates/core/src/portfolio/snapshot/holdings_calculator_tests.rs` (5136).
- Light/Missing:
  - `crates/device-sync/` — no `*_tests.rs` files visible at top level. Lock
    poisoning, pairing phase transitions untested.
  - `crates/connect/` — broker service (1692 lines) has limited visible tests.
  - `crates/ai/src/chat.rs` (1973 lines) — no `chat_tests.rs` found.
  - `crates/core/src/addons/tests.rs` exists but addon path validation &
    permission detection edge cases (see zip-slip code
    `crates/core/src/addons/service.rs:92-130`) should have fuzz coverage.
- Priority: **High** for device-sync engine state transitions; **High** for
  addon zip extraction (security-critical); **Medium** for AI chat.

**E2E coverage:**

- 10 Playwright specs (`e2e/01-happy-path.spec.ts` through
  `e2e/10-symbol-mapping-validation.spec.ts`). Covers happy path, activities,
  FX, CSV import, form validation, data grid, asset creation,
  holdings/performance, bulk holdings, symbol mapping.
- What's not covered: Device sync pairing, addon installation, AI assistant
  flows, Whaleit Connect login, backup/restore.
- Priority: **Medium** — these are the features most likely to have UI
  regressions given their recent churn.

**Addon SDK version compatibility:**

- Issue: `validateAddonCompatibility` in
  `apps/frontend/src/addons/addons-core.ts:49` only warns on SDK version
  mismatch — never blocks. Comment explicitly says "Future: implement proper
  semver compatibility if needed."
- Risk: Old addons break silently on SDK major bumps.
- Priority: **Medium**.

---

_Concerns audit: 2026-04-20_
