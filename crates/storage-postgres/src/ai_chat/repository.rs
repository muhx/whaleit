//! PostgreSQL AI chat repository.
//!
//! Implements `ChatRepositoryTrait` from `whaleit-ai` using diesel-async.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use whaleit_ai::{
    AiError, ChatMessage, ChatMessageContent, ChatMessagePart, ChatMessageRole,
    ChatRepositoryResult, ChatRepositoryTrait, ChatThread, ChatThreadConfig, ListThreadsRequest,
    ThreadPage,
};
use whaleit_core::errors::{DatabaseError, Error as CoreError};

use crate::db::PgPool;
use crate::schema::{ai_messages, ai_thread_tags, ai_threads};

use super::model::{AiMessageDB, AiThreadDB, AiThreadTagDB};

#[allow(dead_code)]
fn core_to_ai_error(e: CoreError) -> AiError {
    AiError::Core(e)
}

fn db_to_thread(db: &AiThreadDB) -> ChatThread {
    let config = db
        .config_snapshot
        .as_ref()
        .and_then(|json| serde_json::from_str::<ChatThreadConfig>(json).ok());

    ChatThread {
        id: db.id.clone(),
        title: db.title.clone(),
        is_pinned: db.is_pinned,
        tags: Vec::new(),
        config,
        created_at: naive_to_datetime(db.created_at),
        updated_at: naive_to_datetime(db.updated_at),
    }
}

fn naive_to_datetime(ndt: NaiveDateTime) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(ndt, Utc)
}

fn datetime_to_naive(dt: DateTime<Utc>) -> NaiveDateTime {
    dt.naive_utc()
}

fn db_to_message(db: &AiMessageDB) -> ChatRepositoryResult<ChatMessage> {
    let content = convert_json_to_content(&db.content_json)?;
    let role = db
        .role
        .parse::<ChatMessageRole>()
        .map_err(AiError::InvalidInput)?;

    Ok(ChatMessage {
        id: db.id.clone(),
        thread_id: db.thread_id.clone(),
        role,
        content,
        created_at: naive_to_datetime(db.created_at),
    })
}

fn convert_content_to_json(content: &ChatMessageContent) -> ChatRepositoryResult<String> {
    // Reuse the SQLite model's serialization format
    let storage_parts: Vec<serde_json::Value> = content
        .parts
        .iter()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .collect();

    let obj = serde_json::json!({
        "schemaVersion": content.schema_version,
        "parts": storage_parts,
        "truncated": content.truncated,
    });

    serde_json::to_string(&obj).map_err(|e| AiError::InvalidInput(e.to_string()))
}

fn convert_json_to_content(json: &str) -> ChatRepositoryResult<ChatMessageContent> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| AiError::InvalidInput(e.to_string()))?;

    let schema_version = parsed
        .get("schemaVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let truncated = parsed
        .get("truncated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let parts_val = parsed
        .get("parts")
        .cloned()
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let parts_arr = parts_val.as_array().cloned().unwrap_or_default();

    let mut parts = Vec::new();
    for part_val in parts_arr {
        let part: ChatMessagePart =
            serde_json::from_value(part_val).map_err(|e| AiError::InvalidInput(e.to_string()))?;
        parts.push(part);
    }

    Ok(ChatMessageContent {
        schema_version,
        parts,
        truncated,
    })
}

fn parse_cursor(cursor: &str) -> ChatRepositoryResult<(bool, String, String)> {
    let parts: Vec<&str> = cursor.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err(AiError::InvalidCursor(format!(
            "Expected format 'is_pinned:updated_at:id', got '{}'",
            cursor
        )));
    }

    let is_pinned: bool = parts[0]
        .parse()
        .map_err(|_| AiError::InvalidCursor(format!("Invalid is_pinned value: {}", parts[0])))?;

    Ok((is_pinned, parts[1].to_string(), parts[2].to_string()))
}

fn encode_cursor(is_pinned: bool, updated_at: &NaiveDateTime, id: &str) -> String {
    format!("{}:{}:{}", if is_pinned { 1 } else { 0 }, updated_at, id)
}

pub struct PgAiChatRepository {
    pool: Arc<PgPool>,
}

impl PgAiChatRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChatRepositoryTrait for PgAiChatRepository {
    async fn create_thread(&self, thread: ChatThread) -> ChatRepositoryResult<ChatThread> {
        let config_snapshot = thread
            .config
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());

        let thread_db = AiThreadDB {
            id: thread.id.clone(),
            title: thread.title.clone(),
            created_at: datetime_to_naive(thread.created_at),
            updated_at: datetime_to_naive(thread.updated_at),
            config_snapshot,
            is_pinned: thread.is_pinned,
        };

        let pool = self.pool.clone();
        let thread_id = thread_db.id.clone();

        let mut conn = pool.get().await.map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                e.to_string(),
            )))
        })?;

        diesel::insert_into(ai_threads::table)
            .values(&thread_db)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        let db = ai_threads::table
            .find(&thread_id)
            .first::<AiThreadDB>(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        Ok(db_to_thread(&db))
    }

    fn get_thread(&self, thread_id: &str) -> ChatRepositoryResult<Option<ChatThread>> {
        let pool = self.pool.clone();
        let thread_id = thread_id.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                        e.to_string(),
                    )))
                })?;

                let result = ai_threads::table
                    .find(&thread_id)
                    .first::<AiThreadDB>(&mut conn)
                    .await
                    .optional()
                    .map_err(|e| {
                        AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                            e.to_string(),
                        )))
                    })?;

                Ok(result.map(|db| db_to_thread(&db)))
            })
        })
    }

    fn list_threads(&self, limit: i64, offset: i64) -> ChatRepositoryResult<Vec<ChatThread>> {
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                        e.to_string(),
                    )))
                })?;

                let threads_db = ai_threads::table
                    .order((ai_threads::is_pinned.desc(), ai_threads::updated_at.desc()))
                    .limit(limit)
                    .offset(offset)
                    .load::<AiThreadDB>(&mut conn)
                    .await
                    .map_err(|e| {
                        AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                            e.to_string(),
                        )))
                    })?;

                let mut threads = Vec::with_capacity(threads_db.len());
                for db in &threads_db {
                    let mut thread = db_to_thread(db);
                    thread.tags = ai_thread_tags::table
                        .filter(ai_thread_tags::thread_id.eq(&db.id))
                        .select(ai_thread_tags::tag)
                        .load::<String>(&mut conn)
                        .await
                        .unwrap_or_default();
                    threads.push(thread);
                }

                Ok(threads)
            })
        })
    }

    fn list_threads_paginated(
        &self,
        request: &ListThreadsRequest,
    ) -> ChatRepositoryResult<ThreadPage> {
        let pool = self.pool.clone();
        let limit = request.limit.unwrap_or(20).min(100) as i64;
        let search = request.search.clone();
        let cursor = request.cursor.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                        e.to_string(),
                    )))
                })?;

                let mut query = ai_threads::table.into_boxed();

                if let Some(search_str) = search.as_deref().map(str::trim).filter(|s| !s.is_empty())
                {
                    let escaped = search_str
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_");
                    let search_pattern = format!("%{}%", escaped);
                    query = query.filter(ai_threads::title.like(search_pattern));
                }

                if let Some(cursor_str) = &cursor {
                    let (cursor_pinned, cursor_updated_at, cursor_id) = parse_cursor(cursor_str)?;
                    let cursor_updated_at = cursor_updated_at
                        .parse::<NaiveDateTime>()
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc());

                    query = query.filter(
                        ai_threads::is_pinned
                            .lt(cursor_pinned)
                            .or(ai_threads::is_pinned
                                .eq(cursor_pinned)
                                .and(ai_threads::updated_at.lt(cursor_updated_at)))
                            .or(ai_threads::is_pinned
                                .eq(cursor_pinned)
                                .and(ai_threads::updated_at.eq(cursor_updated_at))
                                .and(ai_threads::id.lt(cursor_id))),
                    );
                }

                query = query.order((
                    ai_threads::is_pinned.desc(),
                    ai_threads::updated_at.desc(),
                    ai_threads::id.desc(),
                ));

                let threads_db = query
                    .limit(limit + 1)
                    .load::<AiThreadDB>(&mut conn)
                    .await
                    .map_err(|e| {
                        AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                            e.to_string(),
                        )))
                    })?;

                let has_more = threads_db.len() > limit as usize;
                let threads_db: Vec<_> = threads_db.into_iter().take(limit as usize).collect();

                let mut threads = Vec::with_capacity(threads_db.len());
                for db in &threads_db {
                    let mut thread = db_to_thread(db);
                    thread.tags = ai_thread_tags::table
                        .filter(ai_thread_tags::thread_id.eq(&db.id))
                        .select(ai_thread_tags::tag)
                        .load::<String>(&mut conn)
                        .await
                        .unwrap_or_default();
                    threads.push(thread);
                }

                let next_cursor = if has_more {
                    threads_db
                        .last()
                        .map(|t| encode_cursor(t.is_pinned, &t.updated_at, &t.id))
                } else {
                    None
                };

                Ok(ThreadPage {
                    threads,
                    next_cursor,
                    has_more,
                })
            })
        })
    }

    async fn update_thread(&self, thread: ChatThread) -> ChatRepositoryResult<ChatThread> {
        let thread_id = thread.id.clone();
        let title = thread.title.clone();
        let is_pinned = thread.is_pinned;
        let config_snapshot = thread
            .config
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());
        let updated_at = Utc::now().naive_utc();

        let pool = self.pool.clone();

        let mut conn = pool.get().await.map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                e.to_string(),
            )))
        })?;

        diesel::update(ai_threads::table.find(&thread_id))
            .set((
                ai_threads::title.eq(&title),
                ai_threads::is_pinned.eq(is_pinned),
                ai_threads::config_snapshot.eq(&config_snapshot),
                ai_threads::updated_at.eq(&updated_at),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        let db = ai_threads::table
            .find(&thread_id)
            .first::<AiThreadDB>(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        Ok(db_to_thread(&db))
    }

    async fn delete_thread(&self, thread_id: &str) -> ChatRepositoryResult<()> {
        let thread_id = thread_id.to_string();
        let pool = self.pool.clone();

        let mut conn = pool.get().await.map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                e.to_string(),
            )))
        })?;

        diesel::delete(ai_threads::table.find(&thread_id))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        Ok(())
    }

    async fn create_message(&self, message: ChatMessage) -> ChatRepositoryResult<ChatMessage> {
        let content_json = convert_content_to_json(&message.content)?;
        let message_db = AiMessageDB {
            id: message.id.clone(),
            thread_id: message.thread_id.clone(),
            role: message.role.to_string(),
            content_json,
            created_at: datetime_to_naive(message.created_at),
        };

        let thread_id = message_db.thread_id.clone();
        let message_id = message_db.id.clone();
        let pool = self.pool.clone();
        let updated_at = Utc::now().naive_utc();

        let mut conn = pool.get().await.map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                e.to_string(),
            )))
        })?;

        diesel::insert_into(ai_messages::table)
            .values(&message_db)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        // Update thread's updated_at
        diesel::update(ai_threads::table.find(&thread_id))
            .set(ai_threads::updated_at.eq(updated_at))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        let db = ai_messages::table
            .find(&message_id)
            .first::<AiMessageDB>(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        db_to_message(&db)
    }

    fn get_message(&self, message_id: &str) -> ChatRepositoryResult<Option<ChatMessage>> {
        let pool = self.pool.clone();
        let message_id = message_id.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                        e.to_string(),
                    )))
                })?;

                let result = ai_messages::table
                    .find(&message_id)
                    .first::<AiMessageDB>(&mut conn)
                    .await
                    .optional()
                    .map_err(|e| {
                        AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                            e.to_string(),
                        )))
                    })?;

                match result {
                    Some(db) => Ok(Some(db_to_message(&db)?)),
                    None => Ok(None),
                }
            })
        })
    }

    fn get_messages_by_thread(&self, thread_id: &str) -> ChatRepositoryResult<Vec<ChatMessage>> {
        let pool = self.pool.clone();
        let thread_id = thread_id.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                        e.to_string(),
                    )))
                })?;

                let messages_db = ai_messages::table
                    .filter(ai_messages::thread_id.eq(&thread_id))
                    .order(ai_messages::created_at.asc())
                    .load::<AiMessageDB>(&mut conn)
                    .await
                    .map_err(|e| {
                        AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                            e.to_string(),
                        )))
                    })?;

                messages_db
                    .iter()
                    .map(db_to_message)
                    .collect::<ChatRepositoryResult<Vec<_>>>()
            })
        })
    }

    async fn update_message(&self, message: ChatMessage) -> ChatRepositoryResult<ChatMessage> {
        let message_id = message.id.clone();
        let content_json = convert_content_to_json(&message.content)?;
        let pool = self.pool.clone();

        let mut conn = pool.get().await.map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                e.to_string(),
            )))
        })?;

        diesel::update(ai_messages::table.find(&message_id))
            .set(ai_messages::content_json.eq(&content_json))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        let db = ai_messages::table
            .find(&message_id)
            .first::<AiMessageDB>(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        db_to_message(&db)
    }

    async fn add_tag(&self, thread_id: &str, tag: &str) -> ChatRepositoryResult<()> {
        let tag_db = AiThreadTagDB::new(
            uuid::Uuid::new_v4().to_string(),
            thread_id.to_string(),
            tag.to_string(),
        );

        let pool = self.pool.clone();

        let mut conn = pool.get().await.map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                e.to_string(),
            )))
        })?;

        diesel::insert_into(ai_thread_tags::table)
            .values(&tag_db)
            .on_conflict((ai_thread_tags::thread_id, ai_thread_tags::tag))
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                    e.to_string(),
                )))
            })?;

        Ok(())
    }

    async fn remove_tag(&self, thread_id: &str, tag: &str) -> ChatRepositoryResult<()> {
        let thread_id = thread_id.to_string();
        let tag = tag.to_string();
        let pool = self.pool.clone();

        let mut conn = pool.get().await.map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                e.to_string(),
            )))
        })?;

        diesel::delete(
            ai_thread_tags::table
                .filter(ai_thread_tags::thread_id.eq(&thread_id))
                .filter(ai_thread_tags::tag.eq(&tag)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                e.to_string(),
            )))
        })?;

        Ok(())
    }

    fn get_tags(&self, thread_id: &str) -> ChatRepositoryResult<Vec<String>> {
        let pool = self.pool.clone();
        let thread_id = thread_id.to_string();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut conn = pool.get().await.map_err(|e| {
                    AiError::Core(CoreError::Database(DatabaseError::ConnectionFailed(
                        e.to_string(),
                    )))
                })?;

                ai_thread_tags::table
                    .filter(ai_thread_tags::thread_id.eq(&thread_id))
                    .select(ai_thread_tags::tag)
                    .load::<String>(&mut conn)
                    .await
                    .map_err(|e| {
                        AiError::Core(CoreError::Database(DatabaseError::QueryFailed(
                            e.to_string(),
                        )))
                    })
            })
        })
    }
}
