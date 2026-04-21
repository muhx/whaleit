# Technology Stack

**Analysis Date:** 2026-04-20

## Languages

**Primary:**

- TypeScript 5.9 (`tsconfig.base.json`) — React frontend, shared packages,
  addons, scripts
- Rust (edition 2021, workspace version 3.2.1) — Tauri desktop/mobile app, Axum
  web server, core portfolio/market-data/AI crates

**Secondary:**

- JavaScript/MJS — Build and dev scripts (`scripts/dev-web.mjs`,
  `scripts/run-e2e.mjs`, `scripts/prep-e2e.mjs`,
  `scripts/wait-for-both-servers-to-be-ready.sh`)
- SQL — Diesel migrations under `crates/storage-sqlite/migrations/*`
- CSS — Tailwind v4 global styles in `apps/frontend/src/globals.css`
- Shell/Dockerfile — `Dockerfile`, `compose.yml`, `compose.dev.yml`
- Plist/XML — Apple bundle metadata in `apps/tauri/Info.plist`,
  `apps/tauri/Info.ios.plist`, `apps/tauri/Entitlements.plist`

## Runtime

**Environment:**

- Node.js 24 (`.node-version`, CI `node-version: "24"` in
  `.github/workflows/pr-check.yml`)
- Rust stable (`rust:1.91-alpine` in `Dockerfile`,
  `dtolnay/rust-toolchain@stable` in CI)
- Tauri v2 WebView runtime (Chromium on Windows, WebKit on macOS/iOS/Linux) —
  `apps/tauri/tauri.conf.json`
- Tokio async runtime
  (`tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }`
  in workspace `Cargo.toml`)
- Axum 0.8 HTTP server for web mode (`apps/server/Cargo.toml`)

**Package Manager:**

- pnpm 9 (root `package.json`, `pnpm-workspace.yaml`, Dockerfile pins
  `pnpm@9.9.0`)
- Lockfile: `pnpm-lock.yaml` (present, 478 KB)
- Cargo (`Cargo.lock` present at workspace root and in `apps/tauri/`,
  `apps/server/`)

## Frameworks

**Core:**

- React 19.2 (`apps/frontend/package.json`) — SPA UI
- Vite 7.3 + `@vitejs/plugin-react` 5.1 — frontend dev server & bundler
  (`apps/frontend/vite.config.ts`)
- Tailwind CSS 4.1 via `@tailwindcss/vite` — utility-first styling
- React Router 7.13 (`react-router-dom`) — client-side routing
  (`apps/frontend/src/routes.tsx`)
- Tauri v2 (Rust core `tauri 2.10`, JS API `@tauri-apps/api 2.10.1`) — desktop
  and mobile app shell (`apps/tauri/`)
- Axum 0.8 + Tower 0.5 + Tower-HTTP 0.6 — web HTTP server
  (`apps/server/Cargo.toml`)
- Diesel 2.2 (SQLite feature) + `diesel_migrations` — ORM and migrations
  (`crates/storage-sqlite/`)
- rig-core 0.30 — LLM orchestration for AI assistant (`crates/ai/Cargo.toml`)

**Testing:**

- Vitest 3.2 + `@vitest/coverage-v8` — frontend unit tests
  (`apps/frontend/package.json`)
- Playwright 1.58 — end-to-end tests (`playwright.config.ts`, `e2e/*.spec.ts`)
- jsdom 28 — DOM environment for Vitest (`apps/frontend/vite.config.ts` →
  `test: { environment: "jsdom" }`)
- Testing Library (`@testing-library/react 16.3`,
  `@testing-library/jest-dom 6.9`, `@testing-library/user-event 14.6`)
- `cargo test --workspace` for Rust unit tests (CI in
  `.github/workflows/pr-check.yml`)
- `proptest 1.4` property-based tests (`crates/core/Cargo.toml` dev-dep)
- `tempfile 3` for test fixtures (multiple crate dev-deps)

**Build/Dev:**

- TypeScript 5.9 + `tsc -b` (`package.json` → `"tsc": "tsc -b"`)
- tsup 8.5 — library bundling for `packages/addon-sdk`, `packages/ui`,
  `packages/addon-dev-tools`
- ESLint 9.39 (flat config) + `typescript-eslint 8.55`,
  `eslint-plugin-react 7.37`, `eslint-plugin-react-hooks 5.2`,
  `eslint-plugin-react-refresh 0.4`, `@tanstack/eslint-plugin-query 5.91`
  (`eslint.base.config.js`, `eslint.config.js`)
- Prettier 3.8 + `prettier-plugin-tailwindcss 0.6` (`.prettierrc.cjs`)
- `cargo fmt`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` (CI
  enforced)
- `tauri-build 2.5` build script (`apps/tauri/build.rs`)
- Docker multi-stage cross-compile using `tonistiigi/xx` (`Dockerfile`)

## Key Dependencies

**Critical (frontend):**

- `@tanstack/react-query` 5.90 — server-state caching (Tauri invoke + HTTP API)
- `@tanstack/react-table` 8.21 — data grids (activities, holdings)
- `@tanstack/react-virtual` 3.13 — virtualized lists
- `@tauri-apps/api` 2.10 — IPC bridge to Rust
  (`apps/frontend/src/adapters/tauri/core.ts`)
- `@supabase/supabase-js` 2.95 — Supabase auth client for Whaleit Connect
  (`apps/frontend/src/features/whaleit-connect/providers/whaleit-connect-provider.tsx`)
- `@assistant-ui/react` 0.11 + `@assistant-ui/react-markdown` — AI assistant UI
- `react-hook-form` 7.71 + `@hookform/resolvers` + `zod` 3.25 — forms + schema
  validation
- `recharts` 3.7 — charting
- `zustand` 5.0 — lightweight client state
- `date-fns` 4.1, `@internationalized/date` 3.11, `chrono-tz` (Rust side) — date
  handling
- `lodash` 4.17, `clsx` 2.1, `tailwind-merge` 3.4, `class-variance-authority`
  0.7 — utility libs
- `lucide-react` 0.561, `@phosphor-icons/react` 2.1 — icon sets
- `motion` 12.34 (Framer Motion successor) — animations
- `sonner` 2.0 — toast notifications
- `cmdk` 1.1 — command palette
- `remark-gfm` 4.0 — markdown rendering

**Critical (Rust):**

- `tokio 1`, `async-trait 0.1`, `futures 0.3` — async runtime primitives
  (workspace)
- `serde 1`, `serde_json 1`, `serde_with 3` — (de)serialization (workspace)
- `diesel 2.2` (sqlite, chrono, r2d2, numeric,
  returning_clauses_for_sqlite_3_35) and `rusqlite 0.34` (bundled) — SQLite
  access (workspace `Cargo.toml`)
- `r2d2 0.8` — connection pool
- `reqwest 0.12` (default-features = false, features = `json`, `rustls-tls`) —
  HTTP client (workspace)
- `rust_decimal 1.39` (+ `rust_decimal_macros`, `num-traits 0.2`) — decimal math
- `uuid 1` (v4, v7, serde) — IDs
- `chrono 0.4`, `chrono-tz 0.10`, `time 0.3` — time handling
- `thiserror 1`, `anyhow 1` — error handling
- `log 0.4`, `tracing 0.1`, `tracing-subscriber 0.3` (`fmt`, `env-filter`,
  `json`) — logging
- `chacha20poly1305 0.10`, `x25519-dalek 2`, `hkdf 0.12`, `sha2 0.10`,
  `rand 0.8` — E2E encryption primitives (device sync)
- `argon2 0.5`, `jsonwebtoken 10` (aws_lc_rs) — password hashing and JWT auth
  for web server (`apps/server/Cargo.toml`)
- `yahoo_finance_api 4.1` — Yahoo Finance market data
- `rig` (package = `rig-core 0.30`) — LLM orchestration
- `scraper 0.22`, `jsonpath-rust 0.7`, `csv 1.4`, `zip 2.2`,
  `chardetng`/`encoding_rs 0.8` — import/scraping utilities in
  `crates/core/Cargo.toml`
- `rayon 1`, `dashmap 6.1` — parallelism and concurrent maps

**Infrastructure (Rust):**

- Tauri plugins: `tauri-plugin-fs 2`, `tauri-plugin-dialog 2`,
  `tauri-plugin-shell 2`, `tauri-plugin-log 2`, `tauri-plugin-deep-link 2`,
  `tauri-plugin-updater 2` (desktop), `tauri-plugin-window-state 2` (desktop),
  `tauri-plugin-single-instance 2` (desktop, `deep-link` feature),
  `tauri-plugin-haptics 2` (mobile), `tauri-plugin-barcode-scanner` (mobile, git
  branch `v2`), `tauri-plugin-web-auth 1` (iOS only),
  `tauri-plugin-mobile-share 0.1.2` (iOS only) — see `apps/tauri/Cargo.toml`
- `tower-http` features: `cors`, `trace`, `compression-full`, `timeout`,
  `request-id`, `fs` (`apps/server/Cargo.toml`)
- `tower_governor 0.8` — rate limiting on `/auth/login`
  (`apps/server/src/api.rs:141`)
- `utoipa 4` + `utoipa-swagger-ui 4` — OpenAPI generation
  (`apps/server/src/api.rs:61`)
- `tokio-stream 0.1` — SSE streaming
- `hyper 0.14` — HTTP primitives
- `keyring 2.0` — OS keychain access on desktop
  (`apps/tauri/src/secret_store.rs`)
- `dotenvy 0.15` — .env loading in Tauri and server builds
- `local-ip-address 0.6`, `hostname 0.3`, `base64 0.22`, `urlencoding 2.1`
  (`apps/tauri/Cargo.toml`)

**Shared TypeScript packages (`packages/`):**

- `@whaleit/addon-sdk` (`packages/addon-sdk/`) — typed SDK for third-party
  addons
- `@whaleit/ui` (`packages/ui/`) — shadcn/ui-based component library built
  on Radix UI primitives (`@radix-ui/react-*`), `react-aria-components 1.15`,
  `react-day-picker 9.13`, `embla-carousel-react 8.6`, `input-otp 1.4`,
  `react-dropzone 14.4`, `react-number-format 5.4`
- `@whaleit/addon-dev-tools` (`packages/addon-dev-tools/`) — CLI and dev
  server for addon authors

## Configuration

**Environment:**

- Two env layers: `.env` (desktop/Tauri + shared `CONNECT_*` vars) and
  `.env.web` (Axum server)
- `.env.example` (present, example-only) covers `DATABASE_URL`,
  `CONNECT_AUTH_URL`, `CONNECT_AUTH_PUBLISHABLE_KEY`, `CONNECT_API_URL`,
  `CONNECT_OAUTH_CALLBACK_URL`
- `.env.web.example` (present) covers `WF_LISTEN_ADDR`, `WF_DB_PATH`,
  `WF_CORS_ALLOW_ORIGINS`, `WF_REQUEST_TIMEOUT_MS`, `WF_STATIC_DIR`,
  `WF_SECRET_KEY`, `WF_AUTH_PASSWORD_HASH`, `WF_AUTH_TOKEN_TTL_MINUTES`,
  `WF_SECRET_FILE`, `WF_ADDONS_DIR`, `VITE_API_TARGET`
- Additional server envs parsed in `apps/server/src/config.rs`:
  `WF_COOKIE_SECURE` (`auto|true|false`), `WF_AUTH_REQUIRED`
- Tauri build script `apps/tauri/build.rs` reads `CONNECT_API_URL`,
  `CONNECT_AUTH_URL`, `CONNECT_AUTH_PUBLISHABLE_KEY` from env or root `.env` and
  bakes them as `cargo:rustc-env=...` at compile time
- Vite exposes env vars with prefixes `VITE_`, `TAURI_`, `CONNECT_`
  (`apps/frontend/vite.config.ts:81`)
- `.env`, `.env.web` are present but gitignored — never read by the mapper

**Build:**

- `apps/frontend/vite.config.ts` — path aliases `@`, `@whaleit/addon-sdk`,
  `@whaleit/ui`, `@/adapters`, `#platform`; build target switched via
  `BUILD_TARGET=tauri|web`; dev port 1420 (fixed, strictPort)
- `tsconfig.base.json`, `tsconfig.json`, `tsconfig.node.json`,
  `tsconfig.test.json` at root
- `apps/tauri/tauri.conf.json` — bundle config (identifier
  `com.teymz.whaleit`, iOS team `DYDJ2RNL5H`, macOS signing, auto-updater
  endpoint `https://whaleit.app/releases/...`, deep-link schemes
  `whaleit://`)
- `apps/tauri/capabilities/desktop.json`, `mobile.json`, `ios.json` — Tauri
  permission capabilities (fs scope limited to `$APPDATA/**`)
- `crates/storage-sqlite/diesel.toml` — Diesel schema config
- `Dockerfile` (multi-stage: `node:24-alpine` frontend + `rust:1.91-alpine`
  backend + `tonistiigi/xx` cross-compile + `alpine:3.19` runtime)
- `compose.yml` production compose, `compose.dev.yml` dev overlay,
  `.dockerignore`
- `playwright.config.ts` — test dir `./e2e`, Chromium only, single worker,
  `fullyParallel: false` (ordering required for onboarding → activities)
- `.github/workflows/pr-check.yml`, `release.yml`, `docker-publish.yml`
- `eslint.base.config.js`, `eslint.config.js`, `.prettierrc.cjs`,
  `.prettierignore`

## Platform Requirements

**Development:**

- Node.js 24, pnpm 9 (CI uses `pnpm/action-setup@v4 version: 9`,
  `actions/setup-node@v4 node-version: "24"`)
- Rust stable toolchain (`rustfmt`, `clippy`)
- System libs for Tauri on Linux CI: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`,
  `librsvg2-dev` (`.github/workflows/pr-check.yml:72`)
- Docker optional for web-mode containerized build

**Production targets:**

- Desktop: macOS 10.13+, Windows, Linux (Tauri v2 bundle, targets `"all"` in
  `apps/tauri/tauri.conf.json`)
- Mobile: iOS (Apple dev team `DYDJ2RNL5H`, `Info.ios.plist`, signed
  `Apple Distribution: Teymz Inc`), Android (Tauri mobile targets)
- Web/server: Alpine Linux container (`alpine:3.19`) exposing port 8088; data
  volume `/data`, `WF_DB_PATH=/data/whaleit.db`; `muhx/whaleit:latest`
  image on Docker Hub per `compose.yml`
- Auto-updater endpoint:
  `https://whaleit.app/releases/{{target}}/{{arch}}/{{current_version}}`
  (`apps/tauri/tauri.conf.json:40`)

---

_Stack analysis: 2026-04-20_
