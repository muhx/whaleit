//! Database models for market data.

use diesel::prelude::*;
use whaleit_core::quotes::{MarketDataProviderSetting, ProviderCapabilities};

// Re-export QuoteDB from fx module (shared quotes table)
pub use crate::fx::QuoteDB;

#[derive(Queryable, Identifiable, Selectable, AsChangeset, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::quote_sync_state)]
#[diesel(primary_key(asset_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuoteSyncStateDB {
    pub asset_id: String,
    pub position_closed_date: Option<String>,
    pub last_synced_at: Option<chrono::NaiveDateTime>,
    pub data_source: String,
    pub sync_priority: i32,
    pub error_count: i32,
    pub last_error: Option<String>,
    pub profile_enriched_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct QuoteSyncStateUpdateDB {
    pub sync_priority: Option<i32>,
    pub error_count: Option<i32>,
    pub last_error: Option<Option<String>>,
}

#[derive(Queryable, Identifiable, Selectable, AsChangeset, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::market_data_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MarketDataProviderSettingDB {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub logo_filename: Option<String>,
    pub last_synced_at: Option<chrono::NaiveDateTime>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub provider_type: String,
    pub config: Option<String>,
}

impl From<MarketDataProviderSettingDB> for MarketDataProviderSetting {
    fn from(db: MarketDataProviderSettingDB) -> Self {
        let capabilities = ProviderCapabilities::for_provider(&db.id);
        Self {
            id: db.id,
            name: db.name,
            description: db.description,
            url: db.url,
            priority: db.priority,
            enabled: db.enabled,
            logo_filename: db.logo_filename,
            last_synced_at: db.last_synced_at.map(|dt| dt.to_string()),
            last_sync_status: db.last_sync_status,
            last_sync_error: db.last_sync_error,
            capabilities,
            provider_type: Some(db.provider_type),
        }
    }
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::market_data_providers)]
pub struct UpdateMarketDataProviderSettingDB {
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub config: Option<String>,
}
