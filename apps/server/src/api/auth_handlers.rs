use std::sync::Arc;

use crate::auth::{hash_password, hash_token};
use crate::main_lib::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::{Duration, Utc};
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, ApiAuthError> {
    let user_repo = state
        .user_repo
        .as_ref()
        .ok_or(ApiAuthError::NotConfigured)?;
    let email_service = state.email.as_ref().ok_or(ApiAuthError::NotConfigured)?;

    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(ApiAuthError::BadRequest("Invalid email address".into()));
    }
    if payload.password.len() < 8 {
        return Err(ApiAuthError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    let password_hash = hash_password(&payload.password)?;

    let user = user_repo
        .create_user(&email, &password_hash, payload.display_name.as_deref())
        .await
        .map_err(|e| {
            if e.to_string().contains("already registered") {
                ApiAuthError::Conflict("Email already registered".into())
            } else {
                ApiAuthError::Internal(e.to_string())
            }
        })?;

    // Generate verification token
    let token_bytes: [u8; 32] = rand::thread_rng().gen();
    let verification_token = URL_SAFE_NO_PAD.encode(token_bytes);
    let token_hash = hash_token(&verification_token);
    let expires_at = (Utc::now() + Duration::hours(1)).naive_utc();

    user_repo
        .create_token(&user.id, &token_hash, "EMAIL_VERIFY", expires_at)
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?;

    // Send verification email (non-blocking)
    email_service
        .send_verification_email(&email, &verification_token)
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "message": "Registration successful. Check your email to verify your account." }))).into_response())
}

pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<Response, ApiAuthError> {
    let user_repo = state
        .user_repo
        .as_ref()
        .ok_or(ApiAuthError::NotConfigured)?;

    let token_hash = hash_token(&payload.token);

    let token = user_repo
        .find_valid_token(&token_hash, "EMAIL_VERIFY")
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?
        .ok_or(ApiAuthError::BadRequest(
            "Invalid or expired verification token".into(),
        ))?;

    user_repo
        .verify_email(&token.user_id)
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?;

    user_repo
        .consume_token(&token.id)
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "message": "Email verified successfully" })).into_response())
}

pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Response, ApiAuthError> {
    let user_repo = state
        .user_repo
        .as_ref()
        .ok_or(ApiAuthError::NotConfigured)?;
    let email_service = state.email.as_ref().ok_or(ApiAuthError::NotConfigured)?;

    let email = payload.email.trim().to_lowercase();

    // Always return 200 to prevent email enumeration
    if let Ok(Some(user)) = user_repo.find_by_email(&email).await {
        let token_bytes: [u8; 32] = rand::thread_rng().gen();
        let reset_token = URL_SAFE_NO_PAD.encode(token_bytes);
        let token_hash = hash_token(&reset_token);
        let expires_at = (Utc::now() + Duration::hours(1)).naive_utc();

        if let Ok(()) = user_repo
            .create_token(&user.id, &token_hash, "PASSWORD_RESET", expires_at)
            .await
        {
            // Best-effort email send
            let _ = email_service
                .send_password_reset_email(&email, &reset_token)
                .await;
        }
    }

    Ok(Json(serde_json::json!({ "message": "If an account exists with that email, a reset link has been sent." })).into_response())
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Response, ApiAuthError> {
    let user_repo = state
        .user_repo
        .as_ref()
        .ok_or(ApiAuthError::NotConfigured)?;

    if payload.password.len() < 8 {
        return Err(ApiAuthError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }

    let token_hash = hash_token(&payload.token);

    let token = user_repo
        .find_valid_token(&token_hash, "PASSWORD_RESET")
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?
        .ok_or(ApiAuthError::BadRequest(
            "Invalid or expired reset token".into(),
        ))?;

    let new_hash = hash_password(&payload.password)?;

    user_repo
        .update_password(&token.user_id, &new_hash)
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?;

    user_repo
        .consume_token(&token.id)
        .await
        .map_err(|e| ApiAuthError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "message": "Password reset successfully" })).into_response())
}

#[derive(Debug)]
pub enum ApiAuthError {
    NotConfigured,
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for ApiAuthError {
    fn into_response(self) -> Response {
        use axum::http::StatusCode;
        use axum::Json;

        let (status, message) = match self {
            ApiAuthError::NotConfigured => (
                StatusCode::NOT_FOUND,
                "Authentication not configured".into(),
            ),
            ApiAuthError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiAuthError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiAuthError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(serde_json::json!({ "message": message }))).into_response()
    }
}

impl From<crate::auth::AuthError> for ApiAuthError {
    fn from(err: crate::auth::AuthError) -> Self {
        match err {
            crate::auth::AuthError::Internal(msg) => ApiAuthError::Internal(msg),
            other => ApiAuthError::Internal(other.to_string()),
        }
    }
}
