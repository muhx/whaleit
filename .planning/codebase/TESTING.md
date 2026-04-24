# Testing Patterns

**Analysis Date:** 2026-04-20

## Overview

Three test layers:

| Layer                   | Tool                 | Runs against                               |
| ----------------------- | -------------------- | ------------------------------------------ |
| TS unit / integration   | **Vitest 3** + jsdom | Frontend logic, hooks, React components    |
| Rust unit / integration | **`cargo test`**     | `crates/*`, `apps/tauri`, `apps/server`    |
| E2E                     | **Playwright**       | Web build (not Tauri) against real backend |

CI (`.github/workflows/pr-check.yml`) runs the first two in parallel jobs
(`frontend-check`, `rust-check`). E2E is **not** wired into PR CI — it is run
locally via `pnpm test:e2e`.

---

## TypeScript — Vitest

### Framework

- **Runner:** Vitest `^3.2.4`
- **Environment:** `jsdom` (`jsdom ^28.0.0`)
- **Globals:** enabled (`globals: true` in `apps/frontend/vite.config.ts:92-97`)
  — `describe`, `it`, `expect`, `vi` are available without import but code still
  imports them explicitly from `vitest` for clarity.
- **Assertion extras:** `@testing-library/jest-dom` matchers extended onto
  `expect` in `apps/frontend/src/test/setup.ts:5-7`.
- **Coverage provider:** `@vitest/coverage-v8`
- **Type:** `types: ["@tauri-apps/api", "vitest/globals"]` in
  `apps/frontend/tsconfig.json:4`

### Config

`apps/frontend/vite.config.ts:92-98`:

```ts
test: {
  globals: true,
  environment: "jsdom",
  setupFiles: "./src/test/setup.ts",
  include: ["**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
}
```

Setup file (`apps/frontend/src/test/setup.ts`):

- Extends `expect` with `@testing-library/jest-dom/matchers`
- Registers `afterEach(cleanup)` from `@testing-library/react`
- Mocks `window.matchMedia` (needed by Radix / theme components)

### Run Commands

Defined in `package.json:23-26` (root) and `apps/frontend/package.json:13-16`:

```bash
pnpm test              # vitest (runs once in CI because Vitest defaults to run in CI)
pnpm test:watch        # vitest --watch
pnpm test:ui           # vitest --ui
pnpm test:coverage     # vitest --coverage
```

The root `pnpm test` delegates to `pnpm --filter frontend test`.

### Test File Organization

- **Co-located** next to source: `schemas.ts` + `schemas.test.ts`,
  `activity-utils.ts` + `activity-utils.test.ts` in `apps/frontend/src/lib/`.
- **`__tests__/` folder** when multiple related tests exist:
  `apps/frontend/src/pages/activity/components/forms/__tests__/`,
  `.../fields/__tests__/`.
- **Extensions:** `.test.ts` for logic, `.test.tsx` for component tests.

Coverage across the repo (non-exhaustive):

- Schema/validation: `apps/frontend/src/lib/schemas.test.ts`,
  `.../forms/__tests__/form-schemas.test.ts`
- Pure utils: `apps/frontend/src/lib/activity-utils.test.ts`,
  `utils.timezone.test.ts`, `export-utils.test.ts`, `portfolio-helper.test.ts`
- React components:
  `apps/frontend/src/pages/activity/components/forms/__tests__/buy-form.test.tsx`,
  `transfer-form.test.tsx`, `sell-form.test.tsx`, `deposit-form.test.tsx`,
  `dividend-split-forms.test.tsx`, `simple-forms.test.tsx`
- Form fields: `.../forms/fields/date-picker.test.tsx`,
  `.../fields/__tests__/symbol-search.test.tsx`, `account-select.test.tsx`
- Hooks: `apps/frontend/src/pages/activity/hooks/use-activity-form.test.ts`,
  `apps/frontend/src/features/ai-assistant/hooks/use-chat-import-session.test.tsx`
- Services:
  `apps/frontend/src/features/devices-sync/services/sync-service.pairing.test.ts`
- Device-sync components:
  `apps/frontend/src/features/devices-sync/components/pairing-flow/index.test.tsx`,
  `device-sync-section.test.tsx`, `recovery-dialog.test.tsx`
- Pure logic utilities in features: `ai-assistant/types.test.ts`,
  `ai-assistant/components/tool-uis/record-activities-tool-utils.test.ts`

### Test Structure

Standard pattern (`apps/frontend/src/lib/schemas.test.ts:5-56`):

```ts
import { describe, expect, it } from "vitest";
import { importMappingSchema } from "./schemas";

describe("schemas", () => {
  describe("importMappingSchema", () => {
    it("should accept valid quoteMode values in symbolMappingMeta", () => {
      const validMapping = { accountId: "test-account", ... };
      const result = importMappingSchema.safeParse(validMapping);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.symbolMappingMeta?.AAPL.quoteMode).toBe(QuoteMode.MARKET);
      }
    });
  });
});
```

- Nested `describe` for grouping (module → function → scenario).
- Explicit imports from `vitest` (even though globals are on) for readability.
- `beforeEach(() => vi.clearAllMocks())` to reset state between tests
  (`apps/frontend/src/pages/activity/components/forms/__tests__/buy-form.test.tsx:118-120`).

### Mocking

Tool: `vi` from `vitest`.

**Module mocking** — use `vi.mock(path, factory)` at top of file:

```ts
vi.mock("@/hooks/use-settings", () => ({
  useSettings: () => ({
    data: { baseCurrency: "USD" },
    isLoading: false,
    error: null,
  }),
}));
```

**Hoisted refs** — when factory needs to reference mutable mocks, use
`vi.hoisted` so the mocks exist before `vi.mock` factory runs
(`apps/frontend/src/pages/activity/hooks/use-activity-form.test.ts:8-35`):

```ts
const mutationMocks = vi.hoisted(() => ({
  addMutateAsync: vi.fn(),
  updateMutateAsync: vi.fn(),
}));

vi.mock("./use-activity-mutations", () => ({
  useActivityMutations: () => ({
    addActivityMutation: { mutateAsync: mutationMocks.addMutateAsync, ... },
  }),
}));
```

**Heavy adapter mocking** — common in services tests
(`apps/frontend/src/features/devices-sync/services/sync-service.pairing.test.ts:3-66`)
where the entire `@/adapters`, `../storage/keyring`, and `../crypto` modules are
replaced with hoisted `vi.fn()` collections, then imports of the real service
are placed **after** the `vi.mock` calls.

**UI component stubs** — replace `@whaleit/ui/components/ui/*` with minimal DOM
stubs to isolate logic from styling
(`apps/frontend/src/pages/activity/components/forms/__tests__/buy-form.test.tsx:61-107`):

```ts
vi.mock("@whaleit/ui/components/ui/button", () => ({
  Button: ({ children, type, onClick, disabled }) => (
    <button type={type} onClick={onClick} disabled={disabled}>{children}</button>
  ),
}));
```

**What to mock:**

- Hooks that touch the backend (`useSettings`, `useAccounts`, mutation hooks).
- Adapter modules (`@/adapters`) so tests don't rely on Tauri runtime.
- Platform-specific packages (`@tauri-apps/plugin-log`, plugin dialogs).
- Heavy UI components when testing logic, not visuals.

**What NOT to mock:**

- Pure Zod schemas — test them directly.
- Pure utility functions (`lib/activity-utils.ts`, `lib/portfolio-helper.ts`).
- `react-hook-form` — used in real form tests via `FormProvider` wrapper
  (`apps/frontend/src/pages/activity/components/forms/fields/date-picker.test.tsx:39-49`).

### React Component Tests

- `@testing-library/react` + `@testing-library/user-event`.
- Queries preferred in order: `getByRole`, `getByLabelText`, `getByTestId` (for
  stubbed components) — consistent with React Testing Library guidance.
- User interactions via `userEvent.setup()` then `await user.click(...)`.
- Assertions via `expect(el).toBeInTheDocument()`, `.toBeDisabled()`, etc.
  (jest-dom matchers).

### Hooks Tests

Use `renderHook` + `act` from `@testing-library/react`
(`apps/frontend/src/pages/activity/hooks/use-activity-form.test.ts:51-57`):

```ts
const { result } = renderHook(() =>
  useActivityForm({ accounts, selectedType: "DEPOSIT" }),
);
// mockResolvedValue on mutation mocks before invoking
```

### Fixtures & Test Data

No shared fixtures framework. Test data is defined inline per test, usually as
`const mockAccounts: AccountSelectOption[] = [...]` at the top of the describe
block.

### Coverage

- Provider: `@vitest/coverage-v8`
- No enforced threshold in config (no `coverage.thresholds` entry).
- Run with `pnpm test:coverage`, outputs to `coverage/` (gitignored).

---

## Rust — `cargo test`

### Framework

- Standard `#[test]` and `#[tokio::test]` (Tokio `rt-multi-thread` from
  workspace deps).
- **Property-based:** `proptest` (`crates/core/Cargo.toml:59` dev-dep).
- **Temp filesystem:** `tempfile` (`crates/core/Cargo.toml:58` dev-dep,
  `apps/server/Cargo.toml:63` dev-dep).
- **HTTP client for integration tests:** `reqwest` with `json` feature
  (`apps/server/Cargo.toml:62`).
- Async traits via `async-trait`.
- Mock repositories hand-written, not via a framework (no `mockall`).

### Run Commands

```bash
cargo test                       # All workspace tests
cargo test --workspace           # Explicit workspace flag (used in CI)
cargo test -p whaleit-core   # Single crate
cargo test activities_service    # Filter by test name substring
```

CI (`pr-check.yml:87-92`) runs `cargo test --workspace` with
`CONNECT_API_URL=http://test.local` env var (required by storage-sqlite outbox
tests).

### Test File Organization

Three conventions observed:

1. **Inline `#[cfg(test)]` module** in a dedicated file, declared from `mod.rs`:
   - `crates/core/src/activities/mod.rs:13-17`:
     ```rust
     #[cfg(test)]
     mod activities_service_tests;
     #[cfg(test)]
     mod activities_model_tests;
     ```
   - File itself: `crates/core/src/activities/activities_service_tests.rs`
     (starts with `#[cfg(test)] mod tests { ... }`).

2. **`tests/` subdirectory inside `src/`** for submodule-scoped tests:
   - `crates/core/src/health/tests/mod.rs` + `property_tests.rs`.

3. **Crate-level `tests/` directory** for integration tests that use only the
   public API:
   - `crates/core/tests/health_property_tests.rs` — uses `whaleit_core::...` as
     an external consumer.
   - `apps/server/tests/auth.rs` — full Axum integration test.

### Test Structure

**Unit test** (`crates/core/src/activities/activities_model_tests.rs:15-62`):

```rust
#[cfg(test)]
mod tests {
    use crate::activities::activities_model::*;
    use chrono::{TimeZone, Utc};
    use rust_decimal_macros::dec;

    #[test]
    fn test_activity_status_default() {
        let status = ActivityStatus::default();
        assert_eq!(status, ActivityStatus::Posted);
    }

    #[test]
    fn test_activity_status_serialization_posted() {
        let status = ActivityStatus::Posted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""POSTED""#);
    }
}
```

Patterns:

- Test factory functions (e.g. `fn create_test_activity() -> Activity`) near top
  of the `mod tests` block for shared fixtures.
- `rust_decimal_macros::dec!(150.50)` for money literals.
- `serde_json::to_string(&value).unwrap()` for serialization tests.

### Mocking Pattern (hand-written)

Services depend on traits (`ActivityRepositoryTrait`, `AccountServiceTrait`).
Tests implement minimal mocks
(`crates/core/src/activities/activities_service_tests.rs:27-80`):

```rust
#[derive(Clone)]
struct MockAccountService {
    accounts: Arc<Mutex<Vec<Account>>>,
}

#[async_trait]
impl AccountServiceTrait for MockAccountService {
    async fn create_account(&self, _new: NewAccount) -> Result<Account> {
        unimplemented!()
    }
    fn get_account(&self, account_id: &str) -> Result<Account> {
        let accounts = self.accounts.lock().unwrap();
        accounts.iter().find(|a| a.id == account_id).cloned()
            .ok_or_else(|| Error::Unexpected("Account not found".into()))
    }
    // ... implement only the methods this test needs, stub others with
    // unimplemented!()
}
```

### Property-Based Tests

Use `proptest` with generators
(`crates/core/tests/health_property_tests.rs:14-80`):

```rust
use proptest::prelude::*;

fn arb_severity() -> impl Strategy<Value = Severity> {
    prop_oneof![
        Just(Severity::Info),
        Just(Severity::Warning),
        Just(Severity::Error),
        Just(Severity::Critical),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_global_status_reflects_highest_severity(
        issues in arb_health_issues(50),
    ) {
        // ... assert invariant holds for all generated inputs
    }
}
```

### Axum Integration Tests (`apps/server/tests/`)

Full router exercised via `tower::ServiceExt::oneshot`
(`apps/server/tests/auth.rs:15-100`):

```rust
async fn build_test_router(password: &str) -> axum::Router {
    let tmp = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", tmp.path().join("test.db"));
    // ... set WF_AUTH_PASSWORD_HASH, WF_SECRET_KEY, WF_CORS_ALLOW_ORIGINS
    let config = Config::from_env();
    let state = build_state(&config).await.unwrap();
    app_router(state, &config)
}

#[tokio::test]
async fn login_and_access_protected_route() {
    let app = build_test_router("super-secret").await;
    let response = app.clone().oneshot(
        Request::builder().uri("/api/v1/accounts").body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), 401);
}
```

Key details:

- `tempfile::tempdir` for isolated DB path.
- Env vars set before calling `Config::from_env()`; `cleanup_env()` helper
  resets them.
- `tower::ServiceExt::oneshot` drives the router without a TCP listener.
- `ConnectInfo` injected into request extensions for rate-limiter tests.

### Coverage

No coverage tool wired in. CI only runs `cargo test --workspace`.

---

## E2E — Playwright

### Framework

- `@playwright/test ^1.58.2` (dev-dep in root `package.json:48`)
- Config: `playwright.config.ts`
- Target: **web build only** (`http://localhost:1420`), **not Tauri**
  (`e2e/README.md:4-6`).
- Browser: Chromium with system Chrome channel, headless
  (`playwright.config.ts:37-40`).

### Config Highlights (`playwright.config.ts`)

```ts
testDir: "./e2e",
fullyParallel: false,        // Tests share DB state
workers: 1,                   // Serial execution required
retries: process.env.CI ? 2 : 0,
forbidOnly: !!process.env.CI,
reporter: "html",
use: { trace: "on-first-retry" },
```

Tests run **serially** because spec 10 depends on spec 01 seeding data.
`fullyParallel: false` + `workers: 1` is mandatory.

### Run Commands

```bash
pnpm test:e2e            # node scripts/run-e2e.mjs — full automated run
pnpm test:e2e:ui         # adds --ui for Playwright inspector
```

`scripts/run-e2e.mjs` orchestrates:

1. Calls `prepE2eEnv()` (`scripts/prep-e2e.mjs`) — rewrites `WF_DB_PATH` in
   `.env.web` to a fresh timestamped SQLite file (e.g.
   `./db/app-testing-20260411T120000Z.db`).
2. Spawns `pnpm run dev:web` (launches frontend on `:1420` and Axum on `:8088`).
3. Polls both servers via `fetch` with a 60 s / 120 s timeout.
4. Runs `pnpm exec playwright test`.
5. Cleans up dev server on exit/SIGINT/SIGTERM.

**Manual workflow** (for single-spec debugging) documented in
`e2e/README.md:37-94`:

```bash
node scripts/prep-e2e.mjs                    # fresh DB
pnpm run dev:web > /tmp/whaleit-dev2.log 2>&1 &
./scripts/wait-for-both-servers-to-be-ready.sh
npx playwright test e2e/<spec>.spec.ts [--headed|--debug]
```

### Test Structure

**Serial mode + shared `page`** (`e2e/01-happy-path.spec.ts:3-9`):

```ts
test.describe.configure({ mode: "serial" });

test.describe("Onboarding And Main Flow", () => {
  const BASE_URL = "http://localhost:1420";
  const TEST_PASSWORD = "password001";
  let page: Page;

  // test.beforeAll/afterAll for shared page/setup
  // test steps use `page` as closure
});
```

**Helpers** (`e2e/helpers.ts`):

- `BASE_URL`, `TEST_PASSWORD` constants.
- `loginIfNeeded(page)` — handles both onboarding and existing sessions.
- `createAccount(page, name, currency, trackingMode)` — idempotent account
  creation.
- `openAddActivitySheet(page)`, `selectActivityType(page, type)`,
  `searchAndSelectSymbol(page, symbol)`, `expandAdvancedOptions(page)` — domain
  UI helpers.
- `fillDateField(page, daysAgo)` — types into each React Aria `DateInput`
  segment (`data-type="month"`, `"day"`, `"year"`, `"hour"`, `"minute"`,
  `"dayPeriod"`).
- `waitForOverlayClose(page)`, `waitForSyncToast(page, maxWaitMs)` — deals with
  async UI.

### Test Files (`e2e/`)

| File                                   | Scope                                  |
| -------------------------------------- | -------------------------------------- |
| `01-happy-path.spec.ts`                | Onboarding, accounts, deposits, trades |
| `02-activities.spec.ts`                | All activity types                     |
| `03-fx-cash-balance.spec.ts`           | FX cash balances                       |
| `04-csv-import.spec.ts`                | CSV activity import                    |
| `05-form-validation.spec.ts`           | Form validation errors                 |
| `06-activity-data-grid.spec.ts`        | Activity data grid                     |
| `07-asset-creation.spec.ts`            | Manual asset creation/edit             |
| `08-holdings-and-performance.spec.ts`  | Holdings + performance views           |
| `09-bulk-holdings.spec.ts`             | Bulk holdings CSV import               |
| `10-symbol-mapping-validation.spec.ts` | Symbol mapping real-time validation    |

Naming: numeric prefix enforces execution order. Spec 10 depends on spec 01.

### Locator Strategy

In order of preference (observed in specs + helpers):

1. `page.getByRole("button", { name: "Add Activities" })`
2. `page.getByLabel("Account Name")`
3. `page.getByPlaceholder("Search for symbol")`
4. `page.getByTestId("date-picker")` — used for React Aria composites and custom
   widgets
5. `page.getByText(...)` / `page.locator('[data-type="month"]')` — last resort

No mocks of backend services — tests exercise the real Axum server against a
fresh SQLite DB.

### Assertions

Standard Playwright `expect`:

- `await expect(locator).toBeVisible({ timeout: 15000 })`
- `await expect(locator).not.toBeVisible({ timeout: 10000 })`
- `await expect(locator).toBeHidden({ timeout: 15000 })`

### Traces & Reports

- `trace: "on-first-retry"` — traces only captured on retries (CI).
- `reporter: "html"` — view with `npx playwright show-report`.
- Reports and traces excluded from root ESLint (`eslint.config.js:25-27`:
  `playwright-report/**`, `test-results/**`, `e2e/**`).

---

## CI Integration

**`.github/workflows/pr-check.yml`** runs on PRs to `main`, `develop`,
`feature/**`:

### `frontend-check` job

```yaml
- pnpm install --frozen-lockfile
- pnpm run build:types # Build @whaleit/ui + addon-sdk
- pnpm format:check # Prettier
- pnpm lint # ESLint
- pnpm type-check # tsc --noEmit
- pnpm test # Vitest
- pnpm build # Vite web build
```

### `rust-check` job

```yaml
- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets --all-features -- -D warnings
- cargo test --workspace
  env: { CONNECT_API_URL: http://test.local }
- cargo check -p whaleit-server --release
```

### `build-status` job

Gate that fails if either check fails or is cancelled.

**E2E is not part of CI.** It must be run locally before shipping UI-affecting
changes.

---

## Common Patterns

### Async TS testing

```ts
it("waits for mutation", async () => {
  const { result } = renderHook(() => useSomething());
  await act(async () => {
    await result.current.mutate(input);
  });
  expect(mockMutateAsync).toHaveBeenCalledWith(expected);
});
```

### Error testing (Zod)

```ts
const result = schema.safeParse(invalid);
expect(result.success).toBe(false);
if (!result.success) {
  expect(result.error.issues[0].message).toBe("Please select an account.");
}
```

### Rust async test

```rust
#[tokio::test]
async fn login_and_access_protected_route() {
    let app = build_test_router("password").await;
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
}
```

### Rust serialization round-trip

```rust
#[test]
fn test_activity_status_deserialization() {
    let posted: ActivityStatus = serde_json::from_str(r#""POSTED""#).unwrap();
    assert_eq!(posted, ActivityStatus::Posted);
}
```

---

_Testing analysis: 2026-04-20_
