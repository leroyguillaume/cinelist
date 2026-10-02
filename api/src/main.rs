use std::{io::IsTerminal, net::SocketAddr, sync::Arc};

use anyhow::Context;
use chrono::Duration;
use cinelist_api::{AppState, JwtKeys, MIGRATOR, Settings, bootstrap_admin, router};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::EnvFilter;

const MIN_JWT_SECRET_LENGTH: usize = 32;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Address the HTTP server listens on.
    #[arg(long, env = "BIND_ADDR", default_value_t = SocketAddr::from(([0, 0, 0, 0], 8080)))]
    bind_addr: SocketAddr,

    /// Postgres connection string.
    #[arg(long, env = "DATABASE_URL", hide_env_values = true)]
    database_url: String,

    /// Largest number of pooled database connections.
    #[arg(long, env = "DATABASE_MAX_CONNECTIONS", default_value_t = 10)]
    database_max_connections: u32,

    /// Username of the administrator, created or updated at startup.
    #[arg(long, env = "ADMIN_USERNAME", default_value = "admin")]
    admin_username: String,

    /// Password of the administrator, reset at every startup.
    #[arg(long, env = "ADMIN_PASSWORD", hide_env_values = true)]
    admin_password: String,

    /// Secret signing the JWTs (HS256), at least 32 bytes. Changing it signs everyone out.
    #[arg(long, env = "JWT_SECRET", hide_env_values = true)]
    jwt_secret: String,

    /// Days an issued JWT stays valid.
    #[arg(long, env = "TOKEN_TTL_DAYS", default_value_t = 30)]
    token_ttl_days: i64,

    /// Days an invitation link stays valid.
    #[arg(long, env = "INVITATION_TTL_DAYS", default_value_t = 7)]
    invitation_ttl_days: i64,

    /// Mark the session cookie `Secure`; disable only to serve plain HTTP locally.
    #[arg(long, env = "COOKIE_SECURE", default_value_t = true, action = clap::ArgAction::Set)]
    cookie_secure: bool,

    /// `tracing` filter directive (e.g. `info`, `cinelist_api=debug,tower_http=debug`).
    ///
    /// Syntax: <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives>
    #[arg(long, env = "LOG_FILTER", default_value = "info")]
    log_filter: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cli.log_filter))
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .init();

    // HS256 is only as strong as its key; RFC 7518 §3.2 wants at least 256 bits.
    anyhow::ensure!(
        cli.jwt_secret.len() >= MIN_JWT_SECRET_LENGTH,
        "JWT_SECRET must be at least {MIN_JWT_SECRET_LENGTH} bytes long"
    );

    let pool = PgPoolOptions::new()
        .max_connections(cli.database_max_connections)
        .connect(&cli.database_url)
        .await
        .context("connecting to the database")?;
    MIGRATOR
        .run(&pool)
        .await
        .context("running database migrations")?;
    info!("database migrated");
    bootstrap_admin(&pool, &cli.admin_username, cli.admin_password).await?;

    let state = AppState {
        pool: pool.clone(),
        settings: Arc::new(Settings {
            token_ttl: Duration::days(cli.token_ttl_days),
            invitation_ttl: Duration::days(cli.invitation_ttl_days),
            cookie_secure: cli.cookie_secure,
            jwt: JwtKeys::from_secret(cli.jwt_secret.as_bytes()),
        }),
    };
    let listener = tokio::net::TcpListener::bind(cli.bind_addr)
        .await
        .with_context(|| format!("binding {}", cli.bind_addr))?;
    info!(addr = %cli.bind_addr, "listening");
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serving HTTP")?;
    pool.close().await;
    info!("stopped");
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let mut interrupt = signal(SignalKind::interrupt()).expect("SIGINT handler");
    let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler");
    tokio::select! {
        _ = interrupt.recv() => info!("SIGINT received, draining"),
        _ = terminate.recv() => info!("SIGTERM received, draining"),
    }
}
