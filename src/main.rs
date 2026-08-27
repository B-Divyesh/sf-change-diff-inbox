use std::{env, net::SocketAddr, time::Duration};

use anyhow::Context;
use change_diff_inbox::{app, watcher};
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "change_diff_inbox=info,tower_http=info".into()),
        )
        .init();
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/change-diff.db?mode=rwc".into());
    if database_url.starts_with("sqlite://data/") {
        std::fs::create_dir_all("data")?;
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("connect database")?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("run migrations")?;
    tokio::spawn(scheduler(pool.clone()));

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "change diff inbox listening");
    axum::serve(
        listener,
        app(
            pool,
            &env::var("FRONTEND_DIR").unwrap_or_else(|_| "frontend/dist".into()),
        ),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

async fn scheduler(pool: SqlitePool) {
    let mut tick = tokio::time::interval(Duration::from_secs(60));
    tick.tick().await;
    loop {
        tick.tick().await;
        let now = Utc::now().to_rfc3339();
        let ids: Vec<(String,)> = sqlx::query_as("SELECT id FROM sources WHERE enabled=1 AND (next_check IS NULL OR next_check<=?) LIMIT 10")
            .bind(now).fetch_all(&pool).await.unwrap_or_default();
        for (id,) in ids {
            let pool = pool.clone();
            tokio::spawn(async move {
                if let Err(error) = watcher::check_source(&pool, &id).await {
                    tracing::warn!(%id, %error, "scheduled check failed");
                }
            });
        }
    }
}

async fn shutdown() {
    let ctrl_c = async { signal::ctrl_c().await.expect("install Ctrl+C handler") };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("graceful shutdown");
}
