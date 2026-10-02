use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(Clone, FromRow)]
pub struct ParticipantRow {
    pub id: Uuid,
    pub list_id: Uuid,
    pub name: String,
}

/// Participants of every given list, in turn order.
pub async fn for_lists(
    ex: impl PgExecutor<'_>,
    list_ids: &[Uuid],
) -> sqlx::Result<Vec<ParticipantRow>> {
    sqlx::query_as(include_str!("../../queries/participants/for_lists.sql"))
        .bind(list_ids)
        .fetch_all(ex)
        .await
}

/// Appends a participant at the end of the turn order.
pub async fn create(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    list_id: Uuid,
    name: &str,
) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/participants/create.sql"))
        .bind(id)
        .bind(list_id)
        .bind(name)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn rename(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    list_id: Uuid,
    name: &str,
) -> sqlx::Result<bool> {
    let result = sqlx::query(include_str!("../../queries/participants/rename.sql"))
        .bind(id)
        .bind(list_id)
        .bind(name)
        .execute(ex)
        .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn delete(ex: impl PgExecutor<'_>, id: Uuid, list_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/participants/delete.sql"))
        .bind(id)
        .bind(list_id)
        .execute(ex)
        .await?;
    Ok(())
}

/// List of the participant `id`, when `owner_id` owns that list.
pub async fn list_id_for_owner(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    owner_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar(include_str!(
        "../../queries/participants/list_id_for_owner.sql"
    ))
    .bind(id)
    .bind(owner_id)
    .fetch_optional(ex)
    .await
}
