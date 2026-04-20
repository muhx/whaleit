# External Integrations

**Analysis Date:** 2026-04-20

## API Architecture

**Dual Runtime: Desktop (Tauri IPC) and Web (REST)**

The application runs in two modes with a shared codebase:
- **Desktop mode:** React frontend communicates with Rust backend via Tauri IPC (`invoke()`)
- **Web mode:** React frontend communicates with Axum HTTP server via REST API (`/api/v1/*`)

The adapter pattern in `apps/frontend/src/adapters/` abstracts the runtime:
- `apps/frontend/src/adapters/tauri/` - Tauri IPC command wrappers
- `apps/frontend/src/adapters/web/` - REST API fetch wrappers
- `apps/frontend/src/adapters/shared/` - Platform-agnostic logic
- Vite alias `@/adapters` resolves to the correct adapter at build time based on `BUILD_TARGET`

**Web API (REST):**
- Base path: `/api/v1/`
- Versioning: URL-based (`/api/v1/...`)
- Auth: JWT session cookies + Bearer tokens (web mode only)
- OpenAPI docs via `utoipa` + Swagger UI at `/docs`
- SSE endpoint for real-time events: `/api/v1/events/stream`
- SSE endpoint for AI chat streaming: `/api/v1/ai/chat/stream`

**Key API handler locations:**
- Tauri commands: `apps/tauri/src/commands/*.rs`
- Web endpoints: `apps/server/src/api/*.rs`

## External Services

| Service | Purpose | Auth Method | Used In | Critical |
|---------|---------|-------------|---------|----------|
| Wealthfolio Connect API | Broker sync, device sync, subscriptions | OAuth2 (Supabase Auth) + JWT | `crates/connect/`, `crates/device-sync/` | Optional |
| Supabase Auth | Connect feature authentication | Publishable key | `apps/frontend/src/features/wealthfolio-connect/` | Optional |
| Yahoo Finance | Stock quotes and market data | Public (no API key) | `crates/market-data/src/provider/yahoo/` | Yes |
| Alpha Vantage | Market data provider | API key | `crates/market-data/src/provider/alpha_vantage/` | Optional |
| Finnhub | Market data provider | API key | `crates/market-data/src/provider/finnhub/` | Optional |
| MarketData.app | Market data provider | API key | `crates/market-data/src/provider/marketdata_app/` | Optional |
| Boerse Frankfurt | European market data | Public | `crates/market-data/src/provider/boerse_frankfurt/` | Optional |
| Metal Price API | Precious metals pricing | API key | `crates/market-data/src/provider/metal_price_api/` | Optional |
| OpenFIGI | Financial instrument identification | API key | `crates/market-data/src/provider/openfigi/` | Optional |
| US Treasury | Treasury yield calculations | Public | `crates/market-data/src/provider/us_treasury_calc/` | Optional |
| Ollama | Local LLM inference | None (local) | `crates/ai/` | Optional |
| OpenAI | GPT models for AI assistant | API key | `crates/ai/` | Optional |
| Anthropic | Claude models for AI assistant | API key | `crates/ai/` | Optional |
| Google AI | Gemini models for AI assistant | API key | `crates/ai/` | Optional |
| Groq | Fast inference cloud | API key | `crates/ai/` | Optional |
| OpenRouter | Multi-model AI gateway | API key | `crates/ai/` | Optional |

## Data Providers / Market Data

**Architecture:** Provider-agnostic system with trait-based abstraction.

**Core trait:** `MarketDataProvider` in `crates/market-data/src/provider/traits.rs`

**Provider implementations:** `crates/market-data/src/provider/`

| Provider | Module | Data Type | Auth |
|----------|--------|-----------|------|
| Yahoo Finance | `yahoo/mod.rs` | Stock quotes, historical data | None |
| Alpha Vantage | `alpha_vantage/` | Global equities, forex | API key (env) |
| Finnhub | `finnhub/` | Stock quotes | API key (env) |
| MarketData.app | `marketdata_app/` | US market data | API key (env) |
| Boerse Frankfurt | `boerse_frankfurt/` | German/European equities | None |
| Metal Price API | `metal_price_api/` | Gold, silver, metals | API key (env) |
| OpenFIGI | `openfigi/` | Instrument ID mapping (ISIN, CUSIP → ticker) | API key (env) |
| US Treasury | `us_treasury_calc/` | Treasury yield calculations | None |

**Provider resolution:** `crates/market-data/src/resolver/` - Maps canonical `InstrumentId` to provider-specific parameters.

**Provider registry:** `crates/market-data/src/registry/` - Manages provider lifecycle, rate limiting, circuit breakers.

**Custom providers:** Users can configure custom market data sources via `crates/core/src/custom_provider/`.

## Database & Storage

**Engine:** SQLite (single-file, local-first)

**ORM:** Diesel 2.2 with SQLite backend

**Key files:**
- Schema models: `crates/core/src/` (domain modules)
- Repository implementations: `crates/storage-sqlite/`
- Migrations: `crates/storage-sqlite/migrations/` (28 migrations)
- Connection pooling: r2d2

**Migration pattern:** Timestamped directories with `up.sql` files (e.g., `2026-01-01-000001_quotes_market_data/up.sql`)

**Database location:**
- Desktop: Configured via Tauri app data directory
- Web: `WF_DB_PATH` env var (default: `./db/app.db`)
- Docker: `/data/wealthfolio.db` with volume mount

**File Storage:**
- Desktop: Local filesystem via Tauri FS plugin
- Web: Local filesystem via server process
- Database backup/restore: Built-in via `backup_database` / `restore_database` commands

**Caching:**
- In-memory: `dashmap` for concurrent caches in `crates/core/`
- React Query: Frontend query caching with `@tanstack/react-query`

## Authentication & Security

### Desktop Mode
- **No user authentication required** - All data is local
- **Secret storage:** OS keyring via `keyring` crate
  - Implementation: `apps/tauri/src/secret_store.rs` (`KeyringSecretStore`)
  - Trait: `crates/core/src/secrets/` (`SecretStore`)
  - Keys stored: AI provider API keys, Connect tokens, E2EE keys

### Web Mode
- **Password-based auth:** Argon2id password hash
  - Implementation: `apps/server/src/auth.rs`
  - JWT tokens (HS256) with configurable TTL (default 60 min)
  - Session cookies: `wf_session` (HttpOnly, SameSite=Lax)
  - Sliding session refresh at 50% TTL
- **Key derivation:** HKDF-SHA256 derives separate JWT signing key and secrets encryption key from master `WF_SECRET_KEY`
- **Secret storage:** Encrypted in SQLite (using derived key)
  - Implementation: `apps/server/src/secrets/`
- **CORS:** Configurable via `WF_CORS_ALLOW_ORIGINS`
- **Rate limiting:** `tower_governor` 0.8

### Wealthfolio Connect Auth
- **Supabase Auth** for Connect feature OAuth flow
  - Client: `@supabase/supabase-js` in `apps/frontend/src/features/wealthfolio-connect/providers/wealthfolio-connect-provider.tsx`
  - Auth URL: `CONNECT_AUTH_URL` (default: `https://auth.wealthfolio.app`)
  - Token lifecycle: `crates/connect/src/token_lifecycle.rs`
  - Token storage: OS keyring (desktop) / encrypted DB (web)

### E2EE (Device Sync)
- **X25519** key exchange for device pairing
- **ChaCha20-Poly1305** for data encryption
- **HKDF-SHA256** for key derivation
- Implementation: `crates/device-sync/src/crypto.rs`

## Communication Patterns

### Tauri IPC (Desktop)
- Frontend → Backend: `@tauri-apps/api` `invoke()` calls
- Backend → Frontend: Tauri events (`apps/tauri/src/events.rs`)
- Command definitions: `apps/tauri/src/commands/*.rs`

### REST API (Web)
- Frontend → Backend: `fetch()` calls to `/api/v1/*`
- Backend → Frontend: Server-Sent Events (SSE) for real-time updates
  - Events: `/api/v1/events/stream`
  - AI chat: `/api/v1/ai/chat/stream`

### Cloud API (Connect / Device Sync)
- REST API to `https://api.wealthfolio.app`
- Bearer token authentication
- Endpoints:
  - Broker sync: `/api/v1/sync/brokerage/*`
  - Device management: `/api/v1/sync/team/devices/*`
  - Key management: `/api/v1/sync/team/keys/*`
  - Snapshot sync: `/api/v1/sync/snapshots/*`
  - Event sync: `/api/v1/sync/events/*`
  - Pairing: `/api/v1/sync/team/devices/{id}/pairings/*`
  - User info: `/api/v1/user/me`
  - Subscriptions: `/api/v1/subscription/plans`

### Deep Links
- Desktop: `wealthfolio://` custom URL scheme
- Mobile: `connect.wealthfolio.app/deeplink`
- Config: `apps/tauri/tauri.conf.json` → `plugins.deep-link`

## AI / LLM Integration

**Framework:** `rig-core` 0.30 (Rust LLM orchestration)

**Provider catalog:** `crates/ai/src/ai_providers.json` - Static JSON with all supported providers, models, and capabilities.

**Supported providers:**

| Provider ID | Type | Default Model | Capabilities |
|-------------|------|---------------|--------------|
| `ollama` | local | `gemma4:e4b` | tools, vision |
| `openai` | api | `gpt-5.4-mini` | tools, thinking, vision |
| `anthropic` | api | `claude-sonnet-4-6` | tools, vision |
| `google` | api | `gemini-2.5-flash` | tools, thinking, vision |
| `groq` | api | `openai/gpt-oss-120b` | tools, thinking |
| `openrouter` | api | `openrouter/free` | tools, vision |

**Chat orchestration:** `crates/ai/src/chat.rs` - Multi-turn tool execution with streaming.

**AI Tools (function calling):** `crates/ai/src/tools/`

| Tool | Purpose |
|------|---------|
| `GetAccountsTool` | Fetch active investment accounts |
| `GetHoldingsTool` | Fetch portfolio holdings |
| `GetAssetAllocationTool` | Calculate portfolio allocation |
| `GetCashBalancesTool` | Fetch cash balances |
| `SearchActivitiesTool` | Search transactions |
| `GetIncomeTool` | Fetch income summaries |
| `GetValuationHistoryTool` | Fetch valuation history |
| `GetGoalsTool` | Fetch investment goals |
| `GetPerformanceTool` | Fetch performance metrics |
| `RecordActivityTool` | Create activity from natural language |
| `RecordActivitiesTool` | Batch create activities |
| `ImportCsvTool` | Import CSV data |
| `GetHealthStatusTool` | Get portfolio health status |

**API key storage:** Per-provider secrets stored as `ai_<provider_id>` in OS keyring (desktop) or encrypted DB (web).

## File System & OS Integration

**Desktop (Tauri):**
- File system access: `@tauri-apps/plugin-fs`
- File dialogs: `@tauri-apps/plugin-dialog`
- Shell access: `@tauri-apps/plugin-shell`
- Auto-update: `@tauri-apps/plugin-updater` with endpoint `https://wealthfolio.app/releases/...`
- Window state: `@tauri-apps/plugin-window-state`
- Barcode scanner: `@tauri-apps/plugin-barcode-scanner` (mobile)
- Haptics: `@tauri-apps/plugin-haptics` (mobile)
- Web auth: `tauri-plugin-web-auth-api` (iOS)
- Mobile share: `tauri-plugin-mobile-share` (mobile)
- OS keyring: `keyring` crate for secret storage
- Deep links: `tauri-plugin-deep-link`

## Package Ecosystem

### Internal Packages

| Package | Name | Location | Purpose |
|---------|------|----------|---------|
| UI components | `@wealthfolio/ui` | `packages/ui/` | shadcn/ui-based component library (Radix UI + Tailwind) |
| Addon SDK | `@wealthfolio/addon-sdk` | `packages/addon-sdk/` | TypeScript SDK for building Wealthfolio addons |
| Addon Dev Tools | `@wealthfolio/addon-dev-tools` | `packages/addon-dev-tools/` | CLI and hot-reload dev server for addon development |

### Internal Rust Crates

| Crate | Location | Purpose |
|-------|----------|---------|
| `wealthfolio-core` | `crates/core/` | Business logic, domain models, services |
| `wealthfolio-storage-sqlite` | `crates/storage-sqlite/` | Diesel ORM, repositories, migrations |
| `wealthfolio-market-data` | `crates/market-data/` | Market data provider abstractions and implementations |
| `wealthfolio-connect` | `crates/connect/` | Wealthfolio Connect cloud API client and broker sync |
| `wealthfolio-ai` | `crates/ai/` | AI assistant with LLM orchestration (rig-core) |
| `wealthfolio-device-sync` | `crates/device-sync/` | E2EE device synchronization engine |

### Sample Addons (in `addons/`)

- `goal-progress-tracker/`
- `investment-fees-tracker/`
- `swingfolio-addon/`

### Crate Dependency Graph

```
wealthfolio-core
  ├── wealthfolio-market-data

wealthfolio-storage-sqlite
  ├── wealthfolio-core
  ├── wealthfolio-ai
  ├── wealthfolio-connect
  └── wealthfolio-device-sync

wealthfolio-connect
  └── wealthfolio-core

wealthfolio-ai
  └── wealthfolio-core

wealthfolio-device-sync
  └── wealthfolio-core

apps/tauri (wealthfolio-app)
  ├── wealthfolio-core
  ├── wealthfolio-storage-sqlite
  ├── wealthfolio-market-data
  ├── wealthfolio-connect
  ├── wealthfolio-device-sync
  └── wealthfolio-ai

apps/server (wealthfolio-server)
  ├── wealthfolio-core
  ├── wealthfolio-storage-sqlite
  ├── wealthfolio-market-data
  ├── wealthfolio-connect
  ├── wealthfolio-device-sync
  └── wealthfolio-ai
```

## CI/CD & Deployment

**Hosting:**
- Desktop: Native installers via Tauri (macOS, Windows, Linux, iOS, Android)
- Web: Self-hosted Docker container (`afadil/wealthfolio:latest`)
- Auto-update: Tauri updater with `https://wealthfolio.app/releases/...` endpoint

**CI Pipeline:** GitHub Actions (`.github/workflows/`)
- `pr-check.yml` - PR validation
- `docker-publish.yml` - Docker image build and publish
- `release.yml` - Release automation

**Docker:** `Dockerfile` with multi-stage build
- Stage 1: Frontend build (Node 24 Alpine)
- Stage 2: Rust server cross-compilation (rust:1.91 Alpine)
- Final: Alpine 3.19 runtime (minimal, read-only filesystem)

**Docker Compose:** `compose.yml` for production deployment
- Health check: `GET /api/v1/healthz`
- Volume mount for SQLite data persistence
- Memory limits: 512M max, 128M reserved

## Environment Configuration

**Desktop:**
- `DATABASE_URL` - SQLite database path
- `CONNECT_AUTH_URL` - Connect auth provider URL
- `CONNECT_AUTH_PUBLISHABLE_KEY` - Connect auth publishable key
- `CONNECT_API_URL` - Connect API URL
- `CONNECT_OAUTH_CALLBACK_URL` - OAuth callback URL

**Web Server (all prefixed with `WF_`):**
- `WF_LISTEN_ADDR` - Server listen address (default: `0.0.0.0:8088`)
- `WF_DB_PATH` - SQLite database path
- `WF_SECRET_KEY` - Master encryption key (base64 or 32-byte ASCII, **required**)
- `WF_AUTH_PASSWORD_HASH` - Argon2id password hash for web auth
- `WF_AUTH_TOKEN_TTL_MINUTES` - JWT token TTL (default: 60)
- `WF_AUTH_REQUIRED` - Set `false` to skip auth (default: `true` on non-loopback)
- `WF_CORS_ALLOW_ORIGINS` - CORS allowed origins (default: `*`)
- `WF_REQUEST_TIMEOUT_MS` - Request timeout (default: 300000)
- `WF_STATIC_DIR` - Frontend static files directory (default: `dist`)
- `WF_ADDONS_DIR` - Addons root directory
- `WF_COOKIE_SECURE` - Cookie Secure attribute policy (default: `auto`)

**Frontend (Vite, prefixed with `VITE_`):**
- `VITE_API_TARGET` / `WF_API_TARGET` - API server URL for Vite proxy
- `WF_ENABLE_VITE_PROXY` - Enable Vite dev proxy (set `true`)
- `CONNECT_AUTH_URL` - Baked into frontend bundle at build time
- `CONNECT_AUTH_PUBLISHABLE_KEY` - Baked into frontend bundle at build time

**Secrets location:**
- Desktop: OS keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Web: Encrypted in SQLite database via HKDF-derived key

## Webhooks & Callbacks

**Incoming:**
- Deep link callbacks: `wealthfolio://auth/callback` (desktop), `connect.wealthfolio.app/deeplink` (mobile)
- OAuth callback for Connect auth flow

**Outgoing:**
- Connect API requests to `https://api.wealthfolio.app`
- Market data provider API calls (various endpoints)
- AI provider API calls (OpenAI, Anthropic, Google, Groq, OpenRouter endpoints)

---

*Integration audit: 2026-04-20*
