# Project Research Summary

**Project:** WhaleIt (Whaleit expansion — personal finance features) **Domain:**
Personal Finance Management (PFM) — local-first, AI-native **Researched:**
2026-04-20 **Confidence:** HIGH

## Executive Summary

WhaleIt expands an existing local-first investment portfolio tracker (Whaleit)
into a full personal finance management application. The product combines bank
account tracking, budgeting, subscription management, and AI-powered features —
positioned as the only privacy-first, self-hostable PFM with deep AI
integration. Competitors like Monarch ($99/yr), YNAB ($109/yr), and Copilot
($95/yr) are all cloud-only SaaS; WhaleIt is free, runs locally on desktop
(Tauri/SQLite) or self-hosted (Axum/PostgreSQL), and brings AI-native UX that no
competitor matches.

The recommended approach is to extend the existing Rust crate architecture with
six new domain modules (bank accounts, credit cards, transactions, budgets,
subscriptions, recommendations) behind the proven repository trait pattern, add
a PostgreSQL storage engine for web mode, and layer AI features (conversational
entry, OCR receipt scanning, context-aware chat, periodic recommendations) on
top of the existing `crates/ai/` infrastructure. Every new component follows
established patterns — no paradigm shifts, no new frameworks. The architecture
research identified a clear 9-phase build order driven by hard dependencies.

The dominant risks are: (1) Diesel dual-backend schema drift between SQLite and
PostgreSQL — mitigated by a shared migration authoring workflow and CI parity
checks; (2) financial precision loss across backends — mitigated by keeping all
calculations in Rust (`rust_decimal`), never SQL; (3) rebranding complexity with
2,612+ "Whaleit" references — mitigated by keeping internal crate names
unchanged and treating rebrand as a dedicated atomic phase; (4) MCP security
exposure of financial data — mitigated by mandatory auth and read-only defaults.
The research is high-confidence across all areas, backed by official library
documentation (Context7), crates.io version verification, and direct codebase
analysis.

## Key Findings

### Recommended Stack

The stack extends the existing workspace with zero paradigm changes. Every new
dependency fills a specific gap: PostgreSQL async access, OCR, Gmail OAuth, MCP
protocol, and Excel/OFX parsing. Frontend needs no new libraries — existing
Recharts, TanStack Table, react-hook-form, and @assistant-ui/react cover all
requirements.

**Core new technologies:**

- **diesel 2.3.7 + diesel-async 0.8.0 + deadpool 0.13.0**: PostgreSQL
  dual-engine support — diesel-async provides async queries for Axum; deadpool
  is the recommended pool for diesel-async with simpler API than bb8
- **ocr-rs 2.2.2 + multimodal LLM (rig-core 0.35.0)**: Hybrid OCR — local text
  extraction via PaddleOCR models for offline, LLM vision for online/higher
  accuracy; no system dependencies (critical for Tauri cross-compilation)
- **oauth2 5.0 + google-gmail1 7.0 + yup-oauth2 12.1**: Gmail OAuth + API for
  subscription discovery from email receipts
- **rmcp 1.5.0**: Official Rust MCP SDK with first-class Axum integration via
  StreamableHttpService
- **calamine 0.34 + quick-xml**: Excel and OFX bank statement import; ofx-rs
  (0.2.0) is immature — plan custom XML parsing fallback
- **rig-core 0.35.0** (upgrade from 0.30): Latest agent/tool builder patterns
  for expanded AI tool set

**Key upgrade constraints:** Diesel must go from 2.2 → 2.3.7; rig-core from 0.30
→ 0.35.0. Both are minor bumps but need changelog review.

### Expected Features

**Must have (v1 table stakes):**

- Transaction core with manual entry, auto-categorization (AI), search/filter —
  the engine everything builds on
- Bank account management (checking, savings) with CSV/OFX import
- Credit card tracking with balance, limit, statement cycles
- Unified dashboard showing all accounts + net worth across investments, bank,
  credit cards
- Category-based budget tracking with monthly progress
- Spending reports and charts (category breakdowns, monthly trends, income vs.
  expense)
- Subscription/bill detection from transaction patterns with reminders
- AI-powered conversational transaction entry ("I spent $50 on groceries at
  Whole Foods") — WhaleIt's signature feature
- Multi-currency support (extend existing FX infrastructure)
- WhaleIt rebrand (user-facing only, internal crate names unchanged)

**Should have (v1.x differentiators):**

- Context-aware AI chat sidebar — no competitor has this
- OCR receipt scanning — hybrid local OCR + LLM vision pipeline
- Gmail OAuth invoice scanning for auto-discovered subscriptions — no PFM
  competitor does this well
- AI financial recommendations with periodic (daily/weekly/monthly/quarterly)
  insights
- MCP server endpoint for external AI tool integration — first-mover advantage
- Freelancer mode with business/personal toggle and tax category tagging

**Defer (v2+):**

- Invoice management, partner/family sharing, bank API feeds (Plaid), mobile
  native app, real estate tracking

### Architecture Approach

The architecture follows the existing Whaleit pattern: `crates/core/` defines
domain models and async service traits; storage crates implement them for SQLite
(desktop) and PostgreSQL (web); transport layers (Tauri IPC, Axum HTTP, MCP)
delegate to the same trait implementations. Six new domain modules join the
existing investment-focused ones. A new `crates/storage-postgres/` crate
provides PostgreSQL implementations using diesel-async + deadpool (no write
actor needed — PG handles concurrent writes). The AI expansion adds tools to the
existing `crates/ai/` tool set rather than creating a separate recommendation
engine.

**Major components:**

1. **`crates/core/{bank_accounts, credit_cards, transactions, budgets, subscriptions, recommendations}/`**
   — New domain modules with models, service traits, and validation. Mirror
   existing `accounts/` and `activities/` patterns.
2. **`crates/storage-postgres/`** — NEW crate with PostgreSQL implementations of
   all repository traits using diesel-async + deadpool. Shares schema
   definitions with SQLite via `table!` macros (backend-agnostic).
3. **`crates/gmail/`** — NEW crate for Gmail OAuth flow, email scanning, and
   invoice extraction. Follows `crates/connect/` pattern. Tokens stored via
   existing `SecretStore` trait.
4. **`crates/mcp-server/`** — NEW crate wrapping core service traits as MCP
   tools via `rmcp`. Thin transport layer — zero business logic duplication.
5. **`crates/ai/tools/` (expanded)** — New tools (transactions, budgets,
   subscriptions, OCR receipt) extending the existing 15-tool `ToolSet` +
   `AiEnvironment` DI pattern.

### Critical Pitfalls

1. **Diesel dual-backend schema drift** — SQLite and PostgreSQL schemas diverge
   silently because `diesel print-schema` runs independently. Prevent with a
   shared migration authoring format and CI parity check between both
   `schema.rs` files.
2. **Financial amount precision loss** — SQLite stores `Decimal` as text;
   PostgreSQL uses `NUMERIC`. Budget calculations and FX conversions accumulate
   rounding errors. Prevent by doing ALL financial math in Rust
   (`rust_decimal`), never in SQL aggregation.
3. **Rebranding breaks everything** — 2,612+ "Whaleit" references across Cargo
   configs, build files, data paths, and UI strings. Prevent by keeping internal
   crate names as `whaleit-*`, changing only user-facing strings, and treating
   rebrand as a dedicated atomic phase.
4. **MCP security exposure** — Financial data accessible to external AI tools
   without auth. Prevent with mandatory per-session tokens, read-only defaults,
   and rate limiting.
5. **WriteHandle pattern incompatible with PostgreSQL** — The SQLite write actor
   (sync closure over MPSC channel) doesn't translate to async PG connections.
   Repository traits must be async-native from the start, abstracting the write
   pattern behind a trait.
6. **AI financial advice creates legal liability** — LLM-generated "insights"
   can cross into regulated financial advice. Prevent with explicit disclaimers
   on every insight and prompts designed for informational presentation, not
   prescriptions.
7. **Existing web adapter switch statement explosion** — Currently 184 cases,
   projected to hit 300+ with new features. Must refactor to generated mapping
   before adding feature commands.
8. **Existing types god file** — `apps/frontend/src/types.ts` at 1,929 lines
   will absorb all new domain types. Split by domain before adding new feature
   types.

## Implications for Roadmap

Based on combined research — feature dependencies, architecture build order,
pitfall prevention requirements, and MVP scope — the following phase structure
is recommended:

### Phase 1: Codebase Health & Rebrand

**Rationale:** Must happen before new features pile on top of existing technical
debt. The web adapter (184-case switch) and types file (1,929 lines) will become
unmaintainable if new domains are added without cleanup. Rebrand is safest when
no new features are mid-flight. **Delivers:** Clean adapter architecture
(generated mapping), split type files by domain, WhaleIt user-facing rebrand
(internal crate names unchanged). **Addresses:** Anti-features — rebrand, plus
existing codebase concerns from PITFALLS.md. **Avoids:** Mixed branding in UI,
adapter monolith growth, types god file reaching 3,000+ lines.

### Phase 2: Dual Database Engine Foundation

**Rationale:** PostgreSQL support is a hard prerequisite for all web-mode
features. Must be established before any new domain modules that need both
SQLite and PostgreSQL implementations. The repository trait abstraction must be
async-native here to prevent WriteHandle leaking. **Delivers:**
`crates/storage-postgres/` with PostgreSQL pool setup, existing investment
domain repos ported to PG, baseline PG migration (consolidating 28 SQLite
migrations), async repository trait abstraction that works for both sync SQLite
and async PG. **Uses:** diesel 2.3.7, diesel-async 0.8.0, deadpool 0.13.0
**Implements:** Dual DB engine architecture, connection selection at startup.
**Avoids:** Schema drift (CI parity check), WriteHandle leaking (async trait
design), hand-rolled SQL duplication for sync.

### Phase 3: Unified Data Model — Core Finance Domains

**Rationale:** Transaction core is the dependency for everything else (budgets,
subscriptions, reports, AI entry). Bank accounts and credit cards are the other
pillars of the unified view. These three domains must be built together because
the unified dashboard depends on all three. **Delivers:** Domain models, service
traits, and validation for bank accounts, credit cards, and transactions.
Category system with auto-categorization rules. SQLite and PostgreSQL repository
implementations. Tauri IPC + Axum HTTP transport. Frontend adapters, feature
modules, and pages. **Addresses:** P1 features — bank accounts, credit cards,
transaction core, account aggregation, auto-categorization, manual entry,
search/filter, CSV/OFX import. **Uses:** calamine, quick-xml, existing csv,
existing rust_decimal. **Avoids:** Financial precision loss (all math in Rust),
adapter drift (generated mapping from Phase 1), database-specific logic in core
crate.

### Phase 4: Budgets, Subscriptions & Reporting

**Rationale:** Budgets depend on transactions + categories (Phase 3).
Subscriptions depend on transactions for pattern detection. Spending reports
depend on categorized transactions. These features form a natural cluster.
**Delivers:** Budget tracking (category-based monthly), spending reports/charts
(Recharts), subscription/bill detection from patterns, bill reminders, 50/30/20
rule budgeting, net worth tracking across all account types. **Addresses:** P1
features — budgets, spending reports, subscription detection, bill reminders,
net worth. P2 — 50/30/20 rule. **Uses:** Existing recharts, existing TanStack
Table, existing react-day-picker. **Avoids:** Budget precision errors (property
tests for splits), month boundary edge cases, unindexed date-range queries.

### Phase 5: AI Transaction Entry & Smart Categorization

**Rationale:** AI conversational entry is WhaleIt's signature feature and
requires transaction core (Phase 3). Smart auto-categorization uses existing
`crates/ai/` infrastructure. This phase transforms the app from "another finance
tracker" to "AI-native finance." **Delivers:** Conversational transaction entry
via AI chat ("I spent $50 on groceries at Whole Foods" → transaction created).
AI-powered categorization with 80%+ confidence. CreateTransactionTool for
`crates/ai/`. Extended AiEnvironment with transaction/budget/account services.
**Addresses:** P1 — AI-powered transaction entry, auto-categorization. **Uses:**
rig-core 0.35.0 (upgrade from 0.30), existing crates/ai infrastructure.
**Avoids:** Auto-categorization inaccuracy (start with rules + AI fallback,
require confirmation on uncertain ones).

### Phase 6: Unified Dashboard & MVP Polish

**Rationale:** The unified dashboard pulls together all account types from
Phases 3-4. This is the "one place for all finances" promise. Must feel like a
cohesive product, not investment tracker + bolted-on finance features.
**Delivers:** Unified dashboard with all accounts, net worth across
investments + bank + credit cards, spending overview widget, budget status
widget, subscription summary widget. Responsive layout for web mode.
**Addresses:** P1 — unified dashboard, net worth, spending overview. **Avoids:**
Dashboard information overload (lazy-load widgets), performance traps
(incremental budget aggregates, cached per-period summaries).

### Phase 7: Context-Aware AI Chat & OCR Receipt Scanning

**Rationale:** These are the deep AI differentiators. Context-aware chat needs
screen context injection (frontend architecture change). OCR needs the
multimodal pipeline. Both build on AI infrastructure from Phase 5. **Delivers:**
Context-aware AI chat sidebar that knows what screen the user is on. OCR receipt
scanning via hybrid local OCR + LLM vision. ProcessReceiptTool for AI pipeline.
Review step for OCR results (never auto-save). **Addresses:** P2 features —
context-aware AI chat, OCR receipt scanning. **Uses:** ocr-rs 2.2.2, rig-core
multimodal support, existing AI chat infrastructure. **Avoids:** OCR accuracy
issues (multi-step pipeline, confidence scoring, always require user
confirmation), OCR on main thread (background task), AI context window overflow
(RAG pattern for transaction history).

### Phase 8: Gmail Integration & Subscription Auto-Discovery

**Rationale:** Gmail scanning for subscriptions is a unique differentiator but
depends on subscription tracking (Phase 4) and AI email parsing (Phase 5+).
OAuth lifecycle management is complex and needs dedicated attention.
**Delivers:** Gmail OAuth flow with token lifecycle management. Email scanning
for subscription invoices. AI-powered parsing of receipt/subscription emails.
Subscription suggestions from email data. OAuth status UI. **Addresses:** P2 —
Gmail invoice scanning, enhanced subscription discovery. **Uses:** oauth2 5.0,
google-gmail1 7.0, yup-oauth2 12.1, existing SecretStore. **Avoids:** OAuth
token loss (encrypt at rest, proactive health checks, graceful degradation),
minimal scope requests, Google app verification process.

### Phase 9: AI Recommendations Engine & MCP Server

**Rationale:** AI recommendations need enough transaction history (1-2 months of
data) and budget tracking to generate meaningful insights. MCP server needs the
full unified data model to be useful. Both are P2 features that complete the
AI + integration story. **Delivers:** Periodic AI-generated financial insights
(daily/weekly/monthly/quarterly). Recommendation dashboard widget and insights
page. MCP server endpoint with auth, rate limiting, and read-only defaults.
Financial data exposed as MCP tools for external AI tools. **Addresses:** P2 —
AI recommendations, MCP server endpoint. **Uses:** rmcp 1.5.0, existing AI
provider infrastructure. **Avoids:** Legal liability (disclaimers on every
insight, informational-only prompts), MCP security (mandatory auth, scope
limiting, rate limiting), MCP data exposure (aggregated data by default, full
details require explicit approval).

### Phase Ordering Rationale

- **Phase 1 (Health/Rebrand) first** because existing codebase debt (adapter
  monolith, types god file) will block all subsequent feature work. Rebrand is
  safest with no in-flight features.
- **Phase 2 (Dual DB) second** because PostgreSQL support is a hard prerequisite
  for all web-mode features. Repository trait design must be right before domain
  modules are built on top.
- **Phase 3 (Core Domains) before Phase 4 (Budgets/Subs)** because budgets
  depend on categorized transactions, and subscriptions depend on transaction
  pattern detection. The transaction core is the foundation.
- **Phase 5 (AI Entry) after Phase 3** because AI transaction entry needs a
  transaction data model to write into. But before Phase 6 (Dashboard) because
  the AI entry is a signature feature that shapes the UX.
- **Phase 7-9 (Advanced AI/Integrations) after MVP core** because these features
  need working data to be useful. OCR needs transactions, recommendations need
  budget history, Gmail needs subscription tracking.

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 2 (Dual DB Engine):** Complex Diesel dual-backend setup. Need to
  research diesel-async migration patterns, PG connection pooling configuration,
  and write-pattern abstraction. Also need to plan the 28-migration
  consolidation to PG baseline.
- **Phase 7 (OCR):** ocr-rs accuracy on real receipts needs validation. Plan a
  spike to test with real-world receipt images before committing to the
  approach.
- **Phase 8 (Gmail OAuth):** Google OAuth app verification process takes weeks.
  Need to research the exact scope requirements, consent screen configuration,
  and testing mode limitations.
- **Phase 9 (MCP Server):** rmcp is version 1.5 — relatively new. Need to verify
  the Axum integration pattern works as documented and test auth middleware
  composition.

Phases with standard patterns (skip research-phase):

- **Phase 1 (Rebrand/Health):** Mechanical refactoring with well-understood
  patterns.
- **Phase 3 (Core Domains):** Follows existing `accounts/` and `activities/`
  patterns exactly. Well-documented in codebase.
- **Phase 4 (Budgets/Subs):** Standard CRUD + domain events pattern. Budgeting
  rules are straightforward business logic.
- **Phase 5 (AI Entry):** Extends existing `crates/ai/tools/` pattern. The
  `ToolSet` + `AiEnvironment` DI is proven.
- **Phase 6 (Dashboard):** Standard React/Recharts dashboard. No novel technical
  challenges.

## Confidence Assessment

| Area         | Confidence | Notes                                                                                                                                                                                                 |
| ------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Stack        | HIGH       | All crate versions verified on crates.io. Official docs consulted via Context7. Existing codebase patterns well-understood. Only gap: ofx-rs at 0.2.0 is immature.                                    |
| Features     | HIGH       | Competitive analysis across 6 PFM products (Monarch, YNAB, Copilot, Wallet, Lunch Money, Mint). Feature prioritization matrix built from competitor data and dependency analysis.                     |
| Architecture | HIGH       | Based on direct codebase analysis (28 migrations, 32 Diesel tables, existing crate structure). Dual DB pattern verified against diesel-async docs. Build order derived from actual code dependencies. |
| Pitfalls     | HIGH       | 2,612+ brand references counted. WriteHandle pattern analyzed from source. Financial precision risks verified against SQLite text storage. MCP security gap confirmed from official SDK docs.         |

**Overall confidence:** HIGH

### Gaps to Address

- **OFX parsing:** ofx-rs at 0.2.0 is immature. During Phase 3 planning,
  evaluate whether custom OFX parsing with quick-xml is needed from the start,
  or if ofx-rs covers common bank export formats. Test with real OFX files
  early.
- **ocr-rs receipt accuracy:** Pure OCR accuracy on real-world receipts
  (thermal, faded, crumpled) is untested. Plan a Phase 7 spike with 50+ real
  receipts before committing to the local OCR path. May need to default to
  LLM-only for v1.
- **Gmail OAuth app verification:** Google's verification process timeline is
  unknown. Plan to start the verification process during Phase 6 so it's ready
  by Phase 8. Design for "unverified app" mode during development.
- **diesel 2.2 → 2.3 migration:** Changelog review needed for breaking changes
  in migration infra and query builder. Schedule for Phase 2.
- **rig-core 0.30 → 0.35 upgrade:** API changes in agent/tool builder patterns
  need changelog review. Schedule for Phase 5.
- **Web adapter generation strategy:** Phase 1 needs to decide on the adapter
  generation approach (code generation from shared schema, or architectural
  refactor). Research options during planning.

## Sources

### Primary (HIGH confidence)

- Context7: `/diesel-rs/diesel` — Diesel ORM dual-backend patterns, migration
  infra
- Context7: `/weiznich/diesel_async` — AsyncPgConnection, deadpool integration
- Context7: `/modelcontextprotocol/rust-sdk` — rmcp MCP Rust SDK, Axum transport
- Context7: `/ramosbugs/oauth2-rs` — OAuth2 PKCE flow patterns
- Context7: `/0xplaygrounds/rig` — rig-core agent builder, tool trait, streaming
- Context7: `/zibo-chen/rust-paddle-ocr` — ocr-rs local OCR
- crates.io — Version verification for all recommended crates
- Codebase analysis — 28 migrations, 32 Diesel tables, existing crate structure,
  WriteHandle pattern

### Secondary (MEDIUM confidence)

- Monarch Money (monarchmoney.com) — Competitor feature analysis
- YNAB (youneedabudget.com) — Competitor feature analysis
- Copilot Money (copilot.money) — Competitor AI features
- Wallet by BudgetBakers (budgetbakers.com) — Competitor, freelancer features
- Lunch Money (lunchmoney.app) — Competitor, recurring expenses
- Google OAuth documentation — Token lifecycle, revocation policies
- Gmail API v1 documentation — Messages API patterns

### Tertiary (LOW confidence)

- ofx-rs (crates.io) — Version 0.2.0, immature, may need custom parsing
- ocr-rs real-world receipt accuracy — Needs spike testing with real receipts
- Google OAuth app verification timeline — Unknown, needs research during
  planning

---

_Research completed: 2026-04-20_ _Ready for roadmap: yes_
