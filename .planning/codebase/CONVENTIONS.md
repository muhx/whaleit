# Coding Conventions

**Analysis Date:** 2026-04-20

## Overview

This monorepo enforces two parallel conventions — **TypeScript/React** (frontend

- packages) and **Rust** (Tauri desktop, Axum server, core crates). Both are
  kept consistent by `pnpm check` (format + lint + type-check) and
  `cargo fmt/clippy` via CI (`.github/workflows/pr-check.yml`).

```
pnpm check          → format:check + lint:quiet + type-check
cargo fmt --check   → check formatting
cargo clippy -D warnings  → lint (warnings fail CI)
```

---

## TypeScript / React

### Code Style

**Formatter:** Prettier 3 (`.prettierrc.cjs`)

Key settings:

- `printWidth: 100`
- `tabWidth: 2`, `useTabs: false`
- `semi: true`
- `singleQuote: false` → use double quotes (`"foo"`)
- `trailingComma: "all"`
- `arrowParens: "always"` → `(x) => ...`
- `endOfLine: "lf"`
- Markdown overridden to `printWidth: 80`, `proseWrap: "always"`
- `prettier-plugin-tailwindcss` enforces Tailwind class ordering

**Linter:** ESLint flat config (`eslint.base.config.js`, `eslint.config.js`,
`apps/frontend/eslint.config.js`, `packages/*/eslint.config.js`)

Stack:

- `typescript-eslint` recommended + type-checked + stylistic
- `eslint-plugin-react` (recommended + jsx-runtime)
- `eslint-plugin-react-hooks`
- `eslint-plugin-react-refresh`
- `@tanstack/eslint-plugin-query` (recommended)
- `eslint-config-prettier` applied last
- `typescript-eslint` uses `parserOptions.projectService: true` for typed
  linting

Notable rules:

- `no-unused-vars`: **error**, ignores `^_` prefix
- `prefer-const`: **error**
- `no-var`: **error**
- `no-console`: **warn** (allows `warn`, `error`)
- `@typescript-eslint/no-explicit-any`: **warn** (tolerated but discouraged)
- `@typescript-eslint/no-unsafe-*`: **warn** (relaxed)
- `@typescript-eslint/no-floating-promises`: **off**
- `@typescript-eslint/no-misused-promises`: `checksVoidReturn: false` (allow
  async JSX handlers)

Per-workspace ignores: `dist/**`, `coverage/**`, `*.config.*`, `**/*.d.ts`,
generated Recharts/react-qr-code patches.

### TypeScript Compiler

Inherits `tsconfig.base.json`:

- `target: "ES2022"`, `module: "ESNext"`, `moduleResolution: "bundler"`
- `strict: true` plus `noUnusedLocals`, `noUnusedParameters`,
  `noFallthroughCasesInSwitch`, `noImplicitOverride`
- `composite: true` for project references (root `tsconfig.json` references
  `apps/frontend`, `packages/ui`, `packages/addon-sdk`)
- `jsx: "react-jsx"`

Frontend `tsconfig.json` (`apps/frontend/tsconfig.json`) adds:

- `types: ["@tauri-apps/api", "vitest/globals"]`
- Path aliases:
  - `@/*` → `./src/*`
  - `@whaleit/ui` → `../../packages/ui/src`
  - `@whaleit/addon-sdk` → `../../packages/addon-sdk/src`
  - `#platform` → `./src/adapters/tauri/core` (swapped to `web/core` at build
    time)

Vite resolves `@/adapters` conditionally via `BUILD_TARGET` env
(`apps/frontend/vite.config.ts`) — `tauri` vs `web`.

### Naming Patterns

**Files:**

- `kebab-case.ts` / `kebab-case.tsx` for everything except test colocation.
  Examples: `use-accounts.ts`, `buy-form.tsx`, `query-keys.ts`,
  `activity-utils.ts`.
- Tests mirror the source name with `.test.ts` / `.test.tsx`: `schemas.ts` →
  `schemas.test.ts`.
- Some directories use `__tests__/` subfolders:
  `apps/frontend/src/pages/activity/components/forms/__tests__/`.

**Directories:** `lowercase-with-dashes` (per `AGENTS.md`). Example:
`apps/frontend/src/features/devices-sync/`.

**Functions & variables:** `camelCase` (`getActivities`, `searchActivities`,
`normalizeStringArray`).

**React components:** `PascalCase` exported as named exports (`BuyForm`,
`TransferForm`, `Header`).

**Hooks:** `useXxx` camelCase function, file `use-xxx.ts`
(`apps/frontend/src/hooks/use-accounts.ts`, `use-settings-mutation.ts`).

**Types / interfaces:** `PascalCase` (`ActivityFilters`, `RunEnv`,
`PlatformInfo`, `BackendSyncStateResult`). Prefer `interface` over `type` per
`AGENTS.md`; use `type` for unions and utility types.

**Enums:** avoided. Use `as const` object patterns instead
(`apps/frontend/src/adapters/types.ts:6-14`,
`apps/frontend/src/lib/schemas.ts:41-45`):

```ts
export const RunEnvs = { DESKTOP: "desktop", WEB: "web" } as const;
export type RunEnv = (typeof RunEnvs)[keyof typeof RunEnvs];
```

**Constants & query keys:** SCREAMING_SNAKE_CASE inside `as const` object. See
`apps/frontend/src/lib/query-keys.ts:1-136`:

```ts
export const QueryKeys = {
  ACCOUNTS: "accounts",
  valuationHistory: (id: string) => [QueryKeys.HISTORY_VALUATION, id],
} as const;
```

### Import Organization

Observed order (no auto-sort plugin, but consistent across codebase):

1. External packages (`react`, `@tanstack/react-query`, `@tauri-apps/api`)
2. Internal alias imports (`@/lib/...`, `@/adapters`, `@/hooks/...`,
   `@whaleit/ui`)
3. Relative imports (`./constants`, `../schemas`)
4. Type-only imports may be inlined with `import type { Foo }` or grouped with
   values

Example (`apps/frontend/src/adapters/shared/activities.ts:1-22`):

```ts
import { ImportType } from "@/lib/types";
import type { Activity, ActivityCreate, ... } from "@/lib/types";

import { invoke, logger } from "./platform";
```

**Path alias quick reference:**

| Alias                    | Resolves to                                | Purpose                        |
| ------------------------ | ------------------------------------------ | ------------------------------ |
| `@/*`                    | `apps/frontend/src/*`                      | Frontend app internals         |
| `@/adapters`             | `src/adapters/tauri` or `src/adapters/web` | Build-target specific          |
| `#platform`              | `src/adapters/{target}/core`               | Used by shared adapter modules |
| `@whaleit/ui`        | `packages/ui/src`                          | Shared UI package              |
| `@whaleit/addon-sdk` | `packages/addon-sdk/src`                   | Addon SDK types/runtime        |

### Adapter / Backend Call Pattern

Every backend call wraps `invoke` (Tauri) or fetch (web) through a thin module
in `apps/frontend/src/adapters/shared/<domain>.ts`. The adapter:

1. Accepts typed args, returns typed `Promise<T>`.
2. Wraps call in `try/catch`, logs via `logger.error`, rethrows.
3. Uses `invoke<T>("snake_case_command", { camelCaseArgs })` — arguments are
   camelCase, serde rename_all handles conversion to snake_case in Rust.

Example (`apps/frontend/src/adapters/shared/activities.ts:100-107`):

```ts
export const createActivity = async (
  activity: ActivityCreate,
): Promise<Activity> => {
  try {
    return await invoke<Activity>("create_activity", { activity });
  } catch (err) {
    logger.error("Error creating activity.");
    throw err;
  }
};
```

The `invoke` helper (`apps/frontend/src/adapters/tauri/core.ts:29-47`) adds a
120 s timeout and logs failures.

**Never import `@tauri-apps/api` directly from components.** Always go through
`@/adapters`. This keeps web build working.

### React Component Patterns

- **Functional components only.** No class components.
- **Named exports** — no default exports for components
  (`export function BuyForm(...)`).
- Strict mode + React 19.2.
- Props typed inline with `interface` or `type`.
- Hooks placed in `apps/frontend/src/hooks/` (shared) or
  `src/pages/<domain>/hooks/` (domain-specific).

### Data Fetching (TanStack Query)

Patterns enforced by `@tanstack/eslint-plugin-query`:

- `exhaustive-deps`: **warn**
- `no-unstable-deps`: **warn**

**Query key discipline:** always reference `QueryKeys` constants from
`apps/frontend/src/lib/query-keys.ts`. Parameterized keys are factory functions:

```ts
// Static key
useQuery({ queryKey: [QueryKeys.ACCOUNTS, includeArchived], ... })

// Parameterized key
useQuery({ queryKey: QueryKeys.aiThreadMessages(threadId), ... })
```

**Hook shape** (`apps/frontend/src/hooks/use-accounts.ts:7-33`):

```ts
export function useAccounts(options?: {...}) {
  const { data = [], isLoading, isError, error, refetch } = useQuery<T, Error>({
    queryKey: [...],
    queryFn: () => getAccounts(...),
  });
  // transform data with useMemo
  return { accounts, isLoading, isError, error, refetch };
}
```

**Mutation shape** (`apps/frontend/src/hooks/use-settings-mutation.ts`):

```ts
return useMutation({
  mutationFn: updateSettings,
  onSuccess: (data) => {
    queryClient.invalidateQueries({ queryKey: [QueryKeys.SETTINGS] });
    toast({ title: "...", variant: "success" });
  },
  onError: (error) => {
    logger.error(`Error updating settings: ${error}`);
    toast({ variant: "destructive", ... });
  },
});
```

### Forms

- **`react-hook-form` + `zod`** (resolved via `@hookform/resolvers/zod`).
- Form schemas live alongside the form component or in
  `apps/frontend/src/lib/schemas.ts`.
- Shared schemas use `z.object({ ... })`, unions via `z.discriminatedUnion`,
  branded primitives via `z.enum`.
- Schemas exported for reuse in tests
  (`apps/frontend/src/pages/activity/components/forms/__tests__/form-schemas.test.ts`).

Example (`apps/frontend/src/lib/schemas.ts:47-74`):

```ts
export const importMappingSchema = z.object({
  accountId: z.string(),
  importType: z
    .enum([ImportType.ACTIVITY, ImportType.HOLDINGS])
    .default(ImportType.ACTIVITY),
  symbolMappingMeta: z
    .record(
      z.string(),
      z.object({
        exchangeMic: z.string().optional(),
        quoteMode: quoteModeSchema.optional(),
      }),
    )
    .optional(),
});
```

### Error Handling

**Frontend:**

- Adapter functions: `try/catch`, `logger.error(message)`, `throw err`.
- Mutation callers: surface via `toast({ variant: "destructive" })` in
  `onError`.
- Avoid bare `console.log` — ESLint warns. Use `logger.*` from `@/adapters`.
  `console.warn` / `console.error` are permitted.

### Logging

- Use `logger` exported from `@/adapters` (Tauri → `@tauri-apps/plugin-log`; web
  → wrapped `console`). See `apps/frontend/src/adapters/tauri/core.ts:11-27`.
- Never log secrets, passwords, or financial values (`AGENTS.md:149-150`).

### Module Design

- **Named exports** preferred. Default exports only for Vite entry points
  (`main.tsx`) and framework files that require them.
- Barrel files (`index.ts`) are used in adapters and hooks directories to
  centralize public API (`apps/frontend/src/adapters/index.ts`,
  `apps/frontend/src/hooks/index.ts`).

### Comments

- JSDoc blocks for public exports, especially in `adapters/` and `lib/` (e.g.,
  `/** Preview which assets would be created... */`).
- Inline comments explain non-obvious intent, never describe what the code does.
- `TODO` / `FIXME` are fine but should reference context.

---

## Rust

### Code Style

- **Formatter:** `rustfmt` default (no custom `rustfmt.toml` at root) — run
  `cargo fmt --all` before commits. CI enforces `cargo fmt --all -- --check`.
- **Linter:**
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` (CI
  fails on any warning).
- **Workspace lints** (`Cargo.toml:14-18`):
  ```toml
  [workspace.lints.rust]
  unsafe_code = "forbid"
  [workspace.lints.clippy]
  all = "warn"
  ```
  → `unsafe` is **forbidden** repo-wide.
- **Edition:** `2021` (`Cargo.toml:12`).

### Workspace Layout

`Cargo.toml` workspace (`members = ["apps/tauri", "apps/server", "crates/*"]`)
uses `workspace.dependencies` to pin shared crate versions (tokio, serde,
diesel, chrono, thiserror, rust_decimal, async-trait, log, etc.).

Per-crate `Cargo.toml` references workspace deps with `{ workspace = true }`.

### Naming Patterns

- **Crates:** kebab-case on disk (`crates/market-data/`), `snake_case` in
  `Cargo.toml` (`whaleit-core`, `whaleit-market-data`).
- **Modules:** `snake_case` (`activities_model`, `activities_service`,
  `device_sync_engine`).
- **Files:** `snake_case.rs`. Common splits: `<domain>_model.rs`,
  `<domain>_service.rs`, `<domain>_traits.rs`, `<domain>_errors.rs`, plus
  `<domain>_model_tests.rs` / `<domain>_service_tests.rs` for inline test
  modules.
- **Types:** `PascalCase` (`ActivityService`, `ActivityError`, `NewActivity`).
- **Functions:** `snake_case`.
- **Constants:** `SCREAMING_SNAKE_CASE`.

### Module Structure

Each domain follows a consistent shape (`crates/core/src/activities/mod.rs`):

```
<domain>/
├── mod.rs                    # re-exports public API
├── <domain>_constants.rs
├── <domain>_errors.rs
├── <domain>_model.rs         # structs, enums, DTOs
├── <domain>_model_tests.rs   # #[cfg(test)] mod tests
├── <domain>_service.rs       # business logic
├── <domain>_service_tests.rs # #[cfg(test)] mod tests
└── <domain>_traits.rs        # ServiceTrait + RepositoryTrait
```

`mod.rs` declares private submodules and re-exports only the public surface:

```rust
mod activities_model;
mod activities_service;

#[cfg(test)]
mod activities_service_tests;

pub use activities_model::{Activity, NewActivity, ...};
pub use activities_service::ActivityService;
pub use activities_traits::{ActivityRepositoryTrait, ActivityServiceTrait};
```

### Error Handling

- **`thiserror`** for all domain error enums (`crates/core/src/errors.rs`,
  `crates/core/src/activities/activities_errors.rs`).
- Each crate has its own error enum that the core `Error` converts `From` via
  `#[from]`.
- **`Result<T>`** type alias per crate
  (`pub type Result<T> = std::result::Result<T, Error>`, `errors.rs:15`).
- Propagate with `?`.
- **Never `unwrap()` in library code.** Only tests and `main()` helpers.
- `From<Error> for String` lets Tauri commands call
  `.map_err(|e| e.to_string())`.

Example (`crates/core/src/errors.rs:22-74`):

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("Database operation failed: {0}")]
    Database(#[from] DatabaseError),
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("Activity error: {0}")]
    Activity(#[from] ActivityError),
    // ...
}
```

### Tauri IPC Commands

Live in `apps/tauri/src/commands/<domain>.rs` (see
`apps/tauri/src/commands/activity.rs` for the canonical shape). Rules:

1. `#[tauri::command]` + `pub async fn`.
2. Arguments are **snake_case** (serde translates camelCase frontend args).
3. Last argument is always `state: State<'_, Arc<ServiceContext>>`.
4. Return type `Result<T, String>` (Tauri serializes error as string).
5. **Thin** — parse inputs, delegate to `state.<service>()`, map error to
   string.
6. Use `log::debug!` at entry for trace visibility.

Example (`apps/tauri/src/commands/activity.rs:55-67`):

```rust
#[tauri::command]
pub async fn create_activity(
    activity: NewActivity,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Activity, String> {
    debug!("Creating activity...");
    state
        .activity_service()
        .create_activity(activity)
        .await
        .map_err(|e| e.to_string())
}
```

### Axum Handlers (`apps/server/src/api/`)

1. Each module exports `pub fn router() -> Router<Arc<AppState>>` merged in
   `apps/server/src/api.rs:88-120`.
2. Handlers are `async fn` returning `ApiResult<Json<T>>`.
3. Use `State(state): State<Arc<AppState>>` + `Json(body): Json<Body>` /
   `Query<...>` / `Path<...>` extractors.
4. Body/query structs use `#[derive(Deserialize)]` with
   `#[serde(rename = "...")]` or `serde(rename_all = "camelCase")` to match
   frontend.
5. Errors bubble via the shared `ApiError` enum
   (`apps/server/src/error.rs:13-31`) which maps variants to HTTP status codes
   in `IntoResponse`.
6. OpenAPI annotations via `#[utoipa::path(...)]` on public handlers.

Example (`apps/server/src/api/activities.rs:56-98`):

```rust
async fn search_activities(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ActivitySearchBody>,
) -> ApiResult<Json<ActivitySearchResponse>> {
    // ...parse inputs...
    let resp = state.activity_service.search_activities(...)?;
    Ok(Json(resp))
}
```

### Service / Repository Traits

- Services accept repository trait objects to enable mock substitution in tests.
- `#[async_trait]` for async trait methods
  (`crates/core/src/activities/activities_traits.rs`).
- Business logic lives in `crates/core`. Tauri/Axum layers only translate
  transport.

### Core Principles (from `AGENTS.md:140-145`)

- Small, focused functions.
- `Result`/`Option` with `?` propagation.
- `thiserror` for domain errors.
- Thin Tauri/Axum commands, delegate to `crates/core`.
- Migrations in `crates/storage-sqlite/migrations`.
- Secrets in OS keyring, never disk/localStorage.
- Never log secrets or financial data.

### Dependencies

Central workspace deps in root `Cargo.toml:20-65`:

- Async: `tokio` (multi-thread, macros, sync), `async-trait`, `futures`
- Serde: `serde` (derive), `serde_json`
- DB: `diesel` (sqlite, chrono, r2d2), `diesel_migrations`, `rusqlite`, `r2d2`
- Errors: `thiserror`, `anyhow` (anyhow only in apps layer)
- Numbers: `rust_decimal` + `rust_decimal_macros` (money), `num-traits`
- Time: `chrono` (serde feature)
- HTTP: `reqwest` (rustls-tls, json)
- Crypto: `chacha20poly1305`, `x25519-dalek`, `hkdf`, `sha2`, `rand`
- Logging: `log` (tracing/tracing-subscriber in server only)
- IDs: `uuid` (v4, v7, serde)

---

## Git & Commits

Style observed in `git log` — conventional-commits-ish, **not** strictly
enforced:

- `feat(<scope>): ...`
- `fix(<scope>): ...`
- `refactor(<scope>): ...`
- `chore(<scope>): ...`
- `perf(<scope>): ...`

Common scopes: `ai-assistant`, `ai-import`, `ai`, `core`, `clippy`, `import`,
etc.

Per `CONTRIBUTING.md:49-52`: "Write clear, concise commit messages". CLA
required for all PRs.

---

## Validation Workflow (AGENTS.md:156-161)

Before completing any task:

- [ ] Builds: `pnpm build` (web) or `pnpm tauri dev` (desktop) or `cargo check`
- [ ] Tests pass: `pnpm test` **and/or** `cargo test`
- [ ] Both desktop and web compile if touching shared code
- [ ] Changes are minimal and surgical

---

_Convention analysis: 2026-04-20_
