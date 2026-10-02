use aide::axum::{
    ApiRouter,
    routing::{delete_with, get_with, post_with},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use super::{PageQuery, PageResponse};
use crate::{
    auth::{Admin, Token},
    db::{self, users::UserSummaryRow},
    error::{AppError, AppResult, ErrorCode},
    state::AppState,
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/api/v1/users",
            get_with(list_users, |op| op.tag("Users").id("listUsers").summary("List users")).post_with(
                create_user,
                |op| {
                    op.tag("Users")
                        .id("createUser")
                        .summary("Create a user")
                        .description("Returns the token of the invitation the new user activates their account with.")
                        .response::<201, Json<CreatedUserResponse>>()
                },
            ),
        )
        .api_route(
            "/api/v1/users/{userId}",
            delete_with(delete_user, |op| {
                op.tag("Users")
                    .id("deleteUser")
                    .summary("Delete a user and their lists")
                    .response::<204, ()>()
            }),
        )
        .api_route(
            "/api/v1/users/{userId}/invitations",
            post_with(create_invitation, |op| {
                op.tag("Users")
                    .id("createInvitation")
                    .summary("Issue a new invitation")
                    .description(
                        "Replaces any pending invitation of the user. Accepting it sets a new password, \
                         which is how a forgotten password is reset.",
                    )
                    .response::<201, Json<InvitationTokenResponse>>()
            }),
        )
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserPath {
    user_id: Uuid,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    #[validate(length(min = 2, max = 32), custom(function = valid_username))]
    #[schemars(extend("example" = "camille"))]
    username: String,
}

fn valid_username(username: &str) -> Result<(), validator::ValidationError> {
    if username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        Ok(())
    } else {
        Err(validator::ValidationError::new("username_charset"))
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum UserStatus {
    /// The user has a password and no pending invitation.
    Active,
    /// An invitation waits to be accepted.
    Invited,
    /// The user never set a password and their invitation expired.
    InvitationExpired,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    id: Uuid,
    username: String,
    is_admin: bool,
    status: UserStatus,
    /// Expiry of the pending invitation, when there is one.
    invitation_expires_at: Option<DateTime<Utc>>,
    list_count: i64,
}

impl UserResponse {
    fn from_row(row: UserSummaryRow, now: DateTime<Utc>) -> Self {
        let pending = row
            .invitation_expires_at
            .filter(|expires_at| *expires_at > now);
        let status = match (pending, row.has_password) {
            (Some(_), _) => UserStatus::Invited,
            (None, true) => UserStatus::Active,
            (None, false) => UserStatus::InvitationExpired,
        };
        Self {
            id: row.id,
            username: row.username,
            is_admin: row.is_admin,
            status,
            invitation_expires_at: pending,
            list_count: row.list_count,
        }
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitationTokenResponse {
    /// Secret to put in the invitation link. It is shown only once.
    token: String,
    expires_at: DateTime<Utc>,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatedUserResponse {
    user: UserResponse,
    invitation: InvitationTokenResponse,
}

async fn list_users(
    State(state): State<AppState>,
    _admin: Admin,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<PageResponse<UserResponse>>> {
    let pagination = query.pagination()?;
    let rows = db::users::page(&state.pool, pagination.limit(), pagination.offset()).await?;
    let total = db::users::count(&state.pool).await?;
    let now = Utc::now();
    let items = rows
        .into_iter()
        .map(|row| UserResponse::from_row(row, now))
        .collect();
    Ok(Json(pagination.wrap(items, total)))
}

async fn create_user(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<CreatedUserResponse>)> {
    request.validate()?;
    let id = Uuid::new_v4();
    let mut tx = state.pool.begin().await?;
    if !db::users::create(&mut *tx, id, &request.username).await? {
        return Err(AppError::conflict(ErrorCode::UsernameTaken));
    }
    let invitation = issue_invitation(&state, &mut tx, id).await?;
    tx.commit().await?;
    info!(admin_id = %admin.id, user_id = %id, "user created");
    let user = UserResponse {
        id,
        username: request.username,
        is_admin: false,
        status: UserStatus::Invited,
        invitation_expires_at: Some(invitation.expires_at),
        list_count: 0,
    };
    Ok((
        StatusCode::CREATED,
        Json(CreatedUserResponse { user, invitation }),
    ))
}

async fn delete_user(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(path): Path<UserPath>,
) -> AppResult<StatusCode> {
    let user = db::users::find(&state.pool, path.user_id)
        .await?
        .ok_or_else(AppError::not_found)?;
    if user.is_admin {
        return Err(AppError::conflict(ErrorCode::AdminManaged));
    }
    db::users::delete(&state.pool, user.id).await?;
    info!(admin_id = %admin.id, user_id = %user.id, "user deleted");
    Ok(StatusCode::NO_CONTENT)
}

async fn create_invitation(
    State(state): State<AppState>,
    Admin(admin): Admin,
    Path(path): Path<UserPath>,
) -> AppResult<(StatusCode, Json<InvitationTokenResponse>)> {
    let user = db::users::find(&state.pool, path.user_id)
        .await?
        .ok_or_else(AppError::not_found)?;
    if user.is_admin {
        return Err(AppError::conflict(ErrorCode::AdminManaged));
    }
    let mut tx = state.pool.begin().await?;
    let invitation = issue_invitation(&state, &mut tx, user.id).await?;
    tx.commit().await?;
    info!(admin_id = %admin.id, user_id = %user.id, "invitation issued");
    Ok((StatusCode::CREATED, Json(invitation)))
}

async fn issue_invitation(
    state: &AppState,
    tx: &mut sqlx::PgConnection,
    user_id: Uuid,
) -> AppResult<InvitationTokenResponse> {
    let token = Token::generate();
    let expires_at = Utc::now() + state.settings.invitation_ttl;
    db::invitations::revoke_pending(&mut *tx, user_id).await?;
    db::invitations::create(&mut *tx, user_id, &token.hash, expires_at).await?;
    Ok(InvitationTokenResponse {
        token: token.token,
        expires_at,
    })
}
