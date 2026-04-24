use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct UserDB {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUserDB {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
}

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::verification_tokens)]
pub struct VerificationTokenDB {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub token_type: String,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::verification_tokens)]
pub struct NewVerificationTokenDB {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub token_type: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::api_keys)]
pub struct ApiKeyDB {
    pub id: String,
    pub user_id: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub name: String,
    pub last_used_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::api_keys)]
pub struct NewApiKeyDB {
    pub id: String,
    pub user_id: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub name: String,
    pub expires_at: Option<NaiveDateTime>,
}

impl From<UserDB> for whaleit_core::users::User {
    fn from(db: UserDB) -> Self {
        Self {
            id: db.id,
            email: db.email,
            password_hash: db.password_hash,
            display_name: db.display_name,
            email_verified: db.email_verified,
            is_active: db.is_active,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl From<VerificationTokenDB> for whaleit_core::users::VerificationToken {
    fn from(db: VerificationTokenDB) -> Self {
        Self {
            id: db.id,
            user_id: db.user_id,
            token_hash: db.token_hash,
            token_type: db.token_type,
            expires_at: db.expires_at,
            used_at: db.used_at,
            created_at: db.created_at,
        }
    }
}

impl From<ApiKeyDB> for whaleit_core::users::ApiKey {
    fn from(db: ApiKeyDB) -> Self {
        Self {
            id: db.id,
            user_id: db.user_id,
            key_prefix: db.key_prefix,
            key_hash: db.key_hash,
            name: db.name,
            last_used_at: db.last_used_at,
            expires_at: db.expires_at,
            is_active: db.is_active,
            created_at: db.created_at,
        }
    }
}
