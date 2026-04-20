# Codebase Concerns

**Analysis Date:** 2026-04-20

## Architecture Concerns

### Monolithic Web Adapter Command Router
- Issue: `apps/frontend/src/adapters/web/core.ts` (1,394 lines) contains a single `invoke()` function with a 184-case `switch` statement that maps every backend command to HTTP request construction. Each case manually destructures `payload as { ... }` (123 type assertions) with no compile-time validation that payload shapes match what the server expects.
- Files: `apps/frontend/src/adapters/web/core.ts`
- Impact: Adding any new backend command requires editing this giant switch. Payload shape mismatches between frontend and server are only caught at runtime. The file grows linearly with every new API endpoint.
- Fix approach: Generate the command-to-HTTP mapping from a shared schema (e.g., OpenAPI or a typed definition file). At minimum, extract each command group into separate modules (activities, sync, ai, etc.) and compose them.

### Giant Types File
- Issue: `apps/frontend/src/lib/types.ts` is 1,929 lines containing every TypeScript interface in the frontend — from `Account` to `TaxonomyInstrumentMappingJson` to `ProviderTuningOverrides`. This is a "God file" that couples all feature modules together.
- Files: `apps/frontend/src/lib/types.ts`
- Impact: Any change to any type re-compiles every import. Circular import risks. Difficult to navigate and find relevant types.
- Fix approach: Co-locate types with their features. Domain types like `Activity`, `Account`, `Holding` belong in `features/` or `commands/`. Shared utility types stay in `lib/types.ts`. Split by domain: `types/activity.ts`, `types/portfolio.ts`, `types/sync.ts`, etc.

### Massive Rust Service Files
- Issue: Several core service files exceed healthy complexity thresholds: `crates/core/src/activities/activities_service.rs` (4,279 lines), `crates/storage-sqlite/src/sync/app_sync/repository.rs` (3,673 lines), `crates/core/src/quotes/service.rs` (2,740 lines). These contain business-critical logic that is hard to reason about in a single file.
- Files: `crates/core/src/activities/activities_service.rs`, `crates/storage-sqlite/src/sync/app_sync/repository.rs`, `crates/core/src/quotes/service.rs`
- Impact: High cognitive load for any change. Difficult to test individual responsibilities in isolation. Merge conflict magnet.
- Fix approach: Extract sub-responsibilities into dedicated modules. For `activities_service.rs`, split CSV import logic, symbol resolution, and activity CRUD into separate files behind the existing trait interface. For the sync repository, split snapshot export/restore, outbox management, and LWW application into separate modules.

### Dual-Platform Adapter Drift Risk
- Issue: The Tauri (`apps/frontend/src/adapters/tauri/`) and Web (`apps/frontend/src/adapters/web/`) adapters are separate implementations of the same interface. The web adapter is a 1,394-line monolith while Tauri adapter files delegate to Tauri IPC. There is no shared type contract ensuring both adapters accept/return the same shapes.
- Files: `apps/frontend/src/adapters/tauri/*.ts`, `apps/frontend/src/adapters/web/*.ts`
- Impact: API drift between desktop and web modes. A change in one adapter may not be reflected in the other, causing runtime errors in one mode but not the other.
- Fix approach: Define typed adapter interfaces in a shared module that both adapters implement. Add integration tests that verify both adapters produce identical outputs for identical inputs.

## Code Quality Concerns

### Hand-Rolled SQL in Sync Repository
- Issue: The sync repository constructs SQL strings manually using `format!()` with `escape_sqlite_str()` for sanitization (14 call sites) and `quote_identifier()` for column/table names. This includes dynamic INSERT/UPDATE with `json_value_to_sql_literal()` which serializes JSON values to SQL literals.
- Files: `crates/storage-sqlite/src/sync/app_sync/repository.rs` (lines 435-636, 672+)
- Impact: While the `escape_sqlite_str` function handles single quotes, hand-rolled SQL is inherently riskier than parameterized queries. Any missed escaping path could lead to SQL injection. The `json_value_to_sql_literal()` function in particular constructs values by string concatenation.
- Fix approach: Migrate dynamic SQL to Diesel's query builder where possible. For the JSON-to-SQL upsert path (`upsert_json_row`), use parameterized bindings via `diesel::sql_query(...).bind::<Text, _>(...)` instead of string interpolation.

### Type Safety Gaps in Web Adapter
- Issue: 123 `payload as { ... }` type assertions in `apps/frontend/src/adapters/web/core.ts` with no runtime validation. Any payload shape mismatch silently produces incorrect HTTP requests. The `invoke<T>()` function also casts responses with no validation (`JSON.parse(text) as T`).
- Files: `apps/frontend/src/adapters/web/core.ts`
- Impact: Silent type mismatches between frontend and backend cause hard-to-debug errors at runtime. No TypeScript compiler protection for the actual HTTP request payloads.
- Fix approach: Define Zod schemas for command payloads (the project already uses Zod). Validate at the adapter boundary. At minimum, add runtime type checks for critical commands.

### Deprecated `ActivityLegacy` Interface Still Present
- Issue: `ActivityLegacy` interface at line 70 of `types.ts` is marked `@deprecated` but remains in the codebase. The `Activity` interface (line 90) is the replacement. The deprecated type still exists alongside the new one.
- Files: `apps/frontend/src/lib/types.ts` (line 70)
- Impact: Confusion about which type to use. Risk of new code accidentally using the legacy type.
- Fix approach: Audit all usages of `ActivityLegacy` and migrate to `Activity`. Remove the deprecated interface.

### 14 Untyped `Record<string, unknown>` in types.ts
- Issue: The types file contains 14 instances of `Record<string, unknown>` for fields like `metadata`, `providerConfig`, `payload`, `checkpointJson`, etc. These are effectively untyped objects that could contain anything.
- Files: `apps/frontend/src/lib/types.ts`
- Impact: No IDE autocompletion or type checking for these fields. Consumers must cast or use type assertions. Bugs from incorrect property access are not caught at compile time.
- Fix approach: Define proper interfaces for each metadata shape. Start with the most commonly accessed ones (e.g., `Asset.metadata`, `Asset.providerConfig`, `FixAction.payload`).

## Performance Concerns

### No Evidence of Pagination in Activity Queries
- Issue: The `activities_service.rs` (4,279 lines) handles activity search and import. While the frontend sends pagination params, the service's query construction should be verified for proper LIMIT/OFFSET usage and index utilization. The repository file (`crates/storage-sqlite/src/activities/repository.rs`) is 2,020 lines.
- Files: `crates/core/src/activities/activities_service.rs`, `crates/storage-sqlite/src/activities/repository.rs`
- Impact: Large activity datasets could cause slow queries or memory issues if pagination is not properly enforced at the SQL level.
- Fix approach: Verify all activity queries use `LIMIT`/`OFFSET` with proper WHERE clause indexing. Add query plan analysis for common search patterns.

### Quote Service Complexity
- Issue: `crates/core/src/quotes/service.rs` (2,740 lines) handles quote CRUD, provider management, sync, and import — all in one service. Quote sync in particular (`crates/core/src/quotes/sync.rs`, 2,446 lines) orchestrates multi-provider data fetching.
- Files: `crates/core/src/quotes/service.rs`, `crates/core/src/quotes/sync.rs`
- Impact: Quote sync is likely the most I/O-intensive operation in the app. Any inefficiency in the sync orchestration (sequential vs parallel fetching, retry logic, batch size) directly impacts startup and refresh times.
- Fix approach: Profile quote sync with realistic data. Ensure parallel provider fetching where possible. Consider batching quote writes to reduce SQLite transaction overhead.

### Large Frontend Components with Heavy State
- Issue: Several React components exceed 1,000 lines with many `useState` hooks: `provider-settings-card.tsx` (1,476 lines, 8 `useState`), `device-sync-section.tsx` (1,441 lines, 16 `useState`), `source-config-panel.tsx` (1,370 lines). These monolithic components re-render on every state change.
- Files: `apps/frontend/src/features/ai-assistant/components/provider-settings-card.tsx`, `apps/frontend/src/features/devices-sync/components/device-sync-section.tsx`, `apps/frontend/src/pages/settings/market-data/source-config-panel.tsx`
- Impact: Excessive re-renders in complex forms. State management is difficult to follow. Components are hard to test and maintain.
- Fix approach: Extract sub-components with their own state. Use `useReducer` for complex state machines (especially device-sync which has an explicit state machine). Move form state to `react-hook-form` (already in the project) instead of individual `useState` calls.

## Security Concerns

### Dynamic SQL Construction in Device Sync
- Issue: The sync repository constructs SQL dynamically via `format!()` for operations like upserting JSON rows, applying taxonomy events, and exporting/importing snapshots. While `escape_sqlite_str()` and `quote_identifier()` provide basic sanitization, this pattern is inherently more risky than parameterized queries.
- Files: `crates/storage-sqlite/src/sync/app_sync/repository.rs` (lines 435-636, 672+)
- Risk: SQL injection if any escaping path is missed or if a new code path forgets to use the escape functions.
- Current mitigation: `escape_sqlite_str()` handles single-quote doubling, `quote_identifier()` handles backtick wrapping, `validate_sync_table()` restricts tables to a known allowlist.
- Recommendations: Migrate `upsert_json_row()` to use Diesel's parameterized query API. Add fuzz testing for the SQL construction paths. Consider using `diesel::insert_into(...).on_conflict(...).values(...)` instead of raw SQL.

### Secrets Management in Web Mode
- Issue: The web adapter (`apps/frontend/src/adapters/web/core.ts`) sends API keys and secrets via JSON body to the server's `/api/v1/secrets` endpoint. The server stores them (presumably in SQLite). In desktop mode, secrets go through OS keyring via Tauri.
- Files: `apps/frontend/src/adapters/web/core.ts` (lines 770-787), `crates/core/src/secrets/`
- Risk: Web mode stores secrets server-side. If the server is accessed over an insecure network, secrets could be intercepted. The `.env.web` file exists (not read, but present).
- Current mitigation: Credentials use `same-origin` fetch policy. HTTPS should be enforced in production.
- Recommendations: Verify that web server enforces HTTPS. Ensure secret encryption at rest. Document the security model difference between desktop and web modes.

### E2EE Key Material Handling
- Issue: Device sync uses E2EE with key envelopes, pairing challenges, and recovery envelopes. The frontend handles these cryptographic operations and passes key material through the adapter layer.
- Files: `apps/frontend/src/features/devices-sync/`, `crates/device-sync/`
- Risk: Any leak of key material in logs, error messages, or state persistence could compromise encrypted sync data.
- Current mitigation: AGENTS.md explicitly states "Never log secrets or financial data."
- Recommendations: Audit all logging paths in `crates/device-sync/` for accidental key material logging. Ensure React state containing key material is cleared after use.

## Maintainability Concerns

### Low Frontend Test Coverage
- Issue: 579 TypeScript/TSX source files vs 42 test files (~7% file coverage). Testing is concentrated in activity forms (7 test files in `__tests__/`), with many feature modules having zero tests: holdings page, settings pages, AI assistant hooks, market data pages, net worth, alternative assets.
- Files: `apps/frontend/src/` (579 source vs 42 test files)
- Risk: Frontend refactoring and feature changes have low safety net. UI regressions can go undetected.
- Priority: High — new features are being added without corresponding test coverage.

### Rust Test Co-location Creates Large Files
- Issue: Test files are co-located in the same source files as implementation, inflating file sizes: `snapshot_service.rs` tests = 5,314 lines, `activities_service_tests.rs` = 5,147 lines, `holdings_calculator_tests.rs` = 5,136 lines. The actual `activities_service.rs` is 4,279 lines of implementation.
- Files: `crates/core/src/portfolio/snapshot/snapshot_service_tests.rs`, `crates/core/src/activities/activities_service_tests.rs`, `crates/core/src/portfolio/snapshot/holdings_calculator_tests.rs`
- Impact: Very large files make code review harder. Test and implementation in the same PR review diff increases cognitive load.
- Fix approach: Consider moving test files to a `tests/` directory at the crate level. This is a style preference and not urgent — the existing approach is valid Rust convention.

### Web Core Must Track Every Backend Command
- Issue: Every new backend command (Tauri IPC or Axum HTTP endpoint) requires a corresponding entry in the web adapter's `COMMANDS` map AND a `case` in the `switch` statement. There are currently 100+ command entries. Missing either the map entry or the case handler causes a runtime error.
- Files: `apps/frontend/src/adapters/web/core.ts`
- Impact: High maintenance burden. Easy to forget adding both entries when creating a new backend command.
- Fix approach: Generate the COMMANDS map and request construction from a single source of truth (e.g., a shared API definition file or OpenAPI spec).

### COMMANDS Map and Server Routes Must Stay Synchronized
- Issue: The `COMMANDS` map in `apps/frontend/src/adapters/web/core.ts` maps command names to `{ method, path }`. The Axum server in `apps/server/src/api/` must define matching routes. There is no compile-time or build-time verification that these match.
- Files: `apps/frontend/src/adapters/web/core.ts`, `apps/server/src/api/`
- Impact: Route mismatches between frontend and server cause 404s at runtime. Method mismatches (GET vs POST) cause silent failures.
- Fix approach: Share route definitions between frontend and server, or generate them from a common spec. Add an integration test that verifies all COMMANDS entries resolve to valid server endpoints.

## Dependency Risks

### Supabase Client for Web Auth
- Issue: `@supabase/supabase-js` (v2.95.3) is a dependency in `apps/frontend/package.json`. This is likely used for web authentication mode. Supabase is a full backend-as-a-service platform — using only the auth portion adds significant bundle weight.
- Files: `apps/frontend/package.json`
- Impact: Increased bundle size. Supabase client brings its own realtime engine and postgrest client. Version pinning to `^2.x` allows breaking changes in minor versions.
- Fix approach: Evaluate if a lighter auth library could replace Supabase for the web mode use case. Pin to exact version if keeping it.

### `lodash` for Utility Functions
- Issue: `lodash` (v4.17.23) is listed as a dependency. Modern JavaScript/TypeScript has native equivalents for most lodash functions. Tree-shaking may not fully eliminate unused lodash code.
- Files: `apps/frontend/package.json`
- Impact: Potential bundle size bloat from unused lodash utilities.
- Fix approach: Audit which lodash functions are actually used and replace with native alternatives or smaller focused packages (e.g., `just-throttle`, `just-debounce-it`).

### Multiple Tauri Plugin Dependencies
- Issue: The frontend depends on 10+ `@tauri-apps/plugin-*` packages (dialog, fs, haptics, log, shell, updater, window-state, barcode-scanner, mobile-share, web-auth). Each plugin has its own version constraints and update cadence.
- Files: `apps/frontend/package.json`
- Impact: Version compatibility matrix between Tauri core and plugins must be maintained. Plugin updates may introduce breaking changes.
- Fix approach: Regularly audit and update all Tauri plugins together. Consider pinning to exact versions for stability.

## Scaling Concerns

### SQLite Single-Writer Limitation
- Issue: The app uses SQLite via Diesel with a connection pool (`r2d2`). SQLite supports only one concurrent writer. The sync repository, quote sync, and activity import all perform write operations that could contend.
- Files: `crates/storage-sqlite/src/`
- Impact: As data volume grows (more activities, more sync operations), write contention could cause perceived latency. The `WriteHandle` abstraction in `crates/storage-sqlite/src/db.rs` suggests write serialization is already in place.
- Fix approach: Ensure all write paths go through the write handle serialization. Consider WAL mode optimization. For extreme scale, evaluate if certain bulk operations can be batched into single transactions.

### Snapshot Export/Restore Performance
- Issue: The sync repository handles snapshot export/import by iterating over all sync tables, applying filters, and copying data. For users with large datasets (thousands of activities, years of quotes), this could be slow.
- Files: `crates/storage-sqlite/src/sync/app_sync/repository.rs` (lines 372-636)
- Impact: Device sync pairing and bootstrap could take prohibitively long for users with large portfolios.
- Fix approach: Add progress reporting for snapshot operations. Consider incremental sync instead of full snapshot transfer. Benchmark with realistic data sizes (10K+ activities).

## Test Coverage Gaps

### Untested Frontend Feature Modules
- What's not tested: Holdings page, net worth page, alternative assets pages, market data settings, AI assistant core hooks (`use-chat-runtime.ts` at 995 lines untested), device sync service layer, account page, goal management.
- Files: `apps/frontend/src/pages/holdings/`, `apps/frontend/src/pages/fire-planner/`, `apps/frontend/src/features/ai-assistant/hooks/`, `apps/frontend/src/features/devices-sync/services/`
- Risk: UI regressions and state management bugs go undetected.
- Priority: High — AI assistant and device sync are complex features with low test coverage.

### No E2E Test Visibility
- Issue: The project has `playwright.config.ts` and `e2e/` directory but the actual E2E test coverage is unknown. E2E tests for the dual-platform architecture (desktop + web) would be particularly valuable.
- Files: `playwright.config.ts`, `e2e/`
- Risk: Integration between frontend adapters and backend may break without detection.
- Priority: Medium — E2E tests are expensive to maintain but critical for the adapter layer.

---

*Concerns audit: 2026-04-20*
