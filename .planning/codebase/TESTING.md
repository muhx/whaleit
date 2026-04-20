# Testing Patterns

**Analysis Date:** 2026-04-20

## Test Framework

### Frontend (TypeScript)

**Runner:** Vitest 3.2.x
- Config: `apps/frontend/vite.config.ts` (test section)
- Environment: `jsdom`
- Globals: `true` (no need to import `describe`, `it`, `expect` — but tests still import explicitly)
- Setup: `apps/frontend/src/test/setup.ts`

**Assertion Library:**
- Vitest built-in (`expect`)
- `@testing-library/jest-dom` for DOM assertions (`toBeInTheDocument`, `toHaveTextContent`, etc.)
- `@testing-library/react` for component testing
- `@testing-library/user-event` for user interaction simulation

**Build Tool:** Vite 7.x (shared config with dev server)

### Backend (Rust)

**Runner:** `cargo test` (built-in Rust test framework)
- Property-based testing: `proptest` crate (used in `crates/core/tests/`)
- Dev dependencies: `tempfile`, `proptest`

### E2E

**Runner:** Playwright 1.58.x
- Config: `playwright.config.ts` (project root)
- Browser: Chromium only (headless)
- Sequential execution (`fullyParallel: false`, `workers: 1`)

---

## Run Commands

| Command | What it runs | Working directory |
|---------|-------------|-------------------|
| `pnpm test` | All Vitest unit tests | Project root |
| `pnpm test:watch` | Vitest in watch mode | Project root |
| `pnpm test:ui` | Vitest with UI | Project root |
| `pnpm test:coverage` | Vitest with V8 coverage | Project root |
| `pnpm test:e2e` | Playwright E2E tests (starts dev servers) | Project root |
| `pnpm test:e2e:ui` | Playwright with UI | Project root |
| `cargo test` | All Rust tests | Project root |
| `cargo test --workspace` | All workspace crate tests | Project root |
| `cargo test -p wealthfolio-core` | Tests for specific crate | Project root |
| `pnpm check` | format:check + lint + type-check (no tests) | Project root |

---

## Test File Organization

### Frontend

**Location:** Co-located with source files

**Patterns:**
1. **Same directory co-location** — test file next to source file:
   - `src/lib/activity-utils.ts` → `src/lib/activity-utils.test.ts`
   - `src/lib/schemas.ts` → `src/lib/schemas.test.ts`
   - `src/lib/portfolio-helper.ts` → `src/lib/portfolio-helper.test.ts`

2. **`__tests__` subdirectory** — grouped tests for larger components:
   - `src/pages/activity/components/forms/__tests__/buy-form.test.tsx`
   - `src/pages/activity/components/forms/__tests__/sell-form.test.tsx`
   - `src/pages/activity/components/forms/__tests__/deposit-form.test.tsx`
   - `src/pages/activity/components/forms/__tests__/transfer-form.test.tsx`
   - `src/pages/activity/components/forms/__tests__/form-schemas.test.ts`
   - `src/pages/activity/components/mobile-forms/__tests__/validate-transfer-fields.test.ts`

**Naming:** `<source-name>.test.ts` or `<source-name>.test.tsx`

**Current test count:** ~41 test files

### Backend (Rust)

**Location:** Two patterns:

1. **Inline tests** — `#[cfg(test)]` module at bottom of source file:
   - Most Rust source files have inline test modules
   - ~1150 `#[test]` annotations across all Rust source files
   - Examples: `crates/storage-sqlite/src/ai_chat/repository.rs`, `crates/market-data/src/resolver/rules_resolver.rs`

2. **Integration tests** — `crates/<crate>/tests/` directory:
   - `crates/core/tests/health_property_tests.rs` — property-based tests using `proptest`

### E2E

**Location:** `e2e/` directory at project root

**Naming:** Numbered spec files for execution order:
- `e2e/01-happy-path.spec.ts` — Onboarding and main flow
- `e2e/02-activities.spec.ts`
- `e2e/03-fx-cash-balance.spec.ts`
- `e2e/04-csv-import.spec.ts`
- `e2e/05-form-validation.spec.ts`
- `e2e/06-activity-data-grid.spec.ts`
- `e2e/07-asset-creation.spec.ts`
- `e2e/08-holdings-and-performance.spec.ts`
- `e2e/09-bulk-holdings.spec.ts`
- `e2e/10-symbol-mapping-validation.spec.ts`

**Helpers:** `e2e/helpers.ts` — shared utilities (`fillDateField`, `createAccount`, `loginIfNeeded`, etc.)

---

## Test Setup

### Frontend Setup File

**Location:** `apps/frontend/src/test/setup.ts`

```typescript
import "@testing-library/jest-dom";
import { cleanup } from "@testing-library/react";
import { vi, afterEach } from "vitest";
import * as matchers from "@testing-library/jest-dom/matchers";

expect.extend(matchers);

afterEach(() => {
  cleanup();
});

// Mock window.matchMedia for components using media queries
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
```

---

## Test Structure

### Unit Tests (TypeScript)

```typescript
import { describe, expect, it } from "vitest";
import { functionUnderTest } from "./module";

describe("Module Name", () => {
  describe("functionUnderTest", () => {
    it("should handle normal case", () => {
      expect(functionUnderTest(input)).toBe(expected);
    });

    it("should handle edge case", () => {
      expect(functionUnderTest(edgeInput)).toBe(fallback);
    });
  });
});
```

**Pattern:** Nested `describe` blocks grouping related functions, with `it` blocks describing expected behavior.

### Component Tests (TypeScript)

```typescript
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

// Mock external dependencies
vi.mock("@/adapters", () => ({
  calculatePerformanceSummary: vi.fn(),
}));

vi.mock("@/hooks/use-accounts", () => ({
  useAccounts: vi.fn(),
}));

// Create typed mock references
const mockUseAccounts = vi.mocked(useAccounts);

// Factory functions for test data
function createAccount(overrides: Partial<Account>): Account {
  return {
    id: overrides.id ?? "account-1",
    name: overrides.name ?? "Account 1",
    // ... defaults
    ...overrides,
  };
}

// Helper render function with providers
function renderComponent(props) {
  mockUseAccounts.mockReturnValue({ accounts: props.accounts, isLoading: false });
  return render(
    <MemoryRouter>
      <ComponentUnderTest />
    </MemoryRouter>,
  );
}

describe("ComponentName", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render correctly", async () => {
    const user = userEvent.setup();
    renderComponent({ accounts: [createAccount()] });
    expect(screen.getByText("Account 1")).toBeInTheDocument();
  });
});
```

**Key patterns:**
- `vi.mock()` at top level for all external dependencies
- `vi.mocked()` for typed mock references
- Factory functions (`createAccount`, `createValuation`) for test data
- `vi.clearAllMocks()` in `beforeEach`
- Wrap in `MemoryRouter` when testing routed components
- `userEvent.setup()` for realistic user interactions
- `data-testid` attributes for stable selectors

### Mocking Patterns

**Mock hooks:**
```typescript
vi.mock("@/hooks/use-settings", () => ({
  useSettings: () => ({
    data: { baseCurrency: "USD" },
    isLoading: false,
    error: null,
  }),
}));
```

**Mock adapter layer:**
```typescript
vi.mock("@/adapters", () => ({
  calculatePerformanceSummary: vi.fn(),
  getSettings: vi.fn(),
}));
```

**Mock UI components (simplified stubs):**
```typescript
vi.mock("@wealthfolio/ui/components/ui/button", () => ({
  Button: ({ children, ...props }) => <button {...props}>{children}</button>,
}));
```

**Mock with `vi.fn().mockImplementation()` for conditional behavior:**
```typescript
vi.mock("@tanstack/react-query", () => ({
  useQueries: vi.fn(),
}));
// Then in test:
mockUseQueries.mockImplementation(({ queries }) =>
  queries.map(q => ({ isLoading: false, data: mockData[q.queryKey[2]] }))
);
```

### Rust Tests

**Inline unit tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // arrange
        let input = ...;
        // act
        let result = function_under_test(input);
        // assert
        assert_eq!(result, expected);
    }
}
```

**Property-based tests (proptest):**
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
    #[test]
    fn test_property_severity_ordering(severity in arb_severity()) {
        // property that must hold for all generated values
    }
}
```

---

## E2E Testing

### Framework

- **Playwright** — configured in `playwright.config.ts`
- Chromium only, headless mode
- Sequential execution (not parallel) — tests depend on order

### Test Scenarios Covered

| Spec | Description |
|------|-------------|
| `01-happy-path` | Full onboarding → create accounts → add activities → verify dashboard |
| `02-activities` | Activity CRUD operations |
| `03-fx-cash-balance` | Foreign exchange and cash balance handling |
| `04-csv-import` | CSV file import workflow |
| `05-form-validation` | Form validation rules |
| `06-activity-data-grid` | Activity data grid interactions |
| `07-asset-creation` | Asset creation flow |
| `08-holdings-and-performance` | Holdings display and performance calculations |
| `09-bulk-holdings` | Bulk holdings import |
| `10-symbol-mapping-validation` | Symbol mapping during import |

### E2E Helpers

Located in `e2e/helpers.ts`:
- `loginIfNeeded(page)` — handles web mode authentication
- `createAccount(page, name, currency, trackingMode)` — creates test account via UI
- `fillDateField(page, daysAgo)` — fills React Aria date segments
- `searchAndSelectSymbol(page, symbol)` — symbol search combobox interaction
- `openAddActivitySheet(page)` — opens activity form sheet
- `selectActivityType(page, type)` — selects activity type button
- `waitForSyncToast(page)` — waits for market data sync to complete
- `waitForOverlayClose(page)` — waits for dialog/sheet close

### E2E Runner

**Script:** `scripts/run-e2e.mjs`:
1. Runs `scripts/prep-e2e.mjs` to prepare test environment
2. Starts dev web server (`pnpm run dev:web`)
3. Waits for frontend (localhost:1420) and backend (localhost:8088) to be healthy
4. Runs Playwright tests
5. Cleans up dev server on exit

**Environment variables:**
- `WF_E2E_BASE_URL` — frontend URL (default: `http://localhost:1420`)
- `WF_E2E_BACKEND_URL` — backend URL (default: `http://localhost:8088`)

---

## Coverage

**Frontend:**
- Coverage tool: `@vitest/coverage-v8`
- Command: `pnpm test:coverage`
- No enforced coverage threshold detected

**Backend (Rust):**
- No coverage tooling configured
- `cargo test --workspace` runs all tests in CI

---

## CI Integration

**PR Check workflow** (`.github/workflows/pr-check.yml`):

Frontend job:
1. `pnpm install --frozen-lockfile`
2. `pnpm run build:types`
3. `pnpm format:check`
4. `pnpm lint`
5. `pnpm type-check`
6. `pnpm test`
7. `pnpm build`

Rust job:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace` (with `CONNECT_API_URL=http://test.local`)

E2E: Not run in CI (no workflow detected)

---

## Test Gaps

### Untested Areas

**Frontend:**
- **Adapter layer** — `src/adapters/tauri/` and `src/adapters/web/` have no unit tests
- **Context providers** — `auth-context.tsx`, `privacy-context.tsx`, `portfolio-sync-context.tsx`
- **Pages with minimal testing** — Most page-level components lack tests
- **Route definitions** — `routes.tsx` has no tests
- **Addon system** — `src/addons/` runtime loader is untested except for `type-bridge.test.ts`

**Backend (Rust):**
- **Integration tests sparse** — Only `health_property_tests.rs` in `crates/core/tests/`
- Most test coverage comes from inline `#[cfg(test)]` modules
- `crates/ai/` — no dedicated test file detected
- `crates/connect/` — broker integration tests depend on external services

**E2E:**
- No mobile/responsive viewport testing
- Only Chromium browser tested
- No performance or load testing
- Not integrated into CI pipeline

### Critical Paths Needing Coverage

1. **Activity bulk mutation** — `ActivityBulkMutationRequest` path through adapters
2. **Holdings snapshot import** — Full CSV import → validation → commit flow
3. **Currency conversion** — FX rate application across accounts
4. **Device sync pairing** — End-to-end encryption handshake
5. **AI chat tool execution** — `record_activities` and `import_csv` tools

---

*Testing analysis: 2026-04-20*
