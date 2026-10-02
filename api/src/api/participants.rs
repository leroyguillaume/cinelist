use aide::axum::{
    ApiRouter,
    routing::{post_with, put_with},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use super::lists::{self, ListPath};
use crate::{
    auth::CurrentUser,
    db::{self, participants::ParticipantRow},
    domain,
    error::{AppError, AppResult, ErrorCode},
    state::AppState,
};

const MAX_NAME_LENGTH: usize = 40;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/api/v1/lists/{listId}/participants",
            post_with(create_participant, |op| {
                op.tag("Participants")
                    .id("createParticipant")
                    .summary("Add a participant")
                    .description("The participant plays last in turn order.")
                    .response::<201, Json<ParticipantResponse>>()
            }),
        )
        .api_route(
            "/api/v1/participants/{participantId}",
            put_with(rename_participant, |op| {
                op.tag("Participants")
                    .id("updateParticipant")
                    .summary("Rename a participant")
            })
            .delete_with(delete_participant, |op| {
                op.tag("Participants")
                    .id("deleteParticipant")
                    .summary("Remove a participant")
                    .description("Past eliminations keep the participant's name.")
                    .response::<204, ()>()
            }),
        )
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantPath {
    participant_id: Uuid,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateParticipantRequest {
    #[validate(custom(function = valid_participant_name))]
    #[schemars(extend("example" = "Noé"))]
    name: String,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateParticipantRequest {
    #[validate(custom(function = valid_participant_name))]
    name: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantResponse {
    id: Uuid,
    name: String,
}

impl From<&ParticipantRow> for ParticipantResponse {
    fn from(row: &ParticipantRow) -> Self {
        Self {
            id: row.id,
            name: row.name.clone(),
        }
    }
}

pub fn valid_participant_name(name: &str) -> Result<(), ValidationError> {
    super::not_blank(name)?;
    if name.trim().chars().count() > MAX_NAME_LENGTH {
        return Err(ValidationError::new("too_long"));
    }
    Ok(())
}

async fn create_participant(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ListPath>,
    Json(request): Json<CreateParticipantRequest>,
) -> AppResult<(StatusCode, Json<ParticipantResponse>)> {
    request.validate()?;
    let mut tx = state.pool.begin().await?;
    let snapshot = lists::lock(&mut tx, path.list_id, &user).await?;
    lists::ensure_idle(&snapshot)?;
    let count = snapshot.participants.len() + 1;
    if count > domain::MAX_PARTICIPANTS {
        return Err(AppError::unprocessable(ErrorCode::TooManyParticipants));
    }
    let id = Uuid::new_v4();
    let name = request.name.trim();
    db::participants::create(&mut *tx, id, snapshot.list.id, name).await?;
    tx.commit().await?;
    info!(user_id = %user.id, list_id = %snapshot.list.id, participant_id = %id, "participant added");
    Ok((
        StatusCode::CREATED,
        Json(ParticipantResponse {
            id,
            name: name.to_owned(),
        }),
    ))
}

async fn rename_participant(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ParticipantPath>,
    Json(request): Json<UpdateParticipantRequest>,
) -> AppResult<Json<ParticipantResponse>> {
    request.validate()?;
    let list_id = db::participants::list_id_for_owner(&state.pool, path.participant_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let name = request.name.trim();
    if !db::participants::rename(&state.pool, path.participant_id, list_id, name).await? {
        return Err(AppError::not_found());
    }
    Ok(Json(ParticipantResponse {
        id: path.participant_id,
        name: name.to_owned(),
    }))
}

async fn delete_participant(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ParticipantPath>,
) -> AppResult<StatusCode> {
    let list_id = db::participants::list_id_for_owner(&state.pool, path.participant_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let mut tx = state.pool.begin().await?;
    let snapshot = lists::lock(&mut tx, list_id, &user).await?;
    lists::ensure_idle(&snapshot)?;
    if !snapshot
        .participants
        .iter()
        .any(|participant| participant.id == path.participant_id)
    {
        return Err(AppError::not_found());
    }
    let count = snapshot.participants.len() - 1;
    if count == 0 {
        return Err(AppError::conflict(ErrorCode::LastParticipant));
    }
    db::participants::delete(&mut *tx, path.participant_id, list_id).await?;
    tx.commit().await?;
    info!(user_id = %user.id, %list_id, participant_id = %path.participant_id, "participant removed");
    Ok(StatusCode::NO_CONTENT)
}
