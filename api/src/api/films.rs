use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with, put_with},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use super::lists::{self, ListPath};
use crate::{
    auth::CurrentUser,
    db::{self, films::FilmRow},
    domain::{self, Outcome, Run},
    error::{AppError, AppResult, ErrorCode},
    state::AppState,
};

const MAX_TITLE_LENGTH: usize = 200;
/// Runs taken into account by the statistics; far more than a list ever plays.
const MAX_RUNS: i64 = 1000;
const RECENT_RUNS: usize = 20;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/api/v1/lists/{listId}/films",
            post_with(create_film, |op| {
                op.tag("Films")
                    .id("createFilm")
                    .summary("Add a film")
                    .response::<201, Json<FilmResponse>>()
            }),
        )
        .api_route(
            "/api/v1/films/{filmId}",
            put_with(rename_film, |op| {
                op.tag("Films").id("updateFilm").summary("Rename a film")
            })
            .delete_with(delete_film, |op| {
                op.tag("Films")
                    .id("deleteFilm")
                    .summary("Remove a film")
                    .description("Past selections keep the film's title.")
                    .response::<204, ()>()
            }),
        )
        .api_route(
            "/api/v1/films/{filmId}/statistics",
            get_with(get_statistics, |op| {
                op.tag("Films")
                    .id("getFilmStatistics")
                    .summary("How a film fared in past selections")
            }),
        )
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilmPath {
    film_id: Uuid,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFilmRequest {
    #[validate(custom(function = valid_title))]
    #[schemars(extend("example" = "In the Mood for Love"))]
    title: String,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFilmRequest {
    #[validate(custom(function = valid_title))]
    title: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilmResponse {
    id: Uuid,
    title: String,
    added_at: DateTime<Utc>,
}

impl From<&FilmRow> for FilmResponse {
    fn from(row: &FilmRow) -> Self {
        Self {
            id: row.id,
            title: row.title.clone(),
            added_at: row.created_at,
        }
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EliminatorCountResponse {
    participant_name: String,
    count: usize,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunResponse {
    selection_id: Uuid,
    started_at: DateTime<Utc>,
    place: i32,
    film_count: i32,
    eliminated_by: String,
    /// Film watched at the end of that selection, if any.
    winner_title: Option<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FilmStatisticsResponse {
    film: FilmResponse,
    selection_count: usize,
    best_place: Option<i32>,
    worst_place: Option<i32>,
    /// Mean place; absent when the selections did not all have the same number of films.
    average_place: Option<f64>,
    /// Mean relative standing, from 0 (always last) to 1 (always first).
    average_standing: Option<f64>,
    /// Selections the film finished second in.
    finals_lost: usize,
    /// Most frequent first.
    eliminated_by: Vec<EliminatorCountResponse>,
    /// Latest selections first, twenty at most.
    recent_runs: Vec<RunResponse>,
}

pub fn valid_title(title: &str) -> Result<(), ValidationError> {
    super::not_blank(title)?;
    if title.trim().chars().count() > MAX_TITLE_LENGTH {
        return Err(ValidationError::new("too_long"));
    }
    Ok(())
}

async fn create_film(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ListPath>,
    Json(request): Json<CreateFilmRequest>,
) -> AppResult<(StatusCode, Json<FilmResponse>)> {
    request.validate()?;
    let mut tx = state.pool.begin().await?;
    let snapshot = lists::lock(&mut tx, path.list_id, &user).await?;
    lists::ensure_idle(&snapshot)?;
    if snapshot.films.len() >= domain::MAX_FILMS {
        return Err(AppError::conflict(ErrorCode::ListFull));
    }
    let id = Uuid::new_v4();
    db::films::create(&mut *tx, id, snapshot.list.id, request.title.trim()).await?;
    let film = db::films::find(&mut *tx, id, snapshot.list.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    tx.commit().await?;
    info!(user_id = %user.id, list_id = %snapshot.list.id, film_id = %id, "film added");
    Ok((StatusCode::CREATED, Json((&film).into())))
}

async fn rename_film(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<FilmPath>,
    Json(request): Json<UpdateFilmRequest>,
) -> AppResult<Json<FilmResponse>> {
    request.validate()?;
    let list_id = db::films::list_id_for_owner(&state.pool, path.film_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    if !db::films::rename(&state.pool, path.film_id, list_id, request.title.trim()).await? {
        return Err(AppError::not_found());
    }
    let film = db::films::find(&state.pool, path.film_id, list_id)
        .await?
        .ok_or_else(AppError::not_found)?;
    Ok(Json((&film).into()))
}

async fn delete_film(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<FilmPath>,
) -> AppResult<StatusCode> {
    let list_id = db::films::list_id_for_owner(&state.pool, path.film_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let mut tx = state.pool.begin().await?;
    let snapshot = lists::lock(&mut tx, list_id, &user).await?;
    lists::ensure_idle(&snapshot)?;
    if snapshot.films.len() <= 1 {
        return Err(AppError::conflict(ErrorCode::LastFilm));
    }
    db::films::delete(&mut *tx, path.film_id, list_id).await?;
    tx.commit().await?;
    info!(user_id = %user.id, %list_id, film_id = %path.film_id, "film removed");
    Ok(StatusCode::NO_CONTENT)
}

async fn get_statistics(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<FilmPath>,
) -> AppResult<Json<FilmStatisticsResponse>> {
    let list_id = db::films::list_id_for_owner(&state.pool, path.film_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let film = db::films::find(&state.pool, path.film_id, list_id)
        .await?
        .ok_or_else(AppError::not_found)?;
    let runs: Vec<Run> = db::films::runs(&state.pool, film.id, MAX_RUNS)
        .await?
        .into_iter()
        .map(|row| Run {
            selection_id: row.selection_id,
            started_at: row.started_at,
            place: row.place,
            film_count: row.film_count,
            eliminated_by: row.participant_name,
        })
        .collect();
    let statistics = domain::film_statistics(&runs);
    let recent = &runs[..runs.len().min(RECENT_RUNS)];
    let recent_ids: Vec<Uuid> = recent.iter().map(|run| run.selection_id).collect();
    let verdicts = db::selections::verdicts_for(&state.pool, &recent_ids).await?;
    let winner_of = |selection_id: Uuid| {
        verdicts
            .iter()
            .find(|verdict| {
                verdict.selection_id == selection_id
                    && Outcome::from_db(&verdict.outcome) == Some(Outcome::Watched)
            })
            .map(|verdict| verdict.film_title.clone())
    };
    let recent_runs = recent
        .iter()
        .map(|run| RunResponse {
            selection_id: run.selection_id,
            started_at: run.started_at,
            place: run.place,
            film_count: run.film_count,
            eliminated_by: run.eliminated_by.clone(),
            winner_title: winner_of(run.selection_id),
        })
        .collect();
    Ok(Json(FilmStatisticsResponse {
        film: (&film).into(),
        selection_count: statistics.selection_count,
        best_place: statistics.best_place,
        worst_place: statistics.worst_place,
        average_place: statistics.average_place,
        average_standing: statistics.average_standing,
        finals_lost: statistics.finals_lost,
        eliminated_by: statistics
            .eliminated_by
            .into_iter()
            .map(|entry| EliminatorCountResponse {
                participant_name: entry.name,
                count: entry.count,
            })
            .collect(),
        recent_runs,
    }))
}
