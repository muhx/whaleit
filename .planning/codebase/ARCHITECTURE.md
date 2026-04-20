# Architecture

**Analysis Date:** 2026-04-20

## Pattern Overview

**Overall:** Dual-runtime desktop/web application with shared backend core

Wealthfolio is a local-first portfolio tracker that runs in two modes:
1. **Desktop** — Tauri v2 shell (Rust) embeds a WebView rendering the React frontend; backend logic runs as Rust crates in-process
2. **Web** — Axum HTTP server serves the same React frontend as static files plus a REST API; same Rust crates power the backend

Both modes share identical business logic via `crates/*`, and the frontend abstracts the transport layer behind adapter modules.

**Key Characteristics:**
- Local-first: all data stored in SQLite on the user's device; no mandatory cloud
- Adapter pattern: single frontend codebase targets Tauri IPC or HTTP via pluggable adapters
- Domain event system: services emit events after mutations; runtime bridges translate events into side effects (portfolio recalculation, broker sync, etc.)
- Feature-gated compilation: `connect-sync` and `device-sync` features enable optional cloud broker sync and E2EE device sync
- Addon system: third-party extensions loaded at runtime via the `@wealthfolio/addon-sdk`

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      Frontend (React + Vite)                     │
│  apps/frontend/src/                                              │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌───────────────┐  │
│  │  Pages/   │  │Features/ │  │Components/│  │   Hooks/      │  │
│  │  Routes   │  │AI,Sync,  │  │Shared UI  │  │TanStack Query │  │
│  │           │  │Connect   │  │           │  │               │  │
│  └─────┬─────┘  └─────┬───┘  └───────────┘  └───────┬───────┘  │
│        │              │                             │           │
│  ┌─────▼──────────────▼─────────────────────────────▼────────┐  │
│  │                   Adapters Layer                           │  │
│  │  adapters/tauri/              adapters/web/                │  │
│  │  ┌──────────────┐            ┌──────────────┐             │  │
│  │  │ Tauri invoke()│            │ fetch() HTTP │             │  │
│  │  └──────┬───────┘            └──────┬───────┘             │  │
│  │         └──────────┬─────────────────┘                     │  │
│  │         adapters/shared/ (common command wrappers)         │  │
│  └─────────────────────┼─────────────────────────────────────┘  │
└────────────────────────┼────────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    Desktop Mode    Web Mode        (same API shape)
         │               │
┌────────▼────────┐ ┌────▼──────────────┐
│  Tauri IPC      │ │  Axum HTTP API    │
│  apps/tauri/    │ │  apps/server/     │
│  commands/*.rs  │ │  api/*.rs         │
│  (thin IPC      │ │  (thin HTTP       │
│   delegates)    │ │   handlers)       │
└────────┬────────┘ └────┬──────────────┘
         │               │
         └───────┬───────┘
                 │
┌────────────────▼────────────────────────────────────────────────┐
│                     Core Crates (Rust)                          │
│  crates/core/        — Business logic, domain services, traits │
│  crates/storage-sqlite/ — Diesel ORM, repositories, migrations │
│  crates/market-data/ — Market data providers, symbol resolver  │
│  crates/connect/     — Broker sync (Wealthfolio Connect)       │
│  crates/device-sync/ — E2EE device sync engine                 │
│  crates/ai/          — LLM integration, tool calls, streaming  │
└────────────────────────────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────────────┐
│                     SQLite Database                             │
│  crates/storage-sqlite/migrations/ (Diesel migrations)          │
│  Local file: app.db (desktop) or configured path (web)          │
└────────────────────────────────────────────────────────────────┘
```

## Layers

### Frontend (`apps/frontend/`)
- **Purpose:** Single-page React application providing the entire UI
- **Location:** `apps/frontend/src/`
- **Contains:** Pages, feature modules, shared components, hooks, adapters, addon runtime
- **Depends on:** React, TanStack Query (server state), react-router-dom, shadcn/ui (`@wealthfolio/ui`)
- **Used by:** Rendered by Tauri WebView (desktop) or served by Axum (web)
- **State management:** TanStack Query for all server state; React context for local UI state (`portfolio-sync-context.tsx`, `privacy-context.tsx`, `auth-context.tsx`)

### Desktop Shell (`apps/tauri/`)
- **Purpose:** Tauri v2 application shell — native window, IPC bridge, OS integration
- **Location:** `apps/tauri/src/`
- **Contains:**
  - `commands/` — 27 IPC command modules, each a thin delegate to core services
  - `context/` — Service registry (`ServiceContext`) and dependency injection
  - `domain_events/` — Bridge from domain events to Tauri app events
  - `secret_store.rs` — OS keyring-backed secret storage
  - `scheduler.rs` — Periodic market data sync (6h interval)
  - `listeners.rs` — Frontend event listeners (file drop, deep links)
- **Depends on:** All core crates, Tauri plugins (shell, dialog, fs, deep-link, updater, single-instance)
- **Used by:** End users as desktop app (macOS, Windows, Linux, iOS)

### Web Server (`apps/server/`)
- **Purpose:** Self-hosted web mode — serves frontend as static files + REST API
- **Location:** `apps/server/src/`
- **Contains:**
  - `api/` — 25 HTTP handler modules mapping to core services
  - `api.rs` — Router composition, CORS, auth middleware, rate limiting, tracing
  - `auth.rs` — Optional JWT + Argon2id password authentication
  - `config.rs` — Environment-driven configuration
  - `domain_events/` — Bridge from domain events to SSE event bus
  - `main_lib.rs` — `AppState` construction, service wiring identical to Tauri
  - `events.rs` — Server-Sent Events (SSE) bus for real-time frontend updates
- **Depends on:** All core crates, Axum, tower-http middleware, utoipa (OpenAPI)
- **Used by:** Self-hosted users, Docker deployments

### Core Crates (`crates/`)
Each crate has a distinct responsibility. All are database-agnostic where possible, with `storage-sqlite` providing concrete implementations.

**`crates/core/`** — Domain logic and service traits
- **Location:** `crates/core/src/`
- **Contains:** `accounts/`, `activities/`, `assets/`, `fx/`, `goals/`, `health/`, `limits/`, `portfolio/`, `quotes/`, `settings/`, `taxonomies/`, `events/`, `secrets/`
- **Depends on:** Defines traits implemented by `storage-sqlite`
- **Used by:** Both `apps/tauri` and `apps/server` via concrete service constructors

**`crates/storage-sqlite/`** — Database layer
- **Location:** `crates/storage-sqlite/src/`
- **Contains:** Repositories for every domain (`accounts/`, `activities/`, `assets/`, etc.), `db/` (pool, write-actor), `schema.rs` (Diesel schema), `migrations/` (29 migration files)
- **Depends on:** Diesel, rusqlite, r2d2 connection pool
- **Used by:** Core services via repository trait implementations

**`crates/market-data/`** — Market data providers
- **Location:** `crates/market-data/src/`
- **Contains:** `provider/` (Yahoo Finance, custom providers), `resolver/` (symbol resolution), `registry/` (provider management), `models/`
- **Depends on:** `reqwest` for HTTP
- **Used by:** `crates/core` quote service

**`crates/connect/`** — Broker sync integration (Wealthfolio Connect)
- **Location:** `crates/connect/src/`
- **Contains:** `broker/` (mapping, orchestrator, progress tracking), `platform/`, `broker_ingest/`, `client.rs`, `token_lifecycle.rs`
- **Feature-gated:** `connect-sync`
- **Used by:** Tauri commands and server API for broker data import

**`crates/device-sync/`** — E2EE device synchronization
- **Location:** `crates/device-sync/src/`
- **Contains:** `engine/` (sync runtime, ports), `crypto.rs` (X25519, ChaCha20Poly1305, HKDF), `enroll_service.rs`, `client.rs`
- **Feature-gated:** `device-sync`
- **Used by:** Tauri and server for multi-device sync

**`crates/ai/`** — AI/LLM integration
- **Location:** `crates/ai/src/`
- **Contains:** `providers.rs`, `provider_service.rs`, `chat.rs`, `stream_hook.rs`, `tools/` (15 tool implementations — accounts, activities, holdings, etc.), `eval/`, `prompt_template_service.rs`
- **Used by:** Both runtimes for AI assistant features

### Packages (`packages/`)
- **`packages/ui/`** (`@wealthfolio/ui`) — Shared shadcn/ui component library built with tsup
- **`packages/addon-sdk/`** (`@wealthfolio/addon-sdk`) — Types and API for addon developers
- **`packages/addon-dev-tools/`** (`@wealthfolio/addon-dev-tools`) — CLI and dev server for addon development

## Data Flow

### User Action → Data Persistence (Desktop)

1. User interacts with React component in `apps/frontend/src/pages/`
2. Component calls a hook (e.g., `use-accounts.ts`) which invokes TanStack Query mutation
3. Mutation calls adapter function from `apps/frontend/src/adapters/shared/accounts.ts`
4. Adapter calls Tauri `invoke("create_account", payload)` via `apps/frontend/src/adapters/tauri/core.ts`
5. Tauri IPC routes to `apps/tauri/src/commands/account.rs` → `create_account()`
6. Command delegates to `crates/core/src/accounts/AccountService` trait method
7. Service performs validation, calls repository trait from `crates/storage-sqlite/`
8. Repository executes Diesel query against SQLite
9. Service emits a `DomainEvent::AccountsChanged` via the event sink
10. Domain event worker (`apps/tauri/src/domain_events/`) processes event:
    - Triggers portfolio recalculation, asset enrichment, etc.
11. Worker emits Tauri app event → frontend `listenPortfolioUpdateComplete` invalidates TanStack Query cache

### User Action → Data Persistence (Web)

Same flow, except:
- Step 4: Adapter calls `fetch("/api/v1/accounts", { method: "POST", ... })` via `apps/frontend/src/adapters/web/core.ts`
- Step 5: Axum routes to `apps/server/src/api/accounts.rs` handler
- Step 11: SSE event sent from `apps/server/src/events.rs::EventBus` → frontend SSE listener invalidates cache

### Market Data Sync (Periodic)

1. `scheduler::run_periodic_sync()` spawns on startup (2min initial delay, 6h interval)
2. Calls `QuoteService::sync_market_data()` in `crates/core/src/quotes/`
3. QuoteService resolves symbols via `crates/market-data/` providers
4. Results stored via `crates/storage-sqlite/market_data/` repository
5. Domain events emitted → triggers recalculation of affected portfolios

### Domain Event Processing

The domain event system is the backbone of side effects:

```
Service (mutation) → DomainEventSink → Event Channel → Queue Worker
    ↓                                                        ↓
  Persist to DB                                    Route event to handlers:
                                                  - Portfolio recalculation
                                                  - Asset enrichment (quote sync)
                                                  - Broker sync triggers
                                                  - FX rate sync
                                                  - Tauri app events / SSE push
```

**Key event types** (defined in `crates/core/src/events/domain_event.rs`):
- `ActivitiesChanged` — triggers portfolio recalculation + FX sync planning
- `HoldingsChanged` — triggers portfolio recalculation
- `AccountsChanged` — triggers FX sync, portfolio recalculation
- `AssetsCreated` / `AssetsUpdated` — triggers quote sync, enrichment
- `AssetsMerged` — UNKNOWN asset merge propagation
- `TrackingModeChanged` — switches between transactions/holdings/manual tracking
- `ManualSnapshotSaved` — triggers recalculation for affected account
- `DeviceSyncPullComplete` — triggers full portfolio recalculation

**State Management:**
- Server state: TanStack Query with query keys defined in `apps/frontend/src/lib/query-keys.ts`
- Client state: React Context providers in `apps/frontend/src/context/`
- Global mutable state (base_currency, timezone): `Arc<RwLock<String>>` in Rust `ServiceContext`

## Key Abstractions

### Adapter Pattern (Frontend Transport)
- **Purpose:** Decouple frontend from backend transport mechanism
- **Examples:** `apps/frontend/src/adapters/tauri/core.ts` (Tauri `invoke`), `apps/frontend/src/adapters/web/core.ts` (HTTP `fetch`)
- **Pattern:** Both implement identical function signatures; `adapters/shared/` contains common logic using platform-agnostic `invoke()`; Vite resolve alias swaps `tauri/` ↔ `web/` at build time

### Repository Trait Pattern (Backend Data)
- **Purpose:** Decouple core business logic from database implementation
- **Examples:** Traits defined in `crates/core/src/*/` (e.g., `AccountServiceTrait`), implemented in `crates/storage-sqlite/src/*/`
- **Pattern:** Core defines `trait XRepositoryTrait`, `storage-sqlite` implements it using Diesel

### Service Trait Pattern (Backend Logic)
- **Purpose:** Allow runtime-specific implementations and test mocking
- **Examples:** `QuoteServiceTrait`, `HoldingsServiceTrait`, `ActivityServiceTrait` — all in `crates/core/src/`
- **Pattern:** `dyn Trait` objects stored in `Arc<>` in both `ServiceContext` (Tauri) and `AppState` (Server)

### Domain Event Sink
- **Purpose:** Decouple mutation side effects from core business logic
- **Examples:** `crates/core/src/events/sink.rs` (trait), `apps/tauri/src/domain_events/` (Tauri impl), `apps/server/src/domain_events/` (Web impl)
- **Pattern:** Services call `event_sink.send(event)` after successful mutations; runtime-specific workers consume events and trigger appropriate side effects

### Write Actor Pattern (Database Writes)
- **Purpose:** Serialize all database writes through a single tokio actor to avoid SQLite locking issues
- **Examples:** `crates/storage-sqlite/src/db/write_actor.rs`
- **Pattern:** Repositories send write operations to a spawned actor; reads use the connection pool directly

## Entry Points

### Desktop Application
- **Location:** `apps/tauri/src/main.rs` → `lib.rs::run()`
- **Triggers:** User launches the Tauri application
- **Responsibilities:**
  1. Load `.env` configuration
  2. Register Tauri plugins (single-instance, logging, shell, dialog, fs, deep-link, updater)
  3. Initialize `ServiceContext` (DB, repos, services, event sink)
  4. Register all IPC commands (70+ commands)
  5. Start domain event queue worker
  6. Run startup sync (market data, broker connections)
  7. Start periodic market data sync scheduler
  8. Optionally start device sync background engine

### Web Server
- **Location:** `apps/server/src/main.rs` → `main_lib.rs`
- **Triggers:** User runs the server binary or Docker container
- **Responsibilities:**
  1. Load configuration from environment variables
  2. Initialize tracing (text or JSON format)
  3. Build `AppState` (identical service wiring to Tauri)
  4. Compose Axum router with all API routes
  5. Apply middleware (CORS, auth, rate limiting, tracing, timeouts)
  6. Serve frontend static files
  7. Start domain event queue worker + SSE event bus

### Frontend Application
- **Location:** `apps/frontend/src/main.tsx`
- **Triggers:** Browser/WebView loads `index.html`
- **Responsibilities:**
  1. Determine platform (desktop vs web)
  2. Load addons
  3. Expose React/ReactDOM globally for addon system
  4. Render `<App />` with React StrictMode

## Error Handling

**Rust (Backend):**
- `thiserror` for domain error types: `crates/core/src/errors.rs`, `apps/tauri/src/error.rs`, `apps/server/src/error.rs`
- `Result<T, E>` propagation with `?` operator
- Tauri commands convert errors to serializable strings for IPC
- Axum handlers return appropriate HTTP status codes via error conversions

**TypeScript (Frontend):**
- TanStack Query error handling via `onError` callbacks
- Adapter `invoke()` wraps calls with timeout (120s) and error logging
- Web adapter handles 401 responses by notifying the auth context

## Cross-Cutting Concerns

**Logging:**
- Desktop: `tauri_plugin_log` with debug/info level filtering → `apps/tauri/src/lib.rs`
- Web: `tracing_subscriber` with configurable format (text/JSON) → `apps/server/src/main_lib.rs`
- Frontend: Platform-specific logger adapter → `apps/frontend/src/adapters/{tauri,web}/core.ts`

**Validation:**
- Frontend forms: `react-hook-form` + `zod` schemas → `apps/frontend/src/lib/schemas.ts`
- Backend: Rust type system + Diesel schema validation

**Authentication:**
- Desktop: No auth (local data, OS user isolation)
- Web: Optional JWT + Argon2id password auth → `apps/server/src/auth.rs`
  - Cookie-based sessions with configurable `Secure` flag
  - Rate-limited login endpoint (5 req/60s per IP)
  - Refuses to start on non-loopback without auth (unless explicitly opted out)

**Secrets Management:**
- Desktop: OS keyring via `secret_store.rs` (never written to disk)
- Web: Encrypted JSON file (`secrets.json`) with HKDF-derived key from `WF_SECRET_KEY`
- API keys for market data providers and AI providers stored as secrets

**Internationalization/Currency:**
- Base currency and timezone are runtime-mutable settings
- `Arc<RwLock<String>>` pattern allows dynamic updates without restart
- FX service handles multi-currency conversions via exchange rate repository

## Key Design Decisions

1. **Single frontend, dual backend transport** — The adapter pattern (`adapters/tauri/` vs `adapters/web/`) allows one React codebase to work with both Tauri IPC and HTTP REST API. Shared command logic lives in `adapters/shared/`.

2. **Local-first with optional cloud** — SQLite is the only required storage. Cloud features (broker sync, device sync) are feature-gated and optional. The app works fully offline.

3. **Write actor for SQLite** — All writes go through a single tokio actor (`crates/storage-sqlite/src/db/write_actor.rs`) to prevent concurrent write conflicts with SQLite's locking model.

4. **Domain events for side effects** — Rather than calling recalculation/sync directly from service methods, services emit domain events. Runtime bridges handle them asynchronously. This keeps core logic side-effect-free.

5. **Feature flags for optional crates** — `connect-sync` and `device-sync` are Cargo features. This keeps the binary lean for users who don't need broker sync or multi-device sync.

6. **Addon system** — Third-party addons use `@wealthfolio/addon-sdk` to register routes, pages, and data queries. Loaded at runtime via dynamic imports.

7. **Desktop-only features** — FIRE planner, file dialogs, and crypto operations are desktop-only. Web adapters provide stubs that throw at runtime for these features.

## Deployment Topology

### Desktop: Bundled Tauri App
- **Build:** `pnpm build:tauri` — Vite builds frontend, Cargo builds Tauri binary
- **Output:** Platform-specific installer (`.dmg`, `.msi`, `.AppImage`, `.ipa`)
- **Data:** SQLite at OS app data directory, secrets in OS keyring
- **Distribution:** GitHub Releases with auto-update via `tauri-plugin-updater`

### Web: Docker / Self-hosted
- **Build:** `Dockerfile` — Multi-stage build (Rust + Node.js)
- **Compose:** `compose.yml` / `compose.dev.yml` for easy deployment
- **Listen:** Configurable via `WF_LISTEN_ADDR` (default `0.0.0.0:8088`)
- **Data:** SQLite at configured `WF_DB_PATH`, secrets in encrypted JSON file
- **Auth:** Optional password authentication with JWT sessions

---

*Architecture analysis: 2026-04-20*
