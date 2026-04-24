//! PostgreSQL assets repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use super::model::{AssetDB, InsertableAssetDB};
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::assets;
use crate::schema::assets::dsl::*;
use whaleit_core::assets::{Asset, AssetRepositoryTrait, NewAsset, UpdateAssetProfile};
use whaleit_core::errors::Result;

pub struct PgAssetRepository {
    pool: Arc<PgPool>,
}

impl PgAssetRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepositoryTrait for PgAssetRepository {
    async fn create(&self, new_asset: NewAsset) -> Result<Asset> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let now = chrono::Utc::now().naive_utc();
        let id_val = new_asset
            .id
            .clone()
            .unwrap_or_else(|| Uuid::now_v7().to_string());

        let asset_kind = new_asset.kind.as_db_str().to_string();
        let asset_quote_mode = new_asset.quote_mode.as_db_str().to_string();
        let asset_instrument_type = new_asset
            .instrument_type
            .as_ref()
            .map(|t| t.as_db_str().to_string());
        let asset_provider_config = new_asset
            .provider_config
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());
        let asset_metadata = new_asset
            .metadata
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());

        diesel::insert_into(assets::table)
            .values(InsertableAssetDB {
                id: Some(id_val.clone()),
                kind: asset_kind,
                name: new_asset.name,
                display_code: new_asset.display_code,
                notes: new_asset.notes,
                metadata: asset_metadata,
                is_active: new_asset.is_active,
                quote_mode: asset_quote_mode,
                quote_ccy: new_asset.quote_ccy,
                instrument_type: asset_instrument_type,
                instrument_symbol: new_asset.instrument_symbol,
                instrument_exchange_mic: new_asset.instrument_exchange_mic,
                provider_config: asset_provider_config,
                created_at: now,
                updated_at: now,
            })
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db = assets::table
            .find(&id_val)
            .first::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(db.into())
    }

    async fn create_batch(&self, new_assets: Vec<NewAsset>) -> Result<Vec<Asset>> {
        let mut results = Vec::new();
        for asset in new_assets {
            results.push(self.create(asset).await?);
        }
        Ok(results)
    }

    async fn update_profile(
        &self,
        asset_id_param: &str,
        _payload: UpdateAssetProfile,
    ) -> Result<Asset> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(assets::table.find(asset_id_param))
            .set(updated_at.eq(chrono::Utc::now().naive_utc()))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db = assets::table
            .find(asset_id_param)
            .first::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn update_quote_mode(&self, asset_id_param: &str, mode: &str) -> Result<Asset> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(assets::table.find(asset_id_param))
            .set((
                quote_mode.eq(mode),
                updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db = assets::table
            .find(asset_id_param)
            .first::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn get_by_id(&self, asset_id_param: &str) -> Result<Asset> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let db = assets::table
            .find(asset_id_param)
            .first::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn list(&self) -> Result<Vec<Asset>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = assets::table
            .load::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Asset::from).collect())
    }

    async fn list_by_asset_ids(&self, asset_ids_param: &[String]) -> Result<Vec<Asset>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = assets::table
            .filter(id.eq_any(asset_ids_param))
            .load::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Asset::from).collect())
    }

    async fn delete(&self, asset_id_param: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::delete(assets::table.find(asset_id_param))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn search_by_symbol(&self, query: &str) -> Result<Vec<Asset>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let pattern = format!("%{}%", query);
        let results = assets::table
            .filter(instrument_symbol.like(&pattern))
            .load::<AssetDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Asset::from).collect())
    }

    async fn find_by_instrument_key(&self, key: &str) -> Result<Option<Asset>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let result = assets::table
            .filter(instrument_key.eq(key))
            .first::<AssetDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;
        Ok(result.map(Asset::from))
    }

    async fn cleanup_legacy_metadata(&self, _asset_id: &str) -> Result<()> {
        Ok(())
    }

    async fn deactivate(&self, asset_id_param: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(assets::table.find(asset_id_param))
            .set(is_active.eq(false))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn reactivate(&self, asset_id_param: &str) -> Result<()> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(assets::table.find(asset_id_param))
            .set(is_active.eq(true))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(())
    }

    async fn copy_user_metadata(&self, _source_id: &str, _target_id: &str) -> Result<()> {
        Ok(())
    }

    async fn deactivate_orphaned_investments(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}
