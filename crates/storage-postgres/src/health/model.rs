//! Database models for health issue dismissals.

use diesel::prelude::*;

/// Database model for health issue dismissals
#[derive(Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::health_issue_dismissals)]
#[diesel(primary_key(issue_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HealthIssueDismissalDB {
    pub issue_id: String,
    pub dismissed_at: chrono::NaiveDateTime,
    pub data_hash: String,
}

// Conversion to domain model
impl From<HealthIssueDismissalDB> for whaleit_core::health::IssueDismissal {
    fn from(db: HealthIssueDismissalDB) -> Self {
        Self {
            issue_id: db.issue_id,
            dismissed_at: chrono::DateTime::from_naive_utc_and_offset(db.dismissed_at, chrono::Utc),
            data_hash: db.data_hash,
        }
    }
}

impl From<whaleit_core::health::IssueDismissal> for HealthIssueDismissalDB {
    fn from(domain: whaleit_core::health::IssueDismissal) -> Self {
        Self {
            issue_id: domain.issue_id,
            dismissed_at: domain.dismissed_at.naive_utc(),
            data_hash: domain.data_hash,
        }
    }
}
