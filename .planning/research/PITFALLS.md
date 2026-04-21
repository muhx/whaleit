# Pitfalls Research

**Domain:** Personal finance management app (investment tracker expanding to full finance)
**Researched:** 2026-04-20
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Diesel Dual-Backend Schema Drift

**What goes wrong:**
The project uses `diesel = { features = ["sqlite", ...] }` at the workspace level. Diesel only allows **one backend feature per compilation unit** — you cannot compile with both `"sqlite"` and `"postgres"` in the same crate. Creating `crates/storage-postgres/` requires separate Cargo features, separate `schema.rs`, separate `diesel.toml`, and separate migration directories. The two schemas will silently drift apart because `diesel print-schema` runs independently per backend.

**Why it happens:**
Developers add a new table to the SQLite schema via migration, run `diesel print-schema`, and forget to create the equivalent PostgreSQL migration and schema update. Diesel has no cross-backend schema validation tooling. Each backend's `table!` macro invocations are separate files generated independently.

**How to avoid:**
- Create a **single migration authoring format** (e.g., a shared SQL or Rust migration DSL) that generates both SQLite and PostgreSQL migrations.
- Add a CI check that compares table counts and column signatures between both `schema.rs` files.
- Consider a "migration first" workflow: write one migration file, generate both dialects from it.
- The 28 existing SQLite migrations must be replayed into a PostgreSQL baseline migration (one consolidated file for fresh PG installs).

**Warning signs:**
- PostgreSQL schema has fewer tables than SQLite.
- App works on desktop (SQLite) but crashes on web (PostgreSQL) with "table not found" errors.
- CI doesn't run PostgreSQL integration tests.

**Phase to address:**
Phase that introduces `crates/storage-postgres/`. This is foundational — every subsequent feature depends on both backends working.

---

### Pitfall 2: WriteHandle Pattern Doesn't Translate to PostgreSQL

**What goes wrong:**
The existing `WriteHandle` (`crates/storage-sqlite/src/db/write_actor.rs`) serializes all database writes through a single MPSC channel with one dedicated `SqliteConnection`. This exists because SQLite supports only one concurrent writer. The PostgreSQL backend will have its own `AsyncPgConnection` pool via `diesel-async`, and serializing writes through a single connection would **destroy PostgreSQL concurrency performance**. But if the repository trait methods are designed around `WriteHandle::exec()` (a synchronous closure taking `&mut SqliteConnection`), the PostgreSQL implementations can't use async connections properly.

**Why it happens:**
The repository traits in `crates/core/src/` use `async_trait` but many underlying implementations call `WriteHandle::exec()` which takes a synchronous closure. The trait signatures accept `&self` (not `&mut self`) and delegate to the actor internally. PostgreSQL needs genuinely async query execution, not a sync closure sent over a channel.

**How to avoid:**
- Repository traits must be defined in terms of async operations that don't assume a specific connection model.
- Abstract the write pattern behind a trait (e.g., `TransactionExecutor`) that SQLite implements via `WriteHandle` and PostgreSQL implements via `diesel-async`'s `AsyncConnection::transaction()`.
- Don't leak `SqliteConnection` types through the trait boundary — the existing traits already do this correctly for reads, but the write paths need the same treatment.

**Warning signs:**
- PostgreSQL repository implementations wrap every write in `tokio::task::spawn_blocking()`.
- Deadlocks under concurrent load in web mode.
- Repository trait methods return `Result<T>` instead of `async fn` for write operations.

**Phase to address:**
Dual DB engine phase. Must be resolved before any write-heavy features (transactions, budgets) are built.

---

### Pitfall 3: Financial Amount Precision Loss

**What goes wrong:**
The project already uses `rust_decimal::Decimal` for financial amounts, but the SQLite schema stores amounts as `Text` columns (see `activities` table: `quantity -> Nullable<Text>`, `unit_price -> Nullable<Text>`). PostgreSQL stores these differently. Cross-backend queries could truncate or round differently. Budget calculations (envelope balances, percentage splits like 50/30/20) accumulate rounding errors across hundreds of transactions, leading to off-by-a-cent displays that erode user trust.

**Why it happens:**
SQLite has no native `DECIMAL` type, so Diesel stores `Decimal` as text. PostgreSQL has `NUMERIC(p,s)` which maps differently. Calculations in Rust using `rust_decimal` are precise, but if any aggregation happens in SQL (SUM, GROUP BY), the SQL engine's precision rules apply — and they differ between SQLite (text-based) and PostgreSQL (numeric-based).

**How to avoid:**
- All financial calculations MUST happen in Rust (`rust_decimal`), never in SQL.
- SQL queries should only SELECT raw values; aggregation logic stays in the service layer.
- Budget percentage splits must use a rounding strategy with a "remainder" bucket (e.g., 50.00% + 30.00% + 20.00% = 100.00% — but $100.01 splits as $50.01, $30.00, $20.00 with the extra cent in the first bucket).
- Add property-based tests with random amounts that verify `sum(parts) == total` for all split operations.

**Warning signs:**
- Budget totals don't match sum of categories by a few cents.
- Net worth differs between desktop and web modes for the same data.
- Currency conversion accumulates drift over many transactions.

**Phase to address:**
Budgeting phase and unified data model phase. The principle applies everywhere but budgets are where users notice precision errors most.

---

### Pitfall 4: Rebranding a Live Open-Source Project Breaks Everything

**What goes wrong:**
There are **2,612+ references** to "Whaleit" or "whaleit" across the codebase — in Cargo crate names (`whaleit-core`, `whaleit-storage-sqlite`, etc.), package names, file paths (`apps/frontend/src/features/whaleit-connect/`), Xcode project files, trademarks, documentation, GitHub URLs, and user-facing strings. An incomplete rename breaks builds, breaks existing users' data paths, breaks deep links, breaks SEO, and creates confusing mixed-brand artifacts.

**Why it happens:**
Developers do a global find-and-replace without understanding all the places the name is embedded. Cargo crate names are referenced in `Cargo.toml` workspace dependencies. Tauri's bundle identifier is embedded in platform-specific build files. App store listings and update mechanisms use the old name. Users' existing data directories may use the old name in paths.

**How to avoid:**
- Do NOT rename Cargo crate names — they're internal identifiers. Keep `whaleit-*` internally and only change user-facing names. Renaming crates is a massive, risk-laden refactor with zero user benefit.
- Create a comprehensive rename checklist: Tauri bundle config (`tauri.conf.json`), app metadata, user-facing strings, documentation, logos/icons, app store listings.
- Preserve backwards compatibility for data directories — check both old and new paths.
- Do the rename early in the milestone when fewer new features are being built on top.
- The brand rename is a **separate atomic commit/phase** — not interleaved with feature work.

**Warning signs:**
- Mixed "Whaleit" and "WhaleIt" strings visible to users.
- App doesn't find existing data after update because directory name changed.
- Build fails because a Cargo crate rename was incomplete.
- App store reviewers reject because metadata inconsistency.

**Phase to address:**
Dedicated rebrand phase, early in the milestone. Must be complete before any marketing or release.

---

### Pitfall 5: MCP Server Security — Exposing Financial Data to External Tools

**What goes wrong:**
An MCP server endpoint gives external AI tools (Claude Desktop, etc.) direct access to the user's financial data. The MCP TypeScript SDK explicitly states that "authentication and authorization is outside the scope of the SDK." Without robust auth, anyone who can reach the MCP endpoint can query the user's transactions, balances, and investment portfolio. This is catastrophic for a finance app.

**Why it happens:**
MCP servers are designed for developer tooling, not for sensitive personal data. The common pattern is local-only access on `localhost`, but "local-only" isn't secure if the user has browser tabs, other local apps, or if the port is accidentally exposed. Financial data requires a higher security bar than code context.

**How to avoid:**
- MCP endpoint must require authentication — even for local connections. At minimum, a per-session token that the user explicitly approves.
- Scope tools to read-only by default. Write operations (creating transactions, modifying budgets) require explicit user confirmation per-operation.
- Never expose raw account numbers, full transaction details, or secrets through MCP tools. Return aggregated/summarized data.
- Rate-limit MCP tool calls to prevent bulk data extraction.
- Document the threat model clearly — what an MCP-connected AI can and cannot see.

**Warning signs:**
- MCP tools return full transaction data without any scope limiting.
- No authentication required to connect to the MCP server.
- User can't audit what AI tools have accessed.

**Phase to address:**
MCP server phase. Security model must be designed upfront, not bolted on after.

---

### Pitfall 6: Gmail OAuth Token Lifecycle Management

**What goes wrong:**
Gmail OAuth tokens expire (access tokens after ~1 hour) and must be refreshed using refresh tokens. But refresh tokens can be **revoked** by Google if unused for 6 months, or if the user changes their password, or if the OAuth app is in "testing" mode and the user is removed. The app silently stops finding subscription invoices, and the user doesn't understand why. Worse: storing OAuth tokens alongside financial data creates a high-value target — if the database is compromised, the attacker gets both financial data AND email access.

**Why it happens:**
Developers test with short-lived sessions during development and never experience token expiration. The OAuth flow works great in demos. Refresh token revocation is invisible until it happens. The Google OAuth consent screen has a "testing" vs "production" mode that behaves differently — testing mode tokens expire after 7 days.

**How to avoid:**
- Encrypt OAuth tokens at rest with a key derived from the user's app password (web mode) or OS keyring (desktop mode).
- Implement proactive token health checks — verify the refresh token works periodically, not just when the user triggers a sync.
- Surface OAuth status clearly in the UI: "Gmail connected (last sync: 2h ago)" vs "Gmail disconnected — please reconnect".
- Design the subscription tracking to work WITHOUT Gmail (manual entry) so OAuth failure isn't catastrophic.
- Plan for Google's OAuth app verification process — unverified apps show scary warnings to users.

**Warning signs:**
- Users report "subscriptions stopped updating" with no error in the UI.
- OAuth tokens stored in plaintext in the database.
- App assumes Gmail is always connected and breaks when it's not.
- Google OAuth consent screen shows "This app isn't verified" warning.

**Phase to address:**
Subscription tracking / Gmail integration phase.

---

### Pitfall 7: OCR Receipt Scanning Accuracy Is Terrible for Real Receipts

**What goes wrong:**
OCR (Tesseract.js or similar) achieves decent accuracy on clean, high-contrast printed text but performs poorly on real-world receipts: thermal paper fades, crumpled paper has distorted text, store logos use unusual fonts, receipt layouts vary wildly between merchants. The AI auto-fill feature "fills in" wrong amounts ($14.99 becomes $149.90 or $1,499.00), wrong dates, or wrong merchants. Users lose trust immediately when the app auto-fills wrong financial data.

**Why it happens:**
OCR demos use clean test images that don't represent real usage. Developers test with well-lit, flat, high-resolution receipt photos. Real users photograph receipts in their pocket, in dim restaurants, or after the receipt has faded. Tesseract.js explicitly states it "is built around assumptions that only hold for printed text" — and receipt printers use dot-matrix or thermal printing that barely qualifies.

**How to avoid:**
- Treat OCR as an **assistive suggestion**, never as authoritative data entry. Always show the extracted data for user confirmation before saving.
- Use a multi-step pipeline: OCR raw text → LLM extraction → confidence scoring → user confirmation.
- Fall back gracefully: if confidence is low, show the scanned image alongside blank fields rather than wrong auto-filled values.
- Consider using a cloud OCR service (Google Vision, AWS Textract) for better accuracy — but make it optional given the local-first constraint.
- Test with a dataset of 100+ real receipts from different merchants, lighting conditions, and paper states before shipping.

**Warning signs:**
- "Auto-filled" transactions have wrong amounts more than 10% of the time.
- Users stop using OCR and revert to manual entry (check analytics).
- OCR confidence scores are consistently low for real-world images.

**Phase to address:**
OCR receipt scanning phase. Requires dedicated testing with real receipts before declaring "done."

---

### Pitfall 8: AI Financial Recommendations Create Legal Liability

**What goes wrong:**
An AI that provides "daily, weekly, monthly, quarterly, yearly insights" and acts as a "full financial advisor: spending insights + investment advice + tax optimization" is providing **financial advice**. In many jurisdictions, providing personalized financial advice requires registration, licensing, and compliance with financial regulations (SEC/FINRA in the US, FCA in the UK, etc.). The app could face legal action if a user follows AI advice that causes financial loss.

**Why it happens:**
The line between "informational insights" and "financial advice" is legally blurry. "Your spending on dining increased 30%" is informational. "You should reduce dining expenses and invest in index funds" is advice. LLMs naturally generate advice-like responses even when prompted to be informational.

**How to avoid:**
- Include explicit disclaimers on every AI-generated insight: "This is informational only, not financial advice. Consult a licensed financial advisor."
- Design the AI prompts to present data and let users draw conclusions, rather than prescribing actions.
- Avoid specific investment recommendations ("buy X", "sell Y"). Instead present comparisons ("your portfolio is 80% stocks / 20% bonds").
- Tax optimization suggestions must explicitly state "consult a tax professional."
- The hosted AI service option creates additional liability vs. user-provided API keys (where the user controls the model).

**Warning signs:**
- AI responses contain specific buy/sell recommendations.
- No disclaimer visible on AI-generated insights.
- Tax advice that could be construed as tax preparation.

**Phase to address:**
AI recommendations engine phase. Legal review needed before shipping any financial advice feature.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip PostgreSQL migrations, rely on ORM schema sync | Faster dual-engine setup | Silent schema drift, data loss in production | Never |
| Store OCR results directly as transaction fields | Simpler code, fewer clicks | Wrong data silently persisted, user distrust | Never — always require confirmation step |
| Share one `schema.rs` between SQLite and PostgreSQL backends | Less duplication | Diesel can't do this — different type mappings per backend | Never — technically impossible with Diesel |
| Use `diesel-async` only for PostgreSQL, keep sync SQLite | Less refactoring of existing code | Repository trait signatures must support both sync and async, creating messy abstraction | Acceptable as intermediate step, but traits should be async from the start |
| Hardcode budget categories (50/30/20) | Simpler initial implementation | Users can't customize, international users have different norms | MVP only, make configurable in next iteration |
| Skip MCP server auth for local-only mode | Faster MCP development | Port scanning or malicious local apps can access financial data | Never |
| Store Gmail OAuth tokens in SQLite alongside financial data | Simpler implementation | Single point of compromise for both financial data AND email access | Never — encrypt tokens separately |
| Implement rebrand as global find-replace | Quick completion | Breaks Cargo crate references, data directory paths, platform build configs | Never |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Gmail OAuth | Requesting overly broad scopes (`mail.read`) when only subscription invoices are needed | Request minimal scopes (`gmail.readonly` restricted to specific labels/senders if possible) |
| Gmail OAuth | Not handling the Google OAuth app verification process | Start verification early (weeks), design for "unverified" mode during development |
| AI Providers (OpenAI/Anthropic/Google) | Assuming all providers support the same tool-calling format | Use `rig-core` abstraction layer (already in place) with provider-specific adaptations |
| MCP Protocol | Implementing MCP server in TypeScript when the backend is Rust | Use a Rust MCP server implementation or expose the existing Axum HTTP endpoints through a thin MCP adapter |
| OCR | Running OCR on the frontend (Tesseract.js) for desktop app | Desktop mode should use a Rust-native OCR library for better performance; web mode can use Tesseract.js or cloud API |
| Market Data (Yahoo Finance) | Already working — but adding more providers increases maintenance | Each new provider needs error handling, rate limiting, data normalization — budget time for each |
| CSV/OFX Import | Assuming a standard format across banks | Every bank exports differently; need a flexible parser with user-configurable column mapping (already exists for activities) |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Loading all transactions for budget calculation | Budget page takes 5+ seconds to load | Compute budget aggregates incrementally, cache per-period summaries | 1,000+ transactions per month |
| OCR processing on main thread | UI freezes while receipt is being scanned | Run OCR in a web worker (frontend) or background task (Tauri/Rust) | First real receipt scan |
| AI chat with full transaction history in context | Context window overflow, API costs spike | Use RAG pattern — retrieve relevant transactions, don't dump everything | 500+ transactions |
| SQLite without WAL mode (already configured, but PostgreSQL doesn't need it) | Read locks during writes, UI stutters | Already using WAL — ensure PostgreSQL connection pool is sized correctly | Concurrent web users |
| Unindexed date-range queries for budgets | Slow budget calculations at month boundaries | Add composite indexes on `(account_id, date)` for all transaction tables | 10K+ transactions |
| Single PostgreSQL connection for all web users | Concurrent request queuing, timeouts | Use `diesel-async` with connection pool (bb8 or deadpool) sized for expected concurrency | 10+ concurrent web users |
| Eager-loading all subscription data on dashboard load | Dashboard slow for users with many subscriptions | Paginate or lazy-load subscription widgets | 50+ tracked subscriptions |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| MCP tools that return full transaction details | External AI tools see every purchase — merchant, amount, date | Return aggregated data (monthly totals by category) unless explicitly requested with user confirmation |
| OCR images stored permanently in database | Receipt photos contain sensitive data (card numbers, addresses) | Process and discard images immediately; store only extracted text data |
| Gmail refresh tokens in database without encryption | Database compromise gives email access | Encrypt tokens with key from OS keyring (desktop) or user-derived key (web) |
| AI chat history stored with full financial context | Chat logs contain the user's complete financial picture | Allow users to delete chat history; don't include account numbers in stored prompts |
| Web mode API keys stored server-side without encryption | Server compromise exposes all users' AI provider keys | Already flagged in CONCERNS.md — encrypt at rest, document security model |
| No rate limiting on AI tool calls via MCP | Automated scripts extract all financial data through MCP | Rate limit per session, require user approval for bulk data access |
| Budget data exported without access control | Net worth and budget reports are highest-value financial data | Ensure export/download features respect the same auth as the UI |

## UX Pitfalls

Common user experience mistakes in personal finance apps.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Requiring manual category for every transaction | Users abandon daily tracking after a week | Auto-categorize with 80%+ confidence, ask for confirmation only on uncertain ones |
| Showing AI confidence scores to users | Users don't understand what "87% confidence" means | Show categories as "suggested" (blue) vs "confirmed" (green), not numeric scores |
| Budget setup requires defining every category upfront | Users feel overwhelmed, never complete setup | Start with 50/30/20 rule as default, let users refine categories over time |
| No "undo" for AI-categorized transactions | One wrong category creates cascading budget errors | Always allow undo/revert for AI actions; show change history |
| OCR auto-fill saves without review | Wrong data enters the financial record | Always show review step with original image alongside extracted fields |
| Gmail sync runs silently without feedback | Users don't know if subscriptions were found or missed | Show "3 new subscriptions detected" notification, let users review and confirm |
| AI recommendations that users can't dismiss | Annoying notifications about the same insight repeatedly | Mark insights as "dismissed" or "acted on" and don't repeat them |
| Mixed branding (Whaleit + WhaleIt) visible to users | Confusion about which app they're using, distrust | 100% rebrand before any public release; zero tolerance for mixed names in UI |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Dual DB engine:** Often missing PostgreSQL migration parity — verify every SQLite table exists in PostgreSQL schema with matching columns
- [ ] **Budget calculations:** Often missing edge cases — verify month boundaries, leap years, currency conversion in multi-currency budgets, partial months (user starts budgeting mid-month)
- [ ] **OCR receipt scanning:** Often missing real-world testing — verify with 50+ real receipts (thermal, faded, crumpled, dim lighting) before declaring "done"
- [ ] **Gmail integration:** Often missing token refresh failure handling — verify app recovers gracefully from revoked/expired tokens without losing existing subscription data
- [ ] **MCP server:** Often missing auth — verify MCP endpoint rejects unauthenticated requests, even on localhost
- [ ] **AI recommendations:** Often missing legal disclaimers — verify every AI-generated insight includes appropriate disclaimer
- [ ] **Rebrand:** Often missing internal references — verify Cargo crate names, environment variables, config file paths, data directory names don't break existing installations
- [ ] **Freelancer features:** Often missing tax category compliance — verify categories match common tax jurisdictions (US Schedule C, etc.)
- [ ] **Context-aware AI chat:** Often missing actual context — verify the AI knows what screen the user is on, not just that "chat is open"
- [ ] **Subscription tracking:** Often missing billing cycle edge cases — verify annual subscriptions, trial periods, variable-amount subscriptions, canceled-but-active subscriptions

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| SQLite/PostgreSQL schema drift | HIGH | Write reconciliation migration; compare schemas programmatically; data repair for mismatched columns |
| WriteHandle abstraction leaking | MEDIUM | Refactor repository traits to be connection-agnostic; add async abstraction layer; update all implementations |
| Financial precision errors in budgets | MEDIUM | Recompute all budget summaries from raw transactions; add rounding strategy tests; fix display layer |
| Incomplete rebrand | LOW (tedious) | Systematic audit of all 2,612+ references; automated search for missed occurrences |
| MCP security breach | HIGH | Revoke all MCP sessions; add auth; audit what data was accessed; notify users |
| Gmail OAuth token loss | LOW | Re-prompt user for OAuth consent; subscription data preserved from last successful sync |
| OCR wrong auto-fill | MEDIUM | Add review step retroactively; audit auto-filled transactions for outliers; allow bulk correction |
| AI providing regulated financial advice | HIGH | Add disclaimers retroactively; modify AI prompts to be informational; legal review of existing outputs |
| Currency conversion drift | MEDIUM | Recompute all FX-adjusted values from raw amounts + historical rates; add reconciliation check |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Diesel dual-backend schema drift | Dual DB Engine | CI test: schema parity check between SQLite and PostgreSQL |
| WriteHandle pattern doesn't translate | Dual DB Engine | Write operations pass in both desktop and web mode |
| Financial amount precision | Budgeting Phase | Property tests: `sum(budget_categories) == total_income` |
| Rebranding breaks everything | Rebrand Phase | Global search: zero occurrences of old name in user-facing code |
| MCP security | MCP Server Phase | Pen test: unauthenticated request rejected; scope limit enforced |
| Gmail OAuth lifecycle | Subscription Tracking | Test: revoked token → graceful degradation, no data loss |
| OCR accuracy | OCR Phase | Test suite: 50+ real receipts with accuracy metrics |
| AI financial liability | AI Recommendations | Legal review of AI prompts and sample outputs |
| Mixed branding in UI | Rebrand Phase | Visual QA: every screen shows only "WhaleIt" branding |
| Budget edge cases | Budgeting Phase | Test: month boundaries, partial months, multi-currency |
| Transaction auto-categorization accuracy | Daily Transactions | Test: 80%+ accuracy on labeled dataset |
| Adapter drift (desktop vs web) | Every feature phase | Integration test: both adapters return identical results for new commands |
| Web adapter monolith growth | Every feature phase | New commands don't increase the switch statement size (use generated mapping) |

## Context-Specific Pitfalls from Existing Codebase

These pitfalls are specific to THIS codebase based on the CONCERNS.md audit.

### Existing Concern: Web Adapter Switch Statement Will Explode

The web adapter (`apps/frontend/src/adapters/web/core.ts`) already has a 184-case switch statement. Every new feature (bank accounts, credit cards, transactions, budgets, subscriptions, OCR, AI recommendations) adds 10-30 new cases. At current growth, it will hit 300+ cases — making the adapter unmaintainable.

**Phase to address:** Must be resolved before adding new feature commands. Generate the adapter from a shared schema.

### Existing Concern: Types God File Gets Worse

`apps/frontend/src/types.ts` (1,929 lines) will absorb all new domain types: `BankAccount`, `CreditCard`, `Transaction`, `Budget`, `BudgetCategory`, `Subscription`, `Receipt`, `Invoice`, etc. This makes the file approach 3,000+ lines.

**Phase to address:** Split types by domain BEFORE adding new feature types. New types go into feature-specific files.

### Existing Concern: Diesel Migrations Diverge

28 existing SQLite migrations must be consolidated into a single PostgreSQL baseline. Any new migrations during feature development must be applied to BOTH backends simultaneously.

**Phase to address:** Dual DB engine setup phase must create the PG baseline and the dual-migration workflow.

### Existing Concern: Hand-Rolled SQL in Sync Must Not Be Duplicated

The sync repository's hand-rolled SQL (`format!()` with `escape_sqlite_str()`) is SQLite-specific. The PostgreSQL sync implementation must NOT replicate this pattern — it should use Diesel's query builder or parameterized queries.

**Phase to address:** Dual DB engine phase — design sync for both backends from the start.

## Sources

- Diesel ORM documentation (Context7: /diesel-rs/diesel) — dual backend feature constraints
- diesel-async documentation (Context7: /weiznich/diesel_async) — async PostgreSQL connection patterns
- MCP TypeScript SDK (Context7: /modelcontextprotocol/typescript-sdk) — authentication is out of scope
- Tesseract.js FAQ (Context7: /naptha/tesseract.js) — printed text only, accuracy caveats
- Google OAuth documentation — token lifecycle, revocation policies, app verification
- Codebase analysis: 2,612+ brand references, 28 migrations, 32 Diesel tables, WriteHandle actor pattern
- CONCERNS.md audit findings — existing architectural risks that compound with expansion

---
*Pitfalls research for: WhaleIt personal finance app expansion*
*Researched: 2026-04-20*
