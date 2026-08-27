use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use url::Url;
use uuid::Uuid;

use crate::{
    models::{Change, ReviewInput, Source, SourceInput, Stats},
    watcher,
};

pub fn api() -> Router<SqlitePool> {
    Router::new()
        .route("/sources", get(list_sources).post(create_source))
        .route("/sources/{id}", put(update_source).delete(delete_source))
        .route("/sources/{id}/check", post(check_source))
        .route("/changes", get(list_changes))
        .route("/changes/{id}", patch(review_change))
        .route("/stats", get(stats))
}

pub fn build_sha() -> &'static str {
    option_env!("BUILD_SHA").unwrap_or("dev")
}

pub async fn health() -> Json<Value> {
    Json(json!({"status":"ok", "build": build_sha()}))
}

#[derive(Debug)]
pub struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "database error");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The database could not complete that action".into(),
        )
    }
}

fn validate(input: &SourceInput) -> Result<(String, String, f64, i64), ApiError> {
    let name = input.name.trim();
    if name.len() < 2 || name.len() > 80 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Name must be 2–80 characters".into(),
        ));
    }
    let url = Url::parse(input.url.trim())
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "Enter a valid absolute URL".into()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Only public http and https URLs are supported".into(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Authenticated URLs are not supported".into(),
        ));
    }
    let mode = input.extract_mode.as_deref().unwrap_or("selector");
    if !["selector", "table", "jsonld", "code"].contains(&mode) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Choose a supported extraction mode".into(),
        ));
    }
    let selector = input.selector.as_deref().unwrap_or("main").trim();
    if selector.len() > 200 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Selector must be under 200 characters".into(),
        ));
    }
    let threshold = input.threshold.unwrap_or(0.03);
    if !(0.0..=1.0).contains(&threshold) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Threshold must be between 0 and 100%".into(),
        ));
    }
    let interval = input.interval_minutes.unwrap_or(1440);
    if !(15..=43200).contains(&interval) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Interval must be between 15 minutes and 30 days".into(),
        ));
    }
    Ok((selector.to_owned(), mode.to_owned(), threshold, interval))
}

async fn list_sources(State(pool): State<SqlitePool>) -> Result<Json<Vec<Source>>, ApiError> {
    Ok(Json(
        sqlx::query_as("SELECT * FROM sources ORDER BY created_at DESC")
            .fetch_all(&pool)
            .await?,
    ))
}

async fn create_source(
    State(pool): State<SqlitePool>,
    Json(input): Json<SourceInput>,
) -> Result<(StatusCode, Json<Source>), ApiError> {
    let (selector, mode, threshold, interval) = validate(&input)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO sources (id,name,url,selector,extract_mode,threshold,interval_minutes,next_check,created_at) VALUES (?,?,?,?,?,?,?,?,?)")
        .bind(&id).bind(input.name.trim()).bind(input.url.trim()).bind(selector).bind(mode).bind(threshold).bind(interval).bind(&now).bind(&now)
        .execute(&pool).await?;
    let source = sqlx::query_as("SELECT * FROM sources WHERE id=?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok((StatusCode::CREATED, Json(source)))
}

async fn update_source(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(input): Json<SourceInput>,
) -> Result<Json<Source>, ApiError> {
    let (selector, mode, threshold, interval) = validate(&input)?;
    let existing = sqlx::query_as::<_, Source>("SELECT * FROM sources WHERE id=?")
        .bind(&id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Source not found".into()))?;
    let extraction_changed = existing.url != input.url.trim()
        || existing.selector != selector
        || existing.extract_mode != mode;
    let result = sqlx::query("UPDATE sources SET name=?,url=?,selector=?,extract_mode=?,threshold=?,interval_minutes=?,next_check=?, baseline=CASE WHEN ? THEN NULL ELSE baseline END, last_status=CASE WHEN ? THEN 'new' ELSE last_status END WHERE id=?")
        .bind(input.name.trim()).bind(input.url.trim()).bind(selector).bind(mode).bind(threshold).bind(interval)
        .bind((Utc::now() + Duration::minutes(interval)).to_rfc3339()).bind(extraction_changed).bind(extraction_changed).bind(&id).execute(&pool).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "Source not found".into()));
    }
    Ok(Json(
        sqlx::query_as("SELECT * FROM sources WHERE id=?")
            .bind(id)
            .fetch_one(&pool)
            .await?,
    ))
}

async fn delete_source(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query("DELETE FROM sources WHERE id=?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "Source not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn check_source(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<crate::models::CheckResult>, ApiError> {
    watcher::check_source(&pool, &id)
        .await
        .map(Json)
        .map_err(|error| ApiError(StatusCode::BAD_REQUEST, error.to_string()))
}

#[derive(Deserialize)]
struct ChangeQuery {
    state: Option<String>,
    source: Option<String>,
}

async fn list_changes(
    State(pool): State<SqlitePool>,
    Query(query): Query<ChangeQuery>,
) -> Result<Json<Vec<Change>>, ApiError> {
    let state = query.state.unwrap_or_else(|| "all".into());
    let source = query.source.unwrap_or_else(|| "all".into());
    let changes = sqlx::query_as::<_, Change>("SELECT c.*,s.name source_name,s.url source_url,s.selector FROM changes c JOIN sources s ON s.id=c.source_id WHERE (?='all' OR c.review_state=?) AND (?='all' OR c.source_id=?) ORDER BY c.created_at DESC LIMIT 250")
        .bind(&state).bind(&state).bind(&source).bind(&source).fetch_all(&pool).await?;
    Ok(Json(changes))
}

async fn review_change(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(input): Json<ReviewInput>,
) -> Result<Json<Value>, ApiError> {
    if let Some(ref state) = input.review_state {
        if !["unread", "reviewed", "archived"].contains(&state.as_str()) {
            return Err(ApiError(
                StatusCode::BAD_REQUEST,
                "Unsupported review state".into(),
            ));
        }
    }
    let useful = input.useful.map(i64::from);
    let result = sqlx::query("UPDATE changes SET review_state=COALESCE(?,review_state), useful=COALESCE(?,useful) WHERE id=?")
        .bind(input.review_state).bind(useful).bind(id).execute(&pool).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "Change not found".into()));
    }
    Ok(Json(json!({"ok":true})))
}

async fn stats(State(pool): State<SqlitePool>) -> Result<Json<Stats>, ApiError> {
    let result = sqlx::query_as::<_, Stats>("SELECT (SELECT count(*) FROM sources) sources, (SELECT count(*) FROM changes WHERE review_state='unread') unread, (SELECT count(*) FROM changes WHERE useful=1) useful, (SELECT count(*) FROM changes WHERE useful IS NOT NULL) rated")
        .fetch_one(&pool).await?;
    Ok(Json(result))
}
