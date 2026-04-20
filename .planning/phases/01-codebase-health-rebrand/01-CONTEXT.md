# Phase 1: Codebase Health & Rebrand - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Rename all user-facing "Wealthfolio" references to "WhaleIt" identity across app UI, configuration, documentation, and package namespaces. Clean up two critical codebase bottlenecks — the monolithic web adapter switch statement and the God types file — to prepare for feature expansion in Phases 2-12. Deliver new WhaleIt visual identity (icon, color palette, brand presence in UI). Keep internal crate names unchanged.

</domain>

<decisions>
## Implementation Decisions

### Rebrand Boundary
- **D-01:** Rename npm package scope from `@wealthfolio/*` to `@whaleit/*` — covers `@whaleit/ui`, `@whaleit/addon-sdk`, `@whaleit/addon-dev-tools`. All imports across 431 TS/TSX files update accordingly.
- **D-02:** Rename `features/wealthfolio-connect/` directory to `features/connect/` and update route paths from `/wealthfolio-connect` to `/connect`.
- **D-03:** Rename all user-visible Rust strings referencing "Wealthfolio" — window titles in `menu.rs`, AI system prompt in `crates/ai/src/system_prompt.txt`, about page strings, etc. Internal-only strings (crate descriptions, log messages, error types) stay as-is.
- **D-04:** Update Tauri bundle identifier from `com.teymz.wealthfolio` to `com.whaleit.app` (or similar `com.whaleit.*`). Update in `apps/tauri/tauri.conf.json` and `apps/tauri/gen/apple/project.yml`.
- **D-05:** Full documentation rebrand — README.md, Dockerfile labels, compose.yml service names, GitHub repo metadata, all docs in `docs/` directory.
- **D-06:** Internal crate names (`wealthfolio-core`, `wealthfolio-storage-sqlite`, etc.) remain unchanged per BRAND-06. Cargo.toml `name` fields stay as-is.
- **D-07:** Internal Rust crate references in Cargo.toml `package = "wealthfolio-*"` stay unchanged. Only user-facing display strings change.

### Visual Identity
- **D-08:** Whale icon style: soft illustration — warm, approachable, friendly companion feel. Think Notion meets Duolingo illustrations, not cartoonish or overly geometric.
- **D-09:** Fresh color palette from scratch — design a new WhaleIt-specific color system in `globals.css` `@theme` block. Replace existing warm paper tones.
- **D-10:** Whale brand presence in three places: app icon, splash/loading screen, and header/sidebar as small brand element.
- **D-11:** Icon assets: AI-generated placeholder in the chosen style. Sufficient for dev use; professional design can swap in later. Need formats: macOS `.icns`, Windows `.ico`, Linux `.png`, web manifest icons.

### Codebase Health Scope
- **D-12:** Web adapter modular split — break the 184-case switch in `apps/frontend/src/adapters/web/core.ts` (1,394 lines) into domain modules. Each command group (accounts, activities, ai, etc.) becomes its own file. Switch dispatch stays but delegates to modules. Follow the same grouping as `apps/frontend/src/adapters/shared/` (16 modules).
- **D-13:** types.ts domain-based split — break `apps/frontend/src/lib/types.ts` (1,929 lines) into domain files: `types/account.ts`, `types/portfolio.ts`, `types/sync.ts`, `types/ai.ts`, etc. Keep barrel re-export from `types.ts` for backward compatibility during transition.
- **D-14:** Remove deprecated types during types.ts split — specifically `ActivityLegacy` and any `@deprecated` interfaces. Audit all usages first, migrate references, then remove.

### Onboarding Redesign
- **D-15:** Rebrand-only approach for onboarding — swap branding text, update colors/logo references in existing 5 files. Keep existing step structure (step1, step2, appearance, connect, page). No flow redesign.

### OpenCode's Discretion
- Exact domain groupings for types.ts split (which types go in which file)
- Exact module boundaries for web adapter split
- Splash/loading screen implementation approach
- Header whale brand element size and placement
- Specific color values for new palette
- Transition strategy for barrel re-exports during types.ts split

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Brand Requirements
- `.planning/REQUIREMENTS.md` §BRAND-01 through BRAND-06 — Rebrand acceptance criteria and constraints
- `.planning/ROADMAP.md` §Phase 1 — Phase success criteria (5 items)
- `.planning/STATE.md` §Blockers/Concerns — Known issues requiring Phase 1 action (web adapter switch, types.ts size, 2,612+ references)

### Codebase Concerns (must-read for refactoring decisions)
- `.planning/codebase/CONCERNS.md` — Monolithic web adapter, God types file, deprecated ActivityLegacy, dual-platform drift risk, COMMANDS/server sync issue

### Architecture Context
- `.planning/codebase/STRUCTURE.md` — Full file tree, adapter pattern, command flow
- `.planning/codebase/CONVENTIONS.md` — Naming conventions, export patterns, import aliases
- `.planning/codebase/ARCHITECTURE.md` — Adapter pattern, dual-runtime design, how adapters work

### Key Source Files (refactoring targets)
- `apps/frontend/src/adapters/web/core.ts` — 1,394 lines, 184-case switch, COMMANDS map
- `apps/frontend/src/lib/types.ts` — 1,929 lines, all TypeScript interfaces
- `apps/frontend/src/globals.css` — Tailwind v4 theme tokens, current color palette
- `apps/tauri/tauri.conf.json` — Bundle identifier, product name, window title
- `apps/tauri/src/menu.rs` — Desktop menu items with "Wealthfolio" strings
- `apps/frontend/src/pages/onboarding/` — 5 onboarding files to rebrand
- `apps/frontend/src/features/wealthfolio-connect/` — Feature directory to rename

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `apps/frontend/src/adapters/shared/` (16 modules): Already organizes commands by domain — provides natural grouping for web adapter split
- `apps/frontend/src/lib/types.ts` barrel export pattern: Other modules import from `@/lib/types` — barrel re-export preserves compatibility during split
- `apps/frontend/src/globals.css` `@theme` block: Well-structured CSS custom property system for color tokens — easy to swap palette wholesale
- `packages/ui/`: Shared component library — will need package rename but components themselves unchanged

### Established Patterns
- Vite alias resolution: `@wealthfolio/ui` → `../../packages/ui/src` in `apps/frontend/vite.config.ts` — rename scope means updating alias config
- Tauri config: `productName`, `mainBinaryName`, `identifier` all in `apps/tauri/tauri.conf.json` — single file for all bundle identity
- Adapter index: `apps/frontend/src/adapters/index.ts` re-exports from tauri/web — no changes needed for rebrand
- Barrel exports: Project uses `index.ts` barrel files extensively — types.ts split should follow this pattern

### Integration Points
- `pnpm-workspace.yaml`: Defines workspace packages — `@wealthfolio/*` references need update
- `apps/frontend/package.json`: Dependencies on `@wealthfolio/ui`, `@wealthfolio/addon-sdk` — rename scope
- `packages/*/package.json`: Package name declarations — rename from `@wealthfolio/*` to `@whaleit/*`
- `Cargo.toml` workspace: Internal crate references — NO changes (stays `wealthfolio-*`)
- `apps/tauri/gen/apple/project.yml`: iOS/macOS bundle configuration — identifier update

</code_context>

<specifics>
## Specific Ideas

- Soft illustration style for whale: warm, approachable, Notion-meets-Duolingo feel
- Tagline "Your friendly finance companion" should appear on splash screen and onboarding
- Fresh color palette should reflect friendly companion branding — not corporate or clinical
- Web adapter split should follow the same domain grouping as the existing `adapters/shared/` modules
- Types barrel re-export ensures zero breakage during split — downstream imports keep working

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-codebase-health-rebrand*
*Context gathered: 2026-04-20*
