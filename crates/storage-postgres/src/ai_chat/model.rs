//! Database models for AI chat.
//!
//! Defines DB models for threads, messages, and tags using PG-native types.

use chrono::Utc;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Insertable, AsChangeset, PartialEq)]
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

#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Insertable, AsChangeset, PartialEq)]
#[diesel(table_name = crate::schema::ai_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiMessageDB {
    pub id: String,
    pub thread_id: String,
    pub role: String,
    pub content_json: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Insertable, AsChangeset, PartialEq)]
#[diesel(table_name = crate::schema::ai_thread_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AiThreadTagDB {
    pub id: String,
    pub thread_id: String,
    pub tag: String,
    pub created_at: chrono::NaiveDateTime,
}

impl AiThreadDB {
    pub fn new(id: String, title: Option<String>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id,
            title,
            created_at: now.clone(),
            updated_at: now,
            config_snapshot: None,
            is_pinned: false,
        }
    }
}

impl AiThreadTagDB {
    pub fn new(id: String, thread_id: String, tag: String) -> Self {
        Self {
            id,
            thread_id,
            tag,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
