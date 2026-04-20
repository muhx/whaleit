# Codebase Structure

**Analysis Date:** 2026-04-20

## Repository Root

```
whaleit/
├── apps/                    # Application shells (frontend, tauri, server)
├── crates/                  # Rust core crates (business logic, storage, integrations)
├── packages/                # Shared npm packages (UI lib, addon SDK, dev tools)
├── addons/                  # Example/third-party addons (3 addons)
├── scripts/                 # Dev scripts (web dev, e2e runner)
├── e2e/                     # Playwright end-to-end tests
├── docs/                    # Documentation
├── db/                      # Database files (gitignored)
├── assets/                  # Static assets
├── .github/                 # GitHub Actions workflows
├── .claude/                 # Claude AI configuration
├── .planning/               # GSD planning directory
├── package.json             # Root monorepo config (pnpm workspaces)
├── Cargo.toml               # Rust workspace root
├── Cargo.lock               # Rust dependency lockfile
├── pnpm-workspace.yaml      # pnpm workspace definition
├── pnpm-lock.yaml           # Node dependency lockfile
├── tsconfig.base.json       # Shared TypeScript config
├── tsconfig.json            # Root TypeScript config
├── eslint.base.config.js    # Shared ESLint config
├── eslint.config.js         # Root ESLint config
├── playwright.config.ts     # E2E test configuration
├── vite.config.d.ts         # Vite config type declarations
├── Dockerfile               # Multi-stage Docker build
├── compose.yml              # Production Docker Compose
├── compose.dev.yml           # Development Docker Compose
├── AGENTS.md                # AI agent behavioral guide
├── ROADMAP.md               # Project roadmap
└── README.md                # Project overview
```

## apps/frontend/src/ (579 TypeScript files, ~129K lines)

```
apps/frontend/src/
├── adapters/                     # Transport abstraction layer
│   ├── index.ts                  # Re-exports from tauri adapter (default)
│   ├── types.ts                  # Shared adapter types (RunEnv, PlatformInfo, etc.)
│   ├── tauri/                    # Desktop-specific implementations (10 files)
│   │   ├── core.ts               #   Tauri invoke() wrapper, logger, platform flags
│   │   ├── index.ts              #   Barrel export for all Tauri adapters
│   │   ├── activities.ts         #   CSV parsing (Tauri-specific)
│   │   ├── addons.ts             #   Addon zip install (Tauri-specific)
│   │   ├── ai-streaming.ts       #   AI chat streaming via Tauri Channel
│   │   ├── crypto.ts             #   Sync crypto commands (Tauri IPC)
│   │   ├── events.ts             #   Tauri event listeners
│   │   ├── files.ts              #   Native file dialogs
│   │   ├── fire-planner.ts       #   FIRE planner commands
│   │   └── settings.ts           #   Settings + backup/restore/update
│   ├── web/                      # Web-specific implementations (10 files)
│   │   ├── core.ts               #   HTTP fetch() wrapper, COMMANDS map (~1400 lines)
│   │   ├── index.ts              #   Barrel export for all web adapters
│   │   ├── activities.ts         #   CSV parsing (web upload)
│   │   ├── addons.ts             #   Addon install via HTTP
│   │   ├── ai-streaming.ts       #   AI chat streaming via fetch SSE
│   │   ├── crypto.ts             #   Crypto stubs (throws in web mode)
│   │   ├── events.ts             #   SSE event listeners
│   │   ├── files.ts              #   Web file picker fallbacks
│   │   ├── fire-planner.ts       #   FIRE planner stubs (desktop-only)
│   │   └── settings.ts           #   Settings via HTTP API
│   └── shared/                   # Platform-agnostic command wrappers (16 files)
│       ├── accounts.ts           #   Account CRUD
│       ├── activities.ts         #   Activity CRUD + import
│       ├── ai-providers.ts       #   AI provider management
│       ├── ai-threads.ts         #   AI thread management
│       ├── alternative-assets.ts #   Alternative asset CRUD
│       ├── connect.ts            #   Broker + device sync commands
│       ├── contribution-limits.ts#   Contribution limit CRUD
│       ├── custom-provider.ts    #   Custom market data providers
│       ├── exchange-rates.ts     #   FX rate management
│       ├── goals.ts              #   Goal CRUD + allocations
│       ├── health.ts             #   Health check commands
│       ├── market-data.ts        #   Market data sync + quotes
│       ├── portfolio.ts          #   Holdings, snapshots, valuations
│       ├── secrets.ts            #   Secret management
│       ├── taxonomies.ts         #   Taxonomy CRUD
│       └── platform.ts           #   Platform detection
│
├── addons/                       # Addon runtime system (6 files)
│   ├── addons-core.ts            #   Addon lifecycle management
│   ├── addons-dev-mode.ts        #   Development mode support
│   ├── addons-loader.ts          #   Dynamic addon loading
│   ├── addons-runtime-context.ts #   Runtime context for addons (dynamic routes)
│   ├── type-bridge.ts            #   Type bridging for addon API
│   └── type-bridge.test.ts       #   Type bridge tests
│
├── components/                   # Shared UI components (33 entries)
│   ├── classification/           #   Asset classification components
│   ├── page/                     #   Page-level layout components
│   ├── account-selector.tsx      #   Account dropdown selector
│   ├── action-palette.tsx        #   Command palette (Cmd+K)
│   ├── header.tsx                #   App header with navigation
│   ├── history-chart.tsx         #   Portfolio history chart
│   ├── performance-chart.tsx     #   Performance visualization
│   ├── ticker-search.tsx         #   Symbol/ticker search
│   ├── update-dialog.tsx         #   App update notification
│   └── ...                       #   (other shared components)
│
├── context/                      # React context providers (3 files)
│   ├── auth-context.tsx          #   Web auth state management
│   ├── portfolio-sync-context.tsx#   Portfolio sync orchestration
│   └── privacy-context.tsx       #   Balance privacy toggle
│
├── features/                     # Self-contained feature modules (3 features)
│   ├── ai-assistant/             #   AI chat feature
│   │   ├── api/                  #     API calls
│   │   ├── components/           #     Chat UI components
│   │   ├── hooks/                #     Chat-specific hooks
│   │   ├── types.ts              #     AI-specific types
│   │   └── index.ts              #     Feature barrel export
│   ├── devices-sync/             #   Device sync feature
│   │   ├── components/           #     Device management UI
│   │   ├── crypto/               #     Client-side crypto
│   │   ├── hooks/                #     Sync-specific hooks
│   │   ├── services/             #     Sync services
│   │   ├── storage/              #     Sync state persistence
│   │   ├── types.ts              #     Sync-specific types
│   │   └── index.ts              #     Feature barrel export
│   └── wealthfolio-connect/      #   Broker connection feature
│       ├── components/           #     Connection management UI
│       ├── hooks/                #     Connection-specific hooks
│       ├── lib/                  #     Connection utilities
│       ├── pages/                #     Auth callback, connect page
│       ├── providers/            #     Broker-specific adapters
│       ├── services/             #     Connection services
│       ├── types.ts              #     Connection-specific types
│       └── index.ts              #     Feature barrel export
│
├── hooks/                        # Shared React hooks (26 files)
│   ├── index.ts                  #   Barrel export
│   ├── use-accounts.ts           #   Account queries/mutations
│   ├── use-holdings.ts           #   Holdings queries
│   ├── use-settings.ts           #   Settings queries/mutations
│   ├── use-calculate-portfolio.ts#   Portfolio calculation trigger
│   ├── use-platform.ts           #   Platform detection hook
│   ├── use-quote-history.ts      #   Quote history queries
│   └── ...                       #   (other hooks)
│
├── lib/                          # Utility libraries (26 files)
│   ├── query-keys.ts             #   TanStack Query key definitions (~136 lines)
│   ├── schemas.ts                #   Zod validation schemas
│   ├── schemas.test.ts           #   Schema tests
│   ├── constants.ts              #   App-wide constants
│   ├── utils.ts                  #   General utilities
│   ├── is-tauri.ts               #   Tauri runtime detection
│   ├── connect-config.ts         #   Wealthfolio Connect config
│   ├── settings-provider.tsx     #   Settings context provider
│   ├── auth-token.ts             #   JWT token management (web)
│   ├── cookie-utils.ts           #   Cookie helpers (web)
│   ├── portfolio-helper.ts       #   Portfolio calculation helpers
│   ├── asset-utils.ts            #   Asset utility functions
│   ├── activity-utils.ts         #   Activity utility functions
│   ├── export-utils.ts           #   CSV export utilities
│   ├── device-utils.ts           #   Device detection utilities
│   ├── id.ts                     #   ID generation
│   ├── isin.ts                   #   ISIN validation
│   ├── occ-symbol.ts             #   OCC symbol parsing
│   ├── types.ts                  #   Shared TypeScript types
│   ├── types/                    #   Additional type definitions
│   ├── ai-prompt-templates.json  #   AI prompt templates
│   └── ...                       #   (other utils)
│
├── pages/                        # Route pages (17 directories)
│   ├── account/                  #   Account detail page
│   ├── activity/                 #   Activity list + manager + import pages
│   ├── ai-assistant/             #   AI assistant page
│   ├── asset/                    #   Asset list + profile pages
│   ├── auth/                     #   Auth pages (web mode)
│   ├── dashboard/                #   Main portfolio dashboard
│   ├── fire-planner/             #   FIRE planner page
│   ├── health/                   #   Health diagnostics page
│   ├── holdings/                 #   Holdings list + insights pages
│   ├── income/                   #   Income summary page
│   ├── insights/                 #   Portfolio insights page
│   ├── layouts/                  #   App layout + onboarding layout
│   ├── net-worth/                #   Net worth tracking page
│   ├── onboarding/               #   First-run onboarding page
│   ├── performance/              #   Performance analysis page
│   ├── settings/                 #   Settings sub-pages (13 sub-pages)
│   │   ├── about/                #     About page
│   │   ├── accounts/             #     Account management
│   │   ├── addons/               #     Addon management
│   │   ├── ai-providers/         #     AI provider configuration
│   │   ├── appearance/           #     Theme + font settings
│   │   ├── contribution-limits/  #     Contribution limits
│   │   ├── exports/              #     Data export
│   │   ├── fire-planner/         #     FIRE planner settings
│   │   ├── general/              #     General settings
│   │   ├── goals/                #     Goal management
│   │   ├── market-data/          #     Market data providers + import
│   │   ├── taxonomies/           #     Taxonomy management
│   │   └── wealthfolio-connect/  #     Broker connection settings
│   └── not-found.tsx             #   404 page
│
├── types/                        # Global type definitions
│   └── global.d.ts               #   Window type augmentations (addons, debug)
│
├── test/                         # Test configuration
│   └── setup.ts                  #   Vitest global setup
│
├── App.tsx                       # Root app component
├── main.tsx                      # Entry point (platform detection, addon loading)
├── routes.tsx                    # Route definitions (BrowserRouter)
├── globals.css                   # Tailwind v4 global styles + CSS variables
├── lockdowm.ts                   # Desktop security lockdown (context menu, text selection)
├── use-global-event-listener.ts  #   Global keyboard shortcut handler
└── vite-env.d.ts                 #   Vite type declarations
```

## apps/tauri/src/ (48 Rust files, ~11.7K lines)

```
apps/tauri/src/
├── commands/                     # Tauri IPC command handlers (27 modules)
│   ├── mod.rs                    #   Module declarations + feature gates
│   ├── account.rs                #   Account CRUD commands
│   ├── activity.rs               #   Activity CRUD + import commands
│   ├── addon.rs                  #   Addon lifecycle commands
│   ├── ai_chat.rs                #   AI chat streaming + thread management
│   ├── ai_providers.rs           #   AI provider configuration
│   ├── alternative_assets.rs     #   Alternative asset commands
│   ├── asset.rs                  #   Asset CRUD commands
│   ├── brokers_sync.rs           #   Broker sync commands (feature-gated: connect-sync)
│   ├── custom_provider.rs        #   Custom market data provider commands
│   ├── device_enroll_service.rs  #   Device enroll high-level commands (feature-gated: device-sync)
│   ├── device_sync/              #   Device sync commands (feature-gated: device-sync)
│   ├── error.rs                  #   Tauri command error conversion
│   ├── fire.rs                   #   FIRE planner commands
│   ├── goal.rs                   #   Goal CRUD commands
│   ├── health.rs                 #   Health diagnostic commands
│   ├── limits.rs                 #   Contribution limit commands
│   ├── market_data.rs            #   Market data sync + quote commands
│   ├── platform.rs               #   Platform detection commands
│   ├── portfolio.rs              #   Portfolio, holdings, snapshot commands
│   ├── providers_settings.rs     #   Market data provider settings
│   ├── secrets.rs                #   Secret management commands
│   ├── settings.rs               #   Settings + exchange rate commands
│   ├── sync_crypto.rs            #   E2EE crypto commands (feature-gated: device-sync)
│   ├── taxonomy.rs               #   Taxonomy CRUD + migration commands
│   ├── utilities.rs              #   Backup/restore, update check commands
│   └── wealthfolio_connect.rs    #   Sync session management
│
├── context/                      # Service context and DI (4 files)
│   ├── mod.rs                    #   Context module entry
│   ├── ai_environment.rs         #   Tauri-specific AI environment
│   ├── providers.rs              #   Service construction + wiring (~387 lines)
│   └── registry.rs               #   ServiceContext struct (~181 lines)
│
├── domain_events/                # Domain event processing (3 files)
│   ├── mod.rs                    #   Module entry + TauriDomainEventSink
│   ├── queue_worker.rs           #   Event queue consumer
│   └── sink.rs                   #   Domain event sink implementation
│
├── services/                     # App-level services (2 files)
│   ├── mod.rs                    #   Module entry
│   └── connect_service.rs        #   Wealthfolio Connect service
│
├── lib.rs                        # App setup + command registration (~633 lines)
├── main.rs                       # Desktop entry point
├── events.rs                     #   Tauri event definitions + emitters
├── listeners.rs                  #   Frontend event listener setup
├── scheduler.rs                  #   Periodic sync scheduler
├── secret_store.rs               #   OS keyring secret store
├── menu.rs                       #   Application menu (desktop-only)
└── updater.rs                    #   Auto-update handler (desktop-only)
```

## apps/server/src/ (45 Rust files, ~12.3K lines)

```
apps/server/src/
├── api/                          # Axum HTTP handlers (25 modules)
│   ├── accounts.rs               #   /accounts endpoints
│   ├── activities.rs             #   /activities endpoints
│   ├── addons.rs                 #   /addons endpoints
│   ├── ai_chat.rs                #   /ai/chat endpoints
│   ├── ai_providers.rs           #   /ai/providers endpoints
│   ├── alternative_assets.rs     #   /alternative-assets endpoints
│   ├── assets.rs                 #   /assets endpoints
│   ├── connect.rs                #   /connect endpoints (feature-gated)
│   ├── custom_providers.rs       #   /custom-providers endpoints
│   ├── device_sync.rs            #   /device-sync endpoints (feature-gated)
│   ├── device_sync_engine.rs     #   Engine control endpoints (feature-gated)
│   ├── exchange_rates.rs         #   /exchange-rates endpoints
│   ├── goals.rs                  #   /goals endpoints
│   ├── health.rs                 #   /health endpoints
│   ├── holdings/                 #   /holdings endpoints (sub-module)
│   ├── limits.rs                 #   /contribution-limits endpoints
│   ├── market_data.rs            #   /market-data endpoints
│   ├── net_worth.rs              #   /net-worth endpoints
│   ├── performance.rs            #   /performance endpoints
│   ├── portfolio.rs              #   /portfolio endpoints
│   ├── secrets.rs                #   /secrets endpoints
│   ├── settings.rs               #   /settings endpoints
│   ├── shared.rs                 #   Shared API utilities
│   ├── sync_crypto.rs            #   /sync-crypto endpoints (feature-gated)
│   └── taxonomies.rs             #   /taxonomies endpoints
│
├── domain_events/                # Domain event processing (4 files)
│   ├── mod.rs                    #   Module entry
│   ├── planner.rs                #   Event planning logic
│   ├── queue_worker.rs           #   Event queue consumer
│   └── sink.rs                   #   WebDomainEventSink implementation
│
├── secrets/                      # Secret management (1 file)
│   └── mod.rs                    #   File-based encrypted secret store
│
├── api.rs                        # Router composition + middleware (~179 lines)
├── auth.rs                       # JWT + Argon2id authentication (~473 lines)
├── config.rs                     # Environment config (~128 lines)
├── main.rs                       # Server binary entry point
├── main_lib.rs                   # AppState construction (~491 lines)
├── lib.rs                        # Crate public exports
├── models.rs                     # API request/response models
├── events.rs                     # SSE EventBus implementation
├── scheduler.rs                  # Periodic sync scheduler
├── ai_environment.rs             # Server-specific AI environment
└── features.rs                   # Feature flag helpers
```

## crates/ (307 Rust files, ~134K lines)

```
crates/
├── core/                         # Domain logic + service traits
│   └── src/
│       ├── accounts/             #   Account service + traits
│       ├── activities/           #   Activity service + traits
│       ├── addons/               #   Addon service trait
│       ├── assets/               #   Asset, alternative asset services + traits
│       ├── constants.rs          #   Domain constants
│       ├── custom_provider/      #   Custom provider service
│       ├── errors.rs             #   Core error types
│       ├── events/               #   Domain event types + sink trait
│       │   ├── mod.rs            #     Module entry
│       │   ├── domain_event.rs   #     DomainEvent enum (~250 lines)
│       │   └── sink.rs           #     DomainEventSink trait
│       ├── fx/                   #   FX/exchange rate service + traits
│       ├── goals/                #   Goal service + traits
│       ├── health/               #   Health diagnostic service + traits
│       ├── limits/               #   Contribution limit service + traits
│       ├── portfolio/            #   Portfolio domain (largest module)
│       │   ├── allocation/       #     Allocation service + traits
│       │   ├── fire/             #     FIRE planner (Monte Carlo, SORR, sensitivity)
│       │   ├── holdings/         #     Holdings, valuation services + traits
│       │   ├── income/           #     Income service + traits
│       │   ├── net_worth/        #     Net worth service + traits
│       │   ├── performance/      #     Performance calculation service + traits
│       │   ├── snapshot/         #     Snapshot service + traits
│       │   └── valuation/        #     Valuation service + traits
│       ├── quotes/               #   Quote service + traits + scheduler
│       ├── secrets/              #   SecretStore trait
│       ├── settings/             #   Settings service + traits
│       ├── sync/                 #   Sync-related types
│       ├── taxonomies/           #   Taxonomy service + traits
│       ├── utils/                #   Shared utilities
│       └── lib.rs                #   Crate public exports
│
├── storage-sqlite/               # SQLite database layer
│   ├── migrations/               #   29 Diesel migrations (2023-11 → 2026-03)
│   └── src/
│       ├── db/                   #   Connection pool, write actor, initialization
│       ├── accounts/             #   Account repository (Diesel)
│       ├── activities/           #   Activity repository (Diesel)
│       ├── ai_chat/              #   AI chat thread/message repository
│       ├── assets/               #   Asset + alternative asset repositories
│       ├── custom_provider/      #   Custom provider repository
│       ├── fx/                   #   FX rate repository
│       ├── goals/                #   Goal repository
│       ├── health/               #   Health dismissal repository
│       ├── limits/               #   Contribution limit repository
│       ├── market_data/          #   Market data + quote sync state repositories
│       ├── portfolio/            #   Snapshot + valuation repositories
│       │   ├── snapshot/         #     Snapshot repository
│       │   └── valuation/        #     Valuation repository
│       ├── settings/             #   Settings repository (key-value)
│       ├── sync/                 #   Sync state repositories (app, broker, platform, import)
│       ├── taxonomies/           #   Taxonomy repository
│       ├── schema.rs             #   Diesel schema declarations
│       ├── errors.rs             #   Storage error types
│       ├── utils.rs              #   Storage utilities
│       └── lib.rs                #   Crate public exports
│
├── market-data/                  # Market data provider abstraction
│   └── src/
│       ├── errors/               #   Provider error types
│       ├── models/               #   Quote model types
│       ├── provider/             #   Provider implementations (Yahoo Finance, etc.)
│       ├── registry/             #   Provider registry
│       ├── resolver/             #   Symbol resolution logic
│       └── lib.rs                #   Crate public exports
│
├── connect/                      # Broker sync (Wealthfolio Connect)
│   └── src/
│       ├── broker/               #   Broker sync service + orchestrator
│       │   ├── mapping.rs        #     Data mapping logic
│       │   ├── models.rs         #     Broker data models
│       │   ├── orchestrator.rs   #     Sync orchestration
│       │   ├── progress.rs       #     Progress tracking
│       │   ├── service.rs        #     BrokerSyncService
│       │   └── traits.rs         #   BrokerSyncServiceTrait
│       ├── broker_ingest/        #   Broker data ingestion
│       ├── platform/             #   Platform detection
│       ├── client.rs             #   HTTP client for Connect API
│       ├── token_lifecycle.rs    #   Token lifecycle management
│       └── lib.rs                #   Crate public exports
│
├── device-sync/                  # E2EE device synchronization
│   └── src/
│       ├── engine/               #   Sync engine runtime
│       │   ├── mod.rs            #     Engine module
│       │   ├── ports.rs          #     Port interfaces
│       │   └── runtime.rs        #     Runtime state management
│       ├── client.rs             #   Sync API client
│       ├── crypto.rs             #   X25519, ChaCha20Poly1305, HKDF operations
│       ├── enroll_service.rs     #   Device enrollment service
│       ├── error.rs              #   Sync error types
│       ├── time.rs               #   Time utilities
│       ├── types.rs              #   Sync types
│       └── lib.rs                #   Crate public exports
│
└── ai/                           # AI/LLM integration
    └── src/
        ├── tools/                #   AI tool implementations (15 tools)
        │   ├── accounts.rs       #     Account query tool
        │   ├── activities.rs     #     Activity query tool
        │   ├── allocation.rs     #     Allocation query tool
        │   ├── cash_balances.rs  #     Cash balance tool
        │   ├── constants.rs      #     Tool constants
        │   ├── goals.rs          #     Goal query tool
        │   ├── health.rs         #     Health check tool
        │   ├── holdings.rs       #     Holdings query tool
        │   ├── import_csv.rs     #     CSV import tool
        │   ├── income.rs         #     Income query tool
        │   ├── performance.rs    #     Performance query tool
        │   ├── record_activities.rs #  Batch activity recording
        │   ├── record_activity.rs #    Single activity recording
        │   └── valuation.rs      #     Valuation query tool
        ├── eval/                  #   AI evaluation framework
        ├── ai_providers.json     #   Provider catalog (embedded at compile time)
        ├── chat.rs                #   Chat service with streaming
        ├── env.rs                 #   AI environment trait
        ├── error.rs               #   AI error types
        ├── lib.rs                 #   Crate public exports
        ├── prompt_template.rs     #   Prompt template model
        ├── prompt_template_service.rs # Template service
        ├── provider_model.rs     #   Provider model definitions
        ├── provider_service.rs   #   Provider management service
        ├── providers.rs           #   Provider implementations
        ├── stream_hook.rs         #   Streaming hook for AI responses
        ├── system_prompt.txt      #   System prompt for AI assistant
        ├── title_generator.rs     #   Thread title auto-generation
        └── types.rs               #   AI-specific types
```

## packages/ (3 packages)

```
packages/
├── ui/                           # @wealthfolio/ui — Shared component library
│   └── src/
│       ├── components/           #   shadcn/ui components (re-exported)
│       ├── hooks/                #   Shared UI hooks
│       ├── lib/                  #   Utility functions
│       ├── chart.ts              #   Chart component
│       ├── separator.ts          #   Separator component
│       ├── skeleton.ts           #   Skeleton component
│       ├── styles.css            #   Base styles
│       └── index.ts              #   Barrel exports
│
├── addon-sdk/                    # @wealthfolio/addon-sdk — Addon development kit
│   └── src/
│       ├── data-types.ts         #   Shared data type definitions
│       ├── goal-progress.ts      #   Goal progress API
│       ├── host-api.ts           #   Host API for addons
│       ├── manifest.ts           #   Addon manifest types
│       ├── permissions.ts        #   Permission system
│       ├── query-keys.ts         #   Query key definitions for addons
│       ├── types.ts              #   Core addon types
│       ├── utils.ts              #   Utility functions
│       ├── version.ts            #   SDK version
│       └── index.ts              #   Barrel exports
│
└── addon-dev-tools/              # @wealthfolio/addon-dev-tools — Dev tooling
    ├── cli.js                    #   CLI for creating/managing addons
    ├── dev-server.js             #   Dev server for addon development
    ├── index.js                  #   Main entry
    ├── scaffold.js               #   Addon scaffolding
    └── templates/                #   Addon project templates
```

## addons/ (3 example addons)

```
addons/
├── goal-progress-tracker/        # Goal progress tracking addon
├── investment-fees-tracker/      # Investment fee tracking addon
└── swingfolio-addon/             # Swing trading addon
```

## Key Files Index

| File | Purpose |
|------|---------|
| `apps/frontend/src/main.tsx` | Frontend entry point — platform detection, addon loading, React render |
| `apps/frontend/src/routes.tsx` | All route definitions with BrowserRouter |
| `apps/frontend/src/App.tsx` | Root component with providers |
| `apps/frontend/src/adapters/index.ts` | Adapter barrel (swapped at build time) |
| `apps/frontend/src/adapters/tauri/core.ts` | Tauri `invoke()` wrapper with timeout |
| `apps/frontend/src/adapters/web/core.ts` | HTTP `fetch()` wrapper with COMMANDS map |
| `apps/frontend/src/adapters/shared/` | Platform-agnostic command wrappers (16 modules) |
| `apps/frontend/src/lib/query-keys.ts` | TanStack Query key registry |
| `apps/frontend/src/lib/schemas.ts` | Zod validation schemas |
| `apps/frontend/src/lib/is-tauri.ts` | Runtime Tauri detection |
| `apps/frontend/src/context/portfolio-sync-context.tsx` | Portfolio sync orchestration |
| `apps/tauri/src/lib.rs` | Tauri app setup + 70+ IPC command registration |
| `apps/tauri/src/context/providers.rs` | Service construction and DI wiring |
| `apps/tauri/src/context/registry.rs` | ServiceContext struct definition |
| `apps/tauri/src/secret_store.rs` | OS keyring secret storage |
| `apps/server/src/api.rs` | Axum router composition + middleware |
| `apps/server/src/main_lib.rs` | AppState construction + service wiring |
| `apps/server/src/config.rs` | Environment variable configuration |
| `apps/server/src/auth.rs` | JWT + Argon2id authentication |
| `apps/server/src/events.rs` | SSE EventBus for real-time updates |
| `crates/core/src/lib.rs` | Core crate public API |
| `crates/core/src/events/domain_event.rs` | DomainEvent enum (all event types) |
| `crates/core/src/errors.rs` | Core error types |
| `crates/core/src/portfolio/` | Portfolio domain (allocation, fire, holdings, income, performance, snapshot, valuation) |
| `crates/storage-sqlite/src/schema.rs` | Diesel database schema |
| `crates/storage-sqlite/src/db/` | Connection pool + write actor |
| `crates/storage-sqlite/migrations/` | 29 Diesel migrations |
| `crates/ai/src/chat.rs` | AI chat service with streaming |
| `crates/ai/src/tools/` | 15 AI tool implementations |
| `crates/connect/src/broker/service.rs` | Broker sync service |
| `crates/device-sync/src/crypto.rs` | E2EE crypto primitives |
| `package.json` | Root monorepo config (pnpm workspaces, scripts) |
| `Cargo.toml` | Rust workspace config (members, dependencies, lints) |
| `Dockerfile` | Multi-stage Docker build (Rust + Node) |
| `compose.yml` | Production Docker Compose |
| `pnpm-workspace.yaml` | Workspace package definitions |
| `tsconfig.base.json` | Shared TypeScript configuration |
| `AGENTS.md` | AI agent behavioral guide and playbooks |

## Size Metrics

- **Total source files:** ~1,238 (excluding node_modules, target, .git)
- **Total lines of code:** ~320K (TypeScript + Rust)
- **By component:**
  - Frontend (TS/TSX): 579 files, ~129K lines
  - Tauri app (Rust): 48 files, ~12K lines
  - Server app (Rust): 45 files, ~12K lines
  - Core crates (Rust): 307 files, ~134K lines
- **By language:**
  - TypeScript/TSX: ~161K lines
  - Rust: ~158K lines
- **Database migrations:** 29 migration directories
- **Tauri IPC commands:** 70+ registered commands
- **API endpoints:** 25+ Axum handler modules
- **AI tools:** 15 tool implementations
- **Shared adapter modules:** 16 platform-agnostic command wrappers

---

*Structure analysis: 2026-04-20*
