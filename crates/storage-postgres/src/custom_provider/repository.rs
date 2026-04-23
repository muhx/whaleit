//! PostgreSQL custom provider repository.
//!
//! Implements `CustomProviderRepository` from `whaleit-core`.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use super::model::CustomProviderDB;
use crate::db::PgPool;
use crate::errors::IntoCore;
use crate::schema::market_data_custom_providers as custom_providers;

use whaleit_core::custom_provider::{
    CustomProviderRepository, CustomProviderSource, CustomProviderWithSources, NewCustomProvider,
    NewCustomProviderSource, UpdateCustomProvider,
};
use whaleit_core::errors::Result;

/// JSON wrapper stored in custom_providers.config
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ProviderConfig {
    sources: Vec<NewCustomProviderSource>,
}

fn parse_sources(config_json: Option<&str>, provider_code: &str) -> Vec<CustomProviderSource> {
    let config: ProviderConfig = match config_json {
        Some(s) => match serde_json::from_str(s) {
            Ok(c) => c,
            Err(e) => {
                log::warn!(
                    "Failed to parse config JSON for provider '{}': {}",
                    provider_code,
                    e
                );
                ProviderConfig::default()
            }
        },
        None => ProviderConfig::default(),
    };

    config
        .sources
        .into_iter()
        .map(|s| CustomProviderSource {
            id: format!("{}:{}", provider_code, s.kind),
            provider_id: provider_code.to_string(),
            kind: s.kind,
            format: s.format,
            url: s.url,
            price_path: s.price_path,
            date_path: s.date_path,
            date_format: s.date_format,
            currency_path: s.currency_path,
            factor: s.factor,
            invert: s.invert,
            locale: s.locale,
            headers: s.headers,
            high_path: s.high_path,
            low_path: s.low_path,
            volume_path: s.volume_path,
            default_price: s.default_price,
            date_timezone: s.date_timezone,
        })
        .collect()
}

fn sources_to_config_json(sources: &[NewCustomProviderSource]) -> String {
    let config = ProviderConfig {
        sources: sources.to_vec(),
    };
    serde_json::to_string(&config).unwrap_or_else(|e| {
        log::warn!("Failed to serialize provider config: {}", e);
        r#"{"sources":[]}"#.to_string()
    })
}

fn db_to_domain(row: CustomProviderDB) -> CustomProviderWithSources {
    let sources = parse_sources(row.config.as_deref(), &row.code);
    CustomProviderWithSources {
        id: row.code,
        name: row.name,
        description: row.description,
        enabled: row.enabled,
        priority: row.priority,
        sources,
    }
}

pub struct PgCustomProviderRepository {
    pool: Arc<PgPool>,
}

impl PgCustomProviderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomProviderRepository for PgCustomProviderRepository {
    fn get_all(&self) -> Result<Vec<CustomProviderWithSources>> {
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                let rows: Vec<CustomProviderDB> = custom_providers::table
                    .order(custom_providers::priority.asc())
                    .select(CustomProviderDB::as_select())
                    .load(&mut conn)
                    .await
                    .into_core()?;

                Ok(rows.into_iter().map(db_to_domain).collect())
            })
        })
    }

    fn get_source_by_kind(
        &self,
        provider_code: &str,
        kind: &str,
    ) -> Result<Option<CustomProviderSource>> {
        let pool = self.pool.clone();
        let provider_code = provider_code.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                let row: Option<CustomProviderDB> = custom_providers::table
                    .filter(custom_providers::code.eq(&provider_code))
                    .select(CustomProviderDB::as_select())
                    .first(&mut conn)
                    .await
                    .optional()
                    .into_core()?;

                match row {
                    Some(r) if r.enabled => {
                        let sources = parse_sources(r.config.as_deref(), &provider_code);
                        Ok(sources.into_iter().find(|s| s.kind == kind))
                    }
                    _ => Ok(None),
                }
            })
        })
    }

    async fn create(&self, payload: &NewCustomProvider) -> Result<CustomProviderWithSources> {
        let code = payload.code.clone();
        let name = payload.name.clone();
        let description = payload.description.clone().unwrap_or_default();
        let config_json = sources_to_config_json(&payload.sources);
        let now = chrono::Utc::now().naive_utc();

        let row = CustomProviderDB {
            id: uuid::Uuid::new_v4().to_string(),
            code: code.clone(),
            name,
            description,
            enabled: true,
            priority: payload.priority.unwrap_or(50),
            config: Some(config_json),
            created_at: now,
            updated_at: now,
        };

        let mut conn =
            self.pool.get().await.map_err(|e| {
                whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
            })?;

        diesel::insert_into(custom_providers::table)
            .values(&row)
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(db_to_domain(row))
    }

    async fn update(
        &self,
        provider_code: &str,
        payload: &UpdateCustomProvider,
    ) -> Result<CustomProviderWithSources> {
        let code = provider_code.to_string();
        let payload = payload.clone();
        let now = chrono::Utc::now().naive_utc();

        let mut conn =
            self.pool.get().await.map_err(|e| {
                whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
            })?;

        let existing: CustomProviderDB = custom_providers::table
            .filter(custom_providers::code.eq(&code))
            .select(CustomProviderDB::as_select())
            .first(&mut conn)
            .await
            .into_core()?;

        let new_name = payload.name.unwrap_or(existing.name);
        let new_desc = payload.description.unwrap_or(existing.description);
        let new_enabled = payload.enabled.unwrap_or(existing.enabled);
        let new_priority = payload.priority.unwrap_or(existing.priority);
        let new_config = match &payload.sources {
            Some(sources) => Some(sources_to_config_json(sources)),
            None => existing.config,
        };

        diesel::update(custom_providers::table.filter(custom_providers::code.eq(&code)))
            .set((
                custom_providers::name.eq(&new_name),
                custom_providers::description.eq(&new_desc),
                custom_providers::enabled.eq(new_enabled),
                custom_providers::priority.eq(new_priority),
                custom_providers::config.eq(&new_config),
                custom_providers::updated_at.eq(&now),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        let updated = CustomProviderDB {
            id: existing.id,
            code,
            name: new_name,
            description: new_desc,
            enabled: new_enabled,
            priority: new_priority,
            config: new_config,
            created_at: existing.created_at,
            updated_at: now,
        };

        Ok(db_to_domain(updated))
    }

    async fn delete(&self, provider_code: &str) -> Result<()> {
        let code = provider_code.to_string();

        let mut conn =
            self.pool.get().await.map_err(|e| {
                whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
            })?;

        diesel::delete(custom_providers::table.filter(custom_providers::code.eq(&code)))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(())
    }

    fn get_asset_count_for_provider(&self, provider_code: &str) -> Result<i64> {
        // In PG mode, we use a simplified count query since json_extract is SQLite-specific
        let pool = self.pool.clone();
        let provider_code = provider_code.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                // Use PostgreSQL's JSON operators instead of SQLite's json_extract
                use diesel::sql_types::{BigInt, Text};

                #[derive(QueryableByName)]
                struct CountRow {
                    #[diesel(sql_type = BigInt)]
                    cnt: i64,
                }

                let row: CountRow = diesel::sql_query(
                    "SELECT COUNT(*) as cnt FROM assets WHERE \
                     provider_config IS NOT NULL AND provider_config != '' \
                     AND provider_config::jsonb->>'custom_provider_code' = $1",
                )
                .bind::<Text, _>(&provider_code)
                .get_result(&mut conn)
                .await
                .into_core()?;

                Ok(row.cnt)
            })
        })
    }
}
