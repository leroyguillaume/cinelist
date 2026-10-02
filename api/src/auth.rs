use aide::{
    OperationInput,
    generate::GenContext,
    openapi::{Operation, SecurityRequirement},
};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, header, request::Parts},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::debug;
use uuid::Uuid;

use crate::{
    db::{self, users::UserRow},
    error::{AppError, AppResult},
    state::{AppState, JwtKeys},
};

pub const TOKEN_COOKIE: &str = "cinelist_token";
pub const COOKIE_SCHEME: &str = "TokenCookie";
pub const BEARER_SCHEME: &str = "BearerAuth";

/// A freshly minted secret: `token` goes to the client once, only `hash` is stored.
pub struct Token {
    pub token: String,
    pub hash: Vec<u8>,
}

impl Token {
    pub fn generate() -> Self {
        let mut bytes = [0_u8; 32];
        rand::fill(&mut bytes);
        let token = URL_SAFE_NO_PAD.encode(bytes);
        let hash = hash_token(&token);
        Self { token, hash }
    }
}

pub fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// Argon2 is deliberately slow, so it runs off the async workers.
pub async fn hash_password(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(AppError::PasswordHash)
    })
    .await
    .expect("password hashing task panicked")
}

pub async fn verify_password(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .verify_password(password.as_bytes(), hash.as_str())
            .is_ok()
    })
    .await
    .expect("password verification task panicked")
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    /// The user's `token_version` when the token was issued.
    ver: i32,
    iat: i64,
    exp: i64,
}

pub struct IssuedToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub fn issue_token(
    keys: &JwtKeys,
    ttl: Duration,
    user_id: Uuid,
    token_version: i32,
) -> AppResult<IssuedToken> {
    let now = Utc::now();
    let expires_at = now + ttl;
    let claims = Claims {
        sub: user_id,
        ver: token_version,
        iat: now.timestamp(),
        exp: expires_at.timestamp(),
    };
    let token = jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &keys.encoding)
        .map_err(AppError::Jwt)?;
    Ok(IssuedToken { token, expires_at })
}

pub fn token_cookie(token: String, ttl: Duration, secure: bool) -> Cookie<'static> {
    Cookie::build((TOKEN_COOKIE, token))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(ttl.num_seconds()))
        .build()
}

pub fn removal_cookie() -> Cookie<'static> {
    Cookie::build((TOKEN_COOKIE, "")).path("/").build()
}

/// The `Authorization: Bearer` token, else the token cookie.
fn presented_token(headers: &HeaderMap) -> Option<String> {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token.trim().to_owned());
    bearer.or_else(|| {
        CookieJar::from_headers(headers)
            .get(TOKEN_COOKIE)
            .map(|cookie| cookie.value().to_owned())
    })
}

/// The signed-in user.
pub struct CurrentUser {
    pub id: Uuid,
    pub username: String,
    pub is_admin: bool,
}

impl From<UserRow> for CurrentUser {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            is_admin: row.is_admin,
        }
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(token) = presented_token(&parts.headers) else {
            debug!("no token");
            return Err(AppError::unauthenticated());
        };
        let validation = Validation::new(Algorithm::HS256);
        let claims =
            match jsonwebtoken::decode::<Claims>(&token, &state.settings.jwt.decoding, &validation)
            {
                Ok(data) => data.claims,
                Err(err) => {
                    debug!(error = %err, "invalid token");
                    return Err(AppError::unauthenticated());
                }
            };
        let Some(user) = db::users::find(&state.pool, claims.sub).await? else {
            debug!(user_id = %claims.sub, "token of a deleted user");
            return Err(AppError::unauthenticated());
        };
        if user.token_version != claims.ver {
            debug!(user_id = %user.id, "revoked token");
            return Err(AppError::unauthenticated());
        }
        Ok(user.into())
    }
}

impl OperationInput for CurrentUser {
    fn operation_input(_ctx: &mut GenContext, operation: &mut Operation) {
        require_token(operation);
    }
}

/// The signed-in user, who must be the administrator.
pub struct Admin(pub CurrentUser);

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = CurrentUser::from_request_parts(parts, state).await?;
        if user.is_admin {
            Ok(Self(user))
        } else {
            Err(AppError::forbidden())
        }
    }
}

impl OperationInput for Admin {
    fn operation_input(_ctx: &mut GenContext, operation: &mut Operation) {
        require_token(operation);
    }
}

/// Either scheme is enough: `OpenAPI` reads separate requirements as alternatives.
fn require_token(operation: &mut Operation) {
    for scheme in [BEARER_SCHEME, COOKIE_SCHEME] {
        let mut requirement = SecurityRequirement::new();
        requirement.insert(scheme.to_owned(), Vec::new());
        operation.security.push(requirement);
    }
}
