//! Database models for taxonomies.

use diesel::prelude::*;

#[derive(Queryable, Identifiable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::taxonomies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaxonomyDB {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_single_select: bool,
    pub sort_order: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::taxonomies)]
pub struct NewTaxonomyDB {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_single_select: bool,
    pub sort_order: i32,
}

#[derive(Queryable, Identifiable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::taxonomy_categories)]
#[diesel(primary_key(id, taxonomy_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CategoryDB {
    pub id: String,
    pub taxonomy_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub key: String,
    pub color: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::taxonomy_categories)]
pub struct NewCategoryDB {
    pub id: String,
    pub taxonomy_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub key: String,
    pub color: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Queryable, Identifiable, Insertable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_taxonomy_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetTaxonomyAssignmentDB {
    pub id: String,
    pub asset_id: String,
    pub taxonomy_id: String,
    pub category_id: String,
    pub weight: i32,
    pub source: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::asset_taxonomy_assignments)]
pub struct NewAssetTaxonomyAssignmentDB {
    pub id: String,
    pub asset_id: String,
    pub taxonomy_id: String,
    pub category_id: String,
    pub weight: i32,
    pub source: String,
}

impl From<TaxonomyDB> for whaleit_core::taxonomies::Taxonomy {
    fn from(db: TaxonomyDB) -> Self {
        Self {
            id: db.id,
            name: db.name,
            color: db.color,
            description: db.description,
            is_system: db.is_system,
            is_single_select: db.is_single_select,
            sort_order: db.sort_order,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl From<CategoryDB> for whaleit_core::taxonomies::Category {
    fn from(db: CategoryDB) -> Self {
        Self {
            id: db.id,
            taxonomy_id: db.taxonomy_id,
            parent_id: db.parent_id,
            name: db.name,
            key: db.key,
            color: db.color,
            description: db.description,
            sort_order: db.sort_order,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl From<AssetTaxonomyAssignmentDB> for whaleit_core::taxonomies::AssetTaxonomyAssignment {
    fn from(db: AssetTaxonomyAssignmentDB) -> Self {
        Self {
            id: db.id,
            asset_id: db.asset_id,
            taxonomy_id: db.taxonomy_id,
            category_id: db.category_id,
            weight: db.weight,
            source: db.source,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}
