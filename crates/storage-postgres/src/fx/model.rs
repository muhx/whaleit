//! Database models for FX quotes (shared with market_data QuoteDB).

use diesel::prelude::*;

/// Database read model for quotes (used by both FX and market_data modules).
#[derive(Queryable, QueryableByName, Identifiable, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::quotes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuoteDB {
    pub id: String,
    pub asset_id: String,
    pub day: String,
    pub source: String,
    pub open: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub close: String,
    pub adjclose: Option<String>,
    pub volume: Option<String>,
    pub currency: String,
    pub notes: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub timestamp: chrono::NaiveDateTime,
}

/// Database insert model for quotes.
#[derive(Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = crate::schema::quotes)]
pub struct NewQuoteDB {
    pub id: String,
    pub asset_id: String,
    pub day: String,
    pub source: String,
    pub open: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub close: String,
    pub adjclose: Option<String>,
    pub volume: Option<String>,
    pub currency: String,
    pub notes: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub timestamp: chrono::NaiveDateTime,
}

impl From<QuoteDB> for whaleit_core::quotes::Quote {
    fn from(db: QuoteDB) -> Self {
        use chrono::{TimeZone, Utc};
        use rust_decimal::Decimal;
        use std::str::FromStr;

        Self {
            id: db.id,
            asset_id: db.asset_id,
            timestamp: Utc.from_utc_datetime(&db.timestamp),
            open: db
                .open
                .as_deref()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO),
            high: db
                .high
                .as_deref()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO),
            low: db
                .low
                .as_deref()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO),
            close: Decimal::from_str(&db.close).unwrap_or(Decimal::ZERO),
            adjclose: db
                .adjclose
                .as_deref()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO),
            volume: db
                .volume
                .as_deref()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO),
            currency: db.currency,
            data_source: db.source,
            created_at: Utc.from_utc_datetime(&db.created_at),
            notes: db.notes,
        }
    }
}

impl From<&whaleit_core::quotes::Quote> for NewQuoteDB {
    fn from(quote: &whaleit_core::quotes::Quote) -> Self {
        use chrono::Utc;

        Self {
            id: quote.id.clone(),
            asset_id: quote.asset_id.clone(),
            day: quote.timestamp.format("%Y-%m-%d").to_string(),
            source: quote.data_source.clone(),
            open: Some(quote.open.to_string()),
            high: Some(quote.high.to_string()),
            low: Some(quote.low.to_string()),
            close: quote.close.to_string(),
            adjclose: Some(quote.adjclose.to_string()),
            volume: Some(quote.volume.to_string()),
            currency: quote.currency.clone(),
            notes: quote.notes.clone(),
            created_at: Utc::now().naive_utc(),
            timestamp: quote.timestamp.naive_utc(),
        }
    }
}
