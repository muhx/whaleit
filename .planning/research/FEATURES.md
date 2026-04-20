# Feature Research

**Domain:** Personal Finance Management (PFM) — expanding investment portfolio tracker to full financial life management
**Researched:** 2026-04-20
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist in any personal finance app. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Transaction list with search/filter** | Core of any finance app — users need to see, search, and filter all transactions across accounts | LOW | Reuses existing activities model; add search, date range, category filters. Monarch, YNAB, Copilot all center on this. |
| **Account aggregation view** | Users expect to see all accounts (bank, credit card, investment) in one dashboard with balances | MEDIUM | Existing investment accounts already do this. Extend to bank accounts and credit cards with unified model. |
| **Auto-categorization of transactions** | Every competitor (Monarch, Copilot, Wallet) auto-categorizes — manual-only feels primitive | MEDIUM | Use existing `crates/ai/` LLM integration. Start with rule-based + AI fallback. Copilot markets "AI learns your patterns." |
| **Manual transaction entry** | Not everything auto-imports — cash spending, split bills need quick entry | LOW | Simple form: amount, category, date, payee, notes. Must be fast (< 3 taps). Wallet and YNAB optimize for speed. |
| **CSV/OFX file import** | Bank statement import is the #1 alternative to bank API feeds. Users expect it. | MEDIUM | Existing CSV import for investment activities. Extend parser for bank statement formats (OFX, CSV with flexible column mapping). |
| **Budget tracking by category** | Category-based budgeting is the default mental model. YNAB, Monarch, Mint all do this. | MEDIUM | Monthly budget per category, spent vs. remaining, progress bars. Must support custom categories and groups. |
| **Spending reports/charts** | Users need to see where money goes — pie charts, trend lines, income vs. expense | MEDIUM | Existing Recharts charts. Add spending by category, monthly trends, income vs. expense comparison. Monarch's reports are a key differentiator. |
| **Net worth tracking** | Single number showing total financial health — assets minus liabilities across all accounts | LOW | Extend existing net worth from investments-only to include bank accounts, credit card balances, loans. Monarch and Copilot both feature this prominently. |
| **Recurring expense/subscription detection** | Users want to see all subscriptions in one place. Monarch auto-detects. Copilot highlights forgotten ones. | MEDIUM | Pattern detection on recurring transactions (same amount, same merchant, regular interval). Calendar + list views. |
| **Bill reminders/notifications** | Upcoming bills and payment due dates. Prevents missed payments. | LOW | Simple notification system. Due date tracking on recurring expenses. Monarch and YNAB both do this. |
| **Multi-currency support** | Already exists for investments — must extend to all new finance features | LOW | Reuse existing `crates/core/src/fx/` infrastructure. Wallet and Lunch Money both highlight multi-currency as key feature. |
| **Search across all transactions** | Users need to find specific transactions quickly by merchant, amount, date, category | LOW | Full-text search on transaction fields. Monarch touts "search any transaction across all accounts." |

### Differentiators (Competitive Advantage)

Features that set WhaleIt apart from competitors. Not required, but create significant competitive moat.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **AI-powered conversational transaction entry** | "I spent $50 on groceries at Whole Foods" → transaction created. Zero friction. No competitor does this natively — they all use forms. | HIGH | Uses existing `crates/ai/` with streaming. Natural language → structured transaction. Massive UX advantage over form-based entry. |
| **Context-aware AI chat sidebar** | AI knows what screen you're on and can help accordingly. "Why is my grocery budget so high this month?" while looking at budget page. | HIGH | Requires screen context injection into AI prompts. No competitor has this — Monarch has no AI chat, YNAB has none. Copilot has basic AI categorization but no chat. |
| **OCR receipt scanning** | Snap receipt → auto-fill transaction. Eliminates manual entry for physical purchases. | HIGH | Use AI vision model (existing provider infrastructure). Extract merchant, amount, date, items, category. Wave does this but poorly. Expense tracking apps do it but are standalone. |
| **Gmail invoice scanning via OAuth** | Automatically discovers subscriptions and bills from email receipts. No manual tracking needed. | HIGH | Gmail API + OAuth. Parse receipt/invoice emails for subscription data. No PFM app does this well — users love auto-discovery. Monarch detects from bank transactions but not email. |
| **MCP server endpoint** | External AI tools (Claude Desktop, etc.) can query user's financial data. First-mover advantage — no PFM app offers this. | MEDIUM | Expose financial data via MCP protocol. Enables power users to build custom AI workflows. Positions WhaleIt as "open" finance platform. |
| **Local-first with dual runtime (desktop + web)** | Data stays on user's machine (desktop) or their server (web). No cloud dependency. Privacy-first. | MEDIUM | Already exists for investments. Extending to all finance features. Every major competitor is cloud-only SaaS. This is a strong differentiator for privacy-conscious users. |
| **Freelancer mode: business expense tracking** | Toggle between personal and business views. Track deductible expenses, categorize by client/project. | MEDIUM | BudgetBakers' "Board" product is separate app. WhaleIt integrates it. Freelancers currently use separate tools (QuickBooks Self-Employed, Wave) or manual spreadsheets. |
| **Invoice management for freelancers** | Create, track, and manage invoices. Track payment status. Essential for freelancer cash flow. | HIGH | Not typical in PFM apps (they're personal, not business). BudgetBakers Board does it. Including it in one app eliminates need for separate invoicing tool. |
| **Tax category tagging** | Tag expenses with tax categories (Schedule C, etc.). Makes tax time painless for freelancers. | MEDIUM | Pre-built tax category templates. Export for tax software. QuickBooks Self-Employed does this. Integrating into personal finance is rare. |
| **50/30/20 rule budgeting** | Percentage-based budgeting alongside category-based. Monarch's "Flex Budgeting" is similar — they found users love simplified models. | LOW | Simple: allocate income percentages to needs/wants/savings. Provide alongside traditional category budgets. Gives users choice. |
| **AI financial recommendations (periodic)** | Daily, weekly, monthly, quarterly, yearly insights. "You spent 20% more on dining this month. Consider cooking 2 more meals/week." | MEDIUM | Scheduled AI analysis of spending patterns, investment performance, budget adherence. Multi-channel delivery: notifications, dashboard widget, insights page. No competitor provides this comprehensively. |
| **Unified investment + spending view** | Single app sees both your portfolio AND daily finances. Most competitors are one or the other. Monarch has basic investment tracking. | LOW | Already exists for investments. The unification IS the value. Show total financial picture: net worth includes investments + bank balances - credit card debt - loans. |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems or misalign with WhaleIt's values.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Real-time bank API feeds (Plaid, etc.)** | Users want auto-import from banks. It's the #1 requested feature in PFM apps. | Creates cloud dependency, costs per-connection ($0.50-2.50/user/month), only works in supported countries, breaks local-first architecture, privacy concerns sending credentials to third party. | Manual + CSV/OFX import + AI-assisted entry. Defer bank API to v2 when architecture supports optional cloud integrations. |
| **Double-entry bookkeeping** | Accountants and some power users want it. Sounds more "correct." | Way too complex for target users (personal + freelancers, not businesses). Confusing UX. Creates barrier to entry. Every failed PFM app tried this. | Single-entry with clear income/expense tracking. Tagging for business vs. personal. Simple, approachable. |
| **Social/sharing features** | Couples want shared budgets. Friends want to split expenses. | Violates privacy-first ethos. Adds massive complexity (real-time sync, permissions, conflict resolution). Expands scope enormously. | v1 is single-user. v2 can add partner sharing following Monarch's model. Split expenses can be tracked manually with tags. |
| **Payment processing / bill pay** | Users want to pay bills from the app. One-stop shop. | Regulatory nightmare (money transmitter licenses). Totally different product. Security liability. | Tracking only. WhaleIt tells you WHAT you owe, not pays it for you. Point users to payment portals. |
| **Crypto-specific features** | Crypto holders want DeFi tracking, wallet connections, NFT valuations. | Extremely niche, rapidly changing ecosystem, valuation is unreliable. Adds massive scope for tiny user segment. | Existing investment tracking handles crypto as holdings (manual entry). No special crypto features. Users can track crypto via the same portfolio mechanism. |
| **Mobile native app** | Users want mobile. Competitors are mobile-first. | Huge engineering investment (separate codebase or React Native). Desktop + web first aligns with local-first philosophy. Mobile browsers work for web mode. | v1: desktop (Tauri) + responsive web. v2: consider mobile when core features are validated. Copilot started iOS-only and added web later — reverse is fine. |
| **Credit score monitoring** | Mint popularized this. Users expect "free credit score." | Requires integration with credit bureaus (Expedia, TransUnion). Complex API, regulatory requirements, US-only. Not core to PFM. | Show credit card utilization metrics (balance/limit ratio). Don't show actual credit score. Link to free credit score services. |
| **Multi-user/family accounts** | Families want shared tracking. Monarch and YNAB do this. | Adds auth complexity, real-time sync, permission models, data isolation. v1 should nail single-user first. | v2 feature after single-user experience is validated. Family sharing is a v2 differentiator, not a v1 requirement. |

## Feature Dependencies

```
[Unified Data Model]
    └──requires──> [Bank Account Management]
    └──requires──> [Credit Card Tracking]
    └──requires──> [Transaction Core (entry, categorization, search)]
                       └──requires──> [Category System]

[Budget Tracking]
    └──requires──> [Transaction Core]
    └──requires──> [Category System]

[Subscription Detection]
    └──requires──> [Transaction Core]
    └──enhances──> [Bill Reminders]

[AI Conversational Entry]
    └──requires──> [Transaction Core]
    └──requires──> [crates/ai Provider System] (existing)

[OCR Receipt Scanning]
    └──requires──> [Transaction Core]
    └──requires──> [AI Vision Model Access]
    └──enhances──> [AI Conversational Entry]

[Gmail Invoice Scanning]
    └──requires──> [Subscription/Bill Tracking]
    └──requires──> [Gmail OAuth Integration]
    └──requires──> [AI Email Parsing]

[AI Financial Recommendations]
    └──requires──> [Transaction Core]
    └──requires──> [Budget Tracking]
    └──requires──> [crates/ai Provider System] (existing)
    └──enhances──> [Context-Aware AI Chat]

[Context-Aware AI Chat]
    └──requires──> [crates/ai Provider System] (existing)
    └──requires──> [Frontend screen context injection]

[MCP Server Endpoint]
    └──requires──> [Unified Data Model]
    └──requires──> [Transaction Core]

[Freelancer Mode]
    └──requires──> [Transaction Core]
    └──requires──> [Category System (with tax categories)]
    └──requires──> [Invoice Management]
    └──enhances──> [Budget Tracking] (business vs. personal budgets)

[Net Worth Tracking]
    └──requires──> [Unified Data Model]
    └──requires──> [Investment Portfolio] (existing)

[Spending Reports]
    └──requires──> [Transaction Core]
    └──requires──> [Category System]
    └──enhances──> [Budget Tracking]

[Credit Card Tracking]
    └──requires──> [Account Management]
    └──enhances──> [Net Worth Tracking] (liabilities)
```

### Dependency Notes

- **Unified Data Model requires Bank Account + Credit Card + Transaction Core:** The unified view depends on having all three account types feeding into a shared transaction and balance model. Without any one of them, the "unified" promise is broken.
- **AI Conversational Entry requires Transaction Core + AI Provider:** Can't build conversational entry until there's a transaction data model to write into. The existing `crates/ai/` with `rig-core` provides the LLM orchestration foundation.
- **Budget Tracking requires Transaction Core + Categories:** Budgets are limits on categorized spending. You need both transactions with categories and a category system before budgets mean anything.
- **Freelancer Mode requires Tax Categories + Invoice Management:** Business expense tracking without tax categories is just personal tracking with a label. Invoices are essential for freelancer cash flow visibility.
- **OCR Receipt Scanning enhances AI Conversational Entry:** Both create transactions, but OCR provides physical receipt data (line items, merchant details) that conversational entry alone can't capture. They complement each other.
- **MCP Server requires Unified Data Model:** External AI tools need the full data model to be useful. Exposing only investment data (existing) is insufficient — the value is in querying across all financial data.
- **Context-Aware AI Chat depends on frontend context injection:** The AI needs to know which screen/page the user is viewing. This requires frontend architecture changes to pass current context to the chat sidebar.

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] **Bank account management** — checking, savings with manual entry + CSV/OFX import. Without bank accounts, this is still just an investment tracker.
- [ ] **Credit card tracking** — balance, limit, statement tracking, bill reminders. Credit cards are the primary spending vehicle for most users.
- [ ] **Transaction core** — quick manual entry, auto-categorization (AI), search/filter, category system. This is the engine everything else builds on.
- [ ] **Unified dashboard** — all accounts (bank + credit + investments) in one view with net worth. The "one place for all finances" promise.
- [ ] **Budget tracking** — category-based monthly budgets with progress tracking. The #1 reason users pick up a PFM app.
- [ ] **Spending reports** — category breakdowns, monthly trends, income vs. expense charts. Makes the data actionable.
- [ ] **Subscription/bill detection** — auto-detect recurring expenses from transaction patterns. Users love this feature in Monarch.
- [ ] **AI-powered transaction entry** — conversational ("I spent $50 on groceries") + smart auto-categorize. This is WhaleIt's signature feature.
- [ ] **WhaleIt rebrand** — app name, logo, icons, colors. Must feel like a fresh product, not a re-skinned investment app.

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] **OCR receipt scanning** — trigger: users want to reduce manual entry friction. Depends on AI vision integration working reliably.
- [ ] **Context-aware AI chat sidebar** — trigger: AI transaction entry is validated, users want more AI interaction. Requires frontend context injection.
- [ ] **Gmail OAuth invoice scanning** — trigger: subscription tracking is working, users want auto-discovery of subscriptions from email.
- [ ] **50/30/20 rule budgeting** — trigger: category budgets validated, users want simpler budgeting model. Low complexity add-on.
- [ ] **AI financial recommendations** — trigger: enough transaction history accumulated for meaningful insights. Needs at least 1-2 months of data.
- [ ] **Freelancer mode: business expense tracking** — trigger: personal features validated, freelancer demand signals. Add tax category tagging first.
- [ ] **MCP server endpoint** — trigger: power users and developer community request it. Requires stable unified data model.

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Invoice management for freelancers** — requires significant new domain logic (invoice templates, PDF generation, payment tracking). Complex and niche.
- [ ] **Partner/family sharing** — requires multi-user auth, real-time sync, permission system. Major architectural undertaking.
- [ ] **Bank API feeds (Plaid)** — requires cloud infrastructure, per-user cost, privacy model reconsideration. Defer until local-first approach is validated.
- [ ] **Mobile native app** — significant engineering investment. Web responsive covers basic mobile use for v1.
- [ ] **Real estate tracking** — Zillow integration for property values. Nice-to-have for net worth, not critical.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Transaction core (entry, categorize, search, categories) | HIGH | MEDIUM | P1 |
| Bank account management | HIGH | MEDIUM | P1 |
| Credit card tracking | HIGH | MEDIUM | P1 |
| Budget tracking (category-based) | HIGH | MEDIUM | P1 |
| Unified dashboard + net worth | HIGH | LOW | P1 |
| Spending reports/charts | HIGH | MEDIUM | P1 |
| Subscription/bill detection | HIGH | MEDIUM | P1 |
| AI-powered transaction entry | HIGH | HIGH | P1 |
| WhaleIt rebrand | HIGH | LOW | P1 |
| Multi-currency (extend existing) | MEDIUM | LOW | P1 |
| Bill reminders | MEDIUM | LOW | P1 |
| Context-aware AI chat sidebar | HIGH | HIGH | P2 |
| OCR receipt scanning | HIGH | HIGH | P2 |
| Gmail invoice scanning | MEDIUM | HIGH | P2 |
| 50/30/20 budgeting | MEDIUM | LOW | P2 |
| AI financial recommendations | HIGH | MEDIUM | P2 |
| Freelancer expense tracking | MEDIUM | MEDIUM | P2 |
| MCP server endpoint | MEDIUM | MEDIUM | P2 |
| Tax category tagging | MEDIUM | MEDIUM | P2 |
| Invoice management | MEDIUM | HIGH | P3 |
| Partner/family sharing | MEDIUM | HIGH | P3 |
| Bank API feeds | MEDIUM | HIGH | P3 |
| Mobile native | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for launch — core finance management + signature AI feature
- P2: Should have, add when possible — deepens AI advantage and reaches new user segments
- P3: Nice to have, future consideration — expand market reach after core is validated

## Competitor Feature Analysis

| Feature | Monarch Money | YNAB | Copilot | Wallet by BudgetBakers | Our Approach |
|---------|---------------|------|---------|----------------------|--------------|
| **Account types** | Bank, credit, investment, loan, real estate | Bank, credit, loan, cash | Bank, credit, investment, crypto, real estate | Bank, credit, cash, loan | Same breadth — bank, credit, investment, loan. No real estate initially. |
| **Budget model** | Flex + category-based | Envelope (give every dollar a job) | Category-based with rollovers | Category-based + planned payments | Category-based + 50/30/20 rule. Simpler than YNAB, more flexible than Monarch. |
| **Bank connection** | Plaid + MX (13K+ institutions) | Plaid direct import | Plaid | 15K+ bank connections (own AISP) | Manual + CSV/OFX import. No bank API for v1 (local-first advantage). |
| **AI features** | Basic auto-categorization | None | AI auto-categorization, "learns patterns" | Auto-categorization, bank sync | AI conversational entry, context-aware chat, OCR, recommendations. Deepest AI integration in the space. |
| **Investment tracking** | Basic (holdings, performance, allocation) | None | Basic (stocks, crypto, ETFs) | None | Full portfolio tracking (existing Wealthfolio strength). Best-in-class investment tracking combined with daily finance. |
| **Subscription detection** | Auto-detect from transactions | None (manual budget categories) | "Spot subscriptions" auto-detect | Recurring payment tracking | Auto-detect from transactions + Gmail invoice scanning (deeper discovery). |
| **Multi-currency** | Limited | No | No | Yes (50 languages, multi-currency) | Full multi-currency (existing FX infrastructure). Strong advantage for international users. |
| **Freelancer features** | None | None | None | Separate "Board" product | Integrated freelancer mode. Personal + business in one app. Unique positioning. |
| **Privacy model** | Cloud SaaS | Cloud SaaS | Cloud SaaS | Cloud SaaS | Local-first (desktop) or self-hosted (web). Only privacy-first PFM with this feature set. |
| **Pricing** | $99.99/yr | $109/yr | $95/yr | Free (premium for bank sync) | Free + optional AI API costs (user's own key). No subscription. |
| **Platform** | Web, iOS, Android | Web, iOS, Android | iOS, Mac, Web | iOS, Android, Web | Desktop (Tauri) + self-hosted web. Desktop is unique in market. |

### Competitive Positioning

**Where WhaleIt wins:**
1. **AI depth** — No competitor has conversational transaction entry, context-aware chat, OCR scanning, AND AI recommendations. This is the widest AI feature set in PFM.
2. **Privacy** — Only local-first, self-hostable PFM with this feature set. Monarch/YNAB/Copilot all require cloud accounts.
3. **Investment + daily finance** — Monarch has basic investment tracking. YNAB has none. WhaleIt inherits Wealthfolio's full portfolio management.
4. **Freelancer integration** — BudgetBakers requires a separate app ("Board"). WhaleIt integrates personal + business in one place.
5. **Free forever** — No subscription. User pays only for AI API usage (optional, their own key).

**Where competitors win (for now):**
1. **Bank connectivity** — Monarch and Wallet auto-sync with 13K+ banks. WhaleIt requires manual/file import. Significant UX gap for users who want zero-effort tracking.
2. **Mobile** — All competitors are mobile-first. WhaleIt is desktop + web. Mobile is deferred to v2.
3. **Partner sharing** — Monarch and YNAB support couples/families. WhaleIt is single-user for v1.
4. **Maturity** — Competitors have years of UX refinement, edge case handling, and reliability. WhaleIt is new in daily finance features.

## Sources

- Monarch Money official site (monarchmoney.com) — feature pages for tracking, budgeting, planning
- YNAB official site (youneedabudget.com/features) — feature overview, goal tracking, debt management
- Copilot Money official site (copilot.money) — feature overview, AI categorization, spending line
- Wallet by BudgetBakers official site (budgetbakers.com) — product overview, Board product for business
- Lunch Money official site (lunchmoney.app/features) — feature list, rules engine, recurring expenses
- Mint (mint.intuit.com) — now migrated to Credit Karma; reference for historical PFM feature baseline
- PROJECT.md — project requirements, constraints, architecture, existing capabilities

---
*Feature research for: WhaleIt personal finance expansion*
*Researched: 2026-04-20*
