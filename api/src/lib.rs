mod api;
mod auth;
mod db;
mod domain;
mod error;
mod state;
mod view;

use sqlx::{PgPool, migrate::Migrator};
use tracing::info;

pub use api::router;
pub use state::{AppState, JwtKeys, Settings};

pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Creates the administrator, or resets their password, and demotes any
/// other account that held the role.
///
/// # Errors
///
/// Fails when the password cannot be hashed or the database is unreachable.
pub async fn bootstrap_admin(
    pool: &PgPool,
    username: &str,
    password: String,
) -> anyhow::Result<()> {
    let hash = auth::hash_password(password)
        .await
        .map_err(|err| anyhow::anyhow!("hashing the administrator password: {err}"))?;
    let mut tx = pool.begin().await?;
    db::users::upsert_admin(&mut *tx, username, &hash).await?;
    let demoted = db::users::demote_admins(&mut *tx, username).await?;
    tx.commit().await?;
    info!(%username, demoted, "administrator ready");
    Ok(())
}
