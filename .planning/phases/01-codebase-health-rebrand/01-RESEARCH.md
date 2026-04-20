# Phase 1: Codebase Health & Rebrand - Research

**Researched:** 2026-04-20
**Status:** Complete

## Research Question
What do I need to know to PLAN the codebase health and rebrand phase well?

## Summary

Phase 1 involves four distinct workstreams: (1) comprehensive rebrand of all user-facing "Wealthfolio" references to "WhaleIt", (2) visual identity creation with new color palette and icon, (3) modular split of the monolithic web adapter (1,394 lines, 184-case switch), and (4) domain-based split of the God types file (1,929 lines). Internal Rust crate names (`wealthfolio-*`) remain unchanged. The work is largely mechanical with high volume (1,583+ frontend references) but zero architectural risk since all changes are cosmetic or refactoring-only.

---

## 1. Rebrand Scope Analysis

### Frontend (TypeScript/TSX)
- **371 files** contain "Wealthfolio/wealthfolio" references in `apps/frontend/src/`
- **1,583 total line matches** across frontend, tauri, and server
- Heaviest concentration: `features/wealthfolio-connect/` directory (29 files, ~180 refs)
- Import references to `@wealthfolio/ui` and `@wealthfolio/addon-sdk` in ~30 files

### Package Scope Rename
Three packages need `@wealthfolio/*` → `@whaleit/*` rename:
| Package | Current Name | New Name |
|---------|-------------|----------|
| UI lib | `@wealthfolio/ui` | `@whaleit/ui` |
| Addon SDK | `@wealthfolio/addon-sdk` | `@whaleit/addon-sdk` |
| Addon Dev Tools | `@wealthfolio/addon-dev-tools` | `@whaleit/addon-dev-tools` |

**Import impact:** All `import { ... } from "@wealthfolio/ui"` across ~30 files change to `@whaleit/ui`.

**Vite alias:** `apps/frontend/vite.config.ts` line 42 maps `@wealthfolio/ui` → must update to `@whaleit/ui`.

### Tauri Configuration
- `apps/tauri/tauri.conf.json`: `productName`, `mainBinaryName`, `identifier`, window `title`, deep-link schemes
- `apps/tauri/gen/apple/project.yml`: `bundleIdPrefix`, `PRODUCT_NAME`, `PRODUCT_BUNDLE_IDENTIFIER`, entitlement paths
- `apps/tauri/src/menu.rs`: Menu label "Wealthfolio" and "About Wealthfolio"
- `apps/tauri/src/lib.rs`: Error message string

### Server/Backend (User-Facing Strings Only)
- `apps/server/src/main_lib.rs`: "Wealthfolio Server" display name
- `apps/server/src/auth.rs`: Internal JWT/secret salt strings ("wealthfolio-jwt", "wealthfolio-secrets") — **these should NOT change** (cryptographic salt change would invalidate existing tokens/secrets)
- `apps/server/src/api/connect.rs`: Comments only
- `apps/server/src/api.rs`: OpenAPI tag name "wealthfolio" — should change to "whaleit"
- `apps/server/src/api/settings.rs`: Backup filename prefix "wealthfolio_backup_"

### Documentation & Config
- `README.md`: 54 references
- `Dockerfile`: Binary names, DB path, comments
- `compose.yml` / `compose.dev.yml`: Service names, container names, image names, volume names
- `apps/frontend/src/features/wealthfolio-connect/`: 29-file directory rename to `features/connect/`
- `apps/frontend/src/pages/settings/wealthfolio-connect/`: Settings page directory rename

### Internal Rust Crates (NO CHANGE per D-06/D-07)
- 577 `use wealthfolio_*` and crate-internal references stay unchanged
- Cargo.toml `name` fields stay `wealthfolio-core`, `wealthfolio-storage-sqlite`, etc.
- `apps/tauri/src/context/providers.rs`, `registry.rs`, `listeners.rs` — all internal

### AI System Prompt
- `crates/ai/src/system_prompt.txt`: "You are Wealthfolio AI" → "You are WhaleIt AI"

---

## 2. Visual Identity Scope

### Current State
- `apps/tauri/icons/`: Full icon set (`.icns`, `.ico`, `.png` at multiple sizes, iOS `AppIcon.icon`)
- `apps/frontend/public/`: `logo.svg`, `logo.png`, `app-icon-192.png`, `app-icon-512.png`, `splashscreen.png`, `wf-vector.png`
- `apps/frontend/src/globals.css`: 961-line CSS with `@theme` block containing:
  - Font families: Inter, Merriweather, IBM Plex Mono
  - Color tokens: `--color-base-*` (warm paper tones, 50-950), `--color-paper`, `--color-black`
  - Semantic colors: red, green, blue, amber scales
  - Disabled scales: lime, emerald, teal, sky, indigo, violet, fuchsia, pink, rose, slate, zinc, neutral, stone

### New Identity Requirements (D-08, D-09, D-10, D-11)
- Fresh color palette (not warm paper tones)
- Soft illustration whale icon (AI-generated placeholder)
- Brand presence: app icon, splash/loading screen, header/sidebar small element
- Tagline: "Your friendly finance companion"

### Technical Approach
- Replace `@theme` block color tokens wholesale in `globals.css`
- Generate placeholder icon assets in required formats (macOS `.icns`, Windows `.ico`, Linux `.png`, web manifest icons)
- Update `splashscreen.png` and logo files in `apps/frontend/public/`
- Update `apps/tauri/icons/` with new icon set

---

## 3. Web Adapter Modular Split

### Current State
- **File:** `apps/frontend/src/adapters/web/core.ts` — 1,394 lines
- **184 switch cases** mapping command names to HTTP API calls
- Structure: COMMANDS map (constants) + `invoke()` function with giant switch
- Only 2 "Wealthfolio" references (in COMMANDS map paths)

### Natural Grouping (from `adapters/shared/`)
The existing shared adapter already organizes commands into 16 modules:
1. `accounts.ts`
2. `activities.ts`
3. `ai-providers.ts`
4. `ai-threads.ts`
5. `alternative-assets.ts`
6. `connect.ts`
7. `contribution-limits.ts`
8. `custom-provider.ts`
9. `exchange-rates.ts`
10. `goals.ts`
11. `health.ts`
12. `market-data.ts`
13. `platform.ts`
14. `portfolio.ts`
15. `secrets.ts`
16. `taxonomies.ts`

### Proposed Architecture
- Keep `core.ts` as the dispatcher: exports `invoke()`, `COMMANDS` map, and re-exports from modules
- Each module handles its subset of switch cases + helper functions
- Module files live in `apps/frontend/src/adapters/web/modules/` (or follow shared pattern)
- Switch statement stays but delegates: `case "get_accounts": return accountsModule.getAccounts(payload)`

### Command-to-Domain Mapping
From the 184 commands, the grouping by domain (based on shared adapter):
- **Accounts** (3): get/create/update/delete_account
- **Activities** (12): search, create, update, delete, save, import/check/mapping/templates
- **Holdings/Snapshots** (10): get_holding(s), snapshots, manual_holdings, import_holdings
- **Portfolio** (5): allocations, recalculate, update, performance
- **Performance** (3): simple, history, summary
- **Goals** (6): CRUD + allocations
- **FX/Exchange Rates** (4): latest, add, update, delete
- **AI** (12): providers, threads, messages, models, tags, chat stream
- **Connect/Broker** (12): sync, ingest, broker accounts/connections, subscription plans, sync session
- **Market Data/Quotes** (7): import, latest, history, check, sync, update mode/quote
- **Taxonomies** (8): CRUD, categories, assignments, import/export, migration
- **Health** (5): status, config, checks, dismiss, restore, fix
- **Device Sync** (18): pairing flow, devices, E2EE keys, team sync, reconcile
- **Settings** (2): get, update, auto-update
- **Secrets** (2): get, set, delete
- **Import** (5): runs, data import runs, templates
- **Assets** (6): create, profile, alternative assets, symbol search/resolve
- **Liabilities** (2): link, unlink
- **Platform/Utilities** (5): backup, restore, app info, check update, net worth
- **Contribution Limits** (3): create, update, calculate deposits
- **Custom Providers** (4): create, update, test, set default
- **Addons** (10): install, uninstall, toggle, ratings, staging, load, check updates

---

## 4. Types.ts Domain Split

### Current State
- **File:** `apps/frontend/src/lib/types.ts` — 1,929 lines
- ~120 exported interfaces and types
- 1 deprecated type: `ActivityLegacy` (line 70, deprecated at line 68) — **zero usages outside types.ts**

### Domain Groupings (proposed)
Based on interface naming patterns and domain ownership:

| Domain | Types | Est. Lines |
|--------|-------|-----------|
| account | Account, AccountSummaryView, AccountGroup, AccountValuation, AccountDeposit | ~80 |
| activity | Activity, ActivityLegacy (remove), ActivityDetails, ActivitySearchResponse, ActivityCreate, ActivityUpdate, ActivityBulkMutation*, ActivityImport, ImportMappingData, ParseConfig, ImportTemplateData, CsvRowData, CsvRowError, ParseError, ParsedCsvResult, ImportValidation*, ImportActivities*, SymbolInput | ~400 |
| asset | Asset, NewAsset, AssetClassifications, AssetTaxonomyAssignment, UpdateAssetProfile, StaleAssetInfo, Instrument | ~120 |
| holding | Holding, HoldingSummary, CashHolding, AllocationHoldings, Position, Lot, MonetaryValue, SnapshotInfo, HoldingsPositionInput, HoldingsSnapshotInput, ImportHoldingsCsvResult, SymbolCheckResult, CheckHoldingsImportResult, AlternativeAssetHolding | ~250 |
| portfolio | PortfolioAllocations, PerformanceMetrics, SimplePerformanceMetrics, ReturnData, DateRange, TimePeriod, IncomeSummary, IncomeByAsset, IncomeByAccount | ~120 |
| quote | Quote, QuoteUpdate, LatestQuoteSnapshot, ResolvedQuote, ExchangeInfo, MarketData, MarketDataProviderInfo, SymbolSearchResult | ~100 |
| goal | Goal, GoalAllocation, GoalProgress | ~40 |
| settings | Settings, SettingsContextType, NetWorthConfig | ~40 |
| fx | ExchangeRate | ~15 |
| taxonomy | Taxonomy*, TaxonomyAllocation, CategoryRef, CategoryWithWeight, CategoryAllocation, MigrationStatus, MigrationResult, TaxonomyJson* | ~120 |
| health | HealthIssue, HealthStatus, HealthConfig, HealthSeverity, HealthCategory, AffectedItem, FixAction, NavigateAction | ~100 |
| ai | AiProvidersResponse, MergedModel, MergedProvider, ModelCapabilities*, ProviderTuning*, ConnectionField, CapabilityInfo, ChatConfig, ListModelsResponse, FetchedModel, UpdateProviderSettingsRequest, SetDefaultProviderRequest, ModelCapabilityOverrideUpdate | ~250 |
| sync | SyncStatus, ImportRun*, BrokerSyncState, BrokerSyncProfileData, SaveBrokerSyncProfileRulesRequest, BrokerProfileScope, TemplateKind, TemplateContextKind | ~120 |
| device | TrackedItem, DeviceSyncState (if any) | ~20 |
| alternative-assets | AlternativeAssetHolding, AlternativeAssetKindApi, CollectibleMetadata, PreciousMetalMetadata, PropertyMetadata, VehicleMetadata, CreateAlternativeAsset*, UpdateAlternativeAsset*, UpdateValuation* | ~120 |
| contributions | ContributionLimit, NewContributionLimit, DepositsCalculation, calculate_deposits_for_contribution_limit | ~30 |
| liabilities | LiabilitiesSection, LiabilityMetadata, LinkLiabilityRequest | ~30 |
| net-worth | NetWorthResponse, NetWorthHistoryPoint | ~20 |
| ui | ReviewMode, BreakdownItem, Tag, Platform, UpdateInfo | ~40 |

### Barrel Re-export Strategy
`types.ts` becomes a barrel file that re-exports everything:
```typescript
export * from './types/account';
export * from './types/activity';
// ... etc
```
This preserves backward compatibility — all existing `import { X } from '@/lib/types'` keep working.

---

## 5. Onboarding Scope

### Current Files
- `apps/frontend/src/pages/onboarding/` — 5 files:
  - `onboarding-page.tsx` (6,605 bytes)
  - `onboarding-step1.tsx` (6,059 bytes)
  - `onboarding-step2.tsx` (18,670 bytes)
  - `onboarding-appearance.tsx` (10,364 bytes)
  - `onboarding-connect.tsx` (3,198 bytes)

### Per D-15
Rebrand-only — swap branding text, update colors/logo references. Keep existing step structure. No flow redesign.

---

## 6. Feature Directory Rename

### Current
- `apps/frontend/src/features/wealthfolio-connect/` → rename to `features/connect/`
- `apps/frontend/src/pages/settings/wealthfolio-connect/` → rename to `pages/settings/connect/`
- Route paths already use `/connect` (line 95, 125 in routes.tsx) — no route change needed
- Import paths: 4 files import from `@/features/wealthfolio-connect/...`

---

## 7. Risk Assessment

### Low Risk
- **Package scope rename** (`@wealthfolio/*` → `@whaleit/*`): Mechanical find-and-replace in package.json files and import statements
- **Feature directory rename**: 4 import path updates
- **Types.ts barrel split**: Zero breaking changes if barrel re-exports correctly
- **Onboarding rebrand**: Text/color swaps only
- **Deprecated type removal**: `ActivityLegacy` has zero external usages

### Medium Risk
- **Web adapter split**: 184 cases must all map correctly. Risk is a missed case or incorrect function signature. Mitigated by shared adapter modules providing the canonical grouping.
- **Tauri config changes**: Must maintain deep-link schemes and updater endpoints. Testing required on both desktop and web builds.
- **Color palette swap**: May expose hardcoded colors not using CSS variables. Need to audit for non-token color references.

### Validation Strategy
- `pnpm build` — catches broken imports immediately
- `pnpm type-check` — catches type errors from types.ts split
- `cargo check` — catches Rust string changes that break compilation
- `pnpm test` — existing tests validate nothing broke
- Manual desktop launch — verify title, icon, menu items
- Manual web launch — verify API endpoints, UI rendering

---

## Validation Architecture

### Dimension 1: Build Integrity
- **V:** `pnpm build && cargo check` pass after every change
- **I:** Build failures from broken imports, missing barrel exports, incorrect type references

### Dimension 2: Type Safety
- **V:** `pnpm type-check` passes — zero type errors from types.ts split
- **I:** Missing barrel re-exports, incorrect import paths

### Dimension 3: Test Continuity
- **V:** `pnpm test && cargo test` pass — existing tests unaffected
- **I:** Test files importing from old paths, mock data using old type names

### Dimension 4: Visual Consistency
- **V:** No "Wealthfolio" text visible in running app (desktop + web)
- **I:** Missed user-facing strings, cached builds

### Dimension 5: Icon/Asset Presence
- **V:** All icon formats exist in expected paths
- **I:** Missing `.icns`/`.ico`/`.png` files breaks desktop build

---

## Key Technical Decisions for Planning

1. **Types.ts split execution order**: Create domain files + barrel first, then migrate, then remove `ActivityLegacy`. This avoids a "big bang" change.
2. **Web adapter split approach**: Create module files first, then wire into switch. Each module can be tested independently.
3. **Rebrand execution order**: Package scope → Tauri config → Rust strings → Frontend text → Documentation. This follows the dependency chain.
4. **Color palette approach**: Replace `@theme` tokens wholesale. Audit for hardcoded colors in components. Update `globals.css` only — components using tokens get automatic update.
5. **Icon generation**: AI-generated placeholder sufficient for dev. Use `iconutil` (macOS) for `.icns`, ImageMagick for `.ico`, sharp/squoosh for PNG sizes.

---

*Research completed: 2026-04-20*
