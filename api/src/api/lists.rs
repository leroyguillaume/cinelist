use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use tracing::info;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use super::{
    PageQuery, PageResponse,
    films::FilmResponse,
    not_blank,
    participants::{ParticipantResponse, valid_participant_name},
    selections::SelectionPhase,
};
use crate::{
    auth::CurrentUser,
    db, domain,
    error::{AppError, AppResult, ErrorCode},
    state::AppState,
    view::{self, ListSnapshot},
};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/api/v1/lists",
            get_with(list_lists, |op| {
                op.tag("Lists").id("listLists").summary("List my lists")
            })
            .post_with(create_list, |op| {
                op.tag("Lists")
                    .id("createList")
                    .summary("Create a list")
                    .response::<201, Json<ListResponse>>()
            }),
        )
        .api_route(
            "/api/v1/lists/{listId}",
            get_with(get_list, |op| {
                op.tag("Lists").id("getList").summary("Get a list")
            })
            .put_with(update_list, |op| {
                op.tag("Lists").id("updateList").summary("Rename a list")
            })
            .delete_with(delete_list, |op| {
                op.tag("Lists")
                    .id("deleteList")
                    .summary("Delete a list and its history")
                    .response::<204, ()>()
            }),
        )
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListPath {
    pub list_id: Uuid,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("example" = serde_json::json!({
    "name": "Soirées ciné",
    "participants": ["Alex", "Sam"],
    "films": ["Paris, Texas", "Stalker"],
})))]
pub struct CreateListRequest {
    #[validate(length(min = 1, max = 100), custom(function = not_blank))]
    name: String,
    /// Names, in turn order.
    #[validate(length(min = 1, max = 10), custom(function = valid_participant_names))]
    participants: Vec<String>,
    /// Titles of the first films; a list is never empty.
    #[validate(length(min = 1, max = 100), custom(function = valid_titles))]
    films: Vec<String>,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateListRequest {
    #[validate(length(min = 1, max = 100), custom(function = not_blank))]
    name: String,
}

fn valid_participant_names(names: &[String]) -> Result<(), ValidationError> {
    names
        .iter()
        .try_for_each(|name| valid_participant_name(name))
}

fn valid_titles(titles: &[String]) -> Result<(), ValidationError> {
    titles
        .iter()
        .try_for_each(|title| super::films::valid_title(title))
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSelectionResponse {
    id: Uuid,
    phase: SelectionPhase,
    elimination_count: usize,
    /// Eliminations needed to leave a single film.
    elimination_total: i32,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListSummaryResponse {
    id: Uuid,
    name: String,
    film_count: i32,
    /// Films to add before a selection can start: `filmCount - 1` must be a
    /// multiple of the number of participants, so that everyone plays as many
    /// turns. 0 when a selection can start.
    films_to_add: usize,
    created_at: DateTime<Utc>,
    /// In turn order.
    participants: Vec<ParticipantResponse>,
    /// Participant who opens the next selection.
    next_starter_id: Option<Uuid>,
    active_selection: Option<ActiveSelectionResponse>,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    #[serde(flatten)]
    summary: ListSummaryResponse,
    /// Oldest first.
    films: Vec<FilmResponse>,
}

impl From<&ListSnapshot> for ListSummaryResponse {
    fn from(snapshot: &ListSnapshot) -> Self {
        Self {
            id: snapshot.list.id,
            name: snapshot.list.name.clone(),
            film_count: snapshot.film_count(),
            films_to_add: domain::films_to_add(snapshot.films.len(), snapshot.participants.len()),
            created_at: snapshot.list.created_at,
            participants: snapshot
                .participants
                .iter()
                .map(ParticipantResponse::from)
                .collect(),
            next_starter_id: snapshot.next_starter().map(|participant| participant.id),
            active_selection: snapshot.active_selection.as_ref().map(|active| {
                ActiveSelectionResponse {
                    id: active.selection.id,
                    phase: active.phase().into(),
                    elimination_count: active.eliminations.len(),
                    elimination_total: active.selection.film_count - 1,
                }
            }),
        }
    }
}

impl From<&ListSnapshot> for ListResponse {
    fn from(snapshot: &ListSnapshot) -> Self {
        Self {
            summary: snapshot.into(),
            films: snapshot.films.iter().map(FilmResponse::from).collect(),
        }
    }
}

/// Locks the list for the rest of the transaction and loads it.
pub async fn lock(
    conn: &mut PgConnection,
    list_id: Uuid,
    user: &CurrentUser,
) -> AppResult<ListSnapshot> {
    let list = db::lists::lock_for_owner(&mut *conn, list_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    Ok(view::load_list(conn, list).await?)
}

pub fn ensure_idle(snapshot: &ListSnapshot) -> AppResult<()> {
    if snapshot.active_selection.is_some() {
        Err(AppError::conflict(ErrorCode::SelectionInProgress))
    } else {
        Ok(())
    }
}

pub async fn load_response(
    conn: &mut PgConnection,
    list_id: Uuid,
    user: &CurrentUser,
) -> AppResult<ListResponse> {
    let list = db::lists::find_for_owner(&mut *conn, list_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    Ok((&view::load_list(conn, list).await?).into())
}

async fn list_lists(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<PageResponse<ListSummaryResponse>>> {
    let pagination = query.pagination()?;
    let mut conn = state.pool.acquire().await?;
    let rows =
        db::lists::page_for_owner(&mut *conn, user.id, pagination.limit(), pagination.offset())
            .await?;
    let total = db::lists::count_for_owner(&mut *conn, user.id).await?;
    let snapshots = view::load_lists(&mut conn, rows).await?;
    let items = snapshots.iter().map(ListSummaryResponse::from).collect();
    Ok(Json(pagination.wrap(items, total)))
}

async fn create_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(request): Json<CreateListRequest>,
) -> AppResult<(StatusCode, Json<ListResponse>)> {
    request.validate()?;
    let id = Uuid::new_v4();
    let mut tx = state.pool.begin().await?;
    db::lists::create(&mut *tx, id, user.id, request.name.trim()).await?;
    for name in &request.participants {
        db::participants::create(&mut *tx, Uuid::new_v4(), id, name.trim()).await?;
    }
    for title in &request.films {
        db::films::create(&mut *tx, Uuid::new_v4(), id, title.trim()).await?;
    }
    let response = load_response(&mut tx, id, &user).await?;
    tx.commit().await?;
    info!(user_id = %user.id, list_id = %id, films = request.films.len(), "list created");
    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ListPath>,
) -> AppResult<Json<ListResponse>> {
    let mut conn = state.pool.acquire().await?;
    Ok(Json(load_response(&mut conn, path.list_id, &user).await?))
}

async fn update_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ListPath>,
    Json(request): Json<UpdateListRequest>,
) -> AppResult<Json<ListResponse>> {
    request.validate()?;
    let mut conn = state.pool.acquire().await?;
    let list = db::lists::find_for_owner(&mut *conn, path.list_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    db::lists::rename(&mut *conn, list.id, request.name.trim()).await?;
    let response = load_response(&mut conn, list.id, &user).await?;
    info!(user_id = %user.id, list_id = %list.id, "list renamed");
    Ok(Json(response))
}

async fn delete_list(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(path): Path<ListPath>,
) -> AppResult<StatusCode> {
    let list = db::lists::find_for_owner(&state.pool, path.list_id, user.id)
        .await?
        .ok_or_else(AppError::not_found)?;
    db::lists::delete(&state.pool, list.id).await?;
    info!(user_id = %user.id, list_id = %list.id, "list deleted");
    Ok(StatusCode::NO_CONTENT)
}
