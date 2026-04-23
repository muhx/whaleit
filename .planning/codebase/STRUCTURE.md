# Codebase Structure

**Analysis Date:** 2026-04-20

## Directory Layout

```
whaleit/                              # repo root (Whaleit)
├── apps/
│   ├── frontend/                     # React + Vite SPA (pnpm workspace)
│   │   ├── index.html                # Vite entry HTML
│   │   ├── vite.config.ts            # Vite config, adapter alias switching
│   │   ├── package.json
│   │   ├── public/                   # Static assets served as-is
│   │   └── src/
│   │       ├── main.tsx              # React root + addon bootstrap
│   │       ├── App.tsx               # Providers + auth gate
│   │       ├── routes.tsx            # React Router routes
│   │       ├── globals.css           # Tailwind + global styles
│   │       ├── lockdown.ts           # Desktop lockdown (SES) install
│   │       ├── use-global-event-listener.ts  # Global event bridge → React Query
│   │       ├── vite-env.d.ts
│   │       ├── adapters/             # Backend adapters (Tauri | Web)
│   │       │   ├── shared/           # Domain wrappers (identical for both)
│   │       │   ├── tauri/            # Tauri IPC implementation
│   │       │   ├── web/              # REST + SSE implementation
│   │       │   ├── index.ts          # Default re-export (for TS type-check)
│   │       │   └── types.ts          # Cross-adapter types
│   │       ├── addons/               # Addon host + loader
│   │       │   ├── addons-core.ts
│   │       │   ├── addons-loader.ts
│   │       │   ├── addons-dev-mode.ts
│   │       │   ├── addons-runtime-context.ts
│   │       │   └── type-bridge.ts
│   │       ├── assets/               # Image/font assets
│   │       ├── components/           # Reusable UI components
│   │       │   ├── classification/
│   │       │   └── page/
│   │       ├── context/              # React contexts (auth, privacy, sync)
│   │       ├── features/             # Feature modules
│   │       │   ├── ai-assistant/
│   │       │   ├── devices-sync/
│   │       │   └── whaleit-connect/
│   │       ├── hooks/                # Reusable React hooks
│   │       ├── lib/                  # Utilities, schemas, types, constants
│   │       │   └── types/            # Ambient TS types
│   │       ├── pages/                # Route-level pages
│   │       │   ├── account/
│   │       │   ├── activity/
│   │       │   ├── ai-assistant/
│   │       │   ├── asset/
│   │       │   ├── auth/
│   │       │   ├── dashboard/
│   │       │   ├── fire-planner/
│   │       │   ├── health/
│   │       │   ├── holdings/
│   │       │   ├── income/
│   │       │   ├── insights/
│   │       │   ├── layouts/
│   │       │   ├── net-worth/
│   │       │   ├── onboarding/
│   │       │   ├── performance/
│   │       │   ├── settings/
│   │       │   └── not-found.tsx
│   │       ├── test/                 # Vitest setup (setup.ts)
│   │       └── types/                # Global .d.ts
│   │
│   ├── tauri/                        # Tauri v2 desktop + mobile host (Rust)
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   ├── build.rs
│   │   ├── Info.plist / Info.ios.plist / Entitlements.plist
│   │   ├── capabilities/             # Tauri capability JSON per platform
│   │   ├── gen/apple/                # Generated Xcode project for iOS
│   │   ├── icons/                    # App icons (desktop + iOS)
│   │   ├── scripts/                  # Build helper scripts
│   │   └── src/
│   │       ├── main.rs               # Binary entry → run()
│   │       ├── lib.rs                # run() + invoke_handler! registration
│   │       ├── commands/             # #[tauri::command] per domain
│   │       ├── context/              # ServiceContext + initialize_context
│   │       ├── domain_events/        # Sink + debounced queue worker + planner
│   │       ├── services/             # Host-only services (ConnectService)
│   │       ├── events.rs             # Event-name constants + emit helpers
│   │       ├── listeners.rs          # Global AppHandle listeners
│   │       ├── scheduler.rs          # Broker + startup sync schedulers
│   │       ├── secret_store.rs       # OS keyring secret store
│   │       ├── menu.rs               # Desktop application menu (desktop only)
│   │       └── updater.rs            # Tauri updater wiring (desktop only)
│   │
│   └── server/                       # Axum web server (Rust, web mode)
│       ├── Cargo.toml
│       ├── tests/                    # Integration tests (auth, health, static)
│       └── src/
│           ├── main.rs               # #[tokio::main] entry
│           ├── main_lib.rs           # AppState builder + init_tracing
│           ├── api.rs                # app_router + OpenAPI + middleware
│           ├── api/                  # One file per domain (handlers)
│           ├── domain_events/        # WebDomainEventSink + queue worker
│           ├── secrets/              # File-based encrypted SecretStore
│           ├── ai_environment.rs     # ServerAiEnvironment impl
│           ├── auth.rs               # JWT + Argon2 authentication
│           ├── config.rs             # Config::from_env
│           ├── error.rs              # ApiError / ApiResult
│           ├── events.rs             # EventBus + ServerEvent
│           ├── features.rs           # Runtime feature helpers
│           ├── models.rs             # API DTOs (Account, NewAccount, …)
│           ├── scheduler.rs          # Broker sync scheduler
│           └── api.rs                # Router composition
│
├── crates/                           # Rust library crates (Cargo workspace)
│   ├── ai/                           # whaleit-ai (LLM orchestration)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── chat.rs, env.rs, error.rs, types.rs
│   │       ├── providers.rs, provider_model.rs, provider_service.rs
│   │       ├── prompt_template.rs, prompt_template_service.rs
│   │       ├── stream_hook.rs, title_generator.rs
│   │       ├── system_prompt.txt
│   │       ├── ai_providers.json    # Compiled-in catalog
│   │       ├── tools/                # Tool implementations
│   │       └── eval/                 # Behavioral test harness
│   ├── connect/                      # whaleit-connect (cloud broker sync)
│   │   └── src/{lib.rs, client.rs, token_lifecycle.rs, broker/, broker_ingest/, platform/}
│   ├── core/                         # whaleit-core (domain logic, traits)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── constants.rs, errors.rs
│   │       ├── accounts/, activities/, addons/, assets/, custom_provider/
│   │       ├── events/               # DomainEvent + DomainEventSink
│   │       ├── fx/, goals/, health/, limits/
│   │       ├── portfolio/            # allocation, fire, holdings, income,
│   │       │                         # net_worth, performance, snapshot, valuation
│   │       ├── quotes/               # QuoteService, providers wrapper
│   │       ├── secrets/, settings/, sync/, taxonomies/, utils/
│   ├── device-sync/                  # whaleit-device-sync (E2EE)
│   │   └── src/{lib.rs, client.rs, crypto.rs, engine/, enroll_service.rs, types.rs, time.rs}
│   ├── market-data/                  # whaleit-market-data
│   │   └── src/{lib.rs, models/, provider/, registry/, resolver/, errors/}
│   └── storage-sqlite/               # whaleit-storage-sqlite (Diesel)
│       ├── Cargo.toml
│       ├── diesel.toml
│       ├── migrations/               # Diesel migrations (embedded via macro)
│       └── src/
│           ├── lib.rs
│           ├── db/                   # Pool + WriteHandle + migrations
│           ├── schema.rs             # Diesel schema
│           ├── utils.rs              # SQLite chunk helpers
│           ├── errors.rs
│           └── {accounts,activities,ai_chat,assets,custom_provider,
│                fx,goals,health,limits,market_data,portfolio,
│                settings,sync,taxonomies}/
│
├── packages/                         # Shared TS packages (pnpm workspace)
│   ├── addon-sdk/                    # Public addon SDK (@whaleit/addon-sdk)
│   │   ├── package.json
│   │   ├── tsup.config.ts
│   │   └── src/{index.ts, host-api.ts, manifest.ts, permissions.ts,
│   │              data-types.ts, goal-progress.ts, query-keys.ts,
│   │              types.ts, utils.ts, version.ts}
│   ├── ui/                           # Reusable components (@whaleit/ui)
│   │   ├── package.json
│   │   ├── tsup.config.ts
│   │   └── src/{index.ts, components/, hooks/, lib/, styles.css}
│   └── addon-dev-tools/              # Addon scaffolding CLI
│       ├── package.json, cli.js, dev-server.js, scaffold.js
│       └── templates/
│
├── addons/                           # Bundled first-party addons
│   ├── goal-progress-tracker/
│   ├── investment-fees-tracker/
│   └── swingfolio-addon/
│
├── e2e/                              # Playwright end-to-end tests
│   ├── playwright fixtures + helpers.ts
│   └── 01-*.spec.ts … 10-*.spec.ts
│
├── scripts/                          # Build / dev / e2e orchestration (Node)
│   ├── dev-web.mjs
│   ├── prep-e2e.mjs
│   ├── run-e2e.mjs
│   └── wait-for-both-servers-to-be-ready.sh
│
├── db/                               # SQL snapshots / seed data
├── docs/                             # Docs assets
├── assets/                           # Repo-level marketing assets
├── .github/ / .devcontainer/ / .claude/ / .cursor/ / .vscode/
│
├── Cargo.toml                        # Cargo workspace manifest
├── Cargo.lock
├── pnpm-workspace.yaml               # pnpm workspace manifest
├── pnpm-lock.yaml
├── package.json                      # Root pnpm scripts (dev, test, build)
├── tsconfig.base.json                # Shared TS compiler config
├── tsconfig.json / tsconfig.node.json / tsconfig.test.json
├── eslint.base.config.js / eslint.config.js
├── .prettierrc.cjs / .prettierignore
├── playwright.config.ts
├── Dockerfile / compose.yml / compose.dev.yml / .dockerignore
├── .env.example / .env.web.example   # (committed) env templates
├── .env.web                          # Runtime env (NOT read by mapper)
├── .node-version                     # Node version pin
├── AGENTS.md / CLA.md / CONTRIBUTING.md / LICENSE / README.md
├── ROADMAP.md / TRADEMARKS.md
└── app-icon.png
```

## Directory Purposes

**`apps/frontend/`:**

- Purpose: Only UI surface. React 19 SPA served by Vite.
- Contains: pages, features, adapters, addons, hooks, contexts, components.
- Key files: `src/main.tsx` (bootstrap), `src/App.tsx` (providers),
  `src/routes.tsx` (router), `vite.config.ts` (adapter alias via
  `BUILD_TARGET`), `package.json` (scripts: `dev` = web mode, `dev:tauri` =
  Tauri mode).

**`apps/frontend/src/adapters/`:**

- Purpose: isolate every backend call behind a typed function so UI code never
  knows whether it runs against Tauri IPC or HTTP.
- Contains: `shared/` (platform-neutral domain wrappers), `tauri/` (Tauri IPC
  impl + platform-specific helpers), `web/` (REST + SSE impl).
- Key files: `shared/*.ts`, `tauri/index.ts`, `tauri/core.ts`, `web/index.ts`,
  `web/core.ts` (contains the `COMMANDS` → REST map).
- Selection: `vite.config.ts` aliases `@/adapters` → `tauri/` or `web/` and
  `#platform` → `tauri/core` or `web/core` based on `BUILD_TARGET`.

**`apps/frontend/src/pages/`:**

- Purpose: route-level screens. One subdirectory per top-level route.
- Contains: one `*-page.tsx` per route plus co-located components and hooks
  specific to that page.
- Key files: `dashboard/portfolio-page.tsx`, `layouts/app-layout.tsx`,
  `layouts/onboarding-layout.tsx`, `settings/settings-layout.tsx`,
  `holdings/holdings-page.tsx`, `performance/performance-page.tsx`,
  `ai-assistant/ai-assistant-page.tsx`, `fire-planner/fire-planner-page.tsx`.

**`apps/frontend/src/features/`:**

- Purpose: cross-page feature modules with their own components/hooks/types.
- Contains: `ai-assistant/` (chat UI, types, streaming hooks),
  `whaleit-connect/` (broker cloud connect onboarding), `devices-sync/` (E2EE
  device sync UI, crypto helpers, pairing flow).

**`apps/frontend/src/components/`:**

- Purpose: reusable UI widgets not tied to one page.
- Notable files: `app-launcher.tsx`, `account-selector.tsx`,
  `benchmark-symbol-selector.tsx`, `symbol-selector-mobile.tsx`,
  `history-chart.tsx`, `performance-chart.tsx`, `theme-selector.tsx`,
  `update-dialog.tsx`, `ticker-search.tsx`, `mobile-actions-menu.tsx`.
- Sub-dirs: `classification/` (taxonomy pickers), `page/` (`swipable-page.tsx`,
  `swipable-routes-page.tsx`).

**`apps/frontend/src/hooks/`:**

- Purpose: reusable React hooks wrapping adapters + TanStack Query.
- Notable: `use-accounts.ts`, `use-holdings.ts`, `use-settings.ts`,
  `use-platform.ts`, `use-pull-to-refresh.ts`, `use-taxonomies.ts`,
  `use-health.ts`, `use-updater.ts`, `use-quote-import.ts`.

**`apps/frontend/src/lib/`:**

- Purpose: utilities, shared types, Zod schemas, query keys, constants.
- Key files: `types.ts` (all TS domain types), `schemas.ts` (Zod + React Hook
  Form), `utils.ts`, `constants.ts`, `query-keys.ts`, `activity-utils.ts`,
  `asset-utils.ts`, `portfolio-helper.ts`, `settings-provider.tsx`,
  `is-tauri.ts`.

**`apps/frontend/src/addons/`:**

- Purpose: runtime host for user-installable addons.
- Key files: `addons-loader.ts` (production + dev-server loading),
  `addons-core.ts`, `addons-runtime-context.ts` (AddonContext + dynamic routes),
  `addons-dev-mode.ts` (local dev servers), `type-bridge.ts` (wraps adapters
  into the SDK host API).

**`apps/tauri/`:**

- Purpose: Tauri v2 desktop + mobile shell.
- Key files: `src/main.rs` (binary entry), `src/lib.rs` (`run()` + full
  `invoke_handler!`), `tauri.conf.json` (bundle + plugins), `Cargo.toml`
  (feature flags `connect-sync`, `device-sync`, `appstore`), `build.rs` (Tauri
  build script).

**`apps/tauri/src/commands/`:**

- Purpose: one file per domain containing `#[tauri::command]` async functions.
- Notable: `account.rs`, `activity.rs`, `portfolio.rs` (large — 39KB),
  `market_data.rs`, `ai_chat.rs`, `addon.rs`, `brokers_sync.rs`, `device_sync/`
  (subdir with multiple files), `health.rs`, `fire.rs`, `taxonomy.rs`,
  `sync_crypto.rs`.
- Pattern: each command pulls `State<'_, Arc<ServiceContext>>`, calls one trait
  method, maps errors to `String`.

**`apps/tauri/src/context/`:**

- Purpose: DI container + builder for services.
- Files: `registry.rs` (`ServiceContext` with ~30 services), `providers.rs`
  (`initialize_context`), `ai_environment.rs` (`TauriAiEnvironment`), `mod.rs`
  (public exports).

**`apps/tauri/src/domain_events/`:**

- Purpose: bridge `DomainEvent`s from core services to Tauri emit + background
  work.
- Files: `sink.rs` (`TauriDomainEventSink` wrapping `tokio::mpsc`),
  `queue_worker.rs` (1s debounce + planners), `planner.rs` (decides
  portfolio/asset/broker actions), `mod.rs`.

**`apps/tauri/src/services/`:**

- Purpose: host-only services that don't belong to a crate.
- Files: `connect_service.rs` (Whaleit Connect cloud bridge +
  `cloud_api_base_url`), `mod.rs`.

**`apps/server/`:**

- Purpose: Axum HTTP server for web mode. Hosts same services as Tauri plus auth
  and static file serving.
- Key files: `src/main.rs`, `src/main_lib.rs` (`AppState`, `build_state`,
  `init_tracing`), `src/api.rs` (router composition + OpenAPI), `src/config.rs`,
  `src/auth.rs`, `tests/*.rs` (integration tests).

**`apps/server/src/api/`:**

- Purpose: HTTP handler modules — one per domain, mirroring
  `apps/tauri/src/commands/`.
- Notable: `accounts.rs`, `holdings/` (subdir `handlers.rs`, `dto.rs`,
  `mappers.rs`, `fixtures/`), `portfolio.rs`, `performance.rs`,
  `market_data.rs`, `ai_chat.rs`, `connect.rs` (very large — 48KB),
  `device_sync.rs` (29KB), `device_sync_engine.rs` (69KB), `taxonomies.rs`,
  `shared.rs` (portfolio job runner), `addons.rs`.

**`crates/core/`:**

- Purpose: domain entities, services, traits, domain events. Framework-free,
  Diesel-free (no direct Diesel types on the public surface).
- Contains one folder per aggregate: `accounts/`, `activities/`, `assets/`,
  `custom_provider/`, `fx/`, `goals/`, `health/`, `limits/`, `portfolio/` (with
  sub-aggregates), `quotes/`, `secrets/`, `settings/`, `sync/`, `taxonomies/`,
  `addons/`, plus cross-cutting `events/`, `errors.rs`, `constants.rs`,
  `utils/`.
- Pattern per aggregate: `mod.rs` + `*_model.rs` + `*_service.rs` +
  `*_traits.rs` + `*_constants.rs` (+ tests).

**`crates/storage-sqlite/`:**

- Purpose: the only place Diesel is used. Implements repository traits from
  `whaleit-core`.
- Contains: `db/` (pool + write actor + migrations harness), `schema.rs`, one
  folder per entity with `model.rs` + `repository.rs`, `migrations/` (dated
  directories, embedded at build time).
- Migration naming: `YYYY-MM-DD-HHMMSS_description/` each containing `up.sql` +
  `down.sql`.

**`crates/market-data/`:**

- Purpose: provider-agnostic quote / symbol fetching.
- Sub-dirs: `models/` (domain types), `provider/` (Yahoo, AlphaVantage, Finnhub,
  BoerseFrankfurt, MarketDataApp, MetalPriceApi, OpenFigi, UsTreasuryCalc,
  CustomScraper), `registry/` (rate-limit, circuit-breaker, validator),
  `resolver/` (instrument resolution chain + MIC/exchange map), `errors/`.

**`crates/connect/`, `crates/device-sync/`, `crates/ai/`:**

- Isolated capabilities behind Cargo features. See ARCHITECTURE.md for module
  roles.

**`packages/`:**

- Purpose: TypeScript libraries published to npm or consumed by workspace.
- `addon-sdk/` — public addon SDK. `ui/` — reusable components.
  `addon-dev-tools/` — CLI for scaffolding new addons.
- Pattern: each package has `package.json`, `tsconfig.json`, `tsup.config.ts`,
  `src/index.ts`, README.

**`addons/`:**

- Purpose: first-party addons shipped with the app. Each has its own `src/`,
  `manifest.json`, `vite.config.ts`, `package.json`.

**`e2e/`:**

- Purpose: Playwright specs that run against the web build via
  `scripts/run-e2e.mjs`.
- Files: numbered `NN-description.spec.ts` + shared `helpers.ts`.

**`scripts/`:**

- Node/Bash helpers invoked from root `package.json` (dev-web, e2e
  orchestration).

**`db/`:**

- Purpose: SQL snapshots / sample databases used by docs and dev setup.

## Key File Locations

**Entry Points:**

- `apps/frontend/src/main.tsx` — React root + addon bootstrap.
- `apps/frontend/src/App.tsx` — Provider tree + `AuthGate`.
- `apps/frontend/src/routes.tsx` — route definitions.
- `apps/tauri/src/main.rs` — Tauri binary entry.
- `apps/tauri/src/lib.rs` — `run()` + `invoke_handler!` (registers ~170
  commands).
- `apps/server/src/main.rs` — Axum binary entry.
- `apps/server/src/main_lib.rs` — `AppState` / `build_state`.
- `apps/server/src/api.rs` — router composition.

**Configuration:**

- `package.json` (root) — pnpm scripts for dev/build/test/e2e.
- `pnpm-workspace.yaml` — `apps/frontend`, `packages/*`, `addons/*`.
- `Cargo.toml` (root) — workspace members `apps/tauri`, `apps/server`,
  `crates/*`.
- `apps/frontend/vite.config.ts` — Vite + BUILD_TARGET alias switching.
- `apps/frontend/package.json` — frontend deps + scripts.
- `apps/tauri/tauri.conf.json` — bundle/plugins/deep-link/updater.
- `apps/tauri/capabilities/*.json` — Tauri capability sets per platform.
- `apps/server/src/config.rs` — env parsing for server mode.
- `.env.example`, `.env.web.example` — committed env templates.
- `tsconfig.base.json`, `tsconfig.json`, `tsconfig.node.json`,
  `tsconfig.test.json` — TS configs.
- `eslint.base.config.js`, `eslint.config.js` — ESLint configs.
- `.prettierrc.cjs` — Prettier config.
- `playwright.config.ts` — E2E config.
- `crates/storage-sqlite/diesel.toml` — Diesel CLI config.

**Core Logic:**

- `crates/core/src/lib.rs` — public module roster.
- `crates/core/src/events/domain_event.rs` — domain events.
- `crates/core/src/accounts/accounts_service.rs` + `accounts_traits.rs` —
  reference pattern for every aggregate.
- `crates/core/src/portfolio/` — all portfolio math (snapshot, valuation,
  performance, holdings, allocation, net_worth, income, fire).
- `crates/core/src/quotes/service.rs` — quote orchestration (large: 96KB).
- `crates/core/src/activities/activities_service.rs` — activity CRUD + CSV
  import (large: 173KB).
- `crates/storage-sqlite/src/db/mod.rs` — pool + migrations bootstrapping.
- `crates/storage-sqlite/src/db/write_actor.rs` — serialized writer handle.
- `crates/storage-sqlite/src/schema.rs` — Diesel schema definition.

**Testing:**

- `apps/frontend/src/test/setup.ts` — Vitest setup.
- `apps/frontend/src/**/*.{test,spec}.{ts,tsx}` — frontend unit tests
  (co-located).
- `crates/*/src/**/*_tests.rs` + `#[cfg(test)] mod tests` — Rust unit tests.
- `crates/ai/src/eval/` — AI behavioral eval harness (test-only).
- `apps/server/tests/*.rs` — Axum integration tests (`auth.rs`, `health.rs`,
  `static_routes.rs`).
- `e2e/*.spec.ts` — Playwright end-to-end suites.
- `scripts/run-e2e.mjs` — orchestrator used by `pnpm test:e2e`.

**Adapters / IPC:**

- `apps/frontend/src/adapters/tauri/core.ts` — `invoke` + Tauri logger.
- `apps/frontend/src/adapters/web/core.ts` — `COMMANDS` map (command name → HTTP
  method + path) + fetch-based `invoke` + base64 helpers.
- `apps/frontend/src/adapters/shared/platform.ts` — re-exports
  `invoke`/`logger`/platform flags from `#platform` alias.
- `apps/frontend/src/adapters/tauri/events.ts` — `listen` wrappers with
  race-safe `adaptUnlisten`.
- `apps/frontend/src/adapters/web/events.ts` — `ServerEventBridge` over
  `EventSource`.

**Events & background work:**

- `apps/tauri/src/events.rs` — Tauri event-name constants + helpers.
- `apps/server/src/events.rs` — `EventBus` + `ServerEvent`.
- `apps/tauri/src/domain_events/queue_worker.rs`,
  `apps/server/src/domain_events/queue_worker.rs` — debounced workers.

## Naming Conventions

**Files (TypeScript):**

- Kebab-case for all source files: `account-selector.tsx`, `use-accounts.ts`,
  `activity-utils.ts`.
- Pages suffixed with `-page.tsx`: `portfolio-page.tsx`, `holdings-page.tsx`.
- Hooks prefixed with `use-`: `use-holdings.ts`, `use-platform.ts`,
  `use-settings-mutation.ts`.
- Tests next to source with `.test.ts`/`.test.tsx`/`.spec.ts`:
  `activity-utils.test.ts`, `type-bridge.test.ts`.
- Context providers: `*-context.tsx` (e.g. `auth-context.tsx`,
  `privacy-context.tsx`).

**Files (Rust):**

- `snake_case.rs` throughout. One concept per file.
- Trait modules suffixed `_traits.rs`, models `_model.rs`, services
  `_service.rs`, constants `_constants.rs`, tests `_tests.rs`.
- Module directories expose `mod.rs`. Re-exports happen in `mod.rs`.

**Directories:**

- `kebab-case` for TS packages and addons (`addon-sdk`,
  `goal-progress-tracker`).
- `snake_case` for Rust modules (`domain_events`, `broker_ingest`,
  `storage-sqlite` is the only kebab-case Rust crate name, mapped from the Cargo
  package name).
- `crates/*`, `apps/*`, `packages/*` — flat one level deep.

**Commands / routes:**

- Rust command names: `snake_case` (`get_accounts`, `create_account`,
  `sync_market_data`, `update_tool_result`).
- Frontend adapter function names: `camelCase` (`getAccounts`, `createAccount`,
  `updateToolResult`). The translation happens inside each adapter wrapper.
- REST paths: `/api/v1/<resource>` kebab-case with REST verbs (see
  `apps/frontend/src/adapters/web/core.ts` `COMMANDS` map).
- Event names: `kebab-case` with `domain:action` format
  (`portfolio:update-start`, `market:sync-complete`, `broker:sync-error`,
  `asset:enrichment-progress`, `app:ready`, `deep-link-received`).

**TypeScript types:**

- PascalCase for types/interfaces (`Account`, `NewAccount`, `AiStreamEvent`).
- Domain types defined in `apps/frontend/src/lib/types.ts`.

**Rust types:**

- PascalCase for structs/enums/traits (`ServiceContext`, `DomainEvent`,
  `AccountServiceTrait`).
- Traits generally suffixed `Trait` (e.g. `AccountServiceTrait`,
  `BrokerSyncServiceTrait`).

**Env vars:**

- `UPPER_SNAKE_CASE`, typically prefixed with `WF_` (`WF_DB_PATH`,
  `WF_SECRET_FILE`, `WF_LOG_FORMAT`, `WF_API_TARGET`, `WF_ENABLE_VITE_PROXY`) or
  `VITE_`/`TAURI_`/`CONNECT_` (honored by `envPrefix` in `vite.config.ts`).

**Migrations:**

- `YYYY-MM-DD-HHMMSS_snake_case_description/` with `up.sql` + `down.sql` (Diesel
  convention).

## Where to Add New Code

**New frontend domain call (CRUD on a resource):**

1. Add the Tauri command in `apps/tauri/src/commands/<domain>.rs` and register
   it in the `generate_handler!` list in `apps/tauri/src/lib.rs`.
2. Add the Axum handler in `apps/server/src/api/<domain>.rs` and mount its
   `router()` inside `apps/server/src/api.rs::app_router`.
3. Add a REST mapping for the command in
   `apps/frontend/src/adapters/web/core.ts` `COMMANDS` map + any body/query
   shaping below.
4. Add/extend the typed adapter function in
   `apps/frontend/src/adapters/shared/<domain>.ts` (platform-neutral) or
   `tauri/` + `web/` if the shapes diverge; re-export from the matching
   `index.ts`.
5. Add a TanStack Query hook in `apps/frontend/src/hooks/use-<domain>.ts`.
6. Add UI in `apps/frontend/src/pages/<area>/` or
   `apps/frontend/src/features/<area>/`.

**New domain entity / aggregate (Rust):**

1. Add a module folder `crates/core/src/<aggregate>/` with `mod.rs` +
   `<aggregate>_model.rs` + `<aggregate>_service.rs` + `<aggregate>_traits.rs`;
   re-export publicly from `mod.rs` and register in `crates/core/src/lib.rs`.
2. Add storage impl
   `crates/storage-sqlite/src/<aggregate>/{model.rs,repository.rs,mod.rs}`; add
   Diesel schema entries to `crates/storage-sqlite/src/schema.rs`.
3. Create a Diesel migration under
   `crates/storage-sqlite/migrations/YYYY-MM-DD-HHMMSS_*/` with `up.sql` +
   `down.sql`.
4. Emit any mutations as `DomainEvent`s via the injected
   `Arc<dyn DomainEventSink>` (see
   `crates/core/src/accounts/accounts_service.rs` for pattern).
5. Wire the service in `apps/tauri/src/context/providers.rs` and
   `apps/server/src/main_lib.rs::build_state`.
6. Add commands + handlers as above.

**New React page/route:**

1. Add `apps/frontend/src/pages/<area>/<area>-page.tsx`.
2. Register the route in `apps/frontend/src/routes.tsx`.
3. Co-locate components/hooks/utilities inside
   `apps/frontend/src/pages/<area>/`.
4. Update navigation in `apps/frontend/src/pages/layouts/navigation/`.

**New reusable component:**

- If used in ≥2 places inside the frontend → `apps/frontend/src/components/`.
- If designed to be consumed by addons too → `packages/ui/src/components/` and
  export from `packages/ui/src/index.ts`.

**New React hook:**

- Cross-feature → `apps/frontend/src/hooks/use-*.ts`.
- Feature-specific → `apps/frontend/src/features/<feature>/hooks/`.

**New addon:**

- Use `pnpm addon:create` (runs `packages/addon-dev-tools/cli.js`). Output goes
  under `addons/<name>/` with `manifest.json` + `src/`.

**New market-data provider:**

- `crates/market-data/src/provider/<provider>.rs`; register with
  `ProviderRegistry`.

**New AI tool:**

- `crates/ai/src/tools/<tool>.rs`; register in the `ToolSet` builder; ensure
  both `TauriAiEnvironment` and `ServerAiEnvironment` expose the needed
  services.

**New integration test:**

- Axum: `apps/server/tests/<name>.rs` (one `#[tokio::test]` per case).
- Frontend unit: `apps/frontend/src/**/*.test.ts(x)` next to source.
- E2E: `e2e/NN-<name>.spec.ts`.

**Shared utilities:**

- TS helpers (frontend-only) → `apps/frontend/src/lib/utils.ts` (growing; be
  selective).
- TS helpers usable by addons → `packages/addon-sdk/src/utils.ts`.
- Rust helpers for core → `crates/core/src/utils/`.
- Rust helpers for storage → `crates/storage-sqlite/src/utils.rs`.

## Special Directories

**`apps/tauri/gen/apple/`:**

- Purpose: generated Xcode project for iOS (created by `tauri ios init`).
- Generated: Yes (by Tauri CLI).
- Committed: Yes (required for iOS builds).

**`crates/storage-sqlite/migrations/`:**

- Purpose: Diesel SQL migrations; embedded at compile time via
  `embed_migrations!` macro (`crates/storage-sqlite/src/db/mod.rs:21`).
- Generated: No (authored by hand).
- Committed: Yes. Never edit a migration after it has been released.

**`apps/frontend/public/`:**

- Purpose: static assets copied verbatim to the final bundle (manifest.json,
  icons, logo, sound files).
- Generated: No.
- Committed: Yes.

**`apps/tauri/icons/`:**

- Purpose: app icons consumed by `tauri.conf.json`. Includes macOS `.icns`,
  Windows `.ico`, iOS AppIcon sets.
- Committed: Yes.

**`target/` (root):**

- Purpose: Cargo build artefacts for the entire workspace.
- Generated: Yes.
- Committed: No (ignored).

**`node_modules/` (root + packages):**

- Purpose: pnpm-installed deps. Single root store with per-package symlinks.
- Committed: No.

**`dist/` (root, unchecked but referenced):**

- Purpose: Vite output from frontend build (`outDir: "../../dist"` in
  `vite.config.ts`). Consumed by `tauri.conf.json`'s
  `frontendDist: "../../dist"` and by Axum's static file service in web mode.
- Generated: Yes.
- Committed: No.

**`.planning/`:**

- Purpose: agent-authored planning docs (codebase map, phase plans). This
  document lives here.
- Generated: Yes (by agents).
- Committed: optional.

**`.claude/`, `.cursor/`, `.vscode/`, `.devcontainer/`, `.github/`:**

- Editor / CI / agent configuration. Committed.

**`e2e/`:**

- Playwright specs. Run against the built web mode via `scripts/run-e2e.mjs`.

**`db/`:**

- SQL fixtures for docs / support workflows. Not used at runtime.

**`.env` files:**

- `.env.example`, `.env.web.example` — committed templates.
- `.env`, `.env.web` — runtime env (existence only; contents must not be read;
  may contain secrets).

---

_Structure analysis: 2026-04-20_
