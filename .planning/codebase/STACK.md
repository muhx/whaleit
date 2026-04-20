# Technology Stack

**Analysis Date:** 2026-04-20

## Languages

**Primary:**
- TypeScript 5.9 - Frontend application (`apps/frontend/`)
- Rust (edition 2021) - Backend logic, desktop shell, web server (`crates/`, `apps/tauri/`, `apps/server/`)

**Secondary:**
- SQL - SQLite migrations (`crates/storage-sqlite/migrations/`)
- CSS (Tailwind v4) - Styling (`apps/frontend/src/globals.css`)
- Shell - Dev scripts (`scripts/`)

## Runtime

**Environment:**
- Node.js 24 (specified in `.node-version`)
- Rust 1.91+ (Dockerfile uses `rust:1.91-alpine`)

**Package Manager:**
- pnpm 9.9+ (workspaces via `pnpm-workspace.yaml`)
- Lockfile: `pnpm-lock.yaml` (present)
- Cargo for Rust (lockfile: `Cargo.lock`)

## Frameworks

### Frontend

**Core:**
- React 19.2 - UI framework
- Vite 7.3 - Build tool and dev server
- Tailwind CSS 4.1 - Utility-first CSS framework
- shadcn/ui (via `@wealthfolio/ui`) - Component library built on Radix UI

**State Management:**
- Zustand 5.0 - Client state stores
- TanStack React Query 5.90 - Server state, caching, async queries
- TanStack React Table 8.21 - Table component logic
- TanStack React Virtual 3.13 - Virtualized lists

**Routing & Forms:**
- React Router DOM 7.13 - Client-side routing
- React Hook Form 7.71 - Form management
- Zod 3.25 - Schema validation

**Desktop Bridge:**
- Tauri 2.10 - Desktop application framework (via `@tauri-apps/api`)
- Tauri plugins: dialog, fs, shell, log, updater, window-state, barcode-scanner, haptics, deep-link

**AI Chat UI:**
- `@assistant-ui/react` 0.11 - Chat interface components
- `@assistant-ui/react-markdown` 0.11 - Markdown rendering in chat

**Visualization:**
- Recharts 3.7 - Charts and graphs
- `@number-flow/react` 0.5 - Animated number display

### Backend (Rust)

**Core:**
- `crates/core` (`wealthfolio-core`) - Business logic, domain models, services
- `crates/storage-sqlite` (`wealthfolio-storage-sqlite`) - Diesel ORM, repositories, migrations
- `crates/market-data` (`wealthfolio-market-data`) - Market data provider abstractions
- `crates/connect` (`wealthfolio-connect`) - Wealthfolio Connect cloud sync
- `crates/ai` (`wealthfolio-ai`) - AI assistant with LLM orchestration
- `crates/device-sync` (`wealthfolio-device-sync`) - E2EE device synchronization

**Desktop Shell:**
- Tauri 2.10 - Desktop framework (`apps/tauri/`)
- `keyring` 2.0 - OS keyring for secret storage

**Web Server:**
- Axum 0.8 - HTTP framework (`apps/server/`)
- Tower / tower-http 0.6 - Middleware (CORS, tracing, compression, timeout)
- `utoipa` 4 / `utoipa-swagger-ui` 4 - OpenAPI docs generation

## Database

**Engine:** SQLite (bundled via `rusqlite` 0.34 with `bundled` feature)

**ORM:** Diesel 2.2 with SQLite backend

**Connection Pooling:** r2d2 0.8

**Migrations:** Diesel migrations in `crates/storage-sqlite/migrations/` (28 migrations from 2023-11 to 2026-03)

## Key Dependencies

### TypeScript

| Package | Version | Purpose | Critical |
|---------|---------|---------|----------|
| `react` | 19.2 | UI framework | Yes |
| `@tanstack/react-query` | 5.90 | Server state management | Yes |
| `zustand` | 5.0 | Client state stores | Yes |
| `react-router-dom` | 7.13 | Routing | Yes |
| `zod` | 3.25 | Schema validation | Yes |
| `react-hook-form` | 7.71 | Form handling | Yes |
| `recharts` | 3.7 | Charting | Yes |
| `lucide-react` | 0.561 | Icon library | Yes |
| `cmdk` | 1.1 | Command palette | No |
| `motion` | 12.34 | Animations | No |
| `sonner` | 2.0 | Toast notifications | No |
| `@supabase/supabase-js` | 2.95 | Connect auth (Supabase Auth) | Conditional |
| `date-fns` | 4.1 | Date utilities | Yes |
| `lodash` | 4.17 | Utility functions | Yes |
| `@tauri-apps/api` | 2.10 | Desktop IPC bridge | Desktop only |

### Rust

| Package | Version | Purpose | Critical |
|---------|---------|---------|----------|
| `tokio` | 1 | Async runtime | Yes |
| `serde` / `serde_json` | 1 | Serialization | Yes |
| `diesel` | 2.2 | SQLite ORM | Yes |
| `rusqlite` | 0.34 | SQLite driver (bundled) | Yes |
| `reqwest` | 0.12 | HTTP client (rustls) | Yes |
| `thiserror` | 1 | Error types | Yes |
| `chrono` | 0.4 | Date/time | Yes |
| `uuid` | 1 | ID generation (v4, v7) | Yes |
| `rust_decimal` | 1.39 | Precise decimals | Yes |
| `rig-core` | 0.30 | LLM orchestration | Yes |
| `yahoo_finance_api` | 4.1 | Yahoo Finance data | Yes |
| `chacha20poly1305` | 0.10 | E2EE encryption | Yes |
| `x25519-dalek` | 2 | Key exchange (E2EE) | Yes |
| `keyring` | 2.0 | OS secret store (Tauri) | Desktop only |
| `axum` | 0.8 | HTTP server (web mode) | Web only |
| `jsonwebtoken` | 10 | JWT auth (web mode) | Web only |
| `argon2` | 0.5 | Password hashing (web mode) | Web only |

## Build & Run Commands

| Command | Purpose |
|---------|---------|
| `pnpm tauri dev` | Desktop app development (Tauri + Vite) |
| `pnpm run dev:web` | Web mode development (Axum server + Vite proxy) |
| `pnpm run build` | Build frontend (web target) |
| `pnpm run build:tauri` | Build frontend (Tauri target) |
| `pnpm test` | Run Vitest unit tests (frontend) |
| `pnpm test:watch` | Run Vitest in watch mode |
| `pnpm test:coverage` | Run tests with coverage |
| `pnpm test:e2e` | Run Playwright E2E tests |
| `pnpm run lint` | ESLint check (frontend + packages) |
| `pnpm run lint:fix` | ESLint auto-fix |
| `pnpm run format` | Prettier format all |
| `pnpm run type-check` | TypeScript type checking (all workspaces) |
| `pnpm run check` | Full check: format + lint + type-check |
| `cargo test` | Rust unit tests |
| `cargo check` | Rust type checking |
| `pnpm run build:addons` | Build all addons |
| `pnpm run bundle:addons` | Bundle all addons for distribution |

## Testing

**Frontend:**
- Vitest 3.2 - Unit test runner
- jsdom - Test environment
- Testing Library (`@testing-library/react` 16, `@testing-library/jest-dom` 6, `@testing-library/user-event` 14)
- Coverage via `@vitest/coverage-v8`

**E2E:**
- Playwright 1.58 - E2E tests (Chromium)
- Config: `playwright.config.ts`

**Rust:**
- Built-in `#[test]` framework
- `proptest` 1.4 for property-based testing
- `tempfile` 3 for test fixtures

## Build Configuration

**Vite:** `apps/frontend/vite.config.ts`
- Conditional alias resolution: `@/adapters` → `tauri/` or `web/` based on `BUILD_TARGET`
- `#platform` alias for platform-specific core modules
- Path alias: `@` → `apps/frontend/src/`
- Environment prefix: `VITE_`, `TAURI_`, `CONNECT_`

**TypeScript:** `tsconfig.base.json`
- Target: ES2022
- Module: ESNext with bundler resolution
- Strict mode with all strictness flags enabled
- JSX: react-jsx

**ESLint:** `eslint.config.js` + `eslint.base.config.js`
- Flat config format (ESLint 9)
- React, TanStack Query, React Hooks plugins

**Prettier:** `.prettierrc.cjs`
- 100 char print width
- Double quotes, trailing commas
- Tailwind CSS class sorting plugin

**Rust:**
- Workspace lints: `unsafe_code = "forbid"`, clippy all = warn
- Edition 2021, resolver 2

## Version Summary

- **Node.js:** 24 (from `.node-version`)
- **Rust:** 1.91+ (from Dockerfile)
- **pnpm:** 9.9+
- **React:** 19.2
- **Tauri:** 2.10
- **Vite:** 7.3
- **Tailwind CSS:** 4.1
- **TypeScript:** 5.9
- **Diesel:** 2.2
- **Axum:** 0.8

---

*Stack analysis: 2026-04-20*
