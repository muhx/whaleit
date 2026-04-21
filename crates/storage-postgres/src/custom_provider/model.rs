//! Database model for custom providers.

use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable, Insertable, AsChangeset, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::market_data_custom_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CustomProviderDB {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub priority: i32,
    pub config: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
