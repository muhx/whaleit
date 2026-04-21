# AGENTS.md

AI agent guide for this repository. Covers behavioral rules, architecture, and
common task playbooks.

---

## Behavioral Guidelines

**These come first because they prevent the most mistakes.**

### 1. Think Before Coding

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them—don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.

### 2. Simplicity First

- No features beyond what was asked.
- No abstractions for single-use code.
- No error handling for impossible scenarios.
- If 200 lines could be 50, rewrite it.

### 3. Surgical Changes

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated issues, mention them—don't fix them.
- Remove only what YOUR changes made unused.

### 4. Goal-Driven Execution

- Transform tasks into verifiable goals.
- For multi-step tasks, state a brief plan with verification steps.
- Unverified work is incomplete work.

### 5. Output Precision

- Lead with findings, not process descriptions.
- Use structured formats (lists, tables, code blocks).
- Include absolute file paths—never relative.

---

## Overview

- **Frontend**: React + Vite + Tailwind v4 + shadcn (`apps/frontend/`)
- **Desktop**: Tauri/Rust with SQLite (`apps/tauri/`, `crates/`)
- **Web mode**: Axum HTTP server (`apps/server/`)
- **Packages**: `@whaleit/ui`, addon-sdk, addon-dev-tools (`packages/`)

## Code Layout

```
apps/frontend/
├── src/
│   ├── pages/          # Route pages
│   ├── components/     # Shared components
│   ├── features/       # Self-contained feature modules
│   ├── commands/       # Backend call wrappers (Tauri/Web)
│   ├── adapters/       # Runtime detection (desktop vs web)
│   └── addons/         # Addon runtime

apps/tauri/src/
└── commands/           # Tauri IPC commands

apps/server/src/
└── api/                # Axum HTTP handlers

crates/
├── core/               # Business logic, models, services
├── storage-sqlite/     # Diesel ORM, repositories, migrations
├── market-data/        # Market data providers
├── connect/            # External integrations
├── device-sync/        # Device sync, E2EE
└── ai/                 # AI providers and LLM integration
```

## Run Targets

| Task         | Command            |
| ------------ | ------------------ |
| Desktop dev  | `pnpm tauri dev`   |
| Web dev      | `pnpm run dev:web` |
| Tests (TS)   | `pnpm test`        |
| Tests (Rust) | `cargo test`       |
| Type check   | `pnpm type-check`  |
| Lint         | `pnpm lint`        |
| All checks   | `pnpm check`       |

---

## Agent Playbook

### Adding a feature with backend data

1. **Frontend route/UI** → `apps/frontend/src/pages/`,
   `apps/frontend/src/routes.tsx`
2. **Command wrapper** → `apps/frontend/src/commands/<domain>.ts` (follow
   `RUN_ENV` pattern)
3. **Tauri command** → `apps/tauri/src/commands/*.rs`, wire in `mod.rs` +
   `lib.rs`
4. **Web endpoint** → `apps/server/src/api/`, call `crates/core` service
5. **Core logic** → `crates/core/` services/repos
6. **Tests** → Vitest for TS, `#[test]` for Rust

### UI patterns

- Components: `@whaleit/ui` and `packages/ui/src/components/`
- Forms: `react-hook-form` + `zod` schemas from
  `apps/frontend/src/lib/schemas.ts`
- Theme: tokens in `apps/frontend/src/globals.css`

### Architecture pattern

```
Frontend → Adapter (tauri/web) → Command wrapper
                ↓
        Tauri IPC  |  Axum HTTP
                ↓
           crates/core (business logic)
                ↓
           crates/storage-sqlite
```

---

## Conventions

### TypeScript

- Strict mode, no unused locals/params
- Prefer interfaces over types, avoid enums
- Functional components, named exports
- Directory names: lowercase-with-dashes

### Rust

- Idiomatic Rust, small focused functions
- `Result`/`Option`, propagate with `?`, `thiserror` for domain errors
- Keep Tauri/Axum commands thin—delegate to `crates/core`
- Migrations in `crates/storage-sqlite/migrations`

### Security

- All data local (SQLite), no cloud
- Secrets via OS keyring—never disk/localStorage
- Never log secrets or financial data

---

## Validation Checklist

Before completing any task:

- [ ] Builds: `pnpm build` or `pnpm tauri dev` or `cargo check`
- [ ] Tests pass: `pnpm test` and/or `cargo test`
- [ ] Both desktop and web compile if touching shared code
- [ ] Changes are minimal and surgical

---

## Plan Mode

- Make plans extremely concise. Sacrifice grammar for brevity.
- End with unresolved questions, if any.

---

When in doubt, follow the nearest existing pattern.

<!-- GSD:project-start source:PROJECT.md -->
## Project

**WhaleIt**

WhaleIt (rebranded from Whaleit) is a local-first personal finance management application that helps individuals and freelancers track their entire financial life — investments, bank accounts, credit cards, subscriptions, budgets, and daily transactions — all in one place. It runs as a desktop app (Tauri/Rust) and a self-hosted web app (Axum), with an AI-powered assistant that makes financial record-keeping effortless. Your friendly finance companion.

**Core Value:** Users can effortlessly track and understand their complete financial picture — investments, spending, budgets, and subscriptions — with AI doing the heavy lifting to categorize, suggest, and advise.

### Constraints

- **Dual DB engine:** Must support both SQLite (desktop) and PostgreSQL (web) through shared repository traits — existing Diesel ORM supports both backends
- **Local-first:** Desktop mode must work fully offline with local SQLite; no mandatory cloud dependency
- **Existing architecture:** Must extend, not replace, the current adapter pattern, repository traits, and domain event system
- **Tech stack:** Rust backend (crates), React/TypeScript frontend, Diesel ORM, Tauri v2 + Axum — all established
- **Self-hosted:** Web mode must remain self-hostable via Docker; no mandatory SaaS dependencies
- **Privacy:** All financial data stays local or on user's own server; AI calls go to user-chosen providers
- **AI providers:** Support OpenAI, Anthropic, Google — user brings their own API key or uses hosted option
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- TypeScript 5.9 - Frontend application (`apps/frontend/`)
- Rust (edition 2021) - Backend logic, desktop shell, web server (`crates/`, `apps/tauri/`, `apps/server/`)
- SQL - SQLite migrations (`crates/storage-sqlite/migrations/`)
- CSS (Tailwind v4) - Styling (`apps/frontend/src/globals.css`)
- Shell - Dev scripts (`scripts/`)
## Runtime
- Node.js 24 (specified in `.node-version`)
- Rust 1.91+ (Dockerfile uses `rust:1.91-alpine`)
- pnpm 9.9+ (workspaces via `pnpm-workspace.yaml`)
- Lockfile: `pnpm-lock.yaml` (present)
- Cargo for Rust (lockfile: `Cargo.lock`)
## Frameworks
### Frontend
- React 19.2 - UI framework
- Vite 7.3 - Build tool and dev server
- Tailwind CSS 4.1 - Utility-first CSS framework
- shadcn/ui (via `@whaleit/ui`) - Component library built on Radix UI
- Zustand 5.0 - Client state stores
- TanStack React Query 5.90 - Server state, caching, async queries
- TanStack React Table 8.21 - Table component logic
- TanStack React Virtual 3.13 - Virtualized lists
- React Router DOM 7.13 - Client-side routing
- React Hook Form 7.71 - Form management
- Zod 3.25 - Schema validation
- Tauri 2.10 - Desktop application framework (via `@tauri-apps/api`)
- Tauri plugins: dialog, fs, shell, log, updater, window-state, barcode-scanner, haptics, deep-link
- `@assistant-ui/react` 0.11 - Chat interface components
- `@assistant-ui/react-markdown` 0.11 - Markdown rendering in chat
- Recharts 3.7 - Charts and graphs
- `@number-flow/react` 0.5 - Animated number display
### Backend (Rust)
- `crates/core` (`whaleit-core`) - Business logic, domain models, services
- `crates/storage-sqlite` (`whaleit-storage-sqlite`) - Diesel ORM, repositories, migrations
- `crates/market-data` (`whaleit-market-data`) - Market data provider abstractions
- `crates/connect` (`whaleit-connect`) - Whaleit Connect cloud sync
- `crates/ai` (`whaleit-ai`) - AI assistant with LLM orchestration
- `crates/device-sync` (`whaleit-device-sync`) - E2EE device synchronization
- Tauri 2.10 - Desktop framework (`apps/tauri/`)
- `keyring` 2.0 - OS keyring for secret storage
- Axum 0.8 - HTTP framework (`apps/server/`)
- Tower / tower-http 0.6 - Middleware (CORS, tracing, compression, timeout)
- `utoipa` 4 / `utoipa-swagger-ui` 4 - OpenAPI docs generation
## Database
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
- Vitest 3.2 - Unit test runner
- jsdom - Test environment
- Testing Library (`@testing-library/react` 16, `@testing-library/jest-dom` 6, `@testing-library/user-event` 14)
- Coverage via `@vitest/coverage-v8`
- Playwright 1.58 - E2E tests (Chromium)
- Config: `playwright.config.ts`
- Built-in `#[test]` framework
- `proptest` 1.4 for property-based testing
- `tempfile` 3 for test fixtures
## Build Configuration
- Conditional alias resolution: `@/adapters` → `tauri/` or `web/` based on `BUILD_TARGET`
- `#platform` alias for platform-specific core modules
- Path alias: `@` → `apps/frontend/src/`
- Environment prefix: `VITE_`, `TAURI_`, `CONNECT_`
- Target: ES2022
- Module: ESNext with bundler resolution
- Strict mode with all strictness flags enabled
- JSX: react-jsx
- Flat config format (ESLint 9)
- React, TanStack Query, React Hooks plugins
- 100 char print width
- Double quotes, trailing commas
- Tailwind CSS class sorting plugin
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
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## TypeScript Conventions
### Naming
- Components: `kebab-case.tsx` — e.g., `privacy-toggle.tsx`, `header.tsx`, `accounts-summary.tsx`
- Hooks: `use-kebab-case.ts` — e.g., `use-accounts.ts`, `use-settings.ts`
- Utilities/lib: `kebab-case.ts` — e.g., `activity-utils.ts`, `query-keys.ts`
- Types: `types.ts` (single file per domain area) — e.g., `src/lib/types.ts`
- Tests: co-located `*.test.ts` or `*.test.tsx` — e.g., `activity-utils.test.ts`
- Test groups within `__tests__/` subdirectory — e.g., `components/forms/__tests__/buy-form.test.tsx`
- `camelCase` for variables and functions — e.g., `formatAmount`, `parseDecimalInput`
- `PascalCase` for React components, types, interfaces, enums — e.g., `AccountSummaryView`, `ActivityType`
- `UPPER_SNAKE_CASE` for constants — e.g., `DECIMAL_PRECISION`, `DISPLAY_DECIMAL_PRECISION`
- Prefer `interface` for object shapes — e.g., `Account`, `Settings`, `Activity`
- Use `type` for unions, intersections, utility types — e.g., `type TrackingMode = "TRANSACTIONS" | "HOLDINGS" | "NOT_SET"`
- Avoid `enum` — use `const` objects with `as const` and derived types instead:
### Export Patterns
- **Named exports** for everything — no default exports except `App.tsx`
- Components: `export function ComponentName()` — always named function declarations
- Utilities: `export function utilityName()` or `export const utilityName = () => {}`
- Barrel exports via `index.ts` files — e.g., `src/hooks/index.ts` uses `export * from "./use-accounts"`
- Types file re-exports from constants: `export { AccountType, ActivityType } from "./constants"`
### Import Organization
- `@/*` → `./src/*`
- `@/adapters` → resolves to `./src/adapters/tauri` or `./src/adapters/web` based on `BUILD_TARGET`
- `#platform` → `./src/adapters/tauri/core` or `./src/adapters/web/core`
- `@whaleit/ui` → `../../packages/ui/src`
- `@whaleit/addon-sdk` → `../../packages/addon-sdk/src`
### Component Patterns
- **Functional components only** — no class components
- Named function declarations: `export function ComponentName(props: Props)`
- Props defined as inline `interface` above the component
- Destructure props in function signature
- Use `cn()` utility for conditional class merging:
- Wrap with `React.StrictMode` at root (`src/main.tsx`)
### State Management
- **Server state:** `@tanstack/react-query` — all backend data goes through React Query
- **Client state:** `zustand` for complex client state
- **Form state:** `react-hook-form` + `zod` schemas
- **Context:** React Context for providers — e.g., `auth-context.tsx`, `privacy-context.tsx`
### Error Handling
- Use `Result`-like patterns in Rust, try/catch in TypeScript
- React Query handles async errors — hooks return `{ isError, error }`
- Validation via `zod` schemas with `.safeParse()` — check `result.success`
- Logger abstraction via `@/adapters` (`logger.warn()`, `logger.error()`)
- Never log secrets or financial data (security requirement from AGENTS.md)
### Type Safety
- **Strict mode enabled** with all strict flags (`strictNullChecks`, `noImplicitAny`, etc.)
- `noUnusedLocals: true` and `noUnusedParameters: true` — no dead code
- `noImplicitOverride: true` — explicit overrides
- Unused vars/params prefixed with `_` (ESLint rule: `argsIgnorePattern: "^_"`)
- `no-explicit-any: warn` — avoid `any`, use `unknown` when type is truly unknown
## React Conventions
### Component Structure
### Hook Patterns
- Custom hooks in `src/hooks/` with `use-` prefix
- Follow React Query patterns:
- Hooks barrel-exported from `src/hooks/index.ts`
### Props Typing
- Inline interface above component (not in separate file)
- Optional props use `?` with reasonable defaults destructured in function signature
- Children typed as `React.ReactNode`
- Event handlers typed inline or with `React.ButtonHTMLAttributes<HTMLElement>`
### Component Composition
- Page components in `src/pages/<domain>/` directories
- Shared components in `src/components/`
- Feature modules in `src/features/<feature-name>/` with self-contained components, hooks, services
- UI primitives from `@whaleit/ui` package (shadcn-based)
## Rust Conventions
### Naming
- `snake_case` for files, functions, variables, modules
- `PascalCase` (CamelCase) for types, traits, enums, structs
- `SCREAMING_SNAKE_CASE` for constants
- Module directories: `snake_case` with `mod.rs` — e.g., `crates/core/src/portfolio/mod.rs`
### Error Handling
- Use `thiserror` for domain errors — `#[derive(Error, Debug)]`
- Propagate with `?` operator
- Type alias: `pub type Result<T> = std::result::Result<T, Error>;`
- Error hierarchy:
- Module-specific error enums: `ActivityError`, `FxError`, `MarketDataError`, `CalculatorError`
- Use `anyhow` for application-level errors where specificity isn't needed
### Module Organization
- `mod.rs` as module entry point — re-exports sub-modules
- `model.rs` for data structures (DB models, DTOs)
- `repository.rs` for database operations
- Domain modules grouped by feature: `accounts/`, `activities/`, `portfolio/`, `assets/`
- Core crate is database-agnostic; `storage-sqlite` implements storage traits
- Keep Tauri/Axum command handlers thin — delegate to `crates/core`
### Trait Patterns
- Domain traits defined in `crates/core` — e.g., repository traits
- Implemented in `crates/storage-sqlite`
- `async_trait` for async trait methods
### Async Patterns
- `tokio` runtime with `rt-multi-thread` feature
- `async/await` with `?` propagation
- `futures` crate for combinators
### Linting
- `unsafe_code = "forbid"` at workspace level
- `clippy::all = "warn"` at workspace level
- `cargo fmt` enforced in CI (check via `cargo fmt --all -- --check`)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` in CI
## CSS/Styling Conventions
### Approach
- **Tailwind CSS v4** with `@tailwindcss/vite` plugin
- **No CSS modules** — utility-first with Tailwind classes
- Custom CSS only in `src/globals.css` using Tailwind v4 `@theme`, `@utility`, `@custom-variant`
- `prettier-plugin-tailwindcss` for automatic class sorting
### Theme/Token System
- Custom color palette defined as CSS custom properties in `@theme {}` block in `globals.css`
- Base colors: `--color-base-*` (50-950 scale, warm paper tones)
- Semantic colors: `--color-red-*`, `--color-green-*`, `--color-blue-*` etc.
- Special tokens: `--color-paper` (background), `--color-black`
- Font families: `--font-sans` (Inter), `--font-serif` (Merriweather), `--font-mono` (IBM Plex Mono)
### Responsive Design
- Mobile-first approach using Tailwind breakpoints (`sm:`, `md:`, `lg:`)
- Mobile-specific components suffixed with `-mobile.tsx` — e.g., `account-selector-mobile.tsx`
- Test responsive layouts in both desktop (Tauri) and web
### Dark Mode
- Class-based dark mode via `@custom-variant dark (&:where(.dark, .dark *));`
- Dark theme tokens also defined in `globals.css` `@theme {}` block
## Git Conventions
### Commit Messages
- No enforced conventional commits format detected
- CI checks: format, lint, type-check, test, build
### CI Pipeline
- **PR Check** (`.github/workflows/pr-check.yml`): Two parallel jobs:
### Branch Strategy
- PRs target `main`, `develop`, or `feature/**` branches
- Concurrency groups prevent parallel runs on same PR
## File Organization Patterns
### Where New Files Go
- Create directory: `apps/frontend/src/pages/<domain>/`
- Add component file: `apps/frontend/src/pages/<domain>/<page-name>.tsx`
- Register route in `apps/frontend/src/routes.tsx`
- Create: `apps/frontend/src/hooks/use-<name>.ts`
- Export from: `apps/frontend/src/hooks/index.ts`
- Create: `apps/frontend/src/components/<component-name>.tsx`
- Create directory: `apps/frontend/src/features/<feature-name>/`
- Structure: `components/`, `hooks/`, `services/`, `types.ts`
- Domain logic: `crates/core/src/<domain>/`
- Storage impl: `crates/storage-sqlite/src/<domain>/`
- Tauri command: `apps/tauri/src/commands/`
- Web endpoint: `apps/server/src/api/`
- Add to both: `apps/frontend/src/adapters/tauri/` AND `apps/frontend/src/adapters/web/`
- The `@/adapters` alias auto-resolves based on `BUILD_TARGET`
### Barrel Exports
- `src/hooks/index.ts` — re-exports all hooks
- `src/lib/types.ts` — central type definitions with re-exports from constants
- `src/adapters/index.ts` — re-exports from platform-specific adapter
- Rust: `mod.rs` files serve as module entry points
### Index File Patterns
- TypeScript: `index.ts` for barrel exports only (not component files)
- Rust: `mod.rs` for module declarations and re-exports
## Anti-patterns to Avoid
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- Local-first: all data stored in SQLite on the user's device; no mandatory cloud
- Adapter pattern: single frontend codebase targets Tauri IPC or HTTP via pluggable adapters
- Domain event system: services emit events after mutations; runtime bridges translate events into side effects (portfolio recalculation, broker sync, etc.)
- Feature-gated compilation: `connect-sync` and `device-sync` features enable optional cloud broker sync and E2EE device sync
- Addon system: third-party extensions loaded at runtime via the `@whaleit/addon-sdk`
## Architecture Diagram
```
```
## Layers
### Frontend (`apps/frontend/`)
- **Purpose:** Single-page React application providing the entire UI
- **Location:** `apps/frontend/src/`
- **Contains:** Pages, feature modules, shared components, hooks, adapters, addon runtime
- **Depends on:** React, TanStack Query (server state), react-router-dom, shadcn/ui (`@whaleit/ui`)
- **Used by:** Rendered by Tauri WebView (desktop) or served by Axum (web)
- **State management:** TanStack Query for all server state; React context for local UI state (`portfolio-sync-context.tsx`, `privacy-context.tsx`, `auth-context.tsx`)
### Desktop Shell (`apps/tauri/`)
- **Purpose:** Tauri v2 application shell — native window, IPC bridge, OS integration
- **Location:** `apps/tauri/src/`
- **Contains:**
- **Depends on:** All core crates, Tauri plugins (shell, dialog, fs, deep-link, updater, single-instance)
- **Used by:** End users as desktop app (macOS, Windows, Linux, iOS)
### Web Server (`apps/server/`)
- **Purpose:** Self-hosted web mode — serves frontend as static files + REST API
- **Location:** `apps/server/src/`
- **Contains:**
- **Depends on:** All core crates, Axum, tower-http middleware, utoipa (OpenAPI)
- **Used by:** Self-hosted users, Docker deployments
### Core Crates (`crates/`)
- **Location:** `crates/core/src/`
- **Contains:** `accounts/`, `activities/`, `assets/`, `fx/`, `goals/`, `health/`, `limits/`, `portfolio/`, `quotes/`, `settings/`, `taxonomies/`, `events/`, `secrets/`
- **Depends on:** Defines traits implemented by `storage-sqlite`
- **Used by:** Both `apps/tauri` and `apps/server` via concrete service constructors
- **Location:** `crates/storage-sqlite/src/`
- **Contains:** Repositories for every domain (`accounts/`, `activities/`, `assets/`, etc.), `db/` (pool, write-actor), `schema.rs` (Diesel schema), `migrations/` (29 migration files)
- **Depends on:** Diesel, rusqlite, r2d2 connection pool
- **Used by:** Core services via repository trait implementations
- **Location:** `crates/market-data/src/`
- **Contains:** `provider/` (Yahoo Finance, custom providers), `resolver/` (symbol resolution), `registry/` (provider management), `models/`
- **Depends on:** `reqwest` for HTTP
- **Used by:** `crates/core` quote service
- **Location:** `crates/connect/src/`
- **Contains:** `broker/` (mapping, orchestrator, progress tracking), `platform/`, `broker_ingest/`, `client.rs`, `token_lifecycle.rs`
- **Feature-gated:** `connect-sync`
- **Used by:** Tauri commands and server API for broker data import
- **Location:** `crates/device-sync/src/`
- **Contains:** `engine/` (sync runtime, ports), `crypto.rs` (X25519, ChaCha20Poly1305, HKDF), `enroll_service.rs`, `client.rs`
- **Feature-gated:** `device-sync`
- **Used by:** Tauri and server for multi-device sync
- **Location:** `crates/ai/src/`
- **Contains:** `providers.rs`, `provider_service.rs`, `chat.rs`, `stream_hook.rs`, `tools/` (15 tool implementations — accounts, activities, holdings, etc.), `eval/`, `prompt_template_service.rs`
- **Used by:** Both runtimes for AI assistant features
### Packages (`packages/`)
- **`packages/ui/`** (`@whaleit/ui`) — Shared shadcn/ui component library built with tsup
- **`packages/addon-sdk/`** (`@whaleit/addon-sdk`) — Types and API for addon developers
- **`packages/addon-dev-tools/`** (`@whaleit/addon-dev-tools`) — CLI and dev server for addon development
## Data Flow
### User Action → Data Persistence (Desktop)
### User Action → Data Persistence (Web)
- Step 4: Adapter calls `fetch("/api/v1/accounts", { method: "POST", ... })` via `apps/frontend/src/adapters/web/core.ts`
- Step 5: Axum routes to `apps/server/src/api/accounts.rs` handler
- Step 11: SSE event sent from `apps/server/src/events.rs::EventBus` → frontend SSE listener invalidates cache
### Market Data Sync (Periodic)
### Domain Event Processing
```
```
- `ActivitiesChanged` — triggers portfolio recalculation + FX sync planning
- `HoldingsChanged` — triggers portfolio recalculation
- `AccountsChanged` — triggers FX sync, portfolio recalculation
- `AssetsCreated` / `AssetsUpdated` — triggers quote sync, enrichment
- `AssetsMerged` — UNKNOWN asset merge propagation
- `TrackingModeChanged` — switches between transactions/holdings/manual tracking
- `ManualSnapshotSaved` — triggers recalculation for affected account
- `DeviceSyncPullComplete` — triggers full portfolio recalculation
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
### Web Server
- **Location:** `apps/server/src/main.rs` → `main_lib.rs`
- **Triggers:** User runs the server binary or Docker container
- **Responsibilities:**
### Frontend Application
- **Location:** `apps/frontend/src/main.tsx`
- **Triggers:** Browser/WebView loads `index.html`
- **Responsibilities:**
## Error Handling
- `thiserror` for domain error types: `crates/core/src/errors.rs`, `apps/tauri/src/error.rs`, `apps/server/src/error.rs`
- `Result<T, E>` propagation with `?` operator
- Tauri commands convert errors to serializable strings for IPC
- Axum handlers return appropriate HTTP status codes via error conversions
- TanStack Query error handling via `onError` callbacks
- Adapter `invoke()` wraps calls with timeout (120s) and error logging
- Web adapter handles 401 responses by notifying the auth context
## Cross-Cutting Concerns
- Desktop: `tauri_plugin_log` with debug/info level filtering → `apps/tauri/src/lib.rs`
- Web: `tracing_subscriber` with configurable format (text/JSON) → `apps/server/src/main_lib.rs`
- Frontend: Platform-specific logger adapter → `apps/frontend/src/adapters/{tauri,web}/core.ts`
- Frontend forms: `react-hook-form` + `zod` schemas → `apps/frontend/src/lib/schemas.ts`
- Backend: Rust type system + Diesel schema validation
- Desktop: No auth (local data, OS user isolation)
- Web: Optional JWT + Argon2id password auth → `apps/server/src/auth.rs`
- Desktop: OS keyring via `secret_store.rs` (never written to disk)
- Web: Encrypted JSON file (`secrets.json`) with HKDF-derived key from `WF_SECRET_KEY`
- API keys for market data providers and AI providers stored as secrets
- Base currency and timezone are runtime-mutable settings
- `Arc<RwLock<String>>` pattern allows dynamic updates without restart
- FX service handles multi-currency conversions via exchange rate repository
## Key Design Decisions
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
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.OpenCode/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using edit, write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-OpenCode-profile` -- do not edit manually.
<!-- GSD:profile-end -->
