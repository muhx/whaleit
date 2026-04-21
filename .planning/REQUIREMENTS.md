# Requirements: WhaleIt

**Defined:** 2026-04-20
**Core Value:** Users can effortlessly track and understand their complete financial picture — investments, spending, budgets, and subscriptions — with AI doing the heavy lifting to categorize, suggest, and advise.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Database Engine

- [ ] **DB-01**: Dual database engine runs SQLite for desktop mode and PostgreSQL for web mode through shared repository traits
- [x] **DB-02**: PostgreSQL crate (`crates/storage-postgres/`) implements all existing repository traits using diesel-async + deadpool
- [ ] **DB-03**: Separate migration directories per database dialect (SQLite migrations, PostgreSQL migrations)
- [ ] **DB-04**: Runtime database selection based on build target — desktop auto-selects SQLite, web auto-selects PostgreSQL
- [x] **DB-05**: Unified data model supports bank accounts, credit cards, transactions, and investments through a single schema abstraction

### Accounts

- [ ] **ACCT-01**: User can create bank accounts (checking, savings) with name, institution, currency, and opening balance
- [ ] **ACCT-02**: User can create credit card accounts with name, institution, currency, credit limit, and statement cycle day
- [ ] **ACCT-03**: User can view all accounts (bank, credit card, investment) in a unified account list with current balances
- [ ] **ACCT-04**: User can edit and archive accounts while preserving historical transaction data
- [ ] **ACCT-05**: Credit card tracking shows outstanding balance, available credit, utilization percentage, and next payment due date
- [ ] **ACCT-06**: User can record credit card statement details including statement balance, minimum payment, and due date
- [ ] **ACCT-07**: User can track reward points/cashback balance per credit card account

### Transactions

- [ ] **TXN-01**: User can quickly add a manual transaction with amount, payee, category, date, account, and notes
- [ ] **TXN-02**: Transactions auto-categorize based on rules (merchant name patterns, amount patterns) with AI fallback for unknowns
- [ ] **TXN-03**: User can search and filter transactions across all accounts by merchant, amount range, date range, category, and account
- [ ] **TXN-04**: User can import transactions from CSV files with flexible column mapping
- [ ] **TXN-05**: User can import transactions from OFX files (bank statement format)
- [ ] **TXN-06**: System detects and flags potential duplicate transactions for user review
- [ ] **TXN-07**: All transaction types support multi-currency with automatic FX conversion using existing exchange rate infrastructure
- [ ] **TXN-08**: User can split a single transaction across multiple categories
- [ ] **TXN-09**: Transaction list shows running balance per account

### Budgeting

- [ ] **BUDG-01**: User can create monthly budgets by spending category with target amounts
- [ ] **BUDG-02**: Budget dashboard shows spent vs. remaining per category with progress indicators
- [ ] **BUDG-03**: User can set up 50/30/20 rule budget allocating income percentages to needs, wants, and savings
- [ ] **BUDG-04**: Budget rolls over month-to-month with option to reset or carry over unspent amounts
- [ ] **BUDG-05**: User can assign transactions to budget categories individually or via auto-categorization rules

### Reporting

- [ ] **RPT-01**: User can view spending breakdown by category (pie chart and table)
- [ ] **RPT-02**: User can view monthly spending trends over time (line chart)
- [ ] **RPT-03**: User can view income vs. expense comparison by month
- [ ] **RPT-04**: Net worth dashboard shows total assets (investments + bank) minus total liabilities (credit cards + loans) with historical trend
- [ ] **RPT-05**: Reports support multi-currency with conversion to user's base currency

### Subscriptions

- [ ] **SUBS-01**: System auto-detects recurring transactions from transaction history (same merchant, similar amount, regular interval)
- [ ] **SUBS-02**: User can manually add subscriptions with name, amount, billing cycle, next billing date, and category
- [ ] **SUBS-03**: User sees subscription calendar showing upcoming bills across all accounts
- [ ] **SUBS-04**: Bill reminder notifications fire before upcoming payment due dates (configurable advance days)
- [ ] **SUBS-05**: Subscription dashboard shows total monthly and annual subscription spend

### AI Chat & Assistant

- [ ] **AI-01**: AI chat sidebar panel is accessible from any screen in the application
- [ ] **AI-02**: Chat sidebar is context-aware — knows which screen/page the user is viewing and offers relevant assistance
- [ ] **AI-03**: User can create transactions via conversational AI entry ("I spent $50 on groceries at Whole Foods" → transaction created)
- [ ] **AI-04**: User can upload receipt image and AI extracts merchant, amount, date, items, and suggested category
- [ ] **AI-05**: AI auto-categorizes transactions using merchant patterns, amount patterns, and historical behavior
- [ ] **AI-06**: AI suggests category and payee based on partial input during manual transaction entry
- [ ] **AI-07**: Dual AI provider support — user can bring their own API key (OpenAI, Anthropic, Google) or use hosted service

### AI Recommendations

- [ ] **AIREC-01**: System generates daily spending insights (unusual charges, budget pace, category spikes)
- [ ] **AIREC-02**: System generates weekly spending summaries with comparison to previous week
- [ ] **AIREC-03**: System generates monthly financial health reports with budget adherence, net worth change, top spending categories
- [ ] **AIREC-04**: System generates quarterly investment performance reviews with portfolio allocation analysis
- [ ] **AIREC-05**: System generates yearly financial overview with trends, achievements, and forward-looking suggestions
- [ ] **AIREC-06**: AI recommendations cover spending insights, investment advice, and tax optimization suggestions
- [ ] **AIREC-07**: Recommendations display on a dedicated AI Insights page with full history and drill-down
- [ ] **AIREC-08**: Recommendation widget shows latest insights on the main dashboard
- [ ] **AIREC-09**: Push-style notifications deliver time-sensitive recommendations at configurable intervals
- [ ] **AIREC-10**: All AI recommendations include disclaimer that they are informational, not professional financial advice

### Gmail Integration

- [ ] **GMAIL-01**: User can connect their Gmail account via OAuth to scan for subscription invoices and receipts
- [ ] **GMAIL-02**: System scans incoming emails for subscription/billing patterns and suggests new subscriptions to track
- [ ] **GMAIL-03**: System extracts invoice details (merchant, amount, date, billing cycle) from email receipts
- [ ] **GMAIL-04**: OAuth tokens stored securely via existing secret store (OS keyring on desktop, encrypted file on web)

### MCP Server

- [ ] **MCP-01**: MCP server endpoint exposes user's financial data (accounts, transactions, budgets, investments) to external AI tools
- [ ] **MCP-02**: MCP server authenticates external tools via API key or web mode JWT token
- [ ] **MCP-03**: MCP tools implement read-only access to financial data with explicit user consent
- [ ] **MCP-04**: MCP server runs as part of the Axum web server at `/mcp` endpoint

### Freelancer Mode

- [ ] **FREEL-01**: User can tag transactions as personal or business with a single toggle
- [ ] **FREEL-02**: User can assign business expenses to clients or projects
- [ ] **FREEL-03**: System provides pre-built tax category templates (Schedule C categories, common deductible expenses)
- [ ] **FREEL-04**: User can tag expenses with tax categories for tax-time export
- [ ] **FREEL-05**: User can create and send invoices with line items, due dates, and payment status tracking
- [ ] **FREEL-06**: Freelancer dashboard shows business income vs. expenses, outstanding invoices, and tax-deductible totals

### Brand

- [ ] **BRAND-01**: All user-facing references to "Whaleit" renamed to "WhaleIt" (app title, window title, package display name, docs, UI copy)
- [ ] **BRAND-02**: New app icon featuring friendly whale in soft illustration style
- [ ] **BRAND-03**: Updated color palette and visual identity reflecting approachable, companion-style branding
- [ ] **BRAND-04**: Updated onboarding/welcome screens with WhaleIt branding and tagline direction
- [ ] **BRAND-05**: GitHub repository metadata, README, and documentation updated to WhaleIt branding
- [ ] **BRAND-06**: Internal crate names remain as `whaleit-*` (no rename — internal only, high risk for zero user benefit)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Future Integrations

- **BANK-01**: Real-time bank API feeds via Plaid or similar aggregation service
- **BANK-02**: Auto-import transactions from connected bank accounts

### Social & Sharing

- **SOCIAL-01**: Partner/spouse shared budget with real-time sync
- **SOCIAL-02**: Family account with permission levels

### Mobile

- **MOB-01**: Progressive web app (PWA) for mobile browser access
- **MOB-02**: Native mobile app (iOS/Android) via React Native or similar

### Advanced

- **ADV-01**: Real estate tracking with property value estimation
- **ADV-02**: Loan tracking with amortization schedules
- **ADV-03**: Credit score monitoring integration

## Out of Scope

| Feature | Reason |
|---------|--------|
| Payment processing / bill pay | Regulatory nightmare (money transmitter licenses), different product entirely |
| Double-entry bookkeeping | Too complex for personal + freelancer users, creates barrier to entry |
| Social/sharing features | Violates privacy-first ethos, massive sync complexity for v1 |
| Crypto-specific features (DeFi, wallets, NFTs) | Niche, rapidly changing ecosystem, unreliable valuations |
| Multi-user/family accounts | Auth complexity, data isolation, real-time sync — single-user first |
| Credit score monitoring | Requires credit bureau integration, regulatory requirements, US-only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BRAND-01 | Phase 1: Codebase Health & Rebrand | Pending |
| BRAND-02 | Phase 1: Codebase Health & Rebrand | Pending |
| BRAND-03 | Phase 1: Codebase Health & Rebrand | Pending |
| BRAND-04 | Phase 1: Codebase Health & Rebrand | Pending |
| BRAND-05 | Phase 1: Codebase Health & Rebrand | Pending |
| BRAND-06 | Phase 1: Codebase Health & Rebrand | Pending |
| DB-01 | Phase 2: Dual Database Engine | Pending |
| DB-02 | Phase 2: Dual Database Engine | Complete |
| DB-03 | Phase 2: Dual Database Engine | Pending |
| DB-04 | Phase 2: Dual Database Engine | Pending |
| DB-05 | Phase 2: Dual Database Engine | Complete |
| ACCT-01 | Phase 3: Bank Accounts & Credit Cards | Pending |
| ACCT-02 | Phase 3: Bank Accounts & Credit Cards | Pending |
| ACCT-03 | Phase 3: Bank Accounts & Credit Cards | Pending |
| ACCT-04 | Phase 3: Bank Accounts & Credit Cards | Pending |
| ACCT-05 | Phase 3: Bank Accounts & Credit Cards | Pending |
| ACCT-06 | Phase 3: Bank Accounts & Credit Cards | Pending |
| ACCT-07 | Phase 3: Bank Accounts & Credit Cards | Pending |
| TXN-01 | Phase 4: Transaction Core | Pending |
| TXN-02 | Phase 4: Transaction Core | Pending |
| TXN-03 | Phase 4: Transaction Core | Pending |
| TXN-04 | Phase 4: Transaction Core | Pending |
| TXN-05 | Phase 4: Transaction Core | Pending |
| TXN-06 | Phase 4: Transaction Core | Pending |
| TXN-07 | Phase 4: Transaction Core | Pending |
| TXN-08 | Phase 4: Transaction Core | Pending |
| TXN-09 | Phase 4: Transaction Core | Pending |
| BUDG-01 | Phase 5: Budgeting | Pending |
| BUDG-02 | Phase 5: Budgeting | Pending |
| BUDG-03 | Phase 5: Budgeting | Pending |
| BUDG-04 | Phase 5: Budgeting | Pending |
| BUDG-05 | Phase 5: Budgeting | Pending |
| RPT-01 | Phase 6: Reporting & Net Worth | Pending |
| RPT-02 | Phase 6: Reporting & Net Worth | Pending |
| RPT-03 | Phase 6: Reporting & Net Worth | Pending |
| RPT-04 | Phase 6: Reporting & Net Worth | Pending |
| RPT-05 | Phase 6: Reporting & Net Worth | Pending |
| SUBS-01 | Phase 7: Subscription & Bill Tracking | Pending |
| SUBS-02 | Phase 7: Subscription & Bill Tracking | Pending |
| SUBS-03 | Phase 7: Subscription & Bill Tracking | Pending |
| SUBS-04 | Phase 7: Subscription & Bill Tracking | Pending |
| SUBS-05 | Phase 7: Subscription & Bill Tracking | Pending |
| AI-01 | Phase 8: AI Chat & Smart Entry | Pending |
| AI-02 | Phase 8: AI Chat & Smart Entry | Pending |
| AI-03 | Phase 8: AI Chat & Smart Entry | Pending |
| AI-04 | Phase 8: AI Chat & Smart Entry | Pending |
| AI-05 | Phase 8: AI Chat & Smart Entry | Pending |
| AI-06 | Phase 8: AI Chat & Smart Entry | Pending |
| AI-07 | Phase 8: AI Chat & Smart Entry | Pending |
| AIREC-01 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-02 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-03 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-04 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-05 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-06 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-07 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-08 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-09 | Phase 9: AI Recommendations Engine | Pending |
| AIREC-10 | Phase 9: AI Recommendations Engine | Pending |
| GMAIL-01 | Phase 10: Gmail Integration | Pending |
| GMAIL-02 | Phase 10: Gmail Integration | Pending |
| GMAIL-03 | Phase 10: Gmail Integration | Pending |
| GMAIL-04 | Phase 10: Gmail Integration | Pending |
| MCP-01 | Phase 11: MCP Server | Pending |
| MCP-02 | Phase 11: MCP Server | Pending |
| MCP-03 | Phase 11: MCP Server | Pending |
| MCP-04 | Phase 11: MCP Server | Pending |
| FREEL-01 | Phase 12: Freelancer Mode | Pending |
| FREEL-02 | Phase 12: Freelancer Mode | Pending |
| FREEL-03 | Phase 12: Freelancer Mode | Pending |
| FREEL-04 | Phase 12: Freelancer Mode | Pending |
| FREEL-05 | Phase 12: Freelancer Mode | Pending |
| FREEL-06 | Phase 12: Freelancer Mode | Pending |

**Coverage:**
- v1 requirements: 73 total
- Mapped to phases: 73
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-20*
*Last updated: 2026-04-20 after roadmap creation*
