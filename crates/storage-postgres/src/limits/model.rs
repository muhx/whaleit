//! Database models for contribution limits.

use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable, AsChangeset, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::contribution_limits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContributionLimitDB {
    pub id: String,
    pub group_name: String,
    pub contribution_year: i32,
    pub limit_amount: f64,
    pub account_ids: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::contribution_limits)]
pub struct NewContributionLimitDB {
    pub id: String,
    pub group_name: String,
    pub contribution_year: i32,
    pub limit_amount: f64,
    pub account_ids: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl From<ContributionLimitDB> for whaleit_core::limits::ContributionLimit {
    fn from(db: ContributionLimitDB) -> Self {
        Self {
            id: db.id,
            group_name: db.group_name,
            contribution_year: db.contribution_year,
            limit_amount: db.limit_amount,
            account_ids: db.account_ids,
            created_at: db.created_at,
            updated_at: db.updated_at,
            start_date: db.start_date,
            end_date: db.end_date,
        }
    }
}

impl From<whaleit_core::limits::NewContributionLimit> for NewContributionLimitDB {
    fn from(domain: whaleit_core::limits::NewContributionLimit) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            group_name: domain.group_name,
            contribution_year: domain.contribution_year,
            limit_amount: domain.limit_amount,
            account_ids: domain.account_ids,
            created_at: now,
            updated_at: now,
            start_date: domain.start_date,
            end_date: domain.end_date,
        }
    }
}
