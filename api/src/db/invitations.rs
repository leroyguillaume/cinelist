use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(FromRow)]
pub struct PendingInvitationRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn revoke_pending(ex: impl PgExecutor<'_>, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/invitations/revoke_pending.sql"))
        .bind(user_id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn create(
    ex: impl PgExecutor<'_>,
    user_id: Uuid,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/invitations/create.sql"))
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(ex)
        .await?;
    Ok(())
}

/// Locks the invitation row, so that it can be accepted only once.
pub async fn find_pending(
    ex: impl PgExecutor<'_>,
    token_hash: &[u8],
) -> sqlx::Result<Option<PendingInvitationRow>> {
    sqlx::query_as(include_str!("../../queries/invitations/find_pending.sql"))
        .bind(token_hash)
        .fetch_optional(ex)
        .await
}

pub async fn mark_used(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/invitations/mark_used.sql"))
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}
