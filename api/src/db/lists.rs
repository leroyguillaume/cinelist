use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub struct ListRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

pub async fn page_for_owner(
    ex: impl PgExecutor<'_>,
    owner_id: Uuid,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<ListRow>> {
    sqlx::query_as(include_str!("../../queries/lists/page_for_owner.sql"))
        .bind(owner_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(ex)
        .await
}

pub async fn count_for_owner(ex: impl PgExecutor<'_>, owner_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar(include_str!("../../queries/lists/count_for_owner.sql"))
        .bind(owner_id)
        .fetch_one(ex)
        .await
}

pub async fn find_for_owner(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    owner_id: Uuid,
) -> sqlx::Result<Option<ListRow>> {
    sqlx::query_as(include_str!("../../queries/lists/find_for_owner.sql"))
        .bind(id)
        .bind(owner_id)
        .fetch_optional(ex)
        .await
}

/// Same as [`find_for_owner`], holding a row lock until the transaction ends:
/// every change to a list's content goes through it, one at a time.
pub async fn lock_for_owner(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    owner_id: Uuid,
) -> sqlx::Result<Option<ListRow>> {
    sqlx::query_as(include_str!("../../queries/lists/lock_for_owner.sql"))
        .bind(id)
        .bind(owner_id)
        .fetch_optional(ex)
        .await
}

pub async fn create(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    owner_id: Uuid,
    name: &str,
) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/lists/create.sql"))
        .bind(id)
        .bind(owner_id)
        .bind(name)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn rename(ex: impl PgExecutor<'_>, id: Uuid, name: &str) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/lists/rename.sql"))
        .bind(id)
        .bind(name)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn delete(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/lists/delete.sql"))
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}
