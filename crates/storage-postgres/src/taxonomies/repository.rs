//! PostgreSQL taxonomies repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use super::model::*;
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::{asset_taxonomy_assignments, taxonomies, taxonomy_categories};
use crate::schema::taxonomies::dsl::*;
use whaleit_core::errors::Result;
use whaleit_core::taxonomies::{
    AssetTaxonomyAssignment, Category, NewAssetTaxonomyAssignment, NewCategory, NewTaxonomy,
    Taxonomy, TaxonomyRepositoryTrait, TaxonomyWithCategories,
};

pub struct PgTaxonomyRepository {
    pool: Arc<PgPool>,
}

impl PgTaxonomyRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaxonomyRepositoryTrait for PgTaxonomyRepository {
    async fn get_taxonomies(&self) -> Result<Vec<Taxonomy>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = taxonomies::table
            .order(taxonomies::sort_order)
            .load::<TaxonomyDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Taxonomy::from).collect())
    }

    async fn get_taxonomy(&self, taxonomy_id: &str) -> Result<Option<Taxonomy>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let result = taxonomies::table
            .find(taxonomy_id)
            .first::<TaxonomyDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;
        Ok(result.map(Taxonomy::from))
    }

    async fn create_taxonomy(&self, new_taxonomy: NewTaxonomy) -> Result<Taxonomy> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let now = chrono::Utc::now().naive_utc();
        let id_val = Uuid::now_v7().to_string();

        diesel::insert_into(taxonomies::table)
            .values(NewTaxonomyDB {
                id: id_val.clone(),
                name: new_taxonomy.name,
                color: new_taxonomy.color,
                description: new_taxonomy.description,
                is_system: new_taxonomy.is_system,
                is_single_select: new_taxonomy.is_single_select,
                sort_order: new_taxonomy.sort_order,
            })
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db = taxonomies::table
            .find(&id_val)
            .first::<TaxonomyDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn update_taxonomy(&self, taxonomy: Taxonomy) -> Result<Taxonomy> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(taxonomies::table.find(&taxonomy.id))
            .set((
                taxonomies::name.eq(&taxonomy.name),
                taxonomies::color.eq(&taxonomy.color),
                taxonomies::description.eq(&taxonomy.description),
                taxonomies::is_single_select.eq(taxonomy.is_single_select),
                taxonomies::sort_order.eq(taxonomy.sort_order),
                taxonomies::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db = taxonomies::table
            .find(&taxonomy.id)
            .first::<TaxonomyDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn delete_taxonomy(&self, taxonomy_id: &str) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let affected = diesel::delete(taxonomies::table.find(taxonomy_id))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(affected)
    }

    async fn get_categories(&self, taxonomy_id_param: &str) -> Result<Vec<Category>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = taxonomy_categories::table
            .filter(taxonomy_categories::taxonomy_id.eq(taxonomy_id_param))
            .order(taxonomy_categories::sort_order)
            .load::<CategoryDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(Category::from).collect())
    }

    async fn get_category(&self, taxonomy_id_param: &str, category_id_param: &str) -> Result<Option<Category>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let result = taxonomy_categories::table
            .filter(taxonomy_categories::taxonomy_id.eq(taxonomy_id_param))
            .filter(taxonomy_categories::id.eq(category_id_param))
            .first::<CategoryDB>(&mut conn)
            .await
            .optional()
            .map_err(StoragePgError::from)?;
        Ok(result.map(Category::from))
    }

    async fn create_category(&self, new_category: NewCategory) -> Result<Category> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let now = chrono::Utc::now().naive_utc();
        let id_val = Uuid::now_v7().to_string();

        diesel::insert_into(taxonomy_categories::table)
            .values(NewCategoryDB {
                id: id_val.clone(),
                taxonomy_id: new_category.taxonomy_id,
                parent_id: new_category.parent_id,
                name: new_category.name,
                key: new_category.key,
                color: new_category.color,
                description: new_category.description,
                sort_order: new_category.sort_order,
            })
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let db = taxonomy_categories::table
            .filter(taxonomy_categories::id.eq(&id_val))
            .first::<CategoryDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(db.into())
    }

    async fn update_category(&self, category: Category) -> Result<Category> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        diesel::update(
            taxonomy_categories::table
                .filter(taxonomy_categories::id.eq(&category.id))
                .filter(taxonomy_categories::taxonomy_id.eq(&category.taxonomy_id)),
        )
        .set((
            taxonomy_categories::name.eq(&category.name),
            taxonomy_categories::key.eq(&category.key),
            taxonomy_categories::color.eq(&category.color),
            taxonomy_categories::description.eq(&category.description),
            taxonomy_categories::sort_order.eq(category.sort_order),
            taxonomy_categories::parent_id.eq(&category.parent_id),
            taxonomy_categories::updated_at.eq(chrono::Utc::now().naive_utc()),
        ))
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;

        Ok(category)
    }

    async fn delete_category(&self, taxonomy_id_param: &str, category_id_param: &str) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let affected = diesel::delete(
            taxonomy_categories::table
                .filter(taxonomy_categories::taxonomy_id.eq(taxonomy_id_param))
                .filter(taxonomy_categories::id.eq(category_id_param)),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;
        Ok(affected)
    }

    async fn bulk_create_categories(&self, categories: Vec<NewCategory>) -> Result<usize> {
        let mut count = 0;
        for cat in categories {
            self.create_category(cat).await?;
            count += 1;
        }
        Ok(count)
    }

    async fn get_asset_assignments(&self, asset_id_param: &str) -> Result<Vec<AssetTaxonomyAssignment>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = asset_taxonomy_assignments::table
            .filter(asset_taxonomy_assignments::asset_id.eq(asset_id_param))
            .load::<AssetTaxonomyAssignmentDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(AssetTaxonomyAssignment::from).collect())
    }

    async fn get_category_assignments(&self, taxonomy_id_param: &str, category_id_param: &str) -> Result<Vec<AssetTaxonomyAssignment>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let results = asset_taxonomy_assignments::table
            .filter(asset_taxonomy_assignments::taxonomy_id.eq(taxonomy_id_param))
            .filter(asset_taxonomy_assignments::category_id.eq(category_id_param))
            .load::<AssetTaxonomyAssignmentDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(results.into_iter().map(AssetTaxonomyAssignment::from).collect())
    }

    async fn upsert_assignment(&self, assignment: NewAssetTaxonomyAssignment) -> Result<AssetTaxonomyAssignment> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let now = chrono::Utc::now().naive_utc();
        let id_val = Uuid::now_v7().to_string();

        let db = NewAssetTaxonomyAssignmentDB {
            id: id_val,
            asset_id: assignment.asset_id,
            taxonomy_id: assignment.taxonomy_id,
            category_id: assignment.category_id,
            weight: assignment.weight,
            source: assignment.source,
        };

        diesel::insert_into(asset_taxonomy_assignments::table)
            .values(&db)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(AssetTaxonomyAssignment {
            id: db.id,
            asset_id: db.asset_id,
            taxonomy_id: db.taxonomy_id,
            category_id: db.category_id,
            weight: db.weight,
            source: db.source,
            created_at: now,
            updated_at: now,
        })
    }

    async fn delete_assignment(&self, assignment_id: &str) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let affected = diesel::delete(asset_taxonomy_assignments::table.find(assignment_id))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(affected)
    }

    async fn delete_asset_assignments(&self, asset_id_param: &str, taxonomy_id_param: &str) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let affected = diesel::delete(
            asset_taxonomy_assignments::table
                .filter(asset_taxonomy_assignments::asset_id.eq(asset_id_param))
                .filter(asset_taxonomy_assignments::taxonomy_id.eq(taxonomy_id_param)),
        )
        .execute(&mut conn)
        .await
        .map_err(StoragePgError::from)?;
        Ok(affected)
    }

    async fn get_taxonomy_with_categories(&self, taxonomy_id: &str) -> Result<Option<TaxonomyWithCategories>> {
        let taxonomy = self.get_taxonomy(taxonomy_id).await?;
        match taxonomy {
            Some(t) => {
                let cats = self.get_categories(taxonomy_id).await?;
                Ok(Some(TaxonomyWithCategories {
                    taxonomy: t,
                    categories: cats,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_all_taxonomies_with_categories(&self) -> Result<Vec<TaxonomyWithCategories>> {
        let all_taxonomies = self.get_taxonomies().await?;
        let mut result = Vec::new();
        for t in all_taxonomies {
            let cats = self.get_categories(&t.id).await?;
            result.push(TaxonomyWithCategories {
                taxonomy: t,
                categories: cats,
            });
        }
        Ok(result)
    }
}
