//! Database model for assets.

use diesel::prelude::*;

/// Database read model for assets
#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug, Clone, Default)]
#[diesel(table_name = crate::schema::assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetDB {
    pub id: String,
    pub kind: String,
    pub name: Option<String>,
    pub display_code: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<String>,
    pub is_active: bool,
    pub quote_mode: String,
    pub quote_ccy: String,
    pub instrument_type: Option<String>,
    pub instrument_symbol: Option<String>,
    pub instrument_exchange_mic: Option<String>,
    pub instrument_key: Option<String>,
    pub provider_config: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

/// Database write model for assets
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = crate::schema::assets)]
pub struct InsertableAssetDB {
    pub id: Option<String>,
    pub kind: String,
    pub name: Option<String>,
    pub display_code: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<String>,
    pub is_active: bool,
    pub quote_mode: String,
    pub quote_ccy: String,
    pub instrument_type: Option<String>,
    pub instrument_symbol: Option<String>,
    pub instrument_exchange_mic: Option<String>,
    pub provider_config: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<AssetDB> for whaleit_core::assets::Asset {
    fn from(db: AssetDB) -> Self {
        use whaleit_core::assets::{AssetKind, InstrumentType, QuoteMode};
        let kind = AssetKind::from_db_str(&db.kind).unwrap_or_default();
        let quote_mode = match db.quote_mode.as_str() {
            "MANUAL" => QuoteMode::Manual,
            _ => QuoteMode::Market,
        };
        let instrument_type = db
            .instrument_type
            .as_deref()
            .and_then(InstrumentType::from_db_str);
        let metadata = db
            .metadata
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());
        let provider_config = db
            .provider_config
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        Self {
            id: db.id,
            kind,
            name: db.name,
            display_code: db.display_code,
            notes: db.notes,
            metadata,
            is_active: db.is_active,
            quote_mode,
            quote_ccy: db.quote_ccy,
            instrument_type,
            instrument_symbol: db.instrument_symbol,
            instrument_exchange_mic: db.instrument_exchange_mic,
            instrument_key: db.instrument_key,
            provider_config,
            exchange_name: None,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}
