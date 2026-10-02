use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub struct SelectionRow {
    pub id: Uuid,
    pub list_id: Uuid,
    pub starter_id: Option<Uuid>,
    pub starter_name: String,
    pub film_count: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, FromRow)]
pub struct EliminationRow {
    pub id: Uuid,
    pub selection_id: Uuid,
    pub film_id: Option<Uuid>,
    pub film_title: String,
    pub participant_id: Option<Uuid>,
    pub participant_name: String,
    pub place: i32,
}

#[derive(Clone, FromRow)]
pub struct VerdictRow {
    pub selection_id: Uuid,
    pub film_title: String,
    pub place: i32,
    pub outcome: String,
    pub created_at: DateTime<Utc>,
}

pub struct NewSelection<'a> {
    pub id: Uuid,
    pub list_id: Uuid,
    pub starter_id: Uuid,
    pub starter_name: &'a str,
    pub film_count: i32,
}

pub struct NewElimination<'a> {
    pub id: Uuid,
    pub selection_id: Uuid,
    pub film_id: Uuid,
    pub film_title: &'a str,
    pub participant_id: Uuid,
    pub participant_name: &'a str,
    pub place: i32,
}

pub struct NewVerdict<'a> {
    pub id: Uuid,
    pub selection_id: Uuid,
    pub film_id: Uuid,
    pub film_title: &'a str,
    pub place: i32,
    pub outcome: &'static str,
}

pub async fn create(ex: impl PgExecutor<'_>, selection: &NewSelection<'_>) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/selections/create.sql"))
        .bind(selection.id)
        .bind(selection.list_id)
        .bind(selection.starter_id)
        .bind(selection.starter_name)
        .bind(selection.film_count)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn find(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    list_id: Uuid,
) -> sqlx::Result<Option<SelectionRow>> {
    sqlx::query_as(include_str!("../../queries/selections/find.sql"))
        .bind(id)
        .bind(list_id)
        .fetch_optional(ex)
        .await
}

pub async fn active_for_lists(
    ex: impl PgExecutor<'_>,
    list_ids: &[Uuid],
) -> sqlx::Result<Vec<SelectionRow>> {
    sqlx::query_as(include_str!(
        "../../queries/selections/active_for_lists.sql"
    ))
    .bind(list_ids)
    .fetch_all(ex)
    .await
}

/// The most recent selection of every given list that has one.
pub async fn latest_for_lists(
    ex: impl PgExecutor<'_>,
    list_ids: &[Uuid],
) -> sqlx::Result<Vec<SelectionRow>> {
    sqlx::query_as(include_str!(
        "../../queries/selections/latest_for_lists.sql"
    ))
    .bind(list_ids)
    .fetch_all(ex)
    .await
}

pub async fn page_for_list(
    ex: impl PgExecutor<'_>,
    list_id: Uuid,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<SelectionRow>> {
    sqlx::query_as(include_str!("../../queries/selections/page_for_list.sql"))
        .bind(list_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(ex)
        .await
}

pub async fn count_for_list(ex: impl PgExecutor<'_>, list_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar(include_str!("../../queries/selections/count_for_list.sql"))
        .bind(list_id)
        .fetch_one(ex)
        .await
}

pub async fn complete(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/selections/complete.sql"))
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/selections/delete.sql"))
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}

/// Eliminations of every given selection, latest first.
pub async fn eliminations_for(
    ex: impl PgExecutor<'_>,
    selection_ids: &[Uuid],
) -> sqlx::Result<Vec<EliminationRow>> {
    sqlx::query_as(include_str!(
        "../../queries/selections/eliminations_for.sql"
    ))
    .bind(selection_ids)
    .fetch_all(ex)
    .await
}

pub async fn create_elimination(
    ex: impl PgExecutor<'_>,
    elimination: &NewElimination<'_>,
) -> sqlx::Result<()> {
    sqlx::query(include_str!(
        "../../queries/selections/create_elimination.sql"
    ))
    .bind(elimination.id)
    .bind(elimination.selection_id)
    .bind(elimination.film_id)
    .bind(elimination.film_title)
    .bind(elimination.participant_id)
    .bind(elimination.participant_name)
    .bind(elimination.place)
    .execute(ex)
    .await?;
    Ok(())
}

pub async fn delete_elimination(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!(
        "../../queries/selections/delete_elimination.sql"
    ))
    .bind(id)
    .execute(ex)
    .await?;
    Ok(())
}

/// Verdicts of every given selection, best place first.
pub async fn verdicts_for(
    ex: impl PgExecutor<'_>,
    selection_ids: &[Uuid],
) -> sqlx::Result<Vec<VerdictRow>> {
    sqlx::query_as(include_str!("../../queries/selections/verdicts_for.sql"))
        .bind(selection_ids)
        .fetch_all(ex)
        .await
}

pub async fn create_verdict(ex: impl PgExecutor<'_>, verdict: &NewVerdict<'_>) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/selections/create_verdict.sql"))
        .bind(verdict.id)
        .bind(verdict.selection_id)
        .bind(verdict.film_id)
        .bind(verdict.film_title)
        .bind(verdict.place)
        .bind(verdict.outcome)
        .execute(ex)
        .await?;
    Ok(())
}

/// List of the selection `id`, when `owner_id` owns that list.
pub async fn list_id_for_owner(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    owner_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar(include_str!(
        "../../queries/selections/list_id_for_owner.sql"
    ))
    .bind(id)
    .bind(owner_id)
    .fetch_optional(ex)
    .await
}
