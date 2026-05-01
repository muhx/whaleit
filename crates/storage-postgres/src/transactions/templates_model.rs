//! Diesel database model for CSV import templates (D-16/17/18).

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use whaleit_core::transactions::{
    NewTransactionTemplate, TransactionTemplate, TransactionTemplateUpdate,
};

use crate::schema::transaction_csv_templates;

/// Diesel model for the `transaction_csv_templates` table.
#[derive(Debug, Clone, Queryable, Identifiable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = transaction_csv_templates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionTemplateDB {
    pub id: String,
    pub name: String,
    /// JSONB column — serialized CsvFieldMapping.
    pub mapping: JsonValue,
    /// TEXT[] column — ordered CSV header names.
    pub header_signature: Vec<Option<String>>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// ── From<NewTransactionTemplate> for TransactionTemplateDB ──────────────────

impl From<NewTransactionTemplate> for TransactionTemplateDB {
    fn from(domain: NewTransactionTemplate) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: Uuid::now_v7().to_string(),
            name: domain.name,
            // CsvFieldMapping is a derive(Serialize) struct of String/Option<String>
            // fields — serialization is infallible in practice.
            mapping: serde_json::to_value(&domain.mapping)
                .expect("CsvFieldMapping serialization is infallible"),
            header_signature: domain.header_signature.into_iter().map(Some).collect(),
            created_at: now,
            updated_at: now,
        }
    }
}

// ── From<TransactionTemplateDB> for TransactionTemplate ─────────────────────

impl From<TransactionTemplateDB> for TransactionTemplate {
    fn from(db: TransactionTemplateDB) -> Self {
        Self {
            id: db.id,
            name: db.name,
            // The JSONB value is schema-controlled — only ever written by the
            // From<NewTransactionTemplate> path above. Deserialization failure
            // here means the JSONB was hand-edited or schema drifted.
            mapping: serde_json::from_value(db.mapping)
                .expect("transaction_csv_templates.mapping must round-trip CsvFieldMapping"),
            header_signature: db.header_signature.into_iter().flatten().collect(),
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

// ── Changeset for partial updates ───────────────────────────────────────────

/// Partial changeset for `diesel::update().set(...)`.
#[derive(Debug, AsChangeset)]
#[diesel(table_name = transaction_csv_templates)]
#[diesel(treat_none_as_null = false)]
pub struct TransactionTemplateChangesetDB {
    pub name: Option<String>,
    pub mapping: Option<JsonValue>,
    pub header_signature: Option<Vec<Option<String>>>,
    pub updated_at: NaiveDateTime,
}

impl From<&TransactionTemplateUpdate> for TransactionTemplateChangesetDB {
    fn from(upd: &TransactionTemplateUpdate) -> Self {
        Self {
            name: upd.name.clone(),
            mapping: upd.mapping.as_ref().map(|m| {
                serde_json::to_value(m).expect("CsvFieldMapping serialization is infallible")
            }),
            header_signature: upd
                .header_signature
                .as_ref()
                .map(|hs| hs.iter().cloned().map(Some).collect()),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}
