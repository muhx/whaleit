//! Database model for application settings.

use diesel::prelude::*;

/// Database model for app settings key-value pairs
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = crate::schema::app_settings)]
pub struct AppSettingDB {
    pub setting_key: String,
    pub setting_value: String,
}
