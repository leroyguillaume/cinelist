use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};
use uuid::Uuid;

#[derive(FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub is_admin: bool,
    pub token_version: i32,
}

#[derive(FromRow)]
pub struct CredentialsRow {
    pub id: Uuid,
    pub username: String,
    pub is_admin: bool,
    pub token_version: i32,
    pub password_hash: Option<String>,
}

#[derive(FromRow)]
pub struct UserSummaryRow {
    pub id: Uuid,
    pub username: String,
    pub is_admin: bool,
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub has_password: bool,
    pub list_count: i64,
}

pub async fn upsert_admin(
    ex: impl PgExecutor<'_>,
    username: &str,
    password_hash: &str,
) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/users/upsert_admin.sql"))
        .bind(Uuid::new_v4())
        .bind(username)
        .bind(password_hash)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn demote_admins(ex: impl PgExecutor<'_>, except_username: &str) -> sqlx::Result<u64> {
    let result = sqlx::query(include_str!("../../queries/users/demote_admins.sql"))
        .bind(except_username)
        .execute(ex)
        .await?;
    Ok(result.rows_affected())
}

pub async fn find_credentials(
    ex: impl PgExecutor<'_>,
    username: &str,
) -> sqlx::Result<Option<CredentialsRow>> {
    sqlx::query_as(include_str!("../../queries/users/find_credentials.sql"))
        .bind(username)
        .fetch_optional(ex)
        .await
}

pub async fn find(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<Option<UserRow>> {
    sqlx::query_as(include_str!("../../queries/users/find.sql"))
        .bind(id)
        .fetch_optional(ex)
        .await
}

/// Returns `false` when the username is taken.
pub async fn create(ex: impl PgExecutor<'_>, id: Uuid, username: &str) -> sqlx::Result<bool> {
    let result = sqlx::query(include_str!("../../queries/users/create.sql"))
        .bind(id)
        .bind(username)
        .execute(ex)
        .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn delete(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/users/delete.sql"))
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn set_password(
    ex: impl PgExecutor<'_>,
    id: Uuid,
    password_hash: &str,
) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/users/set_password.sql"))
        .bind(id)
        .bind(password_hash)
        .execute(ex)
        .await?;
    Ok(())
}

/// Revokes every token issued to the user so far.
pub async fn bump_token_version(ex: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query(include_str!("../../queries/users/bump_token_version.sql"))
        .bind(id)
        .execute(ex)
        .await?;
    Ok(())
}

pub async fn count(ex: impl PgExecutor<'_>) -> sqlx::Result<i64> {
    sqlx::query_scalar(include_str!("../../queries/users/count.sql"))
        .fetch_one(ex)
        .await
}

pub async fn page(
    ex: impl PgExecutor<'_>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<UserSummaryRow>> {
    sqlx::query_as(include_str!("../../queries/users/page.sql"))
        .bind(limit)
        .bind(offset)
        .fetch_all(ex)
        .await
}
