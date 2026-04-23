//! PostgreSQL goals repository implementation.

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use super::model::{GoalDB, GoalsAllocationDB, NewGoalDB};
use crate::db::PgPool;
use crate::errors::StoragePgError;
use crate::schema::goals;
use crate::schema::goals::dsl::*;
use crate::schema::goals_allocation;
use whaleit_core::goals::{Goal, GoalRepositoryTrait, GoalsAllocation, NewGoal};
use whaleit_core::Result;

pub struct PgGoalRepository {
    pool: Arc<PgPool>,
}

impl PgGoalRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        PgGoalRepository { pool }
    }
}

#[async_trait]
impl GoalRepositoryTrait for PgGoalRepository {
    async fn load_goals(&self) -> Result<Vec<Goal>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let goals_db = goals::table
            .load::<GoalDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(goals_db.into_iter().map(Goal::from).collect())
    }

    async fn insert_new_goal(&self, new_goal: NewGoal) -> Result<Goal> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let mut new_goal_db: NewGoalDB = new_goal.into();
        new_goal_db.id = Some(Uuid::now_v7().to_string());

        let result_db = diesel::insert_into(goals::table)
            .values(&new_goal_db)
            .returning(GoalDB::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(result_db.into())
    }

    async fn update_goal(&self, goal_update: Goal) -> Result<Goal> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let goal_id_owned = goal_update.id.clone();
        let goal_db = GoalDB {
            id: goal_update.id,
            title: goal_update.title,
            description: goal_update.description,
            target_amount: goal_update.target_amount,
            is_achieved: goal_update.is_achieved,
        };

        diesel::update(goals::table.find(&goal_id_owned))
            .set(&goal_db)
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        let result_db = goals::table
            .filter(id.eq(&goal_id_owned))
            .first::<GoalDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;

        Ok(result_db.into())
    }

    async fn delete_goal(&self, goal_id_to_delete: String) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let affected = diesel::delete(goals::table.find(&goal_id_to_delete))
            .execute(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(affected)
    }

    async fn load_allocations_for_non_achieved_goals(&self) -> Result<Vec<GoalsAllocation>> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let allocations_db = goals_allocation::table
            .inner_join(goals::table.on(goals::id.eq(goals_allocation::goal_id)))
            .filter(goals::is_achieved.eq(false))
            .select(GoalsAllocationDB::as_select())
            .load::<GoalsAllocationDB>(&mut conn)
            .await
            .map_err(StoragePgError::from)?;
        Ok(allocations_db
            .into_iter()
            .map(GoalsAllocation::from)
            .collect())
    }

    async fn upsert_goal_allocations(&self, allocations: Vec<GoalsAllocation>) -> Result<usize> {
        let mut conn = self.pool.get().await.map_err(|e| StoragePgError::from(e))?;
        let mut affected_rows = 0;
        for allocation in allocations {
            let allocation_db: GoalsAllocationDB = allocation.into();
            affected_rows += diesel::insert_into(goals_allocation::table)
                .values(&allocation_db)
                .on_conflict(goals_allocation::id)
                .do_update()
                .set(&allocation_db)
                .execute(&mut conn)
                .await
                .map_err(StoragePgError::from)?;
        }
        Ok(affected_rows)
    }
}
