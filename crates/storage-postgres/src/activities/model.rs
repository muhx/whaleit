//! Database models for activities.
//!
//! Activity model for PostgreSQL. Most scalar fields are Text (matching SQLite pattern).
//! Timestamp columns use NaiveDateTime to match the Timestamp schema type.

use chrono::NaiveDateTime;
use diesel::prelude::*;

use whaleit_core::activities::{Activity, ActivityStatus};

/// Database model for activities
#[derive(
    Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone,
)]
#[diesel(table_name = crate::schema::activities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ActivityDB {
    pub id: String,
    pub account_id: String,
    pub asset_id: Option<String>,
    pub activity_type: String,
    pub activity_type_override: Option<String>,
    pub source_type: Option<String>,
    pub subtype: Option<String>,
    pub status: String,
    pub activity_date: String,
    pub settlement_date: Option<String>,
    pub quantity: Option<String>,
    pub unit_price: Option<String>,
    pub amount: Option<String>,
    pub fee: Option<String>,
    pub currency: String,
    pub fx_rate: Option<String>,
    pub notes: Option<String>,
    pub metadata: Option<String>,
    pub source_system: Option<String>,
    pub source_record_id: Option<String>,
    pub source_group_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub import_run_id: Option<String>,
    pub is_user_modified: bool,
    pub needs_review: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<ActivityDB> for Activity {
    fn from(db: ActivityDB) -> Self {
        use chrono::{DateTime, Utc};
        use whaleit_core::activities::parse_decimal_string_tolerant;

        let status = match db.status.as_str() {
            "POSTED" => ActivityStatus::Posted,
            "PENDING" => ActivityStatus::Pending,
            "DRAFT" => ActivityStatus::Draft,
            "VOID" => ActivityStatus::Void,
            _ => ActivityStatus::Posted,
        };

        let metadata = db
            .metadata
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        Self {
            id: db.id,
            account_id: db.account_id,
            asset_id: db.asset_id,
            activity_type: db.activity_type,
            activity_type_override: db.activity_type_override,
            source_type: db.source_type,
            subtype: db.subtype,
            status,
            activity_date: DateTime::parse_from_rfc3339(&db.activity_date)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            settlement_date: db.settlement_date.as_ref().and_then(|s| {
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            quantity: db.quantity.as_ref().map(|s| parse_decimal_string_tolerant(s, "quantity")),
            unit_price: db.unit_price.as_ref().map(|s| parse_decimal_string_tolerant(s, "unit_price")),
            amount: db.amount.as_ref().map(|s| parse_decimal_string_tolerant(s, "amount")),
            fee: db.fee.as_ref().map(|s| parse_decimal_string_tolerant(s, "fee")),
            currency: db.currency,
            fx_rate: db.fx_rate.as_ref().map(|s| parse_decimal_string_tolerant(s, "fx_rate")),
            notes: db.notes,
            metadata,
            source_system: db.source_system,
            source_record_id: db.source_record_id,
            source_group_id: db.source_group_id,
            idempotency_key: db.idempotency_key,
            import_run_id: db.import_run_id,
            is_user_modified: db.is_user_modified,
            needs_review: db.needs_review,
            created_at: chrono::DateTime::from_naive_utc_and_offset(db.created_at, chrono::Utc),
            updated_at: chrono::DateTime::from_naive_utc_and_offset(db.updated_at, chrono::Utc),
        }
    }
}

/// Import account template DB
#[derive(
    Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone,
)]
#[diesel(table_name = crate::schema::import_account_templates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ImportAccountTemplateDB {
    pub id: String,
    pub account_id: String,
    pub context_kind: String,
    pub source_system: String,
    pub template_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Import template DB
#[derive(
    Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone,
)]
#[diesel(table_name = crate::schema::import_templates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ImportTemplateDB {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub kind: String,
    pub source_system: String,
    pub config_version: i32,
    pub config: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
