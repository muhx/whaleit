use std::sync::Arc;

use crate::auth::{generate_api_key, AuthenticatedUser};
use crate::main_lib::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub id: String,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
    pub api_key: String,
}

pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Response, ApiKeyError> {
    let user_repo = state.user_repo.as_ref().ok_or(ApiKeyError::NotConfigured)?;

    let keys = user_repo
        .list_api_keys(&user.user_id)
        .await
        .map_err(|e| ApiKeyError::Internal(e.to_string()))?;

    let response: Vec<ApiKeyResponse> = keys
        .into_iter()
        .map(|k| ApiKeyResponse {
            id: k.id,
            key_prefix: k.key_prefix,
            name: k.name,
            created_at: k.created_at.to_string(),
            last_used_at: k.last_used_at.map(|d| d.to_string()),
            api_key: None,
        })
        .collect();

    Ok(Json(response).into_response())
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Response, ApiKeyError> {
    let user_repo = state.user_repo.as_ref().ok_or(ApiKeyError::NotConfigured)?;

    if payload.name.trim().is_empty() {
        return Err(ApiKeyError::BadRequest("Name is required".into()));
    }

    let (full_key, key_hash) = generate_api_key();
    let key_prefix = full_key[..12].to_string();

    let api_key = user_repo
        .create_api_key(&user.user_id, &key_prefix, &key_hash, &payload.name, None)
        .await
        .map_err(|e| ApiKeyError::Internal(e.to_string()))?;

    let response = CreateApiKeyResponse {
        id: api_key.id,
        key_prefix: api_key.key_prefix,
        name: api_key.name,
        created_at: api_key.created_at.to_string(),
        api_key: full_key,
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<DeleteApiKeyRequest>,
) -> Result<Response, ApiKeyError> {
    let user_repo = state.user_repo.as_ref().ok_or(ApiKeyError::NotConfigured)?;

    user_repo
        .delete_api_key(&payload.id, &user.user_id)
        .await
        .map_err(|e| ApiKeyError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
pub struct DeleteApiKeyRequest {
    pub id: String,
}

#[derive(Debug)]
pub enum ApiKeyError {
    NotConfigured,
    BadRequest(String),
    #[allow(dead_code)]
    NotFound(String),
    Internal(String),
}

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        use axum::Json;

        let (status, message) = match self {
            ApiKeyError::NotConfigured => (
                StatusCode::NOT_FOUND,
                "Authentication not configured".into(),
            ),
            ApiKeyError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiKeyError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiKeyError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(serde_json::json!({ "message": message }))).into_response()
    }
}
