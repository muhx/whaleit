# External Integrations

**Analysis Date:** 2026-04-20

## APIs & External Services

**Market Data (Rust `crates/market-data/src/provider/*`):**

- Yahoo Finance — quotes, historical bars, dividends, search, FX
  - SDK/Client: `yahoo_finance_api 4.1` + direct `reqwest` calls
  - Files: `crates/market-data/src/provider/yahoo/mod.rs`
  - Endpoints: `https://fc.yahoo.com`, `https://query1.finance.yahoo.com/*`,
    `https://query2.finance.yahoo.com/v1/finance/search`
  - Auth: anonymous (cookie + crumb dance)
- Alpha Vantage — equities/FX fallback provider
  - File: `crates/market-data/src/provider/alpha_vantage/mod.rs`
  - Auth: user-supplied API key (stored via `SecretStore`)
- Finnhub — company profile and quotes
  - File: `crates/market-data/src/provider/finnhub/mod.rs`
  - Base URL: `https://finnhub.io/api/v1`
  - Auth: user-supplied API key
- Marketdata.app — US equities prices and candles
  - File: `crates/market-data/src/provider/marketdata_app/mod.rs`
  - Base URL: `https://api.marketdata.app/v1`
  - Auth: user-supplied bearer token
- MetalPriceAPI — precious metals spot + history
  - File: `crates/market-data/src/provider/metal_price_api/mod.rs`
  - Endpoints: `https://api.metalpriceapi.com/v1/latest`,
    `https://api.metalpriceapi.com/v1/timeframe`
  - Auth: user-supplied API key
- OpenFIGI — instrument ID (FIGI) lookup for symbol resolution
  - File: `crates/market-data/src/provider/openfigi/mod.rs`
  - Endpoints: `https://api.openfigi.com/v3/mapping`,
    `https://api.openfigi.com/v3/search`
  - Auth: optional API key
- Börse Frankfurt (Deutsche Börse) — European equities
  - File: `crates/market-data/src/provider/boerse_frankfurt/mod.rs`
  - Base URL: `https://api.live.deutsche-boerse.com/v1`
- US Treasury calc — synthetic treasury yields (no network call, local math
  based on Treasury curve)
  - File: `crates/market-data/src/provider/us_treasury_calc/mod.rs`
- Custom user-defined scrapers (`scraper 0.22`, `jsonpath-rust 0.7`)
  - Crate: `crates/core/src/custom_provider/`
  - Server command surface: `apps/server/src/api/custom_providers.rs`,
    `apps/tauri/src/commands/custom_provider.rs`

**AI / LLM providers (Rust `crates/ai/`, config
`crates/ai/src/ai_providers.json`):**

- OpenAI — cloud LLM API
- Anthropic — cloud LLM API
- Google AI (Gemini) — cloud LLM API
- Groq — fast cloud inference
- OpenRouter — cloud LLM gateway
- Ollama — local LLM (default URL `http://localhost:11434`)
- SDK/Client: `rig` (`rig-core 0.30`) via `crates/ai/src/providers.rs` +
  `crates/ai/src/provider_service.rs`
- Auth: per-provider API key stored through `wealthfolio_core::secrets`
  (`SecretStore` trait), env keys per provider listed in
  `crates/ai/src/ai_providers.json` (`OLLAMA_API_KEY`, `GROQ_API_KEY`, etc. —
  stored encrypted, not read from shell env at runtime)
- Streaming: `tokio-stream` + Tauri Channel
  (`apps/frontend/src/adapters/tauri/ai-streaming.ts`) or HTTP SSE
  (`apps/frontend/src/adapters/web/ai-streaming.ts`, endpoint
  `/api/v1/ai/chat/stream`)

**Wealthfolio Connect (first-party cloud — optional feature):**

- Connect API (subscription, broker-sync orchestration, device-sync transport)
  - Default base URL: `https://api.wealthfolio.app`
    (`crates/connect/src/client.rs:25`)
  - Client: `ConnectApiClient` in `crates/connect/src/client.rs`
  - Feature-gated by `connect-sync` and `device-sync` Cargo features
    (`apps/tauri/Cargo.toml`, `apps/server/Cargo.toml`)
- Supabase (auth provider behind Connect)
  - SDK: `@supabase/supabase-js 2.95`
  - File:
    `apps/frontend/src/features/wealthfolio-connect/providers/wealthfolio-connect-provider.tsx`
  - Default URL: `https://auth.wealthfolio.app`
  - Methods used: `signInWithPassword`, `signUp`, `signInWithOAuth`,
    `signInWithOtp`, `verifyOtp`, `exchangeCodeForSession`, `setSession`,
    `signOut`, `onAuthStateChange`
- Connect hosted OAuth callback — `https://connect.wealthfolio.app/deeplink`
  (redirects back to `wealthfolio://auth/callback` on desktop/mobile)
- Connect portal (user UI): `https://connect.wealthfolio.app`
  (`apps/frontend/src/lib/constants.ts:8`)

**Broker integrations (via Connect):**

- `crates/connect/src/broker/` and `crates/connect/src/broker_ingest/`
  orchestrate broker authorization, import runs, account mapping
- Entrypoints: `apps/server/src/api/brokers_sync.rs`,
  `apps/tauri/src/commands/brokers_sync.rs`
- Ingest models: `crates/connect/src/broker_ingest/models.rs`
- Downstream broker services are brokered through `api.wealthfolio.app` — no
  direct broker SDK dependencies in the workspace

## Data Storage

**Databases:**

- SQLite (local-first)
  - Connection: Tauri → `<app_data_dir>/app.db`; server → `WF_DB_PATH` (default
    `./db/app.db`, Docker `/data/wealthfolio.db`)
  - ORM: Diesel 2.2 (sqlite feature) via workspace `Cargo.toml`; raw driver
    `rusqlite 0.34` (bundled)
  - Pool: `r2d2 0.8`
  - Migrations: `diesel_migrations` applied at startup from
    `crates/storage-sqlite/migrations/` (21 migrations,
    `2023-11-08-162221_init_db` through
    `2026-01-24-000001_improve_import_profiles`)
  - Schema: `crates/storage-sqlite/src/schema.rs`
  - Dev DB path: `db/web-dev.db` (gitignored)

**File Storage:**

- Local filesystem only. Tauri fs plugin scoped to `$APPDATA/**`
  (`apps/tauri/capabilities/desktop.json:26`). No S3/GCS integration.
- CSV imports: `crates/core/src/activities/csv/` (CSV parsing via `csv 1.4`,
  encoding detection via `chardetng`/`encoding_rs`)
- ZIP handling (addon bundles): `zip 2.2` in `crates/core`

**Caching:**

- In-process caches: `dashmap 6.1` and `lazy_static 1.4` for market-data crumbs
  (`crates/market-data/src/provider/yahoo/mod.rs:69`)
- React Query cache on frontend (`@tanstack/react-query 5.90`)
- No external cache (Redis/Memcached) — intentional for local-first design

## Authentication & Identity

**Desktop / Tauri:**

- OS keychain via `keyring 2.0` (`apps/tauri/src/secret_store.rs`, struct
  `KeyringSecretStore`). Secret service ID prefix `wealthfolio_`.
- Wealthfolio Connect auth: Supabase session managed by
  `apps/frontend/src/features/wealthfolio-connect/providers/wealthfolio-connect-provider.tsx`
  - Desktop uses deep-link callback scheme `wealthfolio://auth/callback`
    (`apps/tauri/tauri.conf.json:52`)
  - iOS uses `tauri-plugin-web-auth` (`ASWebAuthenticationSession`) — required
    for Google OAuth (`apps/tauri/src/lib.rs:146`)
- Refresh token stored under keyring key `sync_refresh_token`
  (`apps/tauri/src/commands/wealthfolio_connect.rs:21`)

**Web server (Axum):**

- Argon2id password hashing + JWT (HS256 family via `jsonwebtoken 10`
  `aws_lc_rs` feature) — `apps/server/src/auth.rs`
- Required env: `WF_SECRET_KEY` (32-byte base64 or ASCII, HKDF-derived into
  separate JWT + secret-store keys in `apps/server/src/config.rs:49`)
- Optional: `WF_AUTH_PASSWORD_HASH` (Argon2id PHC string),
  `WF_AUTH_TOKEN_TTL_MINUTES` (default 60), `WF_AUTH_REQUIRED`,
  `WF_COOKIE_SECURE` (`auto|true|false`)
- Session cookie with conditional `Secure` attribute, policy in
  `apps/server/src/auth.rs` (`CookieSecurePolicy`)
- Middleware `require_jwt` guards `/api/v1/*` except `/auth/*` and health
  endpoints (`apps/server/src/api.rs:131`)
- Login rate limit: 5 requests / 60s per peer IP via `tower_governor 0.8`
  (`apps/server/src/api.rs:141`)
- Fails-closed when binding non-loopback without auth unless
  `WF_AUTH_REQUIRED=false` (`apps/server/src/config.rs:95`)

**Secret storage (server):**

- Encrypted JSON file at `<data-root>/secrets.json` (override via
  `WF_SECRET_FILE`); encryption via ChaCha20-Poly1305 keyed by HKDF-derived
  `secrets_encryption_key` — `apps/server/src/secrets/`,
  `apps/server/src/config.rs:49`

## Monitoring & Observability

**Error Tracking:**

- None detected (no Sentry/Datadog/Rollbar SDK in dependency manifests)

**Logs:**

- Frontend (Tauri): `@tauri-apps/plugin-log` wired via
  `apps/frontend/src/adapters/tauri/core.ts:2` (`logger` with error/warn/info/
  debug/trace levels)
- Tauri backend: `tauri-plugin-log` with `log::LevelFilter::Debug` in debug
  builds, `Info` in release (`apps/tauri/src/lib.rs:216`), suppresses
  `tauri_plugin_updater` debug spam
- Axum server: `tracing 0.1` + `tracing-subscriber 0.3` (fmt, env-filter, json)
  initialized in `apps/server/src/main_lib.rs` (`init_tracing`); HTTP spans via
  `tower_http::trace::TraceLayer` in `apps/server/src/api.rs:167`
- Request IDs propagated via `SetRequestIdLayer::x_request_id(MakeRequestUuid)`
  and `PropagateRequestIdLayer` (`apps/server/src/api.rs:163`)

**Health endpoints:**

- `GET /api/v1/healthz` → "ok"
- `GET /api/v1/readyz` → "ok"
- OpenAPI document served at `/api/v1/openapi.json`
- Defined in `apps/server/src/api.rs:51` with `utoipa::path` macros
- Docker healthcheck hits `/api/v1/healthz` every 30s (`compose.yml:46`)

## CI/CD & Deployment

**Hosting:**

- Desktop/mobile binaries distributed via auto-updater endpoint
  `https://wealthfolio.app/releases/{{target}}/{{arch}}/{{current_version}}`
  (minisign-signed, pubkey embedded in `apps/tauri/tauri.conf.json:39`)
- Windows install mode: `passive`
- iOS: App Store (`appstore` Cargo feature flag in `apps/tauri/Cargo.toml:64`,
  signing `Apple Distribution: Teymz Inc (DYDJ2RNL5H)`)
- Web/server: Docker image `afadil/wealthfolio:latest` on Docker Hub,
  self-hostable via `compose.yml`

**CI Pipeline (GitHub Actions):**

- `.github/workflows/pr-check.yml` — on PR to `main`, `develop`, `feature/**`:
  frontend check (pnpm install, build:types, format:check, lint, type-check,
  test, build) + rust check (fmt --check, clippy -D warnings, cargo test
  --workspace, cargo check -p wealthfolio-server --release)
- `.github/workflows/release.yml` — release builds
- `.github/workflows/docker-publish.yml` — Docker image publish
- Rust caching: `Swatinem/rust-cache@v2`; rust toolchain:
  `dtolnay/rust-toolchain@stable`

**Docker build:**

- `Dockerfile` — stage 1 `node:24-alpine` builds frontend with
  `BUILD_TARGET=web`; stage 2 `rust:1.91-alpine` + `tonistiigi/xx`
  cross-compiles `wealthfolio-server`; final `alpine:3.19` runtime exposes port
  8080, CMD `/usr/local/bin/wealthfolio-server`
- Build args: `CONNECT_AUTH_URL`, `CONNECT_AUTH_PUBLISHABLE_KEY`,
  `CONNECT_API_URL` (baked into JS bundle and server binary)
- Runtime container is read-only with `/tmp` tmpfs (64M) and `no-new-privileges`
  security option (`compose.yml:58`)

## Environment Configuration

**Required env vars (desktop `.env`):**

- `DATABASE_URL` (dev-only, Diesel CLI)
- `CONNECT_AUTH_URL`, `CONNECT_AUTH_PUBLISHABLE_KEY` — enable Connect feature
  flag (`apps/frontend/src/lib/connect-config.ts:8`)
- `CONNECT_API_URL` — override Connect API base
- `CONNECT_OAUTH_CALLBACK_URL` — override hosted OAuth bridge
- Consumed at compile time by `apps/tauri/build.rs` and `apps/server/build.rs`

**Required env vars (server `.env.web`):**

- `WF_SECRET_KEY` (mandatory; panics if missing)
- `WF_LISTEN_ADDR` (default `0.0.0.0:8088`)
- `WF_DB_PATH`, `WF_STATIC_DIR`, `WF_ADDONS_DIR`, `WF_SECRET_FILE`
- `WF_CORS_ALLOW_ORIGINS` (comma-separated; wildcard `*` disallowed when auth is
  enabled)
- `WF_REQUEST_TIMEOUT_MS`
- `WF_AUTH_PASSWORD_HASH`, `WF_AUTH_TOKEN_TTL_MINUTES`, `WF_AUTH_REQUIRED`,
  `WF_COOKIE_SECURE`
- `VITE_API_TARGET` / `WF_API_TARGET` — Vite dev proxy target
  (`apps/frontend/vite.config.ts:8`)
- `WF_ENABLE_VITE_PROXY` — opt-in flag for proxy
- `BUILD_TARGET` — `tauri` or `web` (selects adapter alias in `vite.config.ts`)
- `TAURI_DEV_HOST` — enables mobile/network dev server binding
- `CI`, `TAURI_DEBUG` — CI and debug build flags

**Secrets location:**

- Desktop: OS keychain via `keyring` (macOS Keychain / Windows Credential
  Manager / Linux Secret Service) — `apps/tauri/src/secret_store.rs`
- Web: encrypted `secrets.json` in server data directory —
  `apps/server/src/secrets/`
- Refresh tokens: Keyring key `sync_refresh_token` (desktop), HttpOnly cookie
  (web)
- `.env` and `.env.web` are present in the repo but gitignored — their contents
  were intentionally not read

## Webhooks & Callbacks

**Incoming:**

- Auto-updater manifest pulls from
  `https://wealthfolio.app/releases/{{target}}/{{arch}}/{{current_version}}`
  (`apps/tauri/tauri.conf.json:40`) — outbound poll, not a true webhook
- Deep links (desktop): custom URL scheme `wealthfolio://` →
  `wealthfolio://auth/callback` handled by `tauri-plugin-deep-link`
  (`apps/tauri/src/lib.rs:256`, event name `deep-link-received`)
- Deep links (mobile): universal links on hosts `connect.wealthfolio.app` and
  `connect-staging.wealthfolio.app` with path prefix `/deeplink`
  (`apps/tauri/tauri.conf.json:45`)
- Server-Sent Events stream: `GET /api/v1/events/stream`
  (`apps/server/src/api/portfolio.rs:90`) — the frontend's web adapter
  subscribes with `EventSource` (`apps/frontend/src/adapters/web/events.ts:35`)
- AI chat SSE stream: `GET /api/v1/ai/chat/stream`
  (`apps/frontend/src/adapters/web/core.ts:15`,
  `apps/server/src/api/ai_chat.rs`)

**Outgoing:**

- Market-data HTTPS calls listed under `## APIs & External Services`
- Connect API calls: `POST`/`GET` against `https://api.wealthfolio.app` via
  `crates/connect/src/client.rs`
- Supabase auth calls from browser (SDK-managed OAuth/OTP redirects)
- OAuth callback bridge: `https://connect.wealthfolio.app/deeplink` redirects
  browsers back into the desktop/mobile app via deep link

## IPC Bridges

**Tauri ⇄ Frontend (desktop/mobile):**

- Frontend invokes via `@tauri-apps/api/core.invoke(command, payload)` wrapped
  in `apps/frontend/src/adapters/tauri/core.ts:34` with a 120 s timeout
- Commands registered in `apps/tauri/src/lib.rs:276`
  (`tauri::generate_handler!`) — modules in `apps/tauri/src/commands/*.rs`
  (account, activity, addon, ai_chat, ai_providers, alternative_assets, asset,
  brokers_sync, custom_provider, device_enroll_service, device_sync, fire, goal,
  health, limits, market_data, platform, portfolio, providers_settings, secrets,
  settings, sync_crypto, taxonomy, utilities, wealthfolio_connect)
- Event emission: `tauri::Emitter` → frontend `@tauri-apps/api/event.listen`
  (`apps/frontend/src/adapters/tauri/events.ts:6`). Event names:
  `tauri://file-drop*`, `portfolio:update-*`, `database-restored`,
  `market:sync-*`, `broker:sync-*`, `navigate-to-route`, `deep-link-received`,
  `asset-enrichment-*`
- Adapter pattern: `@/adapters` alias resolves to `src/adapters/tauri` or
  `src/adapters/web` at build time (`apps/frontend/vite.config.ts:43`), selected
  via `BUILD_TARGET` env var
- Shared adapter modules in `apps/frontend/src/adapters/shared/` contain
  platform-agnostic business logic (accounts, activities, portfolio,
  market-data, custom-provider, goals, taxonomies, alternative-assets,
  contribution-limits, exchange-rates, secrets, connect, ai-providers,
  ai-threads, health). Both `tauri/index.ts` and `web/index.ts` re-export them
  unchanged.

**Frontend ⇄ Axum server (web mode):**

- REST API rooted at `/api/v1`, command-to-route mapping in
  `apps/frontend/src/adapters/web/core.ts:19` (`COMMANDS` map)
- SSE for events (`/api/v1/events/stream`) and AI streaming
  (`/api/v1/ai/chat/stream`)
- Vite dev proxy forwards `/api` and `/docs` to `VITE_API_TARGET`
  (`apps/frontend/vite.config.ts:10`)
- CORS via `tower_http::cors::CorsLayer` with credentials when explicit origins
  are configured (`apps/server/src/api.rs:71`)
- Gzip compression, request timeouts, request-id propagation applied as tower
  middleware layers (`apps/server/src/api.rs:167`)

**Device sync transport (Rust `crates/device-sync/`):**

- `crates/device-sync/src/client.rs` — HTTPS client talking to Connect API
- `crates/device-sync/src/crypto.rs` — end-to-end encryption with ChaCha20-
  Poly1305 + X25519 + HKDF, pairing via SAS codes
- Snapshot/outbox engine: `crates/storage-sqlite/src/sync/app_sync/*` and
  `apps/server/src/api/device_sync_engine.rs`
- Background engine auto-starts when the device is enrolled
  (`apps/tauri/src/lib.rs:114`, `apps/server/src/main.rs:68`)

---

_Integration audit: 2026-04-20_
