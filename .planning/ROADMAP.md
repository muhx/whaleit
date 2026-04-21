# Roadmap: WhaleIt

## Overview

WhaleIt evolves Whaleit from an investment-only tracker into a full personal finance companion. The journey starts with a clean rebrand and codebase health, establishes PostgreSQL alongside SQLite, then builds out the core finance domains (accounts → transactions → budgets → reports → subscriptions). AI features layer on top — conversational entry, context-aware chat, OCR receipts, and periodic recommendations. Integration features (Gmail, MCP server) and freelancer mode complete the picture. Every phase delivers a coherent, verifiable capability that users can interact with.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Codebase Health & Rebrand** - WhaleIt identity and clean architecture for expansion
- [ ] **Phase 2: Dual Database Engine** - PostgreSQL alongside SQLite through shared repository traits
- [ ] **Phase 3: Bank Accounts & Credit Cards** - All account types in a unified view
- [ ] **Phase 4: Transaction Core** - Record, import, search, and categorize transactions
- [ ] **Phase 5: Budgeting** - Category-based and percentage-based spending budgets
- [ ] **Phase 6: Reporting & Net Worth** - Financial visualizations and net worth tracking
- [ ] **Phase 7: Subscription & Bill Tracking** - Recurring payment detection and reminders
- [ ] **Phase 8: AI Chat & Smart Entry** - Conversational transactions and AI-powered suggestions
- [ ] **Phase 9: AI Recommendations Engine** - Periodic financial insights and advice
- [ ] **Phase 10: Gmail Integration** - Auto-discover subscriptions from email receipts
- [ ] **Phase 11: MCP Server** - Secure external AI tool access to financial data
- [ ] **Phase 12: Freelancer Mode** - Business expense tracking and invoice management

## Phase Details

### Phase 1: Codebase Health & Rebrand
**Goal**: The app presents a unified WhaleIt identity and the codebase is clean for expansion
**Depends on**: Nothing (first phase)
**Requirements**: BRAND-01, BRAND-02, BRAND-03, BRAND-04, BRAND-05, BRAND-06
**Success Criteria** (what must be TRUE):
  1. All user-facing text, titles, window labels, and UI copy display "WhaleIt" instead of "Whaleit"
  2. App launches with new WhaleIt icon featuring friendly whale and updated color palette
  3. New users see WhaleIt-branded onboarding/welcome screens with "Your friendly finance companion" messaging
  4. GitHub repository metadata, README, and documentation reflect WhaleIt branding
  5. Internal crate names remain unchanged (whaleit-*) — no code-internal renames
**Plans**: TBD

### Phase 2: Dual Database Engine
**Goal**: Both SQLite and PostgreSQL work as storage backends through shared repository traits
**Depends on**: Phase 1
**Requirements**: DB-01, DB-02, DB-03, DB-04, DB-05
**Success Criteria** (what must be TRUE):
  1. Desktop app starts with SQLite and web app starts with PostgreSQL automatically based on build target
  2. All existing investment domain queries (accounts, activities, holdings, goals) return identical results on both SQLite and PostgreSQL
  3. Separate migration directories exist for SQLite and PostgreSQL with consistent schema definitions
  4. Repository traits are async-native, abstracting the sync SQLite and async PostgreSQL write patterns behind a unified interface
**Plans**: 4 plans

Plans:
- [ ] 02-01-PLAN.md — Async trait conversion, diesel upgrade, storage-common crate
- [ ] 02-02-PLAN.md — storage-postgres crate with all repositories and PG migrations
- [ ] 02-03-PLAN.md — Server wiring with postgres feature flag, Docker Compose PG service
- [ ] 02-04-PLAN.md — Parity tests and CI matrix for both engines

### Phase 3: Bank Accounts & Credit Cards
**Goal**: Users can manage checking, savings, and credit card accounts alongside existing investment accounts
**Depends on**: Phase 2
**Requirements**: ACCT-01, ACCT-02, ACCT-03, ACCT-04, ACCT-05, ACCT-06, ACCT-07
**Success Criteria** (what must be TRUE):
  1. User can create and manage bank accounts (checking, savings) with name, institution, currency, and opening balance
  2. User can create credit card accounts with limit, utilization percentage, and statement cycle tracking
  3. All account types (bank, credit card, investment) appear in a unified account list with current balances
  4. User can edit and archive accounts without losing any historical transaction data
  5. Credit card accounts show outstanding balance, available credit, statement details, and reward points/cashback
**Plans**: TBD
**UI hint**: yes

### Phase 4: Transaction Core
**Goal**: Users can record, import, search, and categorize financial transactions across all accounts
**Depends on**: Phase 3
**Requirements**: TXN-01, TXN-02, TXN-03, TXN-04, TXN-05, TXN-06, TXN-07, TXN-08, TXN-09
**Success Criteria** (what must be TRUE):
  1. User can quickly add a manual transaction with amount, payee, category, date, account, and notes
  2. User can import transactions from CSV and OFX files with flexible column mapping
  3. User can search and filter transactions across all accounts by merchant, amount range, date range, category, and account
  4. System detects and flags potential duplicate transactions for user review
  5. Transactions support multi-currency with FX conversion, split categories across multiple budget lines, and show running balance per account
**Plans**: TBD
**UI hint**: yes

### Phase 5: Budgeting
**Goal**: Users can create and track spending budgets with monthly progress and flexible allocation rules
**Depends on**: Phase 4
**Requirements**: BUDG-01, BUDG-02, BUDG-03, BUDG-04, BUDG-05
**Success Criteria** (what must be TRUE):
  1. User can create monthly budgets by spending category with target amounts
  2. Budget dashboard shows spent vs. remaining per category with visual progress indicators
  3. User can set up 50/30/20 rule budget with automatic income percentage allocation to needs, wants, and savings
  4. Budgets roll over month-to-month with option to reset or carry over unspent amounts
  5. Transactions auto-assign to budget categories via categorization rules or manual assignment
**Plans**: TBD
**UI hint**: yes

### Phase 6: Reporting & Net Worth
**Goal**: Users can visualize their complete financial picture through reports, charts, and net worth tracking
**Depends on**: Phase 4
**Requirements**: RPT-01, RPT-02, RPT-03, RPT-04, RPT-05
**Success Criteria** (what must be TRUE):
  1. User can view spending breakdown by category as pie chart and detailed table
  2. User can view monthly spending trends over time and income vs. expense comparison by month
  3. Net worth dashboard shows total assets (investments + bank) minus total liabilities (credit cards) with historical trend
  4. All reports convert multi-currency amounts to user's base currency
**Plans**: TBD
**UI hint**: yes

### Phase 7: Subscription & Bill Tracking
**Goal**: Users never miss a recurring payment and understand their total subscription spend
**Depends on**: Phase 4
**Requirements**: SUBS-01, SUBS-02, SUBS-03, SUBS-04, SUBS-05
**Success Criteria** (what must be TRUE):
  1. System auto-detects recurring transactions from history (same merchant, similar amount, regular interval) and suggests subscriptions
  2. User can manually add subscriptions with name, amount, billing cycle, next billing date, and category
  3. User sees a subscription calendar showing upcoming bills across all accounts
  4. Bill reminder notifications fire before upcoming payment due dates with configurable advance days
  5. Subscription dashboard shows total monthly and annual subscription spend
**Plans**: TBD
**UI hint**: yes

### Phase 8: AI Chat & Smart Entry
**Goal**: Users interact with AI to create transactions conversationally and receive intelligent suggestions throughout the app
**Depends on**: Phase 4
**Requirements**: AI-01, AI-02, AI-03, AI-04, AI-05, AI-06, AI-07
**Success Criteria** (what must be TRUE):
  1. AI chat sidebar is accessible from any screen and knows which page the user is currently viewing
  2. User can create transactions via natural language ("I spent $50 on groceries at Whole Foods") and the transaction appears in the ledger
  3. User can upload receipt images and AI extracts merchant, amount, date, items, and suggested category for review
  4. AI auto-categorizes transactions using merchant patterns and suggests category/payee during manual entry
  5. User can bring their own API key (OpenAI, Anthropic, Google) or use a hosted AI service option
**Plans**: TBD
**UI hint**: yes

### Phase 9: AI Recommendations Engine
**Goal**: Users receive periodic AI-generated financial insights covering spending, investments, and tax optimization
**Depends on**: Phase 5, Phase 6
**Requirements**: AIREC-01, AIREC-02, AIREC-03, AIREC-04, AIREC-05, AIREC-06, AIREC-07, AIREC-08, AIREC-09, AIREC-10
**Success Criteria** (what must be TRUE):
  1. System generates periodic insights at daily (spending alerts), weekly (summaries), monthly (health reports), quarterly (investment reviews), and yearly (overview) intervals
  2. AI recommendations cover spending insights, investment advice, and tax optimization suggestions
  3. Dedicated AI Insights page shows full recommendation history with drill-down into each insight
  4. Dashboard widget shows latest insights and push-style notifications deliver time-sensitive recommendations at configurable intervals
  5. All AI recommendations display a clear disclaimer that they are informational, not professional financial advice
**Plans**: TBD
**UI hint**: yes

### Phase 10: Gmail Integration
**Goal**: Users can auto-discover subscriptions and extract invoice details from their Gmail inbox
**Depends on**: Phase 7, Phase 8
**Requirements**: GMAIL-01, GMAIL-02, GMAIL-03, GMAIL-04
**Success Criteria** (what must be TRUE):
  1. User can connect Gmail account via OAuth with tokens stored securely via OS keyring (desktop) or encrypted file (web)
  2. System scans incoming emails for subscription and billing patterns and suggests new subscriptions to track
  3. System extracts invoice details (merchant, amount, date, billing cycle) from email receipts with AI-powered parsing
**Plans**: TBD
**UI hint**: yes

### Phase 11: MCP Server
**Goal**: External AI tools can securely read user's financial data through an MCP protocol endpoint
**Depends on**: Phase 4
**Requirements**: MCP-01, MCP-02, MCP-03, MCP-04
**Success Criteria** (what must be TRUE):
  1. MCP server endpoint exposes accounts, transactions, budgets, and investments as tools to external AI tools (Claude Desktop, etc.)
  2. External tools authenticate via API key or web mode JWT token before accessing any data
  3. MCP tools provide read-only access to financial data — no write or delete operations exposed
  4. MCP server runs as part of the Axum web server at `/mcp` endpoint
**Plans**: TBD

### Phase 12: Freelancer Mode
**Goal**: Freelancers can separate business from personal finances, manage invoices, and prepare for tax time
**Depends on**: Phase 4
**Requirements**: FREEL-01, FREEL-02, FREEL-03, FREEL-04, FREEL-05, FREEL-06
**Success Criteria** (what must be TRUE):
  1. User can tag any transaction as personal or business with a single toggle
  2. User can assign business expenses to specific clients or projects
  3. User can apply tax categories from pre-built templates (Schedule C, common deductibles) for tax-time export
  4. User can create and send invoices with line items, due dates, and payment status tracking
  5. Freelancer dashboard shows business income vs. expenses, outstanding invoices, and tax-deductible totals
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5/6/7/8/11/12 (Phase 4 unblocks many) → 9 → 10

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Codebase Health & Rebrand | 0/? | Not started | - |
| 2. Dual Database Engine | 0/? | Not started | - |
| 3. Bank Accounts & Credit Cards | 0/? | Not started | - |
| 4. Transaction Core | 0/? | Not started | - |
| 5. Budgeting | 0/? | Not started | - |
| 6. Reporting & Net Worth | 0/? | Not started | - |
| 7. Subscription & Bill Tracking | 0/? | Not started | - |
| 8. AI Chat & Smart Entry | 0/? | Not started | - |
| 9. AI Recommendations Engine | 0/? | Not started | - |
| 10. Gmail Integration | 0/? | Not started | - |
| 11. MCP Server | 0/? | Not started | - |
| 12. Freelancer Mode | 0/? | Not started | - |
