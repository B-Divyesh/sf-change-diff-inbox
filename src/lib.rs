pub mod models;
pub mod routes;
pub mod watcher;

use axum::{
    body::Body,
    http::{header, HeaderName, HeaderValue, Request},
    middleware::{self, Next},
    response::Response,
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

pub fn app(pool: SqlitePool, frontend_dir: &str) -> Router {
    let fallback = ServeDir::new(frontend_dir)
        .not_found_service(ServeFile::new(format!("{frontend_dir}/index.html")));
    Router::new()
        .route("/health", get(routes::health))
        .nest("/api", routes::api())
        .route_service(
            "/privacy",
            ServeFile::new(format!("{frontend_dir}/index.html")),
        )
        .route_service(
            "/terms",
            ServeFile::new(format!("{frontend_dir}/index.html")),
        )
        .fallback_service(fallback)
        .with_state(pool)
        .layer(middleware::from_fn(security_headers))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(TraceLayer::new_for_http())
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
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' https://api.sociobot.in https://pilot-api.sociobot.in; font-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'; form-action 'self' https://api.sociobot.in"));
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
    } else if path == "/" || path.ends_with(".html") || path == "/sw.js" {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    } else if path.starts_with("/assets/") {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        );
    }
    response
}
