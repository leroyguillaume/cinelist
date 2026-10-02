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
use sqlx::PgConnection;
use tracing::{debug, info};
use uuid::Uuid;

use super::{
    PageQuery, PageResponse,
    lists::{self, ListPath},
    participants::ParticipantResponse,
};
use crate::{
    auth::CurrentUser,
    db::{
        self,
        films::FilmRow,
        participants::ParticipantRow,
        selections::{NewElimination, NewSelection, NewVerdict},
    },
    domain::{self, Outcome, Phase},
    error::{AppError, AppResult, ErrorCode},
    state::AppState,
    view::{self, ListSnapshot, SelectionSnapshot},
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/api/v1/lists/{listId}/selections",
            get_with(list_selections, |op| {
                op.tag("Selections")
                    .id("listSelections")
                    .summary("History of a list's selections, latest first")
            })
            .post_with(start_selection, |op| {
                op.tag("Selections")
                    .id("startSelection")
                    .summary("Start a selection")
                    .description(
                        "`filmCount - 1` must be a multiple of the number of participants, so that \
                         everyone plays as many turns. The participant after the previous selection's \
                         opener plays first.",
                    )
                    .response::<201, Json<SelectionResponse>>()
            }),
        )
        .api_route(
            "/api/v1/selections/{selectionId}",
            get_with(get_selection, |op| op.tag("Selections").id("getSelection").summary("Get a selection"))
                .delete_with(cancel_selection, |op| {
                    op.tag("Selections")
                        .id("cancelSelection")
                        .summary("Cancel a selection")
                        .description("Only before the first verdict; a cancelled selection leaves no history.")
                        .response::<204, ()>()
                }),
        )
        .api_route(
            "/api/v1/selections/{selectionId}/eliminations",
            post_with(eliminate, |op| {
                op.tag("Selections")
                    .id("createElimination")
                    .summary("Eliminate a film")
                    .description("Played on behalf of the participant whose turn it is. Returns the updated selection.")
                    .response::<201, Json<SelectionResponse>>()
            }),
        )
        .api_route(
            "/api/v1/selections/{selectionId}/eliminations/{eliminationId}",
            delete_with(undo_elimination, |op| {
                op.tag("Selections")
                    .id("deleteElimination")
                    .summary("Undo the latest elimination")
                    .description("Only the latest elimination, and only before the first verdict.")
                    .response::<204, ()>()
            }),
        )
        .api_route(
            "/api/v1/selections/{selectionId}/verdicts",
            post_with(judge, |op| {
                op.tag("Selections")
                    .id("createVerdict")
                    .summary("Say whether the chosen film was watched")
                    .description(
                        "Either way the film leaves the list. When it is abandoned, the film eliminated last \
                         becomes the candidate. Returns the updated selection.",
                    )
                    .response::<201, Json<SelectionResponse>>()
            }),
        )
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SelectionPath {
    selection_id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EliminationPath {
    selection_id: Uuid,
    elimination_id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateEliminationRequest {
    film_id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVerdictRequest {
    /// The current candidate; guards against judging twice.
    film_id: Uuid,
    outcome: OutcomeDto,
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum OutcomeDto {
    Watched,
    Abandoned,
}

impl From<OutcomeDto> for Outcome {
    fn from(outcome: OutcomeDto) -> Self {
        match outcome {
            OutcomeDto::Watched => Self::Watched,
            OutcomeDto::Abandoned => Self::Abandoned,
        }
    }
}

impl From<Outcome> for OutcomeDto {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Watched => Self::Watched,
            Outcome::Abandoned => Self::Abandoned,
        }
    }
}

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SelectionPhase {
    Eliminating,
    AwaitingVerdict,
    Completed,
}

impl From<Phase> for SelectionPhase {
    fn from(phase: Phase) -> Self {
        match phase {
            Phase::Eliminating => Self::Eliminating,
            Phase::AwaitingVerdict { .. } => Self::AwaitingVerdict,
            Phase::Completed => Self::Completed,
        }
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilmRefResponse {
    id: Uuid,
    title: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CandidateResponse {
    film_id: Uuid,
    title: String,
    place: i32,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EliminationResponse {
    id: Uuid,
    /// Absent once the film has left the list.
    film_id: Option<Uuid>,
    film_title: String,
    /// Absent once the participant has left the list.
    participant_id: Option<Uuid>,
    participant_name: String,
    place: i32,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerdictResponse {
    film_title: String,
    place: i32,
    outcome: OutcomeDto,
    decided_at: DateTime<Utc>,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SelectionResponse {
    id: Uuid,
    list_id: Uuid,
    starter_name: String,
    film_count: i32,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    phase: SelectionPhase,
    /// Whose turn it is, while films are being eliminated.
    current_participant: Option<ParticipantResponse>,
    /// Film waiting for a verdict.
    candidate: Option<CandidateResponse>,
    /// Films not eliminated yet, while the selection is running.
    remaining_films: Vec<FilmRefResponse>,
    /// Latest first.
    eliminations: Vec<EliminationResponse>,
    /// Best place first.
    verdicts: Vec<VerdictResponse>,
}

impl SelectionResponse {
    fn build(
        snapshot: &SelectionSnapshot,
        participants: &[ParticipantRow],
        films: &[FilmRow],
    ) -> Self {
        let phase = snapshot.phase();
        let running = phase != Phase::Completed;
        Self {
            id: snapshot.selection.id,
            list_id: snapshot.selection.list_id,
            starter_name: snapshot.selection.starter_name.clone(),
            film_count: snapshot.selection.film_count,
            started_at: snapshot.selection.started_at,
            completed_at: snapshot.selection.completed_at,
            phase: phase.into(),
            current_participant: snapshot
                .current_participant(participants)
                .map(ParticipantResponse::from),
            candidate: snapshot
                .candidate(films)
                .map(|candidate| CandidateResponse {
                    film_id: candidate.film_id,
                    title: candidate.title.to_owned(),
                    place: candidate.place,
                }),
            remaining_films: if running {
                snapshot
                    .remaining_films(films)
                    .into_iter()
                    .map(|film| FilmRefResponse {
                        id: film.id,
                        title: film.title.clone(),
                    })
                    .collect()
            } else {
                Vec::new()
            },
            eliminations: snapshot
                .eliminations
                .iter()
                .map(|row| EliminationResponse {
                    id: row.id,
                    film_id: row.film_id,
                    film_title: row.film_title.clone(),
                    participant_id: row.participant_id,
                    participant_name: row.participant_name.clone(),
                    place: row.place,
                })
                .collect(),
            verdicts: snapshot
                .verdicts
                .iter()
                .filter_map(|row| {
                    Some(VerdictResponse {
                        film_title: row.film_title.clone(),
                        place: row.place,
                        outcome: Outcome::from_db(&row.outcome)?.into(),
                        decided_at: row.created_at,
                    })
                })
                .collect(),
        }
    }
}

/// Locks the list holding `selection_id` and returns it with that selection,
/// which must be the one running.
async fn lock_active(
    conn: &mut PgConnection,
    selection_id: Uuid,
    user: &CurrentUser,
    not_running: ErrorCode,
) -> AppResult<ListSnapshot> {
    let list_id = db::selections::list_id_for_owner(&mut *conn, selection_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let snapshot = lists::lock(conn, list_id, user).await?;
    match &snapshot.active_selection {
        Some(active) if active.selection.id == selection_id => Ok(snapshot),
        _ => Err(AppError::conflict(not_running)),
    }
}

/// The running selection of a list locked by [`lock_active`].
fn active(snapshot: &ListSnapshot) -> &SelectionSnapshot {
    snapshot
        .active_selection
        .as_ref()
        .expect("lock_active only returns lists running a selection")
}

async fn reload(
    conn: &mut PgConnection,
    list_id: Uuid,
    selection_id: Uuid,
    user: &CurrentUser,
) -> AppResult<SelectionResponse> {
    let list = db::lists::find_for_owner(&mut *conn, list_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let snapshot = view::load_list(&mut *conn, list).await?;
    let selection = db::selections::find(&mut *conn, selection_id, list_id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let selection = view::load_selections(conn, vec![selection])
        .await?
        .remove(0);
    Ok(SelectionResponse::build(
        &selection,
        &snapshot.participants,
        &snapshot.films,
    ))
}

async fn list_selections(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ListPath>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<PageResponse<SelectionResponse>>> {
    let pagination = query.pagination()?;
    let mut conn = state.pool.acquire().await?;
    let list = db::lists::find_for_owner(&mut *conn, path.list_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let snapshot = view::load_list(&mut conn, list).await?;
    let rows = db::selections::page_for_list(
        &mut *conn,
        path.list_id,
        pagination.limit(),
        pagination.offset(),
    )
    .await?;
    let total = db::selections::count_for_list(&mut *conn, path.list_id).await?;
    let items = view::load_selections(&mut conn, rows)
        .await?
        .iter()
        .map(|selection| {
            SelectionResponse::build(selection, &snapshot.participants, &snapshot.films)
        })
        .collect();
    Ok(Json(pagination.wrap(items, total)))
}

async fn start_selection(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ListPath>,
) -> AppResult<(StatusCode, Json<SelectionResponse>)> {
    let mut tx = state.pool.begin().await?;
    let snapshot = lists::lock(&mut tx, path.list_id, &user).await?;
    lists::ensure_idle(&snapshot)?;
    if !domain::is_playable(snapshot.films.len(), snapshot.participants.len()) {
        return Err(AppError::conflict(ErrorCode::UnevenFilmCount));
    }
    let starter = snapshot.next_starter().ok_or_else(AppError::not_found)?;
    let id = Uuid::new_v4();
    db::selections::create(
        &mut *tx,
        &NewSelection {
            id,
            list_id: snapshot.list.id,
            starter_id: starter.id,
            starter_name: &starter.name,
            film_count: snapshot.film_count(),
        },
    )
    .await?;
    let response = reload(&mut tx, snapshot.list.id, id, &user).await?;
    tx.commit().await?;
    info!(user_id = %user.id, list_id = %snapshot.list.id, selection_id = %id, starter_id = %starter.id, "selection started");
    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_selection(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<SelectionPath>,
) -> AppResult<Json<SelectionResponse>> {
    let mut conn = state.pool.acquire().await?;
    let list_id = db::selections::list_id_for_owner(&mut *conn, path.selection_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    Ok(Json(
        reload(&mut conn, list_id, path.selection_id, &user).await?,
    ))
}

async fn cancel_selection(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<SelectionPath>,
) -> AppResult<StatusCode> {
    let mut tx = state.pool.begin().await?;
    let snapshot =
        lock_active(&mut tx, path.selection_id, &user, ErrorCode::VerdictStarted).await?;
    if !active(&snapshot).verdicts.is_empty() {
        return Err(AppError::conflict(ErrorCode::VerdictStarted));
    }
    db::selections::delete(&mut *tx, path.selection_id).await?;
    tx.commit().await?;
    info!(user_id = %user.id, selection_id = %path.selection_id, "selection cancelled");
    Ok(StatusCode::NO_CONTENT)
}

async fn eliminate(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<SelectionPath>,
    Json(request): Json<CreateEliminationRequest>,
) -> AppResult<(StatusCode, Json<SelectionResponse>)> {
    let mut tx = state.pool.begin().await?;
    let snapshot =
        lock_active(&mut tx, path.selection_id, &user, ErrorCode::NotEliminating).await?;
    let selection = active(&snapshot);
    if selection.phase() != Phase::Eliminating {
        return Err(AppError::conflict(ErrorCode::NotEliminating));
    }
    let Some(film) = snapshot
        .films
        .iter()
        .find(|film| film.id == request.film_id)
    else {
        return Err(AppError::not_found());
    };
    if !selection
        .remaining_films(&snapshot.films)
        .iter()
        .any(|remaining| remaining.id == film.id)
    {
        return Err(AppError::conflict(ErrorCode::FilmAlreadyEliminated));
    }
    let participant = selection
        .current_participant(&snapshot.participants)
        .ok_or_else(|| AppError::conflict(ErrorCode::NotEliminating))?;
    let place =
        domain::elimination_place(selection.selection.film_count, selection.eliminations.len());
    db::selections::create_elimination(
        &mut *tx,
        &NewElimination {
            id: Uuid::new_v4(),
            selection_id: path.selection_id,
            film_id: film.id,
            film_title: &film.title,
            participant_id: participant.id,
            participant_name: &participant.name,
            place,
        },
    )
    .await?;
    let response = reload(&mut tx, snapshot.list.id, path.selection_id, &user).await?;
    tx.commit().await?;
    debug!(selection_id = %path.selection_id, film_id = %film.id, participant_id = %participant.id, place, "film eliminated");
    Ok((StatusCode::CREATED, Json(response)))
}

async fn undo_elimination(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<EliminationPath>,
) -> AppResult<StatusCode> {
    let mut tx = state.pool.begin().await?;
    let snapshot =
        lock_active(&mut tx, path.selection_id, &user, ErrorCode::VerdictStarted).await?;
    let selection = active(&snapshot);
    if !selection.verdicts.is_empty() {
        return Err(AppError::conflict(ErrorCode::VerdictStarted));
    }
    match selection.eliminations.first() {
        Some(latest) if latest.id == path.elimination_id => {}
        _ if selection
            .eliminations
            .iter()
            .any(|row| row.id == path.elimination_id) =>
        {
            return Err(AppError::conflict(ErrorCode::NotLatestElimination));
        }
        _ => return Err(AppError::not_found()),
    }
    db::selections::delete_elimination(&mut *tx, path.elimination_id).await?;
    tx.commit().await?;
    debug!(selection_id = %path.selection_id, elimination_id = %path.elimination_id, "elimination undone");
    Ok(StatusCode::NO_CONTENT)
}

async fn judge(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<SelectionPath>,
    Json(request): Json<CreateVerdictRequest>,
) -> AppResult<(StatusCode, Json<SelectionResponse>)> {
    let mut tx = state.pool.begin().await?;
    let snapshot = lock_active(
        &mut tx,
        path.selection_id,
        &user,
        ErrorCode::NotAwaitingVerdict,
    )
    .await?;
    let selection = active(&snapshot);
    let candidate = selection
        .candidate(&snapshot.films)
        .filter(|candidate| candidate.film_id == request.film_id)
        .ok_or_else(|| AppError::conflict(ErrorCode::NotAwaitingVerdict))?;
    let outcome = Outcome::from(request.outcome);
    db::selections::create_verdict(
        &mut *tx,
        &NewVerdict {
            id: Uuid::new_v4(),
            selection_id: path.selection_id,
            film_id: candidate.film_id,
            film_title: candidate.title,
            place: candidate.place,
            outcome: outcome.as_str(),
        },
    )
    .await?;
    db::films::delete(&mut *tx, candidate.film_id, snapshot.list.id).await?;
    let mut outcomes: Vec<Outcome> = selection
        .verdicts
        .iter()
        .filter_map(|verdict| Outcome::from_db(&verdict.outcome))
        .collect();
    outcomes.push(outcome);
    let completed = domain::phase(
        selection.selection.film_count,
        selection.eliminations.len(),
        &outcomes,
    ) == Phase::Completed;
    if completed {
        db::selections::complete(&mut *tx, path.selection_id).await?;
    }
    let response = reload(&mut tx, snapshot.list.id, path.selection_id, &user).await?;
    tx.commit().await?;
    info!(
        user_id = %user.id,
        selection_id = %path.selection_id,
        film_id = %candidate.film_id,
        outcome = outcome.as_str(),
        place = candidate.place,
        completed,
        "verdict given"
    );
    Ok((StatusCode::CREATED, Json(response)))
}
