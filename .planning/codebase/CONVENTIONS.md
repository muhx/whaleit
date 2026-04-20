# Coding Conventions

**Analysis Date:** 2026-04-20

## TypeScript Conventions

### Naming

**Files:**
- Components: `kebab-case.tsx` — e.g., `privacy-toggle.tsx`, `header.tsx`, `accounts-summary.tsx`
- Hooks: `use-kebab-case.ts` — e.g., `use-accounts.ts`, `use-settings.ts`
- Utilities/lib: `kebab-case.ts` — e.g., `activity-utils.ts`, `query-keys.ts`
- Types: `types.ts` (single file per domain area) — e.g., `src/lib/types.ts`
- Tests: co-located `*.test.ts` or `*.test.tsx` — e.g., `activity-utils.test.ts`
- Test groups within `__tests__/` subdirectory — e.g., `components/forms/__tests__/buy-form.test.tsx`

**Variables & Functions:**
- `camelCase` for variables and functions — e.g., `formatAmount`, `parseDecimalInput`
- `PascalCase` for React components, types, interfaces, enums — e.g., `AccountSummaryView`, `ActivityType`
- `UPPER_SNAKE_CASE` for constants — e.g., `DECIMAL_PRECISION`, `DISPLAY_DECIMAL_PRECISION`

**Types:**
- Prefer `interface` for object shapes — e.g., `Account`, `Settings`, `Activity`
- Use `type` for unions, intersections, utility types — e.g., `type TrackingMode = "TRANSACTIONS" | "HOLDINGS" | "NOT_SET"`
- Avoid `enum` — use `const` objects with `as const` and derived types instead:
  ```typescript
  export const ImportType = { ACTIVITY: "CSV_ACTIVITY", HOLDINGS: "CSV_HOLDINGS" } as const;
  export type ImportType = (typeof ImportType)[keyof typeof ImportType];
  ```

### Export Patterns

- **Named exports** for everything — no default exports except `App.tsx`
- Components: `export function ComponentName()` — always named function declarations
- Utilities: `export function utilityName()` or `export const utilityName = () => {}`
- Barrel exports via `index.ts` files — e.g., `src/hooks/index.ts` uses `export * from "./use-accounts"`
- Types file re-exports from constants: `export { AccountType, ActivityType } from "./constants"`

### Import Organization

Implicit order observed in code:
1. External libraries (`react`, `@tanstack/react-query`, `zod`, `date-fns`)
2. UI library (`@wealthfolio/ui/...`)
3. Adapter layer (`@/adapters`)
4. Hooks (`@/hooks/...`)
5. Library utilities (`@/lib/...`)
6. Relative imports (`./...`)

**Path aliases** (configured in `apps/frontend/vite.config.ts` and `tsconfig.json`):
- `@/*` → `./src/*`
- `@/adapters` → resolves to `./src/adapters/tauri` or `./src/adapters/web` based on `BUILD_TARGET`
- `#platform` → `./src/adapters/tauri/core` or `./src/adapters/web/core`
- `@wealthfolio/ui` → `../../packages/ui/src`
- `@wealthfolio/addon-sdk` → `../../packages/addon-sdk/src`

### Component Patterns

- **Functional components only** — no class components
- Named function declarations: `export function ComponentName(props: Props)`
- Props defined as inline `interface` above the component
- Destructure props in function signature
- Use `cn()` utility for conditional class merging:
  ```typescript
  className={cn("base-classes", conditionalClass, className)}
  ```
- Wrap with `React.StrictMode` at root (`src/main.tsx`)

### State Management

- **Server state:** `@tanstack/react-query` — all backend data goes through React Query
  - Query keys centralized in `src/lib/query-keys.ts` as `QueryKeys` const object
  - Custom hooks wrap queries: `useAccounts()`, `useSettings()`
  - Default stale time: 5 minutes, no refetch on window focus, no retry
- **Client state:** `zustand` for complex client state
- **Form state:** `react-hook-form` + `zod` schemas
  - Schemas in `src/lib/schemas.ts`
  - Resolver: `@hookform/resolvers/zod`
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

---

## React Conventions

### Component Structure

```typescript
// 1. Imports
import { useQuery } from "@tanstack/react-query";
import { cn } from "@/lib/utils";

// 2. Types/Interfaces for props
interface ComponentProps {
  className?: string;
  data: SomeType;
}

// 3. Named export function
export function Component({ className, data }: ComponentProps) {
  // hooks at top
  const query = useQuery(...);
  
  // derived state
  const computed = useMemo(() => ..., [deps]);
  
  // handlers
  const handleClick = () => { ... };
  
  // render
  return (
    <div className={cn("base-classes", className)}>
      ...
    </div>
  );
}
```

### Hook Patterns

- Custom hooks in `src/hooks/` with `use-` prefix
- Follow React Query patterns:
  ```typescript
  export function useAccounts(options?: { filterActive?: boolean }) {
    const { data, isLoading, isError, error, refetch } = useQuery<Account[], Error>({
      queryKey: [QueryKeys.ACCOUNTS],
      queryFn: () => getAccounts(),
    });
    return { accounts: data ?? [], isLoading, isError, error, refetch };
  }
  ```
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
- UI primitives from `@wealthfolio/ui` package (shadcn-based)

---

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
  ```rust
  #[derive(Error, Debug)]
  pub enum Error {
      #[error("Database operation failed: {0}")]
      Database(#[from] DatabaseError),
      #[error("Asset operation failed: {0}")]
      Asset(String),
      #[error("Validation: {0}")]
      Validation(#[from] ValidationError),
      // ...
  }
  ```
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

---

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

---

## Git Conventions

### Commit Messages

- No enforced conventional commits format detected
- CI checks: format, lint, type-check, test, build

### CI Pipeline

- **PR Check** (`.github/workflows/pr-check.yml`): Two parallel jobs:
  1. `frontend-check`: format → lint → type-check → test → build
  2. `rust-check`: fmt → clippy → test → release check

### Branch Strategy

- PRs target `main`, `develop`, or `feature/**` branches
- Concurrency groups prevent parallel runs on same PR

---

## File Organization Patterns

### Where New Files Go

**New page/route:**
- Create directory: `apps/frontend/src/pages/<domain>/`
- Add component file: `apps/frontend/src/pages/<domain>/<page-name>.tsx`
- Register route in `apps/frontend/src/routes.tsx`

**New hook:**
- Create: `apps/frontend/src/hooks/use-<name>.ts`
- Export from: `apps/frontend/src/hooks/index.ts`

**New shared component:**
- Create: `apps/frontend/src/components/<component-name>.tsx`

**New feature module:**
- Create directory: `apps/frontend/src/features/<feature-name>/`
- Structure: `components/`, `hooks/`, `services/`, `types.ts`

**New backend service (Rust):**
- Domain logic: `crates/core/src/<domain>/`
- Storage impl: `crates/storage-sqlite/src/<domain>/`
- Tauri command: `apps/tauri/src/commands/`
- Web endpoint: `apps/server/src/api/`

**New adapter function:**
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

---

## Anti-patterns to Avoid

1. **Never use default exports** except for `App.tsx` — use named exports everywhere
2. **Never import from `@/adapters/core` directly** — use `#platform` alias or `@/adapters`
3. **Never log secrets or financial data** — use `logger` abstraction, not `console.log`
4. **Never store secrets on disk or in localStorage** — use OS keyring
5. **Never put business logic in Tauri/Axum handlers** — keep them thin, delegate to `crates/core`
6. **Never use `any` type** — use `unknown` and narrow with type guards
7. **Never use `enum` in TypeScript** — use `const` objects with `as const`
8. **Never use `unsafe` in Rust** — forbidden at workspace level
9. **Never use `console.log`** — use `logger.warn()` or `logger.error()` (ESLint warns on console)
10. **Never add Tailwind default color scales that are disabled** — many color ranges are set to `initial` in `globals.css`
11. **Avoid improving adjacent code** when making surgical changes — match existing style

---

*Convention analysis: 2026-04-20*
