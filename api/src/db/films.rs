use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub struct FilmRow {
    pub id: Uuid,
    pub list_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct RunRow {
    pub selection_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub place: i32,
    pub film_count: i32,
    pub participant_name: String,
}

/// Films of every given list, oldest first.
pub async fn for_lists(ex: impl PgExecutor<'_>, list_ids: &[Uuid]) -> sqlx::Result<Vec<FilmRow>> {
    sqlx::query_as(include_str!("../../queries/films/for_lists.sql"))
        .bind(list_ids)
        .fetch_all(ex)
        .await
}

pub async fn find(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    list_id: Uuid,
) -> sqlx::Result<Option<FilmRow>> {
    sqlx::query_as(include_str!("../../queries/films/find.sql"))
        .bind(id)
        .bind(list_id)
        .fetch_optional(ex)
        .await
}

pub async fn create(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    list_id: Uuid,
    title: &str,
) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/films/create.sql"))
        .bind(id)
        .bind(list_id)
        .bind(title)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn rename(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    list_id: Uuid,
    title: &str,
) -> sqlx::Result<bool> {
    let result = sqlx::query(include_str!("../../queries/films/rename.sql"))
        .bind(id)
        .bind(list_id)
        .bind(title)
        .execute(ex)
        .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn delete(ex: impl PgExecutor<'_>, id: Uuid, list_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/films/delete.sql"))
        .bind(id)
        .bind(list_id)
        .execute(ex)
        .await?;
    Ok(())
}

/// The film's latest `limit` eliminations, newest first.
pub async fn runs(ex: impl PgExecutor<'_>, film_id: Uuid, limit: i64) -> sqlx::Result<Vec<RunRow>> {
    sqlx::query_as(include_str!("../../queries/films/runs.sql"))
        .bind(film_id)
        .bind(limit)
        .fetch_all(ex)
        .await
}

/// List of the film `id`, when `owner_id` owns that list.
pub async fn list_id_for_owner(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    owner_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar(include_str!("../../queries/films/list_id_for_owner.sql"))
        .bind(id)
        .bind(owner_id)
        .fetch_optional(ex)
        .await
}
