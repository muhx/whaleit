//! PostgreSQL storage implementation for sync.

pub mod app_sync;
pub mod engine_ports;
pub mod import_run;
pub mod platform;
pub mod state;

pub use app_sync::{PgAppSyncRepository, SyncLocalDataSummary, SyncTableRowCount};
pub use engine_ports::PgSyncEngineDbPorts;
pub use import_run::PgImportRunRepository;
pub use platform::PgPlatformRepository;
pub use state::PgBrokerSyncStateRepository;

// Type aliases for compatibility with SQLite storage API
pub type AppSyncRepository = PgAppSyncRepository;
pub type ImportRunRepository = PgImportRunRepository;
pub type PlatformRepository = PgPlatformRepository;
pub type BrokerSyncStateRepository = PgBrokerSyncStateRepository;
