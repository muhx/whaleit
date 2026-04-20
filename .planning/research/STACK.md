# Technology Stack

**Project:** WhaleIt (Wealthfolio expansion — personal finance features)
**Researched:** 2026-04-20
**Focus:** NEW dependencies only — assumes existing stack (Diesel/SQLite, React/TanStack, Tauri/Axum, rig-core AI)

## Overview

This stack adds PostgreSQL dual-engine support, OCR receipt scanning, Gmail OAuth integration, MCP server endpoints, and financial recommendation infrastructure to the existing Wealthfolio codebase. Every recommendation extends existing patterns rather than introducing new paradigms.

---

## Recommended Stack

### 1. PostgreSQL Dual-Engine Support

The app currently uses Diesel 2.2 with SQLite + r2d2 sync pool. The web server (Axum) needs PostgreSQL with async I/O for concurrent access. The desktop (Tauri) stays on sync SQLite.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `diesel` | **2.3.7** (upgrade from 2.2) | ORM for both backends | Add `"postgres"` feature alongside existing `"sqlite"`. 2.3 brings improved migration infra and bug fixes. |
| `diesel-async` | **0.8.0** | Async query execution for PostgreSQL | Required for non-blocking DB in Axum. Only supports `AsyncPgConnection` (no async SQLite). Desktop keeps sync Diesel. |
| `deadpool` | **0.13.0** | Async connection pool for PostgreSQL | Preferred over bb8 for diesel-async — simpler API, fewer generics, same performance. `diesel-async` has first-class deadpool support via `pooled_connection::deadpool` module. |
| `diesel_migrations` | **2.3.1** (upgrade from 2.2) | Schema migrations | Upgrade alongside diesel. Need separate migration dirs: `migrations-sqlite/` and `migrations-postgres/` since SQL dialects differ (e.g., `AUTOINCREMENT` vs `SERIAL`, `BOOLEAN` handling). |

**Confidence:** HIGH — Verified via crates.io + Context7 diesel-async docs.

**Architecture fit:** Create `crates/storage-postgres/` implementing the same repository traits (`AccountRepositoryTrait`, `ActivityRepositoryTrait`, etc.) that `crates/storage-sqlite/` already implements. The `crates/core/` traits use `async_trait`, so both sync and async implementations are natural. The Axum server wire up Postgres repos; the Tauri app wires up SQLite repos.

**Key constraint:** Diesel's `table!` macros are backend-agnostic, so schema definitions in `crates/core/` can be shared. But the `#[derive(Insertable)]` and `#[derive(Queryable)]` may need backend-specific annotations if SQL types differ. Use Diesel's `#[diesel(sql_type = ...)]` annotations carefully.

**Migration strategy:**
- Existing SQLite migrations stay untouched in `crates/storage-sqlite/migrations/`
- New PostgreSQL migrations go in `crates/storage-postgres/migrations/`
- New finance tables (bank accounts, credit cards, transactions, budgets, subscriptions) need SQL for both dialects
- Consider a `crates/storage-shared/` for common migration logic if table schemas are complex

**Do NOT use:**
- `sqlx` — Would require rewriting all existing Diesel queries. The project has 40+ migration files and extensive Diesel usage. Switching ORMs is a non-starter.
- `sea-orm` — Same reason. The repository trait abstraction already provides ORM independence at the trait level.
- `bb8` — More complex generics, no meaningful benefit over deadpool for this use case. diesel-async's deadpool integration is more straightforward.

---

### 2. OCR Receipt Scanning

Two viable approaches. Recommend a **hybrid strategy**.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `ocr-rs` (rust-paddle-ocr) | **2.2.2** | Local OCR text extraction (desktop/offline) | Pure Rust, PaddleOCR models via MNN inference. No system dependencies (no Tesseract). Runs offline. Good enough for receipt text extraction. |
| `image` | **0.25.10** | Image loading/preprocessing | Already used by ocr-rs. Load receipt images, resize, convert to grayscale before OCR. |
| Multimodal LLM via `rig-core` | **0.35.0** (upgrade from 0.30) | AI-powered receipt parsing → structured data | Send OCR text + raw image to GPT-4o/Claude Vision for structured extraction (merchant, date, items, total, tax, category). The existing `crates/ai/` infrastructure already supports this. |

**Confidence:** HIGH for hybrid approach. MEDIUM for ocr-rs receipt accuracy (depends on receipt quality).

**Why hybrid, not just OCR or just LLM:**
- **Desktop/offline mode:** Pure OCR via `ocr-rs` extracts raw text. Then local regex/heuristic parsing fills transaction fields. No API call needed.
- **Web/online mode (or when LLM available):** Send receipt image directly to multimodal LLM. GPT-4o and Claude Sonnet are excellent at receipt parsing — they handle crumpled paper, faded ink, weird layouts that pure OCR struggles with.
- **Fallback chain:** Try LLM vision → fall back to ocr-rs + regex parsing.

**Why NOT Tesseract (`tesseract` crate / `leptess`):**
- Requires system-level Tesseract + Leptonica installation (C++ dependency)
- Breaks cross-compilation for Tauri (needs Tesseract on every build target)
- `ocr-rs` bundles everything as pure Rust + MNN models — zero system deps

**Do NOT use:**
- `tesseract` / `leptess` — System dependency hell, especially for Tauri desktop builds across macOS/Windows/Linux.
- Cloud-only OCR (Google Vision API, AWS Textract) — Violates local-first privacy constraint. User data stays on their machine.

---

### 3. Gmail OAuth Integration for Subscription Discovery

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `oauth2` | **5.0.0** | OAuth2 Authorization Code + PKCE flow | Standard, well-maintained Rust OAuth2 crate. Supports async flows. PKCE for security (no client secret needed for desktop apps). |
| `google-gmail1` | **7.0.0** | Gmail API v1 client | Auto-generated from Google's Discovery docs. Handles messages.list, messages.get, labels. Full type coverage. |
| `yup-oauth2` | **12.1.2** | Google-specific OAuth2 helpers | Handles Google's device flow, installed app flow, and token refresh. Complementary to `oauth2` crate — provides Google-specific token storage and authenticator patterns. |
| `reqwest` | **0.12** (existing) | HTTP client for OAuth2 + Gmail API | Already in workspace. The `oauth2` crate's `reqwest` feature uses it directly. |

**Confidence:** HIGH — Verified via crates.io. `google-gmail1` is the standard way to interact with Gmail from Rust.

**Architecture fit:**
- Add a `crates/connect/src/gmail/` module (extends existing `crates/connect/` which already handles broker sync)
- OAuth token storage: Use OS keyring via existing `keyring` crate (already in Tauri deps) — never store tokens in DB or filesystem
- Gmail scanning flow: User authenticates → scan for subscription-related emails → parse sender/subject/amount → suggest subscriptions
- The subscription emails get processed by the AI crate's LLM for intelligent parsing (reusing `rig-core`)

**Email parsing strategy:**
- Use Gmail API `messages.list` with query `subject:(receipt OR invoice OR payment OR subscription) newer_than:90d`
- For each email, extract HTML body → use existing `scraper` crate (already in `crates/core/`) to extract text
- Feed parsed text to LLM for structured extraction (merchant, amount, date, recurrence)

**Do NOT use:**
- `imap` crate + raw IMAP — Gmail API is far more reliable, handles auth better, and provides structured message data. IMAP would require handling Gmail quirks manually.
- `google-cloud-rust` for Gmail — It's designed for Google Cloud services. `google-gmail1` is the correct crate for Gmail API specifically.

---

### 4. MCP Server Endpoint

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `rmcp` | **1.5.0** | Official Rust SDK for Model Context Protocol | First-class Axum integration via `StreamableHttpService`. Procedural macros for tool definitions (`#[tool]`, `#[tool_router]`). Supports resources, tools, prompts, and completions. |

**Confidence:** HIGH — Verified via crates.io + Context7. Official MCP Rust SDK with native Axum transport.

**Architecture fit:**
- Add `crates/mcp/` as a new workspace crate
- Mount MCP server at `/mcp` on the existing Axum router in `apps/server/`
- MCP tools expose financial data: `get_transactions`, `get_budget_status`, `get_net_worth`, `get_spending_trends`
- Reuse existing repository traits — MCP handler gets `Arc<dyn AccountRepositoryTrait>` etc., same as Tauri/Axum commands
- Authentication: MCP endpoint uses the existing JWT auth middleware from `apps/server/src/auth.rs`

**Key code pattern (from Context7 docs):**
```rust
use rmcp::{tool, tool_router, tool_handler, ServerHandler, ServiceExt};
use rmcp::transport::{StreamableHttpService, StreamableHttpServerConfig};

#[derive(Clone)]
struct WhaleItMcpServer { /* repos */ }

#[tool_router]
impl WhaleItMcpServer {
    #[tool(description = "Get recent transactions")]
    fn get_transactions(&self, #[tool(param)] limit: usize) -> String { /* ... */ }
}

// Mount on Axum:
let mcp_service = StreamableHttpService::new(move || server, config);
let app = Router::new().nest_service("/mcp", mcp_service);
```

**Do NOT use:**
- Custom MCP implementation — The `rmcp` crate handles protocol versioning, JSON-RPC, transport framing. Don't reinvent.
- `stdio` transport — WhaleIt is a web-accessible app. HTTP/SSE transport is correct for Claude Desktop / external tools connecting over network.

---

### 5. Financial Recommendations Engine

This is primarily an **algorithmic + AI feature**, not a library choice. Uses existing infrastructure.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `rig-core` | **0.35.0** (upgrade from 0.30) | LLM orchestration for AI insights | Already used in `crates/ai/`. Upgrade for latest model support and bug fixes. Powers natural language financial insights. |
| `chrono` | **0.4** (existing) | Date/time math for period comparisons | Calculate week-over-week, month-over-month spending changes. Already in workspace. |
| `rust_decimal` | **1.39** (existing) | Precise financial calculations | Budget percentages, spending ratios, savings rate. Already in workspace. No floating point errors. |

**Confidence:** HIGH — Leverages existing stack entirely.

**Recommendation architecture:**
- **Rule-based layer** (Rust code in `crates/core/`): Budget threshold alerts, unusual spending detection, subscription cost analysis, bill reminders. Pure computation, no AI needed.
- **AI insight layer** (`crates/ai/` tools): LLM generates narrative insights ("You spent 30% more on dining this month compared to last month. Your savings rate dropped from 15% to 8%.") using the existing tool pattern (15 tools already implemented).
- **Delivery**: Dashboard widget, notification system, dedicated insights page. Frontend renders pre-computed recommendations from backend.

**New AI tools to add (extending existing pattern in `crates/ai/src/tools/`):**
- `GetSpendingBreakdownTool` — category-wise spending for a period
- `GetBudgetStatusTool` — budget vs actual per category
- `GetSubscriptionSummaryTool` — recurring costs, upcoming renewals
- `GetNetWorthTrendTool` — net worth over time with decomposition
- `GetSpendingInsightsTool` — AI-generated spending observations

**Do NOT use:**
- Dedicated recommendation libraries — Financial recommendations are domain-specific. Generic ML libraries add complexity without value. The rule-based + LLM hybrid is more flexible and maintainable.

---

### 6. File Import Enhancements

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `csv` | **1.4.0** (existing) | CSV parsing | Already in workspace. Handles bank statement CSVs, credit card exports. |
| `calamine` | **0.34.0** | Excel (.xlsx/.xls) parsing | Many banks export statements as Excel. Pure Rust, no system deps. Supports reading sheets, cell types, dates. |
| `ofx-rs` | **0.2.0** | OFX (Open Financial Exchange) parsing | For banks that export OFX format. **LOW confidence** — very young crate (0.2.0), may need custom OFX parsing fallback. |
| `quick-xml` | **latest** | XML parsing for OFX/OFX fallback | OFX files are XML-based. If `ofx-rs` is insufficient, parse directly. Well-maintained, fast XML parser. |

**Confidence:** HIGH for CSV/Excel. LOW for OFX — `ofx-rs` at 0.2.0 is immature. Plan for custom OFX parsing.

**Do NOT use:**
- `roxmltree` — DOM-based, loads entire file. `quick-xml` is event-based and more memory efficient for large OFX files.
- `xlsxwriter` / `umya-spreadsheet` — Only need reading, not writing. `calamine` is read-only and lighter.

---

### 7. Frontend Additions

Minimal new frontend dependencies. Most UI uses existing shadcn/ui + Tailwind + TanStack.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `@tanstack/react-table` | **8.21** (existing) | Transaction tables, budget grids | Already in `package.json`. Virtual scrolling via `@tanstack/react-virtual` for large transaction lists. |
| `react-day-picker` | **9.13** (existing) | Date range selectors for budgets/reports | Already in `package.json`. |
| `recharts` | **3.7** (existing) | Spending charts, budget visualizations | Already in `package.json`. |
| `@assistant-ui/react` | **0.11** (existing) | AI chat sidebar | Already in `package.json`. Context-aware chat panel. |

**Do NOT add:**
- New charting libraries — Recharts handles everything needed.
- State management beyond Zustand — Already have Zustand 5.0 + TanStack Query. No need for Redux, MobX, etc.
- Form libraries beyond react-hook-form + zod — Already in stack.

---

## Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tokio-stream` | **0.1** (existing) | SSE streaming for AI chat + MCP responses | Streaming AI responses to frontend, MCP event streams |
| `uuid` | **1** (existing) | ID generation for new entities | Bank accounts, transactions, budgets, subscriptions all need UUIDs |
| `serde_with` | **3** (existing) | Complex serialization for financial data | Transaction amount serialization, date formats across DB engines |
| `base64` | **0.22** (existing in tauri) | Encode receipt images for LLM API | Sending receipt images to multimodal LLMs |

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| PostgreSQL ORM | Diesel 2.3 + diesel-async | sqlx | Would require rewriting 40+ migrations and all existing queries. Massive migration risk. |
| PostgreSQL ORM | Diesel 2.3 + diesel-async | sea-orm | Same as sqlx. Repository trait pattern already provides DB independence. |
| Connection pool (Postgres) | deadpool | bb8 | bb8 has more complex generics. deadpool is simpler, first-class diesel-async support. |
| OCR | ocr-rs + multimodal LLM | tesseract crate | System dependency, cross-compilation pain for Tauri targets. |
| OCR | ocr-rs + multimodal LLM | Cloud OCR (Google Vision) | Violates local-first privacy. Requires API key + internet. |
| Gmail access | google-gmail1 + oauth2 | imap crate | Gmail API is more reliable, better auth, structured data. IMAP is fragile with Gmail. |
| Gmail auth | oauth2 + yup-oauth2 | google-cloud-rust auth | google-cloud-rust is for GCP services, not consumer Gmail API. |
| MCP | rmcp | Custom JSON-RPC | rmcp handles protocol versioning, transport, framing. Official SDK. |
| MCP transport | Streamable HTTP (SSE) | stdio | WhaleIt serves over HTTP. stdio is for CLI tools. |
| Excel parsing | calamine | umya-spreadsheet | Only need reading. calamine is lighter and read-optimized. |
| OFX parsing | ofx-rs + quick-xml fallback | Custom XML parsing from scratch | ofx-rs provides structure. quick-xml handles the cases it misses. |

---

## Version Upgrade Notes

### Must Upgrade (Breaking or Required)

| Crate | Current | Target | Notes |
|-------|---------|--------|-------|
| `diesel` | 2.2 | **2.3.7** | Minor version bump. New postgres feature. Check migration compat. |
| `diesel_migrations` | 2.2 | **2.3.1** | Match diesel version. |
| `rig-core` | 0.30 | **0.35.0** | Check changelog for API changes in agent/tool builder patterns. |
| `base64` | 0.21 (server) / 0.22 (tauri) | **0.22** | Normalize across workspace. |

### New Workspace Dependencies

Add to `[workspace.dependencies]` in root `Cargo.toml`:

```toml
# PostgreSQL async
diesel-async = { version = "0.8", features = ["postgres", "deadpool"] }
deadpool = "0.13"

# MCP server
rmcp = { version = "1.5", features = ["transport-streamable-http"] }

# OAuth2 + Gmail
oauth2 = "5.0"
google-gmail1 = "7.0"
yup-oauth2 = "12.1"

# OCR
ocr-rs = "2.2"

# File import
calamine = "0.34"
ofx-rs = "0.2"
quick-xml = "0.37"
```

### New Crate Structure

```
crates/
├── storage-postgres/     # NEW — PostgreSQL implementations of core repository traits
│   ├── Cargo.toml        # diesel (postgres feature), diesel-async, deadpool
│   └── migrations/       # PostgreSQL-specific SQL migrations
├── mcp/                  # NEW — MCP server endpoint
│   └── Cargo.toml        # rmcp, wealthfolio-core
└── connect/src/gmail/    # NEW module — Gmail OAuth + scanning (extends existing crate)
```

---

## Installation

```bash
# No global install needed — all Rust deps are crate-level

# For PostgreSQL development:
# Install PostgreSQL locally or use Docker
docker run -d --name whaleit-postgres \
  -e POSTGRES_DB=whaleit \
  -e POSTGRES_USER=whaleit \
  -e POSTGRES_PASSWORD=dev \
  -p 5432:5432 \
  postgres:16

# Run diesel CLI setup for PostgreSQL
cargo install diesel_cli --no-default-features --features postgres
diesel setup --database-url postgres://whaleit:dev@localhost/whaleit \
  --migration-dir crates/storage-postgres/migrations
```

---

## Sources

| Source | URL | Confidence |
|--------|-----|------------|
| Diesel docs (Context7) | https://context7.com/diesel-rs/diesel | HIGH |
| diesel-async docs (Context7) | https://context7.com/weiznich/diesel_async | HIGH |
| rmcp MCP Rust SDK (Context7) | https://context7.com/modelcontextprotocol/rust-sdk | HIGH |
| oauth2-rs docs (Context7) | https://context7.com/ramosbugs/oauth2-rs | HIGH |
| ocr-rs docs (Context7) | https://context7.com/zibo-chen/rust-paddle-ocr | HIGH |
| crates.io version checks | https://crates.io | HIGH |
| Google Cloud Rust (Context7) | https://context7.com/googleapis/google-cloud-rust | MEDIUM |
| Existing codebase analysis | Local files | HIGH |
