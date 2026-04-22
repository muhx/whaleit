use chrono::NaiveDateTime;

pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct VerificationToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub token_type: String,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

pub struct ApiKey {
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
