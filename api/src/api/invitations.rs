use aide::axum::{
    ApiRouter,
    routing::{get_with, put_with},
};
use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

use super::sessions::{CreatedSessionResponse, open_session};
use crate::{
    auth, db,
    error::{AppError, AppResult},
    state::AppState,
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/api/v1/invitations/{token}",
            get_with(get_invitation, |op| {
                op.tag("Invitations")
                    .id("getInvitation")
                    .summary("Look up a pending invitation")
            }),
        )
        .api_route(
            "/api/v1/invitations/{token}/acceptance",
            put_with(accept_invitation, |op| {
                op.tag("Invitations")
                    .id("acceptInvitation")
                    .summary("Accept an invitation")
                    .description(
                        "Sets the account's password, revokes every token issued to it so far and signs in. \
                         An invitation can be accepted once.",
                    )
            }),
        )
}

#[derive(Deserialize, JsonSchema)]
pub struct InvitationPath {
    /// Secret from the invitation link.
    token: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitationResponse {
    username: String,
    expires_at: DateTime<Utc>,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInvitationRequest {
    #[validate(length(min = 8, max = 128))]
    password: String,
}

async fn get_invitation(
    State(state): State<AppState>,
    Path(path): Path<InvitationPath>,
) -> AppResult<Json<InvitationResponse>> {
    let invitation = db::invitations::find_pending(&state.pool, &auth::hash_token(&path.token))
        .await?
        .ok_or_else(AppError::not_found)?;
    Ok(Json(InvitationResponse {
        username: invitation.username,
        expires_at: invitation.expires_at,
    }))
}

async fn accept_invitation(
    State(state): State<AppState>,
    Path(path): Path<InvitationPath>,
    jar: CookieJar,
    Json(request): Json<AcceptInvitationRequest>,
) -> AppResult<(CookieJar, Json<CreatedSessionResponse>)> {
    request.validate()?;
    let password_hash = auth::hash_password(request.password).await?;
    let mut tx = state.pool.begin().await?;
    let invitation = db::invitations::find_pending(&mut *tx, &auth::hash_token(&path.token))
        .await?
        .ok_or_else(AppError::not_found)?;
    db::users::set_password(&mut *tx, invitation.user_id, &password_hash).await?;
    db::invitations::mark_used(&mut *tx, invitation.id).await?;
    db::users::bump_token_version(&mut *tx, invitation.user_id).await?;
    let user = db::users::find(&mut *tx, invitation.user_id)
        .await?
        .ok_or_else(AppError::not_found)?;
    tx.commit().await?;
    info!(user_id = %user.id, "invitation accepted");
    let (jar, response) = open_session(&state, jar, user)?;
    Ok((jar, Json(response)))
}
