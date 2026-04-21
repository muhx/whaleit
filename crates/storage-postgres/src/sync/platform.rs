//! PostgreSQL platform repository.
//!
//! Implements `PlatformRepositoryTrait` from `whaleit-connect`.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use crate::db::PgPool;
use crate::errors::IntoCore;
use crate::schema::platforms;

use whaleit_connect::broker::PlatformRepositoryTrait;
use whaleit_connect::Platform as ConnectPlatform;
use whaleit_core::errors::Result;

/// Local domain model for platforms (mirrors ConnectPlatform fields).
#[derive(Debug, Clone)]
struct Platform {
    id: String,
    name: Option<String>,
    url: String,
    external_id: Option<String>,
    kind: String,
    website_url: Option<String>,
    logo_url: Option<String>,
}

impl From<Platform> for ConnectPlatform {
    fn from(value: Platform) -> Self {
        Self {
            id: value.id,
            name: value.name,
            url: value.url,
            external_id: value.external_id,
            kind: value.kind,
            website_url: value.website_url,
            logo_url: value.logo_url,
        }
    }
}

impl From<ConnectPlatform> for Platform {
    fn from(value: ConnectPlatform) -> Self {
        Self {
            id: value.id,
            name: value.name,
            url: value.url,
            external_id: value.external_id,
            kind: value.kind,
            website_url: value.website_url,
            logo_url: value.logo_url,
        }
    }
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = platforms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PlatformDB {
    id: String,
    name: Option<String>,
    url: String,
    external_id: Option<String>,
    kind: String,
    website_url: Option<String>,
    logo_url: Option<String>,
}

impl From<PlatformDB> for Platform {
    fn from(db: PlatformDB) -> Self {
        Self {
            id: db.id,
            name: db.name,
            url: db.url,
            external_id: db.external_id,
            kind: db.kind,
            website_url: db.website_url,
            logo_url: db.logo_url,
        }
    }
}

pub struct PgPlatformRepository {
    pool: Arc<PgPool>,
}

impl PgPlatformRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlatformRepositoryTrait for PgPlatformRepository {
    fn get_by_id(&self, platform_id: &str) -> Result<Option<ConnectPlatform>> {
        let pool = self.pool.clone();
        let platform_id = platform_id.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                let result = platforms::table
                    .find(&platform_id)
                    .first::<PlatformDB>(&mut conn)
                    .await
                    .optional()
                    .into_core()?;

                Ok(result.map(Platform::from).map(ConnectPlatform::from))
            })
        })
    }

    fn get_by_external_id(&self, ext_id: &str) -> Result<Option<ConnectPlatform>> {
        let pool = self.pool.clone();
        let ext_id = ext_id.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                let result = platforms::table
                    .filter(platforms::external_id.eq(&ext_id))
                    .first::<PlatformDB>(&mut conn)
                    .await
                    .optional()
                    .into_core()?;

                Ok(result.map(Platform::from).map(ConnectPlatform::from))
            })
        })
    }

    fn list(&self) -> Result<Vec<ConnectPlatform>> {
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
                })?;

                let results = platforms::table
                    .order(platforms::name.asc())
                    .load::<PlatformDB>(&mut conn)
                    .await
                    .into_core()?;

                Ok(results
                    .into_iter()
                    .map(Platform::from)
                    .map(ConnectPlatform::from)
                    .collect())
            })
        })
    }

    async fn upsert(&self, platform: ConnectPlatform) -> Result<ConnectPlatform> {
        let p: Platform = platform.into();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        diesel::insert_into(platforms::table)
            .values((
                platforms::id.eq(&p.id),
                platforms::name.eq(&p.name),
                platforms::url.eq(&p.url),
                platforms::external_id.eq(&p.external_id),
                platforms::kind.eq(&p.kind),
                platforms::website_url.eq(&p.website_url),
                platforms::logo_url.eq(&p.logo_url),
            ))
            .on_conflict(platforms::id)
            .do_update()
            .set((
                platforms::name.eq(&p.name),
                platforms::url.eq(&p.url),
                platforms::external_id.eq(&p.external_id),
            ))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(ConnectPlatform::from(p))
    }

    async fn delete(&self, platform_id: &str) -> Result<usize> {
        let id = platform_id.to_string();

        let mut conn = self.pool.get().await.map_err(|e| {
            whaleit_core::errors::DatabaseError::ConnectionFailed(e.to_string())
        })?;

        let affected = diesel::delete(platforms::table.find(&id))
            .execute(&mut conn)
            .await
            .into_core()?;

        Ok(affected)
    }
}
