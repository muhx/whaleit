# Architecture Research

**Domain:** Personal finance management application (expanding investment
portfolio tracker) **Researched:** 2026-04-20 **Confidence:** HIGH (existing
architecture well-documented; new component patterns verified against official
docs)

## Recommended Architecture

### System Overview (Expanded)

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Frontend (React + Vite)                         │
│  apps/frontend/src/                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐ ┌─────────┐ │
│  │ Pages/    │ │Features/ │ │AI Chat   │ │Budgeting  │ │Accounts │ │
│  │ Routes    │ │Invest-   │ │Sidebar   │ │Subs       │ │Bank,Cred│ │
│  │           │ │ments     │ │Panel     │ │           │ │         │ │
│  └─────┬─────┘ └─────┬───┘ └────┬─────┘ └─────┬─────┘ └────┬────┘ │
│        │              │          │              │             │       │
│  ┌─────▼──────────────▼──────────▼──────────────▼─────────────▼────┐│
│  │                  Adapter Layer (shared/tauri/web)                ││
│  │  adapters/shared/ — new modules: bank-accounts.ts, budgets.ts,  ││
│  │  subscriptions.ts, ocr.ts, gmail.ts + existing modules          ││
│  └──────────────────────────┬──────────────────────────────────────┘│
└─────────────────────────────┼───────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            │                 │                 │
       Desktop Mode      Web Mode         MCP Clients
            │                 │           (Claude Desktop, etc.)
┌───────────▼──────────┐ ┌───▼──────────────┐         │
│  Tauri IPC           │ │  Axum HTTP API    │         │
│  apps/tauri/         │ │  apps/server/     │         │
│  commands/*.rs       │ │  api/*.rs         │         │
│  + MCP server (opt)  │ │  + MCP endpoint   │         │
└───────────┬──────────┘ └───┬──────────────┘         │
            │                 │                         │
            └────────┬────────┘                         │
                     │                                  │
┌────────────────────▼──────────────────────────────────▼───────────┐
│                     Core Crates (Rust)                             │
│  crates/core/       — Domain logic, traits, models, events        │
│    + bank_accounts/ — Bank account models & service traits         │
│    + credit_cards/  — Credit card models & service traits          │
│    + transactions/  — Daily transaction models & service traits    │
│    + budgets/       — Budget models, envelope & percentage rules   │
│    + subscriptions/ — Subscription/bill tracking models & traits   │
│    + recommendations/ — AI recommendation models & traits          │
│                                                                    │
│  crates/storage-sqlite/   — SQLite implementations (existing)      │
│    + bank_accounts/, credit_cards/, transactions/, budgets/, etc.  │
│                                                                    │
│  crates/storage-postgres/ — PostgreSQL implementations (NEW)       │
│    + mirrors storage-sqlite structure with PgConnection            │
│    + uses diesel-async + deadpool for async connection pooling     │
│    + no write actor needed (PG handles concurrent writes)          │
│                                                                    │
│  crates/market-data/ — Market data providers (existing)            │
│  crates/connect/     — Broker sync integration (existing)          │
│  crates/device-sync/ — E2EE device synchronization (existing)      │
│  crates/gmail/       — Gmail OAuth + invoice scanning (NEW)        │
│  crates/ai/          — LLM integration + tools + streaming         │
│    + tools/ — expanded tool set (transactions, budgets, subs)      │
│    + recommendations/ — scheduled insight generation               │
│  crates/mcp-server/  — MCP server endpoint (NEW)                   │
│    — wraps core service traits as MCP tools                        │
│    — reuses AiEnvironment-like pattern for DI                      │
└────────────────────────────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────────┐
│                     Database Layer                                  │
│  ┌─────────────────────┐  ┌──────────────────────────────────┐      │
│  │ SQLite (Desktop)     │  │ PostgreSQL (Web/Self-hosted)     │      │
│  │ storage-sqlite/      │  │ storage-postgres/                │      │
│  │ migrations/          │  │ migrations/                      │      │
│  │ Write Actor pattern  │  │ diesel-async + deadpool          │      │
│  └─────────────────────┘  └──────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component                      | Responsibility                                                  | Implementation Notes                                                                           |
| ------------------------------ | --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `crates/core/bank_accounts/`   | Bank account domain models, service traits, validation          | Mirrors `accounts/` pattern — `BankAccountServiceTrait`, `NewBankAccount`, `BankAccountUpdate` |
| `crates/core/credit_cards/`    | Credit card models, balance tracking, rewards, statement cycles | `CreditCardServiceTrait`, `CreditCardBalance`, `RewardPoints`                                  |
| `crates/core/transactions/`    | Daily transactions with categories, auto-categorization rules   | `TransactionServiceTrait`, `TransactionCategory`, `TransactionRule`                            |
| `crates/core/budgets/`         | Envelope budgets, percentage-based rules, period tracking       | `BudgetServiceTrait`, `EnvelopeBudget`, `PercentageRule`                                       |
| `crates/core/subscriptions/`   | Subscription/bill tracking, renewal reminders, detection        | `SubscriptionServiceTrait`, `Subscription`, `BillReminder`                                     |
| `crates/core/recommendations/` | AI-generated financial insights, multi-period scheduling        | `RecommendationServiceTrait`, `Recommendation`, `InsightPeriod`                                |
| `crates/storage-postgres/`     | PostgreSQL implementations of all repository traits             | `diesel-async` + `deadpool`, same trait implementations as `storage-sqlite`                    |
| `crates/gmail/`                | Gmail OAuth flow, email scanning, invoice extraction            | `reqwest` + Gmail API v1, token storage via `SecretStore` trait                                |
| `crates/mcp-server/`           | MCP protocol endpoint for external AI tools                     | `rust-mcp-sdk` hyper_server, wraps core service traits as MCP tools                            |
| `crates/ai/tools/` (expanded)  | New AI tools for transactions, budgets, subscriptions, OCR      | Extend existing `ToolSet` + `AiEnvironment` pattern                                            |

## Recommended Crate Structure

```
crates/
├── core/                    # Domain logic, models, service traits
│   └── src/
│       ├── accounts/        # Existing — investment accounts
│       ├── activities/      # Existing — investment activities
│       ├── assets/          # Existing — asset models
│       ├── bank_accounts/   # NEW — bank account models & traits
│       │   ├── mod.rs       # BankAccountServiceTrait, models
│       │   └── models.rs    # BankAccount, NewBankAccount, BankAccountType
│       ├── credit_cards/    # NEW — credit card models & traits
│       │   ├── mod.rs       # CreditCardServiceTrait, models
│       │   └── models.rs    # CreditCard, CreditCardBalance, RewardPoints
│       ├── transactions/    # NEW — daily transactions & categorization
│       │   ├── mod.rs       # TransactionServiceTrait, models
│       │   ├── models.rs    # Transaction, TransactionCategory, TransactionRule
│       │   └── categorization.rs # Auto-categorization rules engine
│       ├── budgets/         # NEW — envelope & percentage budgeting
│       │   ├── mod.rs       # BudgetServiceTrait, models
│       │   ├── models.rs    # Budget, BudgetPeriod, EnvelopeBudget, PercentageRule
│       │   └── rules.rs     # Budget evaluation & threshold checking
│       ├── subscriptions/   # NEW — subscription/bill tracking
│       │   ├── mod.rs       # SubscriptionServiceTrait, models
│       │   └── models.rs    # Subscription, BillReminder, SubscriptionStatus
│       ├── recommendations/ # NEW — AI recommendation models
│       │   ├── mod.rs       # RecommendationServiceTrait, models
│       │   └── models.rs    # Recommendation, InsightPeriod, RecommendationType
│       ├── events/          # Existing — expanded with new event types
│       │   ├── domain_event.rs  # + TransactionsChanged, BudgetThresholdExceeded, etc.
│       │   └── sink.rs
│       └── lib.rs           # Register new modules
│
├── storage-sqlite/          # Existing — add new domain repositories
│   └── src/
│       ├── bank_accounts/   # NEW — SQLite bank account repo
│       ├── credit_cards/    # NEW — SQLite credit card repo
│       ├── transactions/    # NEW — SQLite transaction repo
│       ├── budgets/         # NEW — SQLite budget repo
│       ├── subscriptions/   # NEW — SQLite subscription repo
│       ├── recommendations/ # NEW — SQLite recommendation repo
│       ├── schema.rs        # Extended with new table definitions
│       └── migrations/      # New migration files for new tables
│
├── storage-postgres/        # NEW — PostgreSQL implementations
│   ├── Cargo.toml           # diesel + diesel-async + deadpool
│   └── src/
│       ├── lib.rs           # Re-exports, pool setup
│       ├── db/
│       │   ├── mod.rs       # PgPool, connection management
│       │   └── migrations.rs # PG-specific migration runner
│       ├── schema.rs        # Diesel PG schema (shared with SQLite where possible)
│       ├── accounts/        # PG impl of AccountServiceTrait
│       ├── activities/      # PG impl of ActivityServiceTrait
│       ├── bank_accounts/   # PG impl of BankAccountServiceTrait
│       ├── credit_cards/    # PG impl of CreditCardServiceTrait
│       ├── transactions/    # PG impl of TransactionServiceTrait
│       ├── budgets/         # PG impl of BudgetServiceTrait
│       ├── subscriptions/   # PG impl of SubscriptionServiceTrait
│       ├── recommendations/ # PG impl of RecommendationServiceTrait
│       └── ai_chat/         # PG impl of ChatRepositoryTrait
│
├── gmail/                   # NEW — Gmail integration
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── oauth.rs         # OAuth2 flow, token lifecycle
│       ├── client.rs        # Gmail API client (messages.list, messages.get)
│       ├── scanner.rs       # Invoice/receipt detection from emails
│       └── models.rs        # GmailMessage, InvoiceData, ParsedSubscription
│
├── ai/                      # Existing — expand tools + recommendations
│   └── src/
│       ├── tools/
│       │   ├── bank_accounts.rs     # NEW — GetBankAccountsTool
│       │   ├── credit_cards.rs      # NEW — GetCreditCardsTool
│       │   ├── transactions.rs      # NEW — SearchTransactionsTool, CreateTransactionTool
│       │   ├── budgets.rs           # NEW — GetBudgetsTool, GetBudgetStatusTool
│       │   ├── subscriptions.rs     # NEW — GetSubscriptionsTool
│       │   ├── recommendations.rs   # NEW — GetRecommendationsTool
│       │   └── ocr_receipt.rs       # NEW — ProcessReceiptTool (extracts from image)
│       ├── recommendations/         # NEW — scheduled insight generation
│       │   ├── mod.rs               # RecommendationEngine
│       │   ├── daily.rs             # Daily spending insights
│       │   ├── weekly.rs            # Weekly budget review
│       │   ├── monthly.rs           # Monthly summary + trends
│       │   └── quarterly.rs         # Quarterly + yearly deep analysis
│       └── env.rs                   # Extended AiEnvironment trait
│
└── mcp-server/              # NEW — MCP server endpoint
    ├── Cargo.toml           # rust-mcp-sdk + core crates
    └── src/
        ├── lib.rs           # MCP server factory
        ├── handler.rs       # ServerHandler impl with tool dispatch
        └── tools/           # MCP tool definitions
            ├── mod.rs       # tool_box! macro grouping
            ├── accounts.rs  # Wrapper around AccountServiceTrait
            ├── transactions.rs
            ├── budgets.rs
            ├── holdings.rs
            └── subscriptions.rs
```

### Frontend Structure Additions

```
apps/frontend/src/
├── adapters/shared/
│   ├── bank-accounts.ts    # NEW — adapter commands for bank accounts
│   ├── credit-cards.ts     # NEW — adapter commands for credit cards
│   ├── transactions.ts     # NEW — adapter commands for transactions
│   ├── budgets.ts          # NEW — adapter commands for budgets
│   ├── subscriptions.ts    # NEW — adapter commands for subscriptions
│   ├── gmail.ts            # NEW — Gmail OAuth + connection adapter
│   └── recommendations.ts  # NEW — adapter commands for recommendations
├── features/
│   ├── bank-accounts/      # NEW — bank account management feature module
│   ├── credit-cards/       # NEW — credit card tracking feature module
│   ├── transactions/       # NEW — daily transactions feature module
│   ├── budgets/            # NEW — budget management feature module
│   ├── subscriptions/      # NEW — subscription tracking feature module
│   ├── ai-chat/            # Enhanced — context-aware sidebar panel
│   └── recommendations/    # NEW — insights & recommendations page
└── pages/
    └── (new route pages for each feature)
```

## Architectural Patterns

### Pattern 1: Dual Database Engine via Repository Traits

**What:** Use the existing repository trait pattern to abstract over SQLite and
PostgreSQL. `crates/core/` defines service traits; `crates/storage-sqlite/` and
`crates/storage-postgres/` each implement them. Runtime selects the appropriate
implementation at startup.

**When to use:** Every new domain module that needs persistence.

**Trade-offs:**

- (+) Core business logic stays database-agnostic
- (+) Testing with mock implementations is trivial (proven by existing
  MockEnvironment pattern)
- (+) Each backend can optimize independently (SQLite uses write actor, PG uses
  async pool)
- (-) Two sets of migrations to maintain
- (-) Some SQL dialect differences require conditional compilation

**Example:**

```rust
// crates/core/bank_accounts/mod.rs — trait definition
#[async_trait]
pub trait BankAccountServiceTrait: Send + Sync {
    fn get_all_bank_accounts(&self) -> Result<Vec<BankAccount>>;
    fn get_bank_account(&self, id: &str) -> Result<BankAccount>;
    async fn create_bank_account(&self, account: NewBankAccount) -> Result<BankAccount>;
    async fn update_bank_account(&self, account: BankAccountUpdate) -> Result<BankAccount>;
    async fn delete_bank_account(&self, id: &str) -> Result<()>;
}

// crates/storage-sqlite/bank_accounts/mod.rs — SQLite impl
pub struct SqliteBankAccountRepository {
    pool: DbPool,
    writer: WriteHandle,
}
#[async_trait]
impl BankAccountServiceTrait for SqliteBankAccountRepository {
    // Uses WriteHandle for all mutations (SQLite write actor pattern)
    async fn create_bank_account(&self, account: NewBankAccount) -> Result<BankAccount> {
        self.writer.exec(move |conn| {
            // diesel::insert_into(...) with SqliteConnection
        }).await
    }
    // Reads use pool directly
    fn get_all_bank_accounts(&self) -> Result<Vec<BankAccount>> { ... }
}

// crates/storage-postgres/bank_accounts/mod.rs — PostgreSQL impl
pub struct PgBankAccountRepository {
    pool: deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}
#[async_trait]
impl BankAccountServiceTrait for PgBankAccountRepository {
    // No write actor needed — PG handles concurrent writes natively
    async fn create_bank_account(&self, account: NewBankAccount) -> Result<BankAccount> {
        let mut conn = self.pool.get().await?;
        diesel::insert_into(bank_accounts::table)
            .values(&NewBankAccountModel::from_domain(account))
            .returning(BankAccountModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| StorageError::from(e).into())
            .map(|m| m.to_domain())
    }
}
```

### Pattern 2: Extended Domain Events for Finance Features

**What:** Extend the existing `DomainEvent` enum with new variants for bank
account, transaction, budget, and subscription changes. New event handlers
process side effects.

**When to use:** Every mutation that triggers downstream processing (budget
recalculation, recommendation generation, notification dispatch).

**Trade-offs:**

- (+) Side-effect-free core logic (proven pattern in existing codebase)
- (+) Easy to add new event consumers without modifying producers
- (-) Event proliferation needs careful naming to stay manageable

**Example:**

```rust
// crates/core/events/domain_event.rs — additions
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    // ... existing variants ...

    // New finance event variants
    TransactionsChanged {
        account_ids: Vec<String>,
        categories: Vec<String>,
        period: Option<String>,
    },
    BudgetThresholdExceeded {
        budget_id: String,
        category: String,
        spent: Decimal,
        limit: Decimal,
        percentage: Decimal,
    },
    SubscriptionDueSoon {
        subscription_id: String,
        due_date: String,
        amount: Decimal,
    },
    BankAccountBalanceChanged {
        account_id: String,
        old_balance: Option<Decimal>,
        new_balance: Decimal,
    },
    RecommendationsGenerated {
        period: String,
        count: usize,
    },
}
```

### Pattern 3: MCP Server as a Thin Wrapper

**What:** The MCP server (`crates/mcp-server/`) wraps existing `crates/core/`
service traits as MCP tools. It does NOT duplicate business logic — it calls the
same trait implementations that Tauri and Axum use.

**When to use:** Exposing financial data to external AI tools (Claude Desktop,
Cursor, etc.)

**Trade-offs:**

- (+) Zero duplication — same service traits, same validation, same data
- (+) Auth/security handled once at the MCP transport layer
- (-) MCP tools must map to trait method signatures (some impedance mismatch)

**Example:**

```rust
// crates/mcp-server/handler.rs
use rust_mcp_sdk::mcp_server::ServerHandler;

pub struct WhaleItMcpHandler<E: McpEnvironment> {
    env: Arc<E>,
}

#[async_trait]
impl<E: McpEnvironment + 'static> ServerHandler for WhaleItMcpHandler<E> {
    async fn handle_list_tools_request(&self, ...) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: WhaleItTools::tools(),
            meta: None, next_cursor: None,
        })
    }

    async fn handle_call_tool_request(&self, params, _runtime) -> Result<CallToolResult, CallToolError> {
        let tool = WhaleItTools::try_from(params).map_err(CallToolError::new)?;
        match tool {
            WhaleItTools::GetBankAccounts(t) => {
                let accounts = self.env.bank_account_service().get_all_bank_accounts()?;
                Ok(CallToolResult::text_content(vec![
                    serde_json::to_string_pretty(&accounts)?.into()
                ]))
            }
            // ... other tools
        }
    }
}
```

### Pattern 4: Gmail Integration via Connect Crate Pattern

**What:** Follow the `crates/connect/` pattern for Gmail OAuth — feature-gated
crate with OAuth lifecycle, token storage via existing `SecretStore`, and
ingestion pipeline.

**When to use:** Gmail integration for subscription discovery from email
receipts/invoices.

**Trade-offs:**

- (+) Consistent with existing broker sync architecture
- (+) Feature-gated means users who don't need Gmail don't pay the cost
- (-) Gmail API has rate limits; must implement backoff and caching

### Pattern 5: OCR via Existing AI Multimodal Pipeline

**What:** Leverage the existing AI chat infrastructure (image/PDF attachment
support already in `chat.rs` via `build_user_prompt`) + a new
`ProcessReceiptTool` that extracts structured transaction data from receipt
images.

**When to use:** Receipt scanning for transaction creation.

**Trade-offs:**

- (+) No new OCR crate needed — uses LLM vision capabilities already supported
- (+) Users control the model choice (cheaper vs. more capable)
- (-) Requires vision-capable model (user must configure)
- (-) Accuracy depends on model quality

## Data Flow

### Transaction Entry Flow (Manual)

```
User types in transaction form
    ↓
Frontend component → TanStack mutation → adapter/shared/transactions.ts
    ↓
┌─── Desktop: Tauri invoke("create_transaction") ───────────┐
│   Web:     fetch("/api/v1/transactions", POST)             │
└────────────────────────┬───────────────────────────────────┘
                         ↓
    Tauri command / Axum handler (thin delegate)
                         ↓
    TransactionService.create_transaction()
        → validates, auto-categorizes, persists
        → emits DomainEvent::TransactionsChanged
                         ↓
    Domain event worker processes:
        → Budget recalculation for affected categories
        → Net worth update
        → Recommendation queue update
        → Tauri app event / SSE push to frontend
                         ↓
    Frontend invalidates TanStack Query cache
```

### Transaction Entry Flow (AI Chat — Conversational)

```
User: "I spent $50 on groceries at Whole Foods"
    ↓
Frontend AI Chat → adapter/shared/ai-threads.ts → stream endpoint
    ↓
ChatService.send_message() → rig-core agent with tools
    ↓
LLM decides to call CreateTransactionTool
    ↓
CreateTransactionTool.call() → TransactionService.create_transaction()
    (creates draft transaction, returns confirmation)
    ↓
AiStreamEvent::ToolResult sent back to chat
    ↓
Frontend renders transaction draft card → user confirms → persisted
```

### OCR Receipt Flow

```
User attaches receipt photo to AI chat
    ↓
ChatService sends multimodal message (image + text)
    ↓
LLM vision model examines receipt
    ↓
Calls ProcessReceiptTool or CreateTransactionTool
    (extracts: merchant, amount, date, category, items)
    ↓
Returns structured transaction draft to chat
    ↓
User reviews/edits → confirms → transaction persisted
```

### Gmail Subscription Discovery Flow

```
User connects Gmail via OAuth settings page
    ↓
Frontend → Gmail OAuth adapter → Tauri/Axum command
    ↓
gmail::oauth initiate OAuth2 flow → browser redirect
    ↓
Callback with auth code → exchange for tokens
    ↓
Store tokens via SecretStore trait
    ↓
gmail::scanner scans recent emails for subscription invoices
    ↓
Extract: merchant, amount, renewal date, frequency
    ↓
Create Subscription entries (draft status)
    ↓
User reviews → confirms → subscriptions tracked
    ↓
Domain events → Bill reminders → Notification delivery
```

### MCP Server Data Flow

```
External AI tool (Claude Desktop) connects to MCP endpoint
    ↓
MCP handshake (Streamable HTTP or SSE transport)
    ↓
External AI calls MCP tool (e.g., "get_transactions")
    ↓
WhaleItMcpHandler.handle_call_tool_request()
    ↓
Dispatches to TransactionService via McpEnvironment trait
    ↓
Service executes query via repository (SQLite or PG)
    ↓
Results serialized → returned as MCP tool result
    ↓
External AI tool receives data, reasons, responds to user
```

### AI Recommendation Generation Flow

```
Scheduler triggers (daily/weekly/monthly/quarterly)
    ↓
RecommendationEngine::generate_insights(period)
    ↓
Queries across all domains:
    → TransactionService (spending patterns)
    → BudgetService (budget adherence)
    → SubscriptionService (upcoming bills)
    → AccountService + HoldingsService (investment health)
    ↓
Constructs context → sends to LLM via ChatService
    (system prompt includes all financial data as context)
    ↓
LLM generates structured insights
    ↓
RecommendationService.persist(recommendations)
    ↓
Domain event → notification delivery
    → Dashboard widget update
    → Insights page refresh
```

### Dual Database Selection at Startup

```
Desktop (Tauri):
    ServiceContext::new()
        → SqliteBankAccountRepository::new(pool, writer)
        → All services use SQLite implementations
        → Write actor serializes mutations

Web (Axum):
    AppState::new(database_url)
        → if database_url starts with "postgres://":
            → PgBankAccountRepository::new(pg_pool)
            → All services use PostgreSQL implementations
            → diesel-async, no write actor
        → else:
            → SqliteBankAccountRepository::new(pool, writer)
            → Same as desktop, SQLite mode
```

## Internal Boundaries

| Boundary                      | Communication                | Notes                                                                        |
| ----------------------------- | ---------------------------- | ---------------------------------------------------------------------------- |
| Frontend ↔ Adapter Layer      | Typed function calls         | Shared modules call `invoke()` (tauri/web) — Vite alias swaps implementation |
| Adapter ↔ Tauri IPC           | `invoke("command", payload)` | JSON-serialized, 120s timeout                                                |
| Adapter ↔ Axum HTTP           | `fetch("/api/v1/...", opts)` | REST API, same command names as IPC                                          |
| Tauri/Axum ↔ Core Services    | `Arc<dyn ServiceTrait>`      | Dependency injection via ServiceContext/AppState                             |
| Core Services ↔ Storage       | Repository trait methods     | Database-agnostic — SQLite or PG implementation                              |
| Core Services ↔ Domain Events | `event_sink.emit(event)`     | Async, non-blocking, best-effort                                             |
| AI Tools ↔ Core Services      | `AiEnvironment` trait        | Same service traits, injected into tool constructors                         |
| MCP Server ↔ Core Services    | `McpEnvironment` trait       | Same pattern as AiEnvironment, wraps service traits                          |
| Gmail ↔ SecretStore           | `SecretStore` trait          | OAuth tokens stored via existing secret abstraction                          |
| Recommendations ↔ AI          | Via existing ChatService     | Recommendations use the same LLM infrastructure                              |

## Anti-Patterns

### Anti-Pattern 1: Database-Specific Logic in Core Crate

**What people do:** Put SQL query logic or Diesel schema references directly in
`crates/core/` **Why it's wrong:** Core crate must remain database-agnostic. It
defines traits, not implementations. **Do this instead:** Keep `crates/core/`
pure trait + model definitions. All Diesel queries, schema references, and
migration code go in `crates/storage-sqlite/` or `crates/storage-postgres/`. The
existing codebase already follows this pattern perfectly — new domains must too.

### Anti-Pattern 2: Duplicating Business Logic in MCP Server

**What people do:** Implement data access and business rules directly in MCP
tool handlers **Why it's wrong:** MCP server is a transport layer, not a
business layer. Logic would diverge from Tauri/Axum paths. **Do this instead:**
MCP tools delegate to existing service traits. The MCP handler is ~5 lines per
tool: parse params → call service trait → format result.

### Anti-Pattern 3: Synchronous Gmail API Calls

**What people do:** Make Gmail API calls synchronously in request handlers **Why
it's wrong:** Gmail API has high latency and rate limits. Blocking an async
runtime thread is catastrophic. **Do this instead:** Gmail scanning runs in
background tasks (like the existing `scheduler::run_periodic_sync()`). Results
are persisted to DB, frontend is notified via domain events / SSE.

### Anti-Pattern 4: Tight-Coupling Budget Engine to Transaction Service

**What people do:** Budget service directly calls transaction service to
calculate spending **Why it's wrong:** Creates circular dependencies. Budget
shouldn't know about transaction internals. **Do this instead:** Budget service
receives `TransactionsChanged` events and recalculates independently. Or budget
queries use a shared read-only repository — budget reads transaction data,
doesn't call transaction service.

### Anti-Pattern 5: Separate Schema for PostgreSQL

**What people do:** Maintain completely independent Diesel schemas for SQLite
and PostgreSQL **Why it's wrong:** Tables drift apart. Queries written for one
don't work on the other. **Do this instead:** Use the same `table!` macro
definitions for both. Diesel's schema macros are backend-agnostic. Only the
connection type and migration SQL differ. Define schema once (in `crates/core/`
or a shared `crates/storage-schema/`), import in both storage crates.

## Build Order (Dependencies Between Components)

Components must be built in this order due to hard dependencies:

```
Phase 1: Foundation
├── crates/core/bank_accounts/     (models + traits)
├── crates/core/transactions/      (models + traits)
├── crates/core/events/            (new DomainEvent variants)
│
Phase 2: Storage Layer
├── crates/storage-sqlite/         (new SQLite repos for Phase 1 domains)
├── crates/storage-postgres/       (NEW crate — PG pool setup + existing domain repos)
│
Phase 3: Transport Layer
├── apps/tauri/commands/           (new Tauri IPC commands)
├── apps/server/api/               (new Axum HTTP handlers)
├── apps/frontend/src/adapters/    (new shared adapter modules)
│
Phase 4: Frontend Features
├── apps/frontend/src/features/bank-accounts/
├── apps/frontend/src/features/transactions/
│
Phase 5: Advanced Domains
├── crates/core/credit_cards/      (depends on transactions for payment tracking)
├── crates/core/budgets/           (depends on transactions for spending data)
├── crates/core/subscriptions/     (depends on transactions for bill detection)
│
Phase 6: Storage + Transport for Phase 5
├── crates/storage-sqlite/         (credit cards, budgets, subscriptions repos)
├── crates/storage-postgres/       (credit cards, budgets, subscriptions repos)
├── apps/tauri/ + apps/server/     (credit cards, budgets, subscriptions endpoints)
├── apps/frontend/                 (credit cards, budgets, subscriptions features)
│
Phase 7: AI Expansion
├── crates/ai/tools/               (new tools for all finance domains)
├── crates/ai/env.rs               (extended AiEnvironment with new services)
│
Phase 8: Integrations
├── crates/gmail/                  (OAuth + email scanning)
├── crates/core/recommendations/   (recommendation models + engine)
├── crates/mcp-server/             (MCP endpoint wrapping all services)
│
Phase 9: AI-Powered Features
├── OCR receipt scanning           (via existing multimodal chat + new tool)
├── Conversational transactions    (via AI chat + CreateTransactionTool)
├── AI recommendations             (scheduled generation + delivery)
└── Context-aware AI sidebar       (frontend UI context injection)
```

## Scaling Considerations

| Scale                           | Architecture Adjustments                                                                         |
| ------------------------------- | ------------------------------------------------------------------------------------------------ |
| Single user (desktop)           | SQLite + write actor is perfect. No changes needed.                                              |
| Family/small team (self-hosted) | PostgreSQL mode. Current architecture handles this — Axum + PG pool. Add connection pool tuning. |
| Multi-tenant SaaS (future)      | Would require schema isolation or tenant IDs. NOT in scope for v1 — single-user per instance.    |

### Scaling Priorities

1. **First bottleneck (PostgreSQL mode):** Connection pool exhaustion under
   heavy concurrent usage. Mitigate with `deadpool` pool size tuning and query
   optimization. Diesel-async handles this well.

2. **Second bottleneck (Gmail scanning):** Gmail API rate limits (~250 quota
   units/user/sec). Mitigate with exponential backoff, batched processing, and
   caching parsed results.

3. **Third bottleneck (MCP server):** Long-running MCP sessions holding service
   trait references. Mitigate with `Arc<>` clones (cheap) and stateless tool
   execution.

## Integration Points

### External Services

| Service                                 | Integration Pattern                      | Notes                                                            |
| --------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------- |
| Gmail API v1                            | OAuth2 + REST via `reqwest`              | Token refresh handled by `gmail::oauth`, stored in `SecretStore` |
| LLM Providers (OpenAI, Anthropic, etc.) | Existing `crates/ai/providers.rs`        | No changes needed — just new tools                               |
| MCP Clients (Claude Desktop)            | `rust-mcp-sdk` Streamable HTTP + SSE     | Runs as Axum sub-router or standalone service                    |
| Vision Models                           | Existing multimodal support in `chat.rs` | `build_user_prompt()` already handles images/PDFs                |

### Key Design Decisions for New Components

| Decision                                                         | Rationale                                                                                                                                                             |
| ---------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/storage-postgres/` as separate crate                     | Clean separation — PG needs `diesel-async`, `deadpool`, different features. SQLite needs `rusqlite`, write actor. Mixing in one crate creates feature flag explosion. |
| `crates/gmail/` as separate crate (not inside `crates/connect/`) | Gmail is fundamentally different from broker sync — it's user-facing OAuth, not server-to-server. Separation keeps concerns clear.                                    |
| `crates/mcp-server/` as separate crate                           | MCP protocol is a distinct transport layer. Separating it means the MCP dependency tree doesn't bloat the core app binary.                                            |
| OCR via existing AI multimodal pipeline                          | No additional dependencies. Users already have vision models available. Accuracy is sufficient for receipt data extraction.                                           |
| Recommendations via LLM (not rule engine)                        | Rule engines are rigid and require constant tuning. LLM generates natural-language insights that adapt to spending patterns.                                          |
| `McpEnvironment` trait (mirrors `AiEnvironment`)                 | Proven DI pattern. MCP tools need the same services as AI tools — use same abstraction.                                                                               |

## Sources

- Existing codebase analysis: `.planning/codebase/ARCHITECTURE.md`,
  `crates/core/`, `crates/ai/`, `crates/storage-sqlite/`
- rust-mcp-sdk: Context7 (`/rust-mcp-stack/rust-mcp-sdk`) — ServerHandler trait,
  tool_box! macro, hyper_server transport
- Diesel dual backend: Context7 (`/diesel-rs/diesel`) — SQLite + PostgreSQL
  connection types, feature flags
- diesel-async: Context7 (`/weiznich/diesel_async`) — AsyncPgConnection,
  deadpool pooling
- rig-core: Context7 (`/0xplaygrounds/rig`) — Agent builder, tool trait,
  streaming (already in use)
- Gmail API v1: Official docs (https://developers.google.com/gmail/api)

---

_Architecture research for: WhaleIt personal finance expansion_ _Researched:
2026-04-20_
