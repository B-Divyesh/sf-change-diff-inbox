pub mod models;
pub mod routes;
pub mod watcher;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    http::{header, HeaderName, HeaderValue, Request},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use sqlx::SqlitePool;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub session_secret: Arc<Vec<u8>>,
    pub billing_base: Arc<String>,
    limiter: Arc<RequestLimiter>,
}

#[derive(Default)]
struct RequestLimiter {
    buckets: Mutex<HashMap<(String, bool), RateBucket>>,
}

struct RateBucket {
    opened: Instant,
    count: u32,
}

impl RequestLimiter {
    fn allowed(&self, client: String, write: bool) -> Result<(), u64> {
        let window = if write {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(1)
        };
        let allowance = if write { 10 } else { 40 };
        let now = Instant::now();
        let mut buckets = self.buckets.lock().expect("rate limiter lock");
        let bucket = buckets.entry((client, write)).or_insert(RateBucket {
            opened: now,
            count: 0,
        });
        if now.duration_since(bucket.opened) >= window {
            bucket.opened = now;
            bucket.count = 0;
        }
        if bucket.count >= allowance {
            let remaining = window.saturating_sub(now.duration_since(bucket.opened));
            return Err(remaining.as_secs().max(1));
        }
        bucket.count += 1;
        if buckets.len() > 10_000 {
            buckets.retain(|_, value| now.duration_since(value.opened) < Duration::from_secs(120));
        }
        Ok(())
    }
}

pub fn app(pool: SqlitePool, frontend_dir: &str) -> Router {
    let mut secret = vec![0_u8; 32];
    getrandom::fill(&mut secret).expect("operating system random source");
    app_with_options(pool, frontend_dir, secret, "https://api.sociobot.in".into())
}

pub fn app_with_options(
    pool: SqlitePool,
    frontend_dir: &str,
    session_secret: Vec<u8>,
    billing_base: String,
) -> Router {
    let state = AppState {
        pool,
        session_secret: Arc::new(session_secret),
        billing_base: Arc::new(billing_base),
        limiter: Arc::new(RequestLimiter::default()),
    };
    let api =
        routes::api(state.clone()).layer(middleware::from_fn_with_state(state.clone(), rate_limit));
    let index = format!("{frontend_dir}/index.html");
    let fallback = ServeDir::new(frontend_dir)
        .not_found_service(ServeFile::new(format!("{frontend_dir}/404.html")));

    Router::new()
        .route("/health", get(routes::health))
        .nest("/api", api)
        .route_service("/", ServeFile::new(&index))
        .route_service("/demo", ServeFile::new(&index))
        .route_service("/privacy", ServeFile::new(&index))
        .route_service("/terms", ServeFile::new(&index))
        .fallback_service(fallback)
        .with_state(state)
        .layer(middleware::from_fn(security_headers))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(TraceLayer::new_for_http())
}

async fn rate_limit(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let client = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned();
    let write = !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    );
    match state.limiter.allowed(client, write) {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let mut response = (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({
                    "error": "Too many requests. Wait before trying again."
                })),
            )
                .into_response();
            response.headers_mut().insert(
                header::RETRY_AFTER,
                HeaderValue::from_str(&retry_after.to_string())
                    .unwrap_or_else(|_| HeaderValue::from_static("1")),
            );
            response
        }
    }
}

async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self'; font-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"));
    headers
        .entry(HeaderName::from_static("permissions-policy"))
        .or_insert(HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=()",
        ));
    if path.starts_with("/assets/index-") || path.starts_with("/fonts/") {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    } else if matches!(
        path.as_str(),
        "/" | "/demo" | "/privacy" | "/terms" | "/sw.js"
    ) || path.ends_with(".html")
    {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    } else if path.starts_with("/assets/") {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        );
    }
    response
}
