use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::{self, CurrentUser},
    db::{self, users::UserRow},
    error::{AppError, AppResult, ErrorCode},
    state::AppState,
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/api/v1/sessions",
            post_with(create_session, |op| {
                op.tag("Sessions")
                    .id("createSession")
                    .summary("Sign in")
                    .description(
                        "Returns a JWT and sets it as the token cookie as well. Send it back either as \
                         `Authorization: Bearer <token>` or through the cookie.",
                    )
                    .response::<201, Json<CreatedSessionResponse>>()
            }),
        )
        .api_route(
            "/api/v1/sessions/current",
            get_with(current_session, |op| {
                op.tag("Sessions")
                    .id("getCurrentSession")
                    .summary("Who is signed in")
            })
            .delete_with(delete_session, |op| {
                op.tag("Sessions")
                    .id("deleteCurrentSession")
                    .summary("Sign out")
                    .description(
                        "Clears the token cookie. A JWT stays valid until it expires; accepting a new \
                         invitation revokes all of the user's tokens.",
                    )
                    .response::<204, ()>()
            }),
        )
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    #[validate(length(min = 1, max = 64))]
    username: String,
    #[validate(length(min = 1, max = 128))]
    password: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    user: SessionUserResponse,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionUserResponse {
    id: Uuid,
    username: String,
    is_admin: bool,
}

/// A signed-in session and its token.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatedSessionResponse {
    user: SessionUserResponse,
    /// JWT to send as `Authorization: Bearer <token>`; also set as a cookie.
    token: String,
    expires_at: DateTime<Utc>,
}

impl From<CurrentUser> for SessionUserResponse {
    fn from(user: CurrentUser) -> Self {
        Self {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
        }
    }
}

impl From<CurrentUser> for SessionResponse {
    fn from(user: CurrentUser) -> Self {
        Self { user: user.into() }
    }
}

async fn create_session(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<CreateSessionRequest>,
) -> AppResult<(StatusCode, CookieJar, Json<CreatedSessionResponse>)> {
    request.validate()?;
    let credentials = db::users::find_credentials(&state.pool, request.username.trim()).await?;
    let Some((user, hash)) = credentials.and_then(|row| {
        let hash = row.password_hash.clone()?;
        Some((row, hash))
    }) else {
        debug!("sign-in with an unknown or inactive username");
        return Err(AppError::unprocessable(ErrorCode::InvalidCredentials));
    };
    if !auth::verify_password(request.password, hash).await {
        debug!(user_id = %user.id, "sign-in with a wrong password");
        return Err(AppError::unprocessable(ErrorCode::InvalidCredentials));
    }
    info!(user_id = %user.id, "user signed in");
    let user = UserRow {
        id: user.id,
        username: user.username,
        is_admin: user.is_admin,
        token_version: user.token_version,
    };
    let (jar, response) = open_session(&state, jar, user)?;
    Ok((StatusCode::CREATED, jar, Json(response)))
}

/// Issues a token for `user` and sets it as the token cookie of `jar`.
pub fn open_session(
    state: &AppState,
    jar: CookieJar,
    user: UserRow,
) -> AppResult<(CookieJar, CreatedSessionResponse)> {
    let settings = &state.settings;
    let issued = auth::issue_token(
        &settings.jwt,
        settings.token_ttl,
        user.id,
        user.token_version,
    )?;
    let jar = jar.add(auth::token_cookie(
        issued.token.clone(),
        settings.token_ttl,
        settings.cookie_secure,
    ));
    let response = CreatedSessionResponse {
        user: CurrentUser::from(user).into(),
        token: issued.token,
        expires_at: issued.expires_at,
    };
    Ok((jar, response))
}

async fn current_session(user: CurrentUser) -> Json<SessionResponse> {
    Json(user.into())
}

async fn delete_session(user: CurrentUser, jar: CookieJar) -> (StatusCode, CookieJar) {
    info!(user_id = %user.id, "user signed out");
    (StatusCode::NO_CONTENT, jar.remove(auth::removal_cookie()))
}
