//! Database models for goals.

use diesel::prelude::*;

/// Database model for goals
#[derive(Queryable, Identifiable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::goals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GoalDB {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_amount: f64,
    pub is_achieved: bool,
}

/// Database model for creating a new goal
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::goals)]
pub struct NewGoalDB {
    pub id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub target_amount: f64,
    pub is_achieved: bool,
}

/// Database model for goal allocations
#[derive(Insertable, Queryable, Identifiable, AsChangeset, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::goals_allocation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GoalsAllocationDB {
    pub id: String,
    pub goal_id: String,
    pub account_id: String,
    pub percent_allocation: i32,
}

// Conversions
impl From<GoalDB> for whaleit_core::goals::Goal {
    fn from(db: GoalDB) -> Self {
        Self {
            id: db.id,
            title: db.title,
            description: db.description,
            target_amount: db.target_amount,
            is_achieved: db.is_achieved,
        }
    }
}

impl From<GoalsAllocationDB> for whaleit_core::goals::GoalsAllocation {
    fn from(db: GoalsAllocationDB) -> Self {
        Self {
            id: db.id,
            goal_id: db.goal_id,
            account_id: db.account_id,
            percent_allocation: db.percent_allocation,
        }
    }
}

impl From<whaleit_core::goals::NewGoal> for NewGoalDB {
    fn from(domain: whaleit_core::goals::NewGoal) -> Self {
        Self {
            id: domain.id,
            title: domain.title,
            description: domain.description,
            target_amount: domain.target_amount,
            is_achieved: domain.is_achieved,
        }
    }
}

impl From<whaleit_core::goals::GoalsAllocation> for GoalsAllocationDB {
    fn from(domain: whaleit_core::goals::GoalsAllocation) -> Self {
        Self {
            id: domain.id,
            goal_id: domain.goal_id,
            account_id: domain.account_id,
            percent_allocation: domain.percent_allocation,
        }
    }
}
