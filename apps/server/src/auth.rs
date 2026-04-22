use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{Error as PasswordHashError, PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{AUTHORIZATION, COOKIE, SET_COOKIE},
        HeaderMap, HeaderValue, Request, StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::main_lib::AppState;

#[derive(Clone, Debug)]
pub enum CookieSecurePolicy {
    Auto,
    Always,
    Never,
}

impl std::fmt::Display for CookieSecurePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto (Secure only when X-Forwarded-Proto: https)"),
            Self::Always => write!(f, "always (Secure flag always set)"),
            Self::Never => write!(f, "never (Secure flag never set)"),
        }
    }
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: Vec<u8>,
    pub access_token_ttl: Duration,
    pub cookie_secure: CookieSecurePolicy,
}

pub struct AuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    token_ttl: Duration,
    cookie_secure: CookieSecurePolicy,
}

#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    InvalidCredentials,
    EmailNotVerified,
    NotConfigured,
    Internal(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Unauthorized => write!(f, "Unauthorized"),
            AuthError::InvalidCredentials => write!(f, "Invalid email or password"),
            AuthError::EmailNotVerified => write!(f, "Email verification required"),
            AuthError::NotConfigured => write!(f, "Authentication not configured"),
            AuthError::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Serialize)]
struct AuthErrorBody {
    code: u16,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    sub: String,
    email: String,
    exp: usize,
    iat: usize,
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LegacyLoginRequest {
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub authenticated: bool,
    pub expires_in: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatusResponse {
    pub requires_password: bool,
    pub multi_user: bool,
}

impl AuthManager {
    pub fn new(config: &AuthConfig) -> anyhow::Result<Self> {
        let encoding_key = EncodingKey::from_secret(&config.jwt_secret);
        let decoding_key = DecodingKey::from_secret(&config.jwt_secret);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            token_ttl: config.access_token_ttl,
            cookie_secure: config.cookie_secure.clone(),
        })
    }

    pub fn issue_token(&self, user_id: &str, email: &str) -> Result<String, AuthError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AuthError::Internal("System clock is before UNIX_EPOCH".into()))?;
        let exp = now + self.token_ttl;
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            iat: now.as_secs() as usize,
            exp: exp.as_secs() as usize,
        };
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to sign token: {e}")))
    }

    pub(crate) fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|err| match err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature
                | jsonwebtoken::errors::ErrorKind::InvalidToken
                | jsonwebtoken::errors::ErrorKind::InvalidSignature
                | jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(_) => {
                    AuthError::Unauthorized
                }
                other => AuthError::Internal(format!("Failed to validate token: {other:?}")),
            })
    }

    pub(crate) fn should_refresh(&self, claims: &Claims) -> bool {
        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return false;
        };
        let elapsed = now.as_secs().saturating_sub(claims.iat as u64);
        elapsed > self.token_ttl.as_secs() / 2
    }

    pub fn expires_in(&self) -> Duration {
        self.token_ttl
    }

    pub fn should_secure_cookie(&self, headers: &HeaderMap) -> bool {
        match &self.cookie_secure {
            CookieSecurePolicy::Always => true,
            CookieSecurePolicy::Never => false,
            CookieSecurePolicy::Auto => headers
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| {
                    v.split(',')
                        .next()
                        .is_some_and(|p| p.trim().eq_ignore_ascii_case("https"))
                }),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password".to_string())
            }
            AuthError::EmailNotVerified => (
                StatusCode::FORBIDDEN,
                "Email verification required".to_string(),
            ),
            AuthError::NotConfigured => (
                StatusCode::NOT_FOUND,
                "Authentication is not configured for this server".to_string(),
            ),
            AuthError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        let body = Json(AuthErrorBody {
            code: status.as_u16(),
            message,
        });
        let mut response = (status, body).into_response();
        if matches!(self, AuthError::EmailNotVerified) {
            if let Ok(val) = HeaderValue::from_str("true") {
                response.headers_mut().insert("X-Verification-Required", val);
            }
        }
        response
    }
}

pub fn derive_keys(master: &[u8]) -> ([u8; 32], [u8; 32]) {
    use hkdf::Hkdf;
    use sha2::Sha256;

    let hk = Hkdf::<Sha256>::new(None, master);
    let mut jwt_key = [0u8; 32];
    hk.expand(b"wealthfolio-jwt", &mut jwt_key)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    let mut secrets_key = [0u8; 32];
    hk.expand(b"wealthfolio-secrets", &mut secrets_key)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    (jwt_key, secrets_key)
}

pub fn decode_secret_key(raw: &str) -> anyhow::Result<Vec<u8>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("JWT secret cannot be empty");
    }
    let decoded = match BASE64.decode(trimmed) {
        Ok(bytes) => bytes,
        Err(_) if trimmed.len() == 32 => trimmed.as_bytes().to_vec(),
        Err(_) => {
            anyhow::bail!("JWT secret must be base64 encoded or a 32-byte ASCII string")
        }
    };

    if decoded.len() != 32 {
        anyhow::bail!("JWT secret must decode to exactly 32 bytes");
    }

    Ok(decoded)
}

const SESSION_COOKIE_NAME: &str = "wf_session";

fn build_session_cookie(token: &str, max_age_secs: u64, secure: bool) -> String {
    let secure_attr = if secure { "; Secure" } else { "" };
    format!(
        "{SESSION_COOKIE_NAME}={token}; HttpOnly; SameSite=Lax; Path=/api; Max-Age={max_age_secs}{secure_attr}"
    )
}

pub fn verify_password_hash(password: &str, hash: &str) -> Result<(), AuthError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AuthError::Internal(format!("Invalid password hash: {e}")))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|err| match err {
            PasswordHashError::Password => AuthError::InvalidCredentials,
            other => AuthError::Internal(format!("Password verification failed: {other}")),
        })
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    use argon2::password_hash::{SaltString, PasswordHasher};
    use rand::rngs::OsRng;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError::Internal(format!("Failed to hash password: {e}")))
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_api_key() -> (String, String) {
    use rand::Rng;
    let random_bytes: [u8; 32] = rand::thread_rng().gen();
    let encoded = BASE64.encode(&random_bytes);
    let key = format!("wfk_live_{encoded}");
    let hash = hash_token(&key);
    (key, hash)
}

pub async fn login(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response, AuthError> {
    let auth = state.auth.as_ref().ok_or(AuthError::NotConfigured)?.clone();
    let user_repo = state.user_repo.as_ref().ok_or(AuthError::NotConfigured)?;

    // Support both old { password } and new { email, password } payloads
    let (email, password) = if let Some(obj) = payload.as_object() {
        if obj.contains_key("email") {
            let e = obj.get("email").and_then(|v| v.as_str()).unwrap_or("");
            let p = obj.get("password").and_then(|v| v.as_str()).unwrap_or("");
            (e.to_string(), p.to_string())
        } else {
            let p = obj.get("password").and_then(|v| v.as_str()).unwrap_or("");
            (String::new(), p.to_string())
        }
    } else {
        return Err(AuthError::InvalidCredentials);
    };

    if email.is_empty() {
        // Legacy single-password mode not supported in multi-user
        return Err(AuthError::InvalidCredentials);
    }

    let user = user_repo
        .find_by_email(&email)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    verify_password_hash(&password, &user.password_hash)?;

    if !user.email_verified {
        return Err(AuthError::EmailNotVerified);
    }

    let token = auth.issue_token(&user.id, &user.email)?;
    let ttl_secs = auth.expires_in().as_secs();
    let cookie_value = build_session_cookie(&token, ttl_secs, auth.should_secure_cookie(&headers));

    let body = LoginResponse {
        authenticated: true,
        expires_in: ttl_secs,
    };

    let mut response = Json(body).into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|e| AuthError::Internal(format!("Failed to set cookie: {e}")))?,
    );
    Ok(response)
}

pub async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let secure = state
        .auth
        .as_ref()
        .is_some_and(|a| a.should_secure_cookie(&headers));
    let cookie_value = build_session_cookie("", 0, secure);
    let mut response = StatusCode::NO_CONTENT.into_response();
    if let Ok(val) = HeaderValue::from_str(&cookie_value) {
        response.headers_mut().insert(SET_COOKIE, val);
    }
    response
}

pub async fn auth_me(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let Some(auth) = state.auth.clone() else {
        return Ok(Json(serde_json::json!({"authenticated": true})));
    };
    let token = extract_token(&request)?;
    let claims = auth.validate_token(&token)?;

    let mut response = serde_json::json!({
        "authenticated": true,
        "user": {
            "id": claims.sub,
            "email": claims.email,
        }
    });

    if let Some(user_repo) = &state.user_repo {
        if let Ok(Some(user)) = user_repo.find_by_id(&claims.sub).await {
            response["user"]["displayName"] = serde_json::json!(user.display_name);
            response["user"]["emailVerified"] = serde_json::json!(user.email_verified);
        }
    }

    Ok(Json(response))
}

pub async fn auth_status(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<AuthStatusResponse> {
    Json(AuthStatusResponse {
        requires_password: state.auth.is_some(),
        multi_user: state.user_repo.is_some(),
    })
}

pub async fn require_jwt(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    request.extensions_mut().insert(state.clone());

    let Some(auth) = state.auth.clone() else {
        return Ok(next.run(request).await);
    };

    let token = extract_token(&request)?;

    // Check for API key prefix
    if token.starts_with("wfk_") {
        let key_hash = hash_token(&token);
        let user_repo = state.user_repo.as_ref().ok_or(AuthError::Unauthorized)?;

        let api_key = user_repo
            .find_api_key_by_hash(&key_hash)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::Unauthorized)?;

        // Check expiry
        if let Some(expires) = api_key.expires_at {
            let now = chrono::Utc::now().naive_utc();
            if expires < now {
                return Err(AuthError::Unauthorized);
            }
        }

        // Load user
        let user = user_repo
            .find_by_id(&api_key.user_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::Unauthorized)?;

        if !user.email_verified {
            return Err(AuthError::EmailNotVerified);
        }

        // Update last used (fire and forget)
        let repo = state.user_repo.clone();
        let key_id = api_key.id.clone();
        tokio::spawn(async move {
            if let Some(r) = repo {
                let _ = r.update_api_key_last_used(&key_id).await;
            }
        });

        // Create synthetic claims for the API key user
        let authenticated_user = AuthenticatedUser {
            user_id: user.id.clone(),
            email: user.email.clone(),
        };
        request.extensions_mut().insert(authenticated_user);

        return Ok(next.run(request).await);
    }

    let claims = auth.validate_token(&token)?;

    // Check email verified
    if let Some(user_repo) = &state.user_repo {
        if let Ok(Some(user)) = user_repo.find_by_id(&claims.sub).await {
            if !user.email_verified {
                return Err(AuthError::EmailNotVerified);
            }
        }
    }

    let authenticated_user = AuthenticatedUser {
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
    };
    request.extensions_mut().insert(authenticated_user);

    // Sliding session: refresh the cookie when past 50% of TTL
    let needs_refresh = auth.should_refresh(&claims);
    let secure = needs_refresh.then(|| auth.should_secure_cookie(request.headers()));

    let mut response = next.run(request).await;

    if needs_refresh {
        if let Ok(new_token) = auth.issue_token(&claims.sub, &claims.email) {
            let ttl_secs = auth.expires_in().as_secs();
            let cookie = build_session_cookie(&new_token, ttl_secs, secure.unwrap_or(false));
            if let Ok(val) = HeaderValue::from_str(&cookie) {
                response.headers_mut().insert(SET_COOKIE, val);
            }
        }
    }

    Ok(response)
}

fn extract_token(request: &Request<Body>) -> Result<String, AuthError> {
    // 1. Authorization header (Bearer token or API key)
    if let Some(header_value) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        let mut parts = header_value.splitn(2, ' ');
        let (Some(scheme), Some(token)) = (parts.next(), parts.next()) else {
            return Err(AuthError::Unauthorized);
        };

        if !scheme.eq_ignore_ascii_case("Bearer") {
            return Err(AuthError::Unauthorized);
        }

        let token = token.trim();
        if token.is_empty() {
            return Err(AuthError::Unauthorized);
        }

        return Ok(token.to_string());
    }

    // 2. HttpOnly cookie (for SSE and page-refresh scenarios)
    if let Some(cookie_header) = request.headers().get(COOKIE).and_then(|v| v.to_str().ok()) {
        for pair in cookie_header.split(';') {
            if let Some((name, value)) = pair.trim().split_once('=') {
                if name.trim() == SESSION_COOKIE_NAME {
                    let value = value.trim();
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }
        }
    }

    Err(AuthError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager(policy: CookieSecurePolicy) -> AuthManager {
        let config = AuthConfig {
            jwt_secret: vec![0u8; 32],
            access_token_ttl: Duration::from_secs(3600),
            cookie_secure: policy,
        };
        AuthManager::new(&config).unwrap()
    }

    fn headers_with_proto(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-proto", HeaderValue::from_str(value).unwrap());
        h
    }

    #[test]
    fn auto_no_header_is_not_secure() {
        let mgr = make_manager(CookieSecurePolicy::Auto);
        assert!(!mgr.should_secure_cookie(&HeaderMap::new()));
    }

    #[test]
    fn auto_http_is_not_secure() {
        let mgr = make_manager(CookieSecurePolicy::Auto);
        assert!(!mgr.should_secure_cookie(&headers_with_proto("http")));
    }

    #[test]
    fn auto_https_is_secure() {
        let mgr = make_manager(CookieSecurePolicy::Auto);
        assert!(mgr.should_secure_cookie(&headers_with_proto("https")));
    }

    #[test]
    fn auto_https_case_insensitive() {
        let mgr = make_manager(CookieSecurePolicy::Auto);
        assert!(mgr.should_secure_cookie(&headers_with_proto("HTTPS")));
    }

    #[test]
    fn auto_multi_value_https_first() {
        let mgr = make_manager(CookieSecurePolicy::Auto);
        assert!(mgr.should_secure_cookie(&headers_with_proto("https, http")));
    }

    #[test]
    fn auto_multi_value_http_first() {
        let mgr = make_manager(CookieSecurePolicy::Auto);
        assert!(!mgr.should_secure_cookie(&headers_with_proto("http, https")));
    }

    #[test]
    fn always_without_header() {
        let mgr = make_manager(CookieSecurePolicy::Always);
        assert!(mgr.should_secure_cookie(&HeaderMap::new()));
    }

    #[test]
    fn always_with_http_header() {
        let mgr = make_manager(CookieSecurePolicy::Always);
        assert!(mgr.should_secure_cookie(&headers_with_proto("http")));
    }

    #[test]
    fn never_without_header() {
        let mgr = make_manager(CookieSecurePolicy::Never);
        assert!(!mgr.should_secure_cookie(&HeaderMap::new()));
    }

    #[test]
    fn never_with_https_header() {
        let mgr = make_manager(CookieSecurePolicy::Never);
        assert!(!mgr.should_secure_cookie(&headers_with_proto("https")));
    }

    #[test]
    fn issue_token_contains_user_id() {
        let mgr = make_manager(CookieSecurePolicy::Auto);
        let token = mgr.issue_token("user-123", "test@example.com").unwrap();
        let claims = mgr.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn hash_token_deterministic() {
        let h1 = hash_token("test-token");
        let h2 = hash_token("test-token");
        assert_eq!(h1, h2);
    }

    #[test]
    fn generate_api_key_has_prefix() {
        let (key, hash) = generate_api_key();
        assert!(key.starts_with("wfk_live_"));
        assert!(!hash.starts_with("wfk_"));
    }
}
