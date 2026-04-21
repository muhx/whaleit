//! Database models for AI chat.

use diesel::prelude::*;

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::ai_threads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiThreadDB {
    pub id: String,
    pub title: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub config_snapshot: Option<String>,
    pub is_pinned: bool,
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::ai_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiMessageDB {
    pub id: String,
    pub thread_id: String,
    pub role: String,
    pub content_json: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Identifiable, Selectable, PartialEq, Debug, Clone)]
#[diesel(table_name = crate::schema::ai_thread_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiThreadTagDB {
    pub id: String,
    pub thread_id: String,
    pub tag: String,
    pub created_at: chrono::NaiveDateTime,
}
