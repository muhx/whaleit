# Architecture

**Analysis Date:** 2026-04-20

## Pattern Overview

**Overall:** Layered, hexagonal-style architecture inside a pnpm + Cargo
monorepo. The React frontend is a single SPA that talks to two interchangeable
Rust backends (Tauri desktop IPC in `apps/tauri` or an Axum HTTP server in
`apps/server`) through a platform adapter layer. Business logic lives in
framework-free Rust crates under `crates/*`. Storage is SQLite via Diesel.

**Key Characteristics:**

- Dual-runtime: identical domain code drives both Tauri (desktop/mobile) and
  Axum (web server). Runtime-specific glue lives in `apps/tauri/src/` and
  `apps/server/src/`.
- Trait-based ports: `crates/core` defines service + repository traits;
  `crates/storage-sqlite` provides the only Diesel-bound implementation (see
  `crates/storage-sqlite/src/lib.rs:12-25` ASCII diagram).
- Command parity: every Tauri command registered in
  `apps/tauri/src/lib.rs:276-610` has a matching REST route in
  `apps/server/src/api/*` and a shared frontend wrapper in
  `apps/frontend/src/adapters/shared/*`.
- Event-driven recalculation: domain mutations emit `DomainEvent`s through an
  `Arc<dyn DomainEventSink>`; runtime sinks fan out portfolio/asset/broker work
  via a debounced queue worker (`apps/tauri/src/domain_events/queue_worker.rs`,
  `apps/server/src/domain_events/queue_worker.rs`).
- Local-first: SQLite database and encrypted secret store live on-device; cloud
  sync (`crates/connect`, `crates/device-sync`) is opt-in via Cargo features
  `connect-sync` / `device-sync`.
- Extensibility: a sandboxed addon system loads third-party UI/logic at runtime
  via `packages/addon-sdk` and `apps/frontend/src/addons/`.

## Layers

**Frontend (React + Vite SPA):**

- Purpose: UI, routing, React Query caches, addon host, streaming UX.
- Location: `apps/frontend/src/`
- Contains: pages (`pages/`), feature modules (`features/`), reusable components
  (`components/`), hooks (`hooks/`), contexts (`context/`), platform adapters
  (`adapters/`), addon runtime (`addons/`).
- Depends on: `@whaleit/ui`, `@whaleit/addon-sdk`, Tauri JS APIs
  (desktop build) or `fetch`/SSE (web build).
- Used by: end user; is the only UI surface.

**Platform Adapter Layer (frontend):**

- Purpose: isolate every backend call behind a typed function so UI code never
  knows whether it runs against Tauri IPC or HTTP.
- Location: `apps/frontend/src/adapters/`
  - `shared/` — domain wrappers (`accounts.ts`, `portfolio.ts`, `connect.ts`,
    ...) that call `invoke` from `#platform`.
  - `tauri/` — `invoke` implemented with `@tauri-apps/api/core.invoke`; logger
    via `@tauri-apps/plugin-log`; event listeners via
    `@tauri-apps/api/event.listen`; sets `isDesktop = true`.
  - `web/` — `invoke` translates command names to REST via a `COMMANDS` map in
    `apps/frontend/src/adapters/web/core.ts:19-288`; events arrive over
    Server-Sent Events (`EVENTS_ENDPOINT = /api/v1/events/stream`); sets
    `isWeb = true`.
- Build-time selection: Vite alias `@/adapters` resolves to `tauri/` or `web/`
  based on `BUILD_TARGET` env var (see `apps/frontend/vite.config.ts:26-55`).
  The alias `#platform` is the hook shared modules use to import the active
  `invoke`/`logger`.
- Depends on: frontend `lib/types`, `lib/schemas`,
  `features/ai-assistant/types`.
- Used by: every hook, page, and feature under `apps/frontend/src/`.

**Tauri Runtime Host:**

- Purpose: desktop/mobile process hosting the Rust services, exposing them as
  IPC commands to the embedded webview.
- Location: `apps/tauri/src/`
- Entry: `apps/tauri/src/main.rs` calls `whaleit_app_lib::run()`
  (`apps/tauri/src/lib.rs:197-633`).
- Key pieces:
  - `context/` — builds `ServiceContext` (`context/registry.rs`,
    `context/providers.rs:59-375`) wiring repositories and services.
  - `commands/` — 1 file per domain (`account.rs`, `portfolio.rs`,
    `activity.rs`, `device_sync.rs`, ...); each `#[tauri::command]` async
    function receives `State<'_, Arc<ServiceContext>>` and delegates to a core
    service.
  - `domain_events/` — `TauriDomainEventSink` (`sink.rs`) pushes `DomainEvent`s
    onto an unbounded `tokio::mpsc`; `queue_worker.rs` debounces 1 second
    batches and calls planners in `planner.rs` to trigger portfolio/asset/broker
    actions and emit webview events.
  - `listeners.rs`, `events.rs` — global event constants
    (`portfolio:update-start`, `market:sync-start`, `broker:sync-*`,
    `asset:enrichment-*`, `app:ready`, `deep-link-received`) and listeners that
    spawn background jobs via `tauri::async_runtime::spawn`.
  - `services/connect_service.rs` — HTTP-based bridge to Whaleit Connect
    cloud used by broker + device sync commands.
  - `scheduler.rs` — startup sync + 4h broker sync scheduler; market-data sync
    scheduler delegated to
    `whaleit_core::quotes::scheduler::run_periodic_sync`.
  - `secret_store.rs` — OS keyring-backed `SecretStore` implementation shared
    across services.
- Depends on: `whaleit-core`, `whaleit-market-data`,
  `whaleit-connect`, `whaleit-storage-sqlite`,
  `whaleit-device-sync`, `whaleit-ai`.
- Used by: the frontend (through the Tauri adapter).

**Axum HTTP Server (web mode):**

- Purpose: headless server that serves the same React bundle plus a JSON REST
  API mirroring the Tauri command surface.
- Location: `apps/server/src/`
- Entry: `apps/server/src/main.rs:62-142` — reads `Config::from_env()`, builds
  `AppState` via `main_lib::build_state`, starts background schedulers, and
  serves the frontend static files from `config.static_dir` with
  `tower_http::services::ServeDir`.
- Key pieces:
  - `main_lib.rs:63-491` — `AppState` struct mirrors the Tauri `ServiceContext`
    but with fewer Tauri-only pieces and adds `EventBus`, optional
    `AuthManager`, `TokenLifecycleState`.
  - `api.rs:70-179` — `app_router(state, config)` composes per-domain `Router`s
    under `/api/v1` and layers CORS, request IDs, tracing, request timeout, and
    JWT middleware when `config.auth` is set.
  - `api/*.rs` — REST handlers (`accounts.rs`, `holdings/`, `portfolio.rs`,
    `ai_chat.rs`, `connect.rs`, `device_sync.rs`, ...). Handlers receive
    `State<Arc<AppState>>` and call the same trait methods the Tauri commands
    use.
  - `api/shared.rs` — shared portfolio job runner; publishes `ServerEvent`s to
    the `EventBus`; converted to SSE at `/api/v1/events/stream`.
  - `auth.rs` — Argon2 password hashing + JWT cookies; rate-limited login via
    `tower_governor`.
  - `domain_events/` — `WebDomainEventSink` (`sink.rs`) uses a two-phase init
    (`start_worker` pattern in `main_lib.rs:425-438`) and a queue worker in
    `queue_worker.rs`.
  - `secrets/` — file-based encrypted `SecretStore` (`build_secret_store`); keys
    are derived from env-provided secret, stored at `WF_SECRET_FILE` path.
  - `scheduler.rs` — background broker sync scheduler (conditional on
    `connect-sync` feature).
  - `ai_environment.rs` — Server implementation of the `AiEnvironment` trait
    from `whaleit-ai`.
- Depends on: identical set of crates as Tauri plus `axum`, `tower`,
  `tower-http`, `tower_governor`, `utoipa`.
- Used by: frontend built with `BUILD_TARGET=web`; supports single-user (auth
  disabled) and multi-user (JWT) modes.

**Core Domain (`crates/core`):**

- Purpose: database-agnostic domain logic, service orchestration, and domain
  event contracts.
- Location: `crates/core/src/`
- Contains modules (`lib.rs:7-25`): `accounts`, `activities`, `addons`,
  `assets`, `custom_provider`, `events`, `fx`, `goals`, `health`, `limits`,
  `portfolio` (submodules `allocation`, `fire`, `holdings`, `income`,
  `net_worth`, `performance`, `snapshot`, `valuation`), `quotes`, `secrets`,
  `settings`, `sync`, `taxonomies`, `utils`, plus shared `constants`, `errors`.
- Pattern per module: each module folder typically contains `*_model.rs` (domain
  entities), `*_service.rs` (use cases), `*_traits.rs` (service + repository
  traits), `*_constants.rs`, and `mod.rs` that re-exports the public surface.
  Example: `crates/core/src/accounts/mod.rs:1-13`.
- Depends on: `whaleit-market-data` (for quote providers), workspace
  primitives (`rust_decimal`, `chrono`, `uuid`, `reqwest`, ...). Does NOT depend
  on Diesel directly beyond the workspace dep surfaced for migration embedding.
- Used by: Tauri, Axum, and indirectly the storage crate (repository traits live
  here).

**Storage Implementation (`crates/storage-sqlite`):**

- Purpose: the one and only Diesel implementation of the repository traits.
- Location: `crates/storage-sqlite/src/` + `crates/storage-sqlite/migrations/`.
- Key files: `lib.rs`, `db/mod.rs` (connection pooling, `WriteHandle`,
  migrations via `diesel_migrations::embed_migrations!`), `schema.rs` (Diesel
  schema), `utils.rs` (SQLite chunking helpers), plus one module per entity
  (`accounts/`, `activities/`, `ai_chat/`, `assets/`, `custom_provider/`, `fx/`,
  `goals/`, `health/`, `limits/`, `market_data/`, `portfolio/`, `settings/`,
  `sync/`, `taxonomies/`), each with `model.rs` (Diesel DTO with
  Queryable/Insertable derives) and `repository.rs`.
- Concurrency: SQLite writes are serialized through a
  `write_actor::spawn_writer` handle obtained once at startup; reads use the
  r2d2 pool. Pragmas set at init: `WAL`, `foreign_keys=ON`,
  `busy_timeout=30000`, `synchronous=NORMAL` (`db/mod.rs:38-46`).
- Used by: Tauri context builder (`apps/tauri/src/context/providers.rs`) and
  Axum state builder (`apps/server/src/main_lib.rs`).

**Market Data (`crates/market-data`):**

- Purpose: provider-agnostic fetching of quotes, profiles, symbols, dividends.
- Location: `crates/market-data/src/`
- Modules: `models` (`InstrumentId`, `Quote`, `AssetProfile`, ...), `provider`
  (Yahoo, Alpha Vantage, Finnhub, Börse Frankfurt, MarketData.app,
  MetalPriceAPI, OpenFIGI, US Treasury, custom scraper), `registry` (rate
  limiter, circuit breaker, validator), `resolver` (instrument resolution chain
  and MIC/exchange mapping), `errors`.
- Architecture diagram: `crates/market-data/src/lib.rs:15-40` — Domain →
  Resolver → Provider → Quote.
- Used by: `whaleit-core::quotes::QuoteService`.

**Connect (`crates/connect`):**

- Purpose: HTTP client and broker-sync orchestration against the Whaleit
  Connect cloud service.
- Location: `crates/connect/src/`
- Modules: `client.rs` (REST client), `token_lifecycle.rs` (access/refresh token
  management, `CLOUD_ACCESS_TOKEN_KEY`, `CLOUD_REFRESH_TOKEN_KEY`), `broker/`
  (sync orchestrator, account/activity sync), `broker_ingest/` (import runs,
  sync state), `platform/` (broker platform catalogue). Gated by `broker`
  feature.
- Used by: both runtimes behind the `connect-sync` Cargo feature.

**Device Sync (`crates/device-sync`):**

- Purpose: E2EE multi-device synchronization via the Connect cloud — enrollment,
  pairing, team-key rotation, engine runtime.
- Location: `crates/device-sync/src/`
- Modules: `client.rs` (HTTP client), `enroll_service.rs`
  (`DeviceEnrollService`, sync state machine), `engine/`
  (`DeviceSyncRuntimeState`, background sync loop), `crypto.rs` (X25519 +
  ChaCha20-Poly1305 + HKDF helpers), `types.rs`, `time.rs`.
- Used by: gated under `device-sync` feature in both runtimes.

**AI (`crates/ai`):**

- Purpose: LLM orchestration built on `rig-core`; exposes streaming
  `AiStreamEvent`s and a tool-calling loop over domain services.
- Location: `crates/ai/src/`
- Modules: `chat.rs` (`ChatService` with tool-execution loop), `env.rs`
  (`AiEnvironment` trait — implemented by `TauriAiEnvironment` and
  `ServerAiEnvironment`), `providers.rs` + `ai_providers.json` (compiled-in
  catalog), `provider_model.rs`, `provider_service.rs`, `tools/`
  (`GetAccountsTool`, `GetGoalsTool`, `GetHoldingsTool`, `SearchActivitiesTool`,
  ...), `prompt_template.rs`, `stream_hook.rs`, `title_generator.rs`,
  `types.rs`, `system_prompt.txt`.
- Streaming transport: Tauri uses `tauri::ipc::Channel` (see
  `apps/frontend/src/adapters/tauri/ai-streaming.ts`); Web uses POST + SSE to
  `/api/v1/ai/chat/stream`.

**Shared TS Packages:**

- `packages/addon-sdk/` — public types, host API surface, permissions model
  consumed by third-party addons.
- `packages/ui/` — reusable React components (`ApplicationShell`,
  `TooltipProvider`, `ErrorBoundary`, ...) and `styles.css`.
- `packages/addon-dev-tools/` — CLI + dev server for authoring addons locally.

**Addons:**

- Purpose: user-installable extensions loaded at runtime.
- In-tree examples: `addons/goal-progress-tracker/`,
  `addons/investment-fees-tracker/`, `addons/swingfolio-addon/`.
- Loader: `apps/frontend/src/addons/addons-loader.ts` + `addons-core.ts`;
  runtime context + dynamic routes in `addons-runtime-context.ts`; type bridge
  in `type-bridge.ts`. Loader installs a global `AddonContext` so addons can
  call typed functions exported by the host.
- Host-side backing: `crates/core/src/addons/` domain logic plus Tauri commands
  in `apps/tauri/src/commands/addon.rs` and HTTP routes in
  `apps/server/src/api/addons.rs`.

## Data Flow

**Mutation Flow (example: create account):**

1. React component calls `createAccount(account)` from `@/adapters`
   (`apps/frontend/src/adapters/shared/accounts.ts:19-25`).
2. In Tauri build, `invoke("create_account", { account })` goes through
   `@tauri-apps/api/core.invoke`
   (`apps/frontend/src/adapters/tauri/core.ts:34-47`); in web build, it is
   translated to `POST /api/v1/accounts` with JSON body
   (`apps/frontend/src/adapters/web/core.ts:19-288`).
3. Tauri: `apps/tauri/src/commands/account.rs:29-44` `create_account` resolves
   `State<Arc<ServiceContext>>` and calls
   `state.account_service().create_account(account).await`. Axum:
   `apps/server/src/api/accounts.rs:37-46` calls the same trait method.
4. `AccountService` (in `crates/core/src/accounts/accounts_service.rs`) applies
   business rules, calls `AccountRepositoryTrait::create` (implemented by
   `crates/storage-sqlite/src/accounts/repository.rs`), and emits a
   `DomainEvent::AccountsChanged` through its injected
   `Arc<dyn DomainEventSink>`.
5. The repository runs an INSERT through the `WriteHandle` actor (serialized
   write path) and returns the inserted row.
6. The sink (Tauri: `TauriDomainEventSink` in
   `apps/tauri/src/domain_events/sink.rs`; Web: `WebDomainEventSink` in
   `apps/server/src/domain_events/sink.rs`) pushes the event onto an mpsc
   channel. A debouncing queue worker batches events over ~1s and dispatches to
   planners (`domain_events/planner.rs`) that trigger:
   - Portfolio snapshot/valuation recalculation
     (`PORTFOLIO_UPDATE_START/COMPLETE/ERROR`).
   - Asset enrichment (`ASSET_ENRICHMENT_START/PROGRESS/COMPLETE`).
   - Broker sync if `connect-sync` feature is enabled.
7. Tauri emits webview events via `AppHandle::emit`; Axum publishes
   `ServerEvent`s to the `EventBus` which streams them as SSE to the web client
   (`apps/server/src/events.rs`).
8. Frontend listeners in `apps/frontend/src/adapters/{tauri,web}/events.ts`
   forward payloads into React state via `use-global-event-listener.ts` and
   TanStack Query invalidations.

**Read Flow:** Same adapter path; commands/handlers call read-only trait methods
(`get_*`, `list_*`, `calculate_*`). No event emission. Caching is handled in the
frontend by `QueryClient` (see `apps/frontend/src/App.tsx:13-24`, default
`staleTime: 5 * 60 * 1000`).

**AI Streaming Flow:**

1. Frontend calls `streamAiChat(request, onEvent)`
   (`apps/frontend/src/adapters/tauri/ai-streaming.ts` or
   `apps/frontend/src/adapters/web/ai-streaming.ts`).
2. Tauri: opens a `Channel<AiStreamEvent>` and invokes `stream_ai_chat`
   (`apps/tauri/src/commands/ai_chat.rs`), which calls
   `ChatService::send_message` from `whaleit-ai`.
3. Web: POSTs to `/api/v1/ai/chat/stream`; handler streams NDJSON/SSE frames
   from `ChatService`.
4. `ChatService` drives a tool-calling loop: provider stream → tool calls →
   executes against `AiEnvironment` (reads from services) → feeds tool results
   back to the model → emits `TextDelta`, `ToolCall`, `ToolResult`, `Done`
   events.
5. Thread + messages are persisted via `AiChatRepository`
   (`crates/storage-sqlite/src/ai_chat/`).

**State Management:**

- Server-state: TanStack Query (`QueryClient` in `apps/frontend/src/App.tsx`).
  Window-global via `window.__whaleit_query_client__` for addon access.
- Transient UI state: React `useState`/`useReducer`, `zustand` (listed in
  `apps/frontend/package.json`).
- Cross-cutting providers (in `apps/frontend/src/App.tsx`): `AuthProvider`,
  `WhaleitConnectProvider`, `PrivacyProvider`, `SettingsProvider`,
  `TooltipProvider`. `PortfolioSyncProvider` and navigation providers are inside
  `AppLayout`.

## Key Abstractions

**`ServiceContext` (Tauri) / `AppState` (Axum):**

- Purpose: dependency-injected container of all `Arc<dyn ...Trait>` services;
  passed to every command/handler.
- Location: `apps/tauri/src/context/registry.rs:18-181`,
  `apps/server/src/main_lib.rs:63-106`.
- Pattern: constructed once at startup
  (`context/providers.rs::initialize_context`, `main_lib::build_state`), then
  stored in `AppHandle`/Axum `State`.

**`DomainEvent` + `DomainEventSink`:**

- Purpose: decouple core mutations from runtime side-effects (portfolio recalc,
  broker sync, asset enrichment).
- Location: `crates/core/src/events/domain_event.rs:1-162`,
  `crates/core/src/events/sink.rs`.
- Variants: `ActivitiesChanged`, `HoldingsChanged`, `AccountsChanged`,
  `AssetsCreated`, `AssetsUpdated`, `AssetsMerged`, `TrackingModeChanged`,
  `ManualSnapshotSaved`, `DeviceSyncPullComplete`.
- Runtime sinks: `apps/tauri/src/domain_events/`,
  `apps/server/src/domain_events/`. Both implement fire-and-forget enqueue with
  a debounced queue worker (`queue_worker.rs` in each).

**Repository + Service Traits:**

- Pattern: every domain module ships `*_traits.rs` with a repository trait
  (persistence port) and a service trait (use-case port). Example:
  `crates/core/src/accounts/accounts_traits.rs:17-91`. Services accept
  `Arc<dyn RepositoryTrait>` and optional `Arc<dyn DomainEventSink>`.
- Benefit: the two runtime hosts never touch Diesel types; tests substitute
  in-memory fakes.

**Platform Adapter (frontend):**

- Purpose: keep UI code identical across Tauri and Web targets.
- Location: `apps/frontend/src/adapters/`
- Pattern: `shared/*.ts` imports `invoke`, `logger`, `isDesktop`, `isWeb` from
  `#platform`; Vite remaps `#platform` to `tauri/core` or `web/core` at build
  time.

**Port Handles (storage):**

- `WriteHandle` — actor-style serialized writer shared by every SQLite
  repository (`crates/storage-sqlite/src/db/write_actor.rs`).
- `DbPool` — r2d2 read pool with SQLite `WAL` mode.

**`AiEnvironment`:**

- Purpose: abstracts how AI tools reach domain services (so Tauri and Axum can
  reuse `ChatService`).
- Location: `crates/ai/src/env.rs`; implementations at
  `apps/tauri/src/context/ai_environment.rs` and
  `apps/server/src/ai_environment.rs`.

**`SecretStore` / `AuthManager`:**

- Tauri uses OS keyring (`apps/tauri/src/secret_store.rs`).
- Axum uses file-based encrypted store with ChaCha20-Poly1305 key derived from
  env (`apps/server/src/secrets/`).
- Both implement the same `whaleit_core::secrets::SecretStore` trait.

## Entry Points

**Frontend (`apps/frontend/src/main.tsx`):**

- Renders React 19 root. Loads addons before mount (`loadAllAddons`), installs
  lockdown for desktop (non-iOS) builds, exposes `React`/`ReactDOM` globals for
  addons, and renders `<App />`.

**`apps/frontend/src/App.tsx`:**

- Wraps `AppRoutes` with `QueryClientProvider` and domain providers. In web mode
  it gates content behind `AuthGate`.

**`apps/frontend/src/routes.tsx`:**

- `BrowserRouter` + `Routes`. Top-level layouts: `/` → `AppLayout`,
  `/onboarding` → `OnboardingLayout`, `/auth/callback` → `AuthCallbackPage`.
  Settings routes are nested under `/settings`. Addon dynamic routes are
  injected via `getDynamicRoutes()` and `subscribeToNavigationUpdates()`.

**Tauri desktop/mobile (`apps/tauri/src/main.rs`):**

- `main()` → `whaleit_app_lib::run()` (`apps/tauri/src/lib.rs:197-633`).
  Sets up plugins (log, shell, dialog, fs, deep-link, single-instance, updater,
  window-state, haptics, barcode-scanner, web-auth, mobile-share), calls
  platform-specific `setup`, registers ~170 `#[tauri::command]` handlers in a
  single `generate_handler!` macro, and hooks exit events to gracefully stop the
  device-sync engine.

**Axum server (`apps/server/src/main.rs`):**

- `#[tokio::main] async fn main()` → reads `Config::from_env()`, builds
  `AppState` via `main_lib::build_state`, warms up device-sync token
  (feature-gated), spawns the broker-sync scheduler, spawns
  `whaleit_core::quotes::scheduler::run_periodic_sync`, serves static
  frontend with fallback to `index.html`, and listens on `config.listen_addr`
  (default `0.0.0.0:8080`).

**Database init:** `whaleit_storage_sqlite::db::init(&app_data_dir)` →
ensures directory, sets pragmas, returns resolved DB path;
`db::run_migrations(&db_path)` runs embedded migrations from
`crates/storage-sqlite/migrations/`.

## Error Handling

**Strategy:**

- Core crate defines `whaleit_core::errors::Error` + `Result` alias
  (`crates/core/src/errors.rs`) covering `DatabaseError`, validation, not-found,
  etc.
- Storage crate converts Diesel/r2d2 errors through `StorageError` + `IntoCore`
  (`crates/storage-sqlite/src/errors.rs`, re-exported from `lib.rs:55-56`).
- Tauri commands return `Result<T, String>` — errors are `format!`'d and logged
  before crossing the IPC boundary (see
  `apps/tauri/src/commands/account.rs:29-44`).
- Axum handlers use `ApiResult<T>` / `ApiError` (`apps/server/src/error.rs`).
  Errors map to HTTP status + JSON body.
- Frontend adapters wrap every call in `try/catch`, log via `logger`, and
  re-throw so TanStack Query surfaces them to the UI.

**Patterns:**

- Early-return with `map_err` in Tauri commands.
- Anyhow conversion in Axum via `ApiError::Anyhow` for background jobs
  (`apps/server/src/api/shared.rs:167-188`).
- Domain-level `thiserror` enums per module (e.g. `activities_errors.rs`).

## Cross-Cutting Concerns

**Logging:**

- Tauri: `tauri_plugin_log` with level `Debug` in debug builds, `Info` in
  release (`apps/tauri/src/lib.rs:215-228`). Frontend adapter logger forwards
  through the plugin.
- Axum: `tracing` + `tracing-subscriber` with JSON/text switch via
  `WF_LOG_FORMAT` (`apps/server/src/main_lib.rs:108-122`). Request-level span
  installed by `TraceLayer`.
- Frontend web build uses `console.*`.

**Validation:**

- Frontend: Zod schemas in `apps/frontend/src/lib/schemas.ts` consumed by React
  Hook Form via `@hookform/resolvers`.
- Backend: per-module validation inside service impls; repository never exposes
  raw DB errors.

**Authentication:**

- Desktop/mobile: no user auth — app is local; secrets live in OS keyring; cloud
  auth (Connect) goes through `ConnectService` + `token_lifecycle`.
- Web: optional JWT auth via `AuthManager` (`apps/server/src/auth.rs`) with
  Argon2 password hashing, cookie-based JWTs, rate-limited `/auth/login`,
  `require_jwt` middleware applied to all protected routes. Frontend gates
  routes with `AuthGate` (see `apps/frontend/src/App.tsx:31-37`).

**Authorization:**

- Axum: `require_jwt` middleware (`apps/server/src/api.rs:131-138`). No
  per-resource ownership checks beyond login (single-user deployment assumed).
- Addons: permission system declared in `packages/addon-sdk/src/permissions.ts`
  and enforced by the host API bridge in
  `apps/frontend/src/addons/type-bridge.ts`.

**Request IDs & Tracing (Axum):**

- `SetRequestIdLayer::x_request_id(MakeRequestUuid)` + `PropagateRequestIdLayer`
  (`apps/server/src/api.rs:164-178`).

**Rate Limiting (Axum):**

- `tower_governor` applied to `/auth/login` (5 requests / 60s per peer IP,
  `api.rs:141-146`).

**CORS (Axum):**

- Configured from `config.cors_allow`; wildcard `*` allowed when single-origin
  (`apps/server/src/api.rs:71-82`).

**Event Bus:**

- Tauri: webview event emission via `AppHandle::emit`; frontend subscribes via
  `@tauri-apps/api/event.listen`.
- Axum: in-process `tokio::sync::broadcast` (`apps/server/src/events.rs`);
  exposed to browser via SSE endpoint `/api/v1/events/stream` consumed by
  `ServerEventBridge` in `apps/frontend/src/adapters/web/events.ts`.

**Background Schedulers:**

- Broker sync: 4h interval (`apps/tauri/src/scheduler.rs`,
  `apps/server/src/scheduler.rs`).
- Market data sync: 6h interval with 2min initial delay, run by
  `whaleit_core::quotes::scheduler::run_periodic_sync` from both hosts.
- Device sync engine: background loop in `crates/device-sync/src/engine/`
  started conditionally at boot.
- Domain event queue worker: debounced 1s batches per host.

**Secrets:**

- Trait `whaleit_core::secrets::SecretStore`. Tauri impl = OS keyring via
  `keyring` crate. Axum impl = encrypted JSON file at `WF_SECRET_FILE` (default
  `<data_dir>/secrets.json`) using `SECRETS_ENCRYPTION_KEY`.

**Feature Flags (Cargo):**

- `connect-sync` — broker cloud sync (default enabled in both binaries).
- `device-sync` — E2EE device sync (default enabled in both binaries).
- `appstore` — Tauri-only flag for App Store builds.

---

_Architecture analysis: 2026-04-20_
