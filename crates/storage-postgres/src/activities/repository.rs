//! PostgreSQL activities repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::model::ActivityDB;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::activities;
use crate::schema::activities::dsl::*;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use whaleit_core::activities::{
    Activity, ActivityBulkMutationResult,
    ActivityRepositoryTrait, ActivitySearchResponse, ActivitySearchResponseMeta,
    ActivityUpdate as CoreActivityUpdate, BulkUpsertResult, ImportMapping,
    ImportTemplate as CoreImportTemplate, NewActivity,
};
use whaleit_core::limits::ContributionActivity;
use whaleit_core::Result;

pub struct PgActivityRepository {
    pool: Arc<PgPool>,
}

impl PgActivityRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActivityRepositoryTrait for PgActivityRepository {
    async fn get_activity(&self, activity_id_param: &str) -> Result<Activity> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let db = activities::table
            .find(activity_id_param)
            .first::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn get_activities(&self) -> Result<Vec<Activity>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = activities::table
            .load::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Activity::from).collect())
    }

    async fn get_activities_by_account_id(&self, account_id_param: &str) -> Result<Vec<Activity>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = activities::table
            .filter(account_id.eq(account_id_param))
            .load::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Activity::from).collect())
    }

    async fn get_activities_by_account_ids(
        &self,
        account_ids_param: &[String],
    ) -> Result<Vec<Activity>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = activities::table
            .filter(account_id.eq_any(account_ids_param))
            .load::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Activity::from).collect())
    }

    async fn get_trading_activities(&self) -> Result<Vec<Activity>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = activities::table
            .filter(activity_type.eq_any(vec!["BUY", "SELL"]))
            .load::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Activity::from).collect())
    }

    async fn get_income_activities(&self) -> Result<Vec<Activity>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = activities::table
            .filter(activity_type.eq_any(vec!["DIVIDEND", "INTEREST"]))
            .load::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Activity::from).collect())
    }

    async fn get_contribution_activities(
        &self,
        account_ids_param: &[String],
        _start_utc: DateTime<Utc>,
        _end_exclusive_utc: DateTime<Utc>,
    ) -> Result<Vec<ContributionActivity>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = activities::table
            .filter(account_id.eq_any(account_ids_param))
            .filter(activity_type.eq_any(vec!["DEPOSIT", "TRANSFER_IN", "TRANSFER_OUT", "CREDIT"]))
            .load::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results
            .into_iter()
            .map(|db| ContributionActivity {
                account_id: db.account_id,
                activity_type: db.activity_type,
                activity_instant: chrono::DateTime::parse_from_rfc3339(&db.activity_date)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                amount: db
                    .amount
                    .as_ref()
                    .map(|s| whaleit_core::activities::parse_decimal_string_tolerant(s, "amount")),
                currency: db.currency,
                metadata: None,
                source_group_id: db.source_group_id,
            })
            .collect())
    }

    #[allow(clippy::too_many_arguments)]
    async fn search_activities(
        &self,
        _page: i64,
        _page_size: i64,
        _account_id_filter: Option<Vec<String>>,
        _activity_type_filter: Option<Vec<String>>,
        _asset_id_keyword: Option<String>,
        _sort: Option<whaleit_core::activities::Sort>,
        _needs_review_filter: Option<bool>,
        _date_from: Option<NaiveDate>,
        _date_to: Option<NaiveDate>,
        _instrument_type_filter: Option<Vec<String>>,
    ) -> Result<ActivitySearchResponse> {
        // Simplified implementation: returns empty results.
        // Full implementation requires JOINs with assets/accounts tables for ActivityDetails.
        Ok(ActivitySearchResponse {
            data: vec![],
            meta: ActivitySearchResponseMeta { total_row_count: 0 },
        })
    }

    async fn create_activity(&self, new_activity: NewActivity) -> Result<Activity> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let id_val = Uuid::now_v7().to_string();
        let now = chrono::Utc::now().naive_utc();

        let asset_id_val = new_activity.get_symbol_id().map(|s| s.to_string());

        let status_str = new_activity
            .status
            .as_ref()
            .map(|s| match s {
                whaleit_core::activities::ActivityStatus::Posted => "POSTED",
                whaleit_core::activities::ActivityStatus::Pending => "PENDING",
                whaleit_core::activities::ActivityStatus::Draft => "DRAFT",
                whaleit_core::activities::ActivityStatus::Void => "VOID",
            })
            .unwrap_or("POSTED")
            .to_string();

        diesel::insert_into(activities::table)
            .values((
                activities::id.eq(&id_val),
                activities::account_id.eq(&new_activity.account_id),
                activities::asset_id.eq(asset_id_val),
                activities::activity_type.eq(&new_activity.activity_type),
                activities::subtype.eq(&new_activity.subtype),
                activities::status.eq(&status_str),
                activities::activity_date.eq(&new_activity.activity_date),
                activities::currency.eq(&new_activity.currency),
                activities::quantity.eq(new_activity.quantity.map(|d| d.to_string())),
                activities::unit_price.eq(new_activity.unit_price.map(|d| d.to_string())),
                activities::amount.eq(new_activity.amount.map(|d| d.to_string())),
                activities::fee.eq(new_activity.fee.map(|d| d.to_string())),
                activities::fx_rate.eq(new_activity.fx_rate.map(|d| d.to_string())),
                activities::notes.eq(&new_activity.notes),
                activities::metadata.eq(&new_activity.metadata),
                activities::source_system.eq(&new_activity.source_system),
                activities::source_record_id.eq(&new_activity.source_record_id),
                activities::source_group_id.eq(&new_activity.source_group_id),
                activities::idempotency_key.eq(&new_activity.idempotency_key),
                activities::is_user_modified.eq(false),
                activities::needs_review.eq(new_activity.needs_review.unwrap_or(false)),
                activities::created_at.eq(now),
                activities::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db = activities::table
            .find(&id_val)
            .first::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(db.into())
    }

    async fn update_activity(&self, _activity_update: CoreActivityUpdate) -> Result<Activity> {
        // Simplified: would need field-by-field update logic matching SQLite impl
        Err(whaleit_core::errors::Error::Unexpected(
            "update_activity not yet fully implemented for PG".to_string(),
        ))
    }

    async fn delete_activity(&self, activity_id_param: String) -> Result<Activity> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let db = activities::table
            .find(&activity_id_param)
            .first::<ActivityDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        diesel::delete(activities::table.find(&activity_id_param))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(db.into())
    }

    async fn bulk_mutate_activities(
        &self,
        creates: Vec<NewActivity>,
        _updates: Vec<CoreActivityUpdate>,
        delete_ids: Vec<String>,
    ) -> Result<ActivityBulkMutationResult> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let mut created = Vec::new();

        for new_act in creates {
            let id_val = Uuid::now_v7().to_string();
            let now = chrono::Utc::now().naive_utc();
            let act_asset_id = new_act.get_symbol_id().map(|s| s.to_string());
            diesel::insert_into(activities::table)
                .values((
                    activities::id.eq(&id_val),
                    activities::account_id.eq(&new_act.account_id),
                    activities::asset_id.eq(act_asset_id),
                    activities::activity_type.eq(&new_act.activity_type),
                    activities::status.eq("POSTED"),
                    activities::activity_date.eq(&new_act.activity_date),
                    activities::currency.eq(&new_act.currency),
                    activities::created_at.eq(now),
                    activities::updated_at.eq(now),
                ))
                .execute(&mut conn)
                .await
                .map_err(StoragePgError::from)?;

            if let Ok(db) = activities::table
                .find(&id_val)
                .first::<ActivityDB>(&mut conn)
                .await
            {
                created.push(db.into());
            }
        }

        let mut deleted = Vec::new();
        if !delete_ids.is_empty() {
            for del_id in &delete_ids {
                if let Ok(db) = activities::table
                    .find(del_id)
                    .first::<ActivityDB>(&mut conn)
                    .await
                {
                    diesel::delete(activities::table.find(del_id))
                        .execute(&mut conn)
                        .await
                        .map_err(StoragePgError::from)?;
                    deleted.push(db.into());
                }
            }
        }

        Ok(ActivityBulkMutationResult {
            created,
            updated: vec![],
            deleted,
            created_mappings: vec![],
            errors: vec![],
        })
    }

    async fn create_activities(&self, new_activities: Vec<NewActivity>) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let mut count = 0;

        for new_act in new_activities {
            let id_val = Uuid::now_v7().to_string();
            let now = chrono::Utc::now().naive_utc();
            let act_asset_id = new_act.get_symbol_id().map(|s| s.to_string());
            diesel::insert_into(activities::table)
                .values((
                    activities::id.eq(&id_val),
                    activities::account_id.eq(&new_act.account_id),
                    activities::asset_id.eq(act_asset_id),
                    activities::activity_type.eq(&new_act.activity_type),
                    activities::status.eq("POSTED"),
                    activities::activity_date.eq(&new_act.activity_date),
                    activities::currency.eq(&new_act.currency),
                    activities::created_at.eq(now),
                    activities::updated_at.eq(now),
                ))
                .execute(&mut conn)
                .await
                .map_err(StoragePgError::from)?;
            count += 1;
        }

        Ok(count)
    }

    async fn get_first_activity_date(
        &self,
        _account_ids: Option<&[String]>,
    ) -> Result<Option<DateTime<Utc>>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let result: Option<String> = activities::table
            .select(diesel::dsl::min(activity_date))
            .first(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(result
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()))
    }

    async fn get_import_mapping(
        &self,
        _account_id: &str,
        _context_kind: &str,
    ) -> Result<Option<ImportMapping>> {
        Ok(None)
    }

    async fn save_import_mapping(&self, _mapping: &ImportMapping) -> Result<()> {
        Ok(())
    }

    async fn link_account_template(
        &self,
        _account_id: &str,
        _template_id: &str,
        _context_kind: &str,
    ) -> Result<()> {
        Ok(())
    }

    async fn list_import_templates(&self) -> Result<Vec<CoreImportTemplate>> {
        Ok(vec![])
    }

    async fn get_import_template(&self, _template_id: &str) -> Result<Option<CoreImportTemplate>> {
        Ok(None)
    }

    async fn save_import_template(&self, _template: &CoreImportTemplate) -> Result<()> {
        Ok(())
    }

    async fn delete_import_template(&self, _template_id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_broker_sync_profile(
        &self,
        _account_id: &str,
        _source_system: &str,
    ) -> Result<Option<CoreImportTemplate>> {
        Ok(None)
    }

    async fn save_broker_sync_profile(&self, _template: &CoreImportTemplate) -> Result<()> {
        Ok(())
    }

    async fn link_broker_sync_profile(
        &self,
        _account_id: &str,
        _template_id: &str,
        _source_system: &str,
    ) -> Result<()> {
        Ok(())
    }

    async fn calculate_average_cost(&self, _account_id: &str, _asset_id: &str) -> Result<Decimal> {
        Ok(Decimal::ZERO)
    }

    async fn get_income_activities_data(
        &self,
        _account_id: Option<&str>,
    ) -> Result<Vec<whaleit_core::activities::IncomeData>> {
        Ok(vec![])
    }

    async fn get_first_activity_date_overall(&self) -> Result<DateTime<Utc>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let result: Option<String> = activities::table
            .select(diesel::dsl::min(activity_date))
            .first(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        let date = result
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Utc::now().date_naive());
        Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc())
    }

    async fn get_activity_bounds_for_assets(
        &self,
        _asset_ids: &[String],
    ) -> Result<HashMap<String, (Option<NaiveDate>, Option<NaiveDate>)>> {
        Ok(HashMap::new())
    }

    async fn check_existing_duplicates(
        &self,
        _idempotency_keys: &[String],
    ) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    async fn bulk_upsert(
        &self,
        _activities: Vec<whaleit_core::activities::ActivityUpsert>,
    ) -> Result<BulkUpsertResult> {
        Ok(BulkUpsertResult {
            upserted: 0,
            created: 0,
            updated: 0,
            skipped: 0,
        })
    }

    async fn reassign_asset(&self, _old_asset_id: &str, _new_asset_id: &str) -> Result<u32> {
        Ok(0)
    }

    async fn get_activity_accounts_and_currencies_by_asset_id(
        &self,
        _asset_id: &str,
    ) -> Result<(Vec<String>, Vec<String>)> {
        Ok((vec![], vec![]))
    }
}
