use std::{
    env,
    fs::OpenOptions,
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use anyhow::Context;
use change_diff_inbox::{app_with_options, watcher};
use chrono::Utc;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
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
    let (data_dir, data_dir_source) = match env::var("DATA_DIR") {
        Ok(value) => (PathBuf::from(value), "supplied"),
        Err(_) if Path::new("/data").is_dir() => (PathBuf::from("/data"), "detected"),
        Err(_) => (PathBuf::from("data"), "local fallback"),
    };
    std::fs::create_dir_all(&data_dir)?;
    let (database_url, database_source) = match env::var("DATABASE_URL") {
        Ok(value) => (value, "supplied"),
        Err(_) => (
            format!(
                "sqlite://{}?mode=rwc",
                data_dir.join("change-diff.db").display()
            ),
            "generated default",
        ),
    };
    let (session_secret, session_source) = load_session_secret(&data_dir)?;
    tracing::info!(
        data_dir = data_dir_source,
        database = database_source,
        session_secret = session_source,
        "runtime configuration ready"
    );
    let connect_options = SqliteConnectOptions::from_str(&database_url)
        .context("parse database URL")?
        .create_if_missing(true)
        // The fleet's durable /data mount is SMB-backed. Dot-file locking is
        // supported there; SQLite's default POSIX byte-range locks are not.
        .vfs("unix-dotfile")
        .busy_timeout(Duration::from_secs(30));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(60))
        .connect_with(connect_options)
        .await
        .context("connect database")?;
    run_migrations(&pool).await?;
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
        app_with_options(
            pool,
            &env::var("FRONTEND_DIR").unwrap_or_else(|_| "frontend/dist".into()),
            session_secret,
            env::var("BILLING_BASE_URL").unwrap_or_else(|_| "https://api.sociobot.in".into()),
        ),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut migrator = sqlx::migrate!();
    // Azure Files does not support SQLx's SQLite migration-wide exclusive
    // lock. The fleet keeps this app at one replica, while each migration is
    // still applied in its own SQLite transaction.
    migrator.set_locking(false);
    for attempt in 1..=12 {
        match migrator.run(pool).await {
            Ok(()) => return Ok(()),
            Err(error) if error.to_string().contains("database is locked") && attempt < 12 => {
                tracing::warn!(attempt, "SQLite migration lock is busy; retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            Err(error) => return Err(error).context("run migrations"),
        }
    }
    unreachable!("migration retry loop returns on its final attempt")
}

async fn scheduler(pool: SqlitePool) {
    let mut tick = tokio::time::interval(Duration::from_secs(60));
    tick.tick().await;
    loop {
        tick.tick().await;
        let now = Utc::now().to_rfc3339();
        let ids: Vec<(String, String)> = sqlx::query_as("SELECT s.tenant_id,s.id FROM sources s JOIN tenants t ON t.id=s.tenant_id WHERE t.kind='real' AND s.enabled=1 AND (s.next_check IS NULL OR s.next_check<=?) LIMIT 10")
            .bind(now).fetch_all(&pool).await.unwrap_or_default();
        for (tenant_id, id) in ids {
            let pool = pool.clone();
            tokio::spawn(async move {
                if let Err(error) = watcher::check_source(&pool, &tenant_id, &id).await {
                    tracing::warn!(%id, %error, "scheduled check failed");
                }
            });
        }
    }
}

fn load_session_secret(data_dir: &Path) -> anyhow::Result<(Vec<u8>, &'static str)> {
    if let Ok(value) = env::var("SESSION_SECRET") {
        if value.len() < 32 {
            anyhow::bail!("SESSION_SECRET must contain at least 32 bytes when supplied");
        }
        return Ok((value.into_bytes(), "supplied"));
    }
    let path = data_dir.join("session.key");
    if path.exists() {
        let value = std::fs::read(path)?;
        if value.len() < 32 {
            anyhow::bail!("persisted session key is invalid");
        }
        return Ok((value, "persisted"));
    }
    let mut value = vec![0_u8; 32];
    getrandom::fill(&mut value)
        .map_err(|error| anyhow::anyhow!("generate session key: {error}"))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(&value)?;
    file.sync_all()?;
    Ok((value, "generated and persisted"))
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
