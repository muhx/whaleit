# WhaleIt

## What This Is

WhaleIt (rebranded from Wealthfolio) is a local-first personal finance management application that helps individuals and freelancers track their entire financial life — investments, bank accounts, credit cards, subscriptions, budgets, and daily transactions — all in one place. It runs as a desktop app (Tauri/Rust) and a self-hosted web app (Axum), with an AI-powered assistant that makes financial record-keeping effortless. Your friendly finance companion.

## Core Value

Users can effortlessly track and understand their complete financial picture — investments, spending, budgets, and subscriptions — with AI doing the heavy lifting to categorize, suggest, and advise.

## Requirements

### Validated

<!-- Existing capabilities from Wealthfolio codebase -->

- ✓ Investment portfolio tracking with multi-account support — existing (`crates/core/src/accounts/`, `crates/core/src/portfolio/`)
- ✓ Activity/transaction history for investments — existing (`crates/core/src/activities/`)
- ✓ Multi-currency support with FX rate management — existing (`crates/core/src/fx/`)
- ✓ Market data integration (Yahoo Finance + custom providers) — existing (`crates/market-data/`)
- ✓ Holdings tracking and portfolio valuation — existing (`crates/core/src/portfolio/`)
- ✓ Financial goals with allocation tracking — existing (`crates/core/src/goals/`)
- ✓ FIRE planner (desktop) — existing (`apps/frontend/src/features/fire-planner/`)
- ✓ Contribution limits tracking — existing (`crates/core/src/limits/`)
- ✓ Asset classification and taxonomies — existing (`crates/core/src/taxonomies/`)
- ✓ Broker sync via Wealthfolio Connect — existing (`crates/connect/`)
- ✓ E2EE device synchronization — existing (`crates/device-sync/`)
- ✓ AI chat integration with LLM providers — existing (`crates/ai/`)
- ✓ Dual-runtime: desktop (Tauri) + web (Axum) — existing (`apps/tauri/`, `apps/server/`)
- ✓ Local-first SQLite storage — existing (`crates/storage-sqlite/`)
- ✓ Optional web authentication (JWT + Argon2id) — existing (`apps/server/src/auth.rs`)
- ✓ CSV import for activities — existing (`apps/frontend/src/adapters/shared/activities.ts`)
- ✓ Domain event system for side effects — existing (`crates/core/src/events/`)
- ✓ Addon system for extensibility — existing (`packages/addon-sdk/`)
- ✓ Dashboard with charts (Recharts) — existing (`apps/frontend/src/pages/`)
- ✓ Complete WhaleIt rebrand — Phase 01 (app name, logo, icons, colors, all user-facing references)
- ✓ Brand identity: whale icon, ocean teal palette, approachable design — Phase 01
- ✓ Tagline: "Your friendly finance companion" — Phase 01

### Active

<!-- New requirements for WhaleIt expansion -->

- [ ] PostgreSQL database engine running alongside SQLite (dual engine: SQLite for desktop, PostgreSQL for web)
- [ ] Bank account management (checking, savings) with manual entry + CSV/OFX import
- [ ] Credit card tracking with balance/limits, statement tracking, bill reminders, and reward points
- [ ] Daily transaction quick entry with smart auto-categorization and suggestions
- [ ] OCR receipt scanning with AI auto-fill of transaction details
- [ ] Subscription/bill tracking with manual entry + Gmail OAuth integration for invoice scanning
- [ ] Budgeting with both envelope/category-based and percentage-based (50/30/20) rule support
- [ ] Multi-currency across all finance features (reusing existing FX infrastructure)
- [ ] In-app AI chat sidebar panel (context-aware — knows current UI screen)
- [ ] Conversational transaction entry via AI chat ("I spent $50 on groceries at Whole Foods")
- [ ] Smart AI assistance throughout app (auto-categorize, duplicate detection, field auto-fill)
- [ ] MCP server endpoint for external AI tools (Claude Desktop, etc.) to interact with user data
- [ ] AI provider support: user-provided API keys AND hosted service option
- [ ] AI financial recommendations: daily, weekly, monthly, quarterly, yearly insights
- [ ] Full financial advisor: spending insights + investment advice + tax optimization
- [ ] Multi-channel recommendation delivery: notifications + dashboard widget + dedicated insights page
- [ ] Unified data model: bank accounts + investments + budgets all connected
- [ ] Freelancer support: business expense tracking, invoice management, tax categories

### Out of Scope

- Real-time bank API feeds (Plaid, etc.) — manual + file import sufficient for v1, bank API feeds deferred
- Mobile native app — desktop + web first, mobile later
- Double-entry bookkeeping — too complex for target users, quick entry model instead
- Crypto-specific features — existing investment tracking covers crypto holdings
- Social/sharing features — personal finance is private
- Payment processing — tracking only, no money movement
- Multi-user/family accounts — single user per instance for v1

## Context

**Current state:** Wealthfolio is a mature, well-architected portfolio tracker with dual-runtime (Tauri desktop + Axum web), local-first SQLite storage, and an existing AI integration layer (`crates/ai/`). The codebase uses a clean adapter pattern that abstracts transport (IPC vs HTTP), repository traits that abstract data access, and a domain event system for side effects.

**Why this expansion:** Wealthfolio currently handles investment portfolios well but lacks everyday financial management — bank accounts, credit cards, daily spending, budgets, and subscriptions. Users need a unified view of their entire financial life, not just investments. The existing AI infrastructure (`crates/ai/` with `rig-core` LLM orchestration, 15 tool implementations, streaming hooks) provides a strong foundation for the AI-powered features.

**Architecture fit:** The dual-engine database requirement (SQLite for desktop, PostgreSQL for web) aligns with the existing repository trait pattern — `crates/core/` defines traits, `crates/storage-sqlite/` implements them. A new `crates/storage-postgres/` crate would implement the same traits with Diesel's PostgreSQL backend. The adapter pattern on the frontend means new features work identically in both modes.

**Existing AI foundation:** `crates/ai/` already has provider management, chat with streaming, tool calls (15 implementations for accounts, activities, holdings, etc.), and prompt templates. This needs expansion for: conversational transaction entry, OCR receipt processing, financial recommendations engine, and MCP server endpoint.

**Brand:** The rebrand from Wealthfolio to WhaleIt reflects the expanded scope — from investment-only to full personal finance companion. The friendly whale represents steady, wise financial guidance.

## Constraints

- **Dual DB engine:** Must support both SQLite (desktop) and PostgreSQL (web) through shared repository traits — existing Diesel ORM supports both backends
- **Local-first:** Desktop mode must work fully offline with local SQLite; no mandatory cloud dependency
- **Existing architecture:** Must extend, not replace, the current adapter pattern, repository traits, and domain event system
- **Tech stack:** Rust backend (crates), React/TypeScript frontend, Diesel ORM, Tauri v2 + Axum — all established
- **Self-hosted:** Web mode must remain self-hostable via Docker; no mandatory SaaS dependencies
- **Privacy:** All financial data stays local or on user's own server; AI calls go to user-chosen providers
- **AI providers:** Support OpenAI, Anthropic, Google — user brings their own API key or uses hosted option

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Dual DB engine (SQLite + PostgreSQL) | Desktop needs embedded DB, web needs concurrent access | — Pending |
| Repository trait pattern for DB abstraction | Existing pattern extends naturally to PostgreSQL | — Pending |
| Context-aware AI chat sidebar | Users stay in flow while AI assists based on current screen | — Pending |
| MCP server endpoint | Enables external AI tools to interact with user's financial data | — Pending |
| Soft illustration brand style | Friendly, approachable — not corporate, not cartoonish | Placeholder whale icon in place, professional illustration deferred |
| Manual + file import for transactions | No bank API dependency, privacy-preserving, works globally | — Pending |
| Gmail OAuth for subscription invoices | Automates subscription discovery from email receipts | — Pending |
| OCR receipt scanning with AI | Reduces manual entry friction for daily transactions | — Pending |
| Unified data model across finance features | Single view of net worth: investments + bank accounts + credit cards | — Pending |
| D-01: @whaleit/* npm scope | All internal packages renamed from @wealthfolio/* | Phase 01 — scope adopted, 1074 imports updated |
| D-02: Preserve internal crate names | wealthfolio-* crate names unchanged (Cargo registry, existing deps) | Phase 01 — internal names kept, user-facing strings renamed |
| D-03: Preserve infrastructure URLs | wealthfolio.app, connect.wealthfolio.app kept (live services) | Phase 01 — URLs deferred to backend/infra phase |
| D-04: Ocean teal color palette | #3d8778 primary, replacing warm paper tones | Phase 01 — globals.css @theme tokens updated |
| D-05: Modular web adapter | Monolithic 1394-line core.ts split into 18 domain modules | Phase 01 — clean dispatcher pattern established |
| D-06: Domain type files | 1929-line types.ts split into 22 domain files with barrel re-export | Phase 01 — backward-compatible imports preserved |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-21 after Phase 01*
