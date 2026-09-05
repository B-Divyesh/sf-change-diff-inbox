use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
    Extension, Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use url::Url;
use uuid::Uuid;

use crate::{
    models::{Change, ReviewInput, Source, SourceInput, Stats},
    watcher, AppState,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TenantKind {
    Real,
    Demo,
}

impl TenantKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::Demo => "demo",
        }
    }

    fn cookie(self) -> &'static str {
        match self {
            Self::Real => "cdi_workspace",
            Self::Demo => "cdi_demo",
        }
    }
}

#[derive(Clone, Debug)]
struct Tenant {
    id: String,
    kind: TenantKind,
    pro: bool,
}

pub fn api(state: AppState) -> Router<AppState> {
    let real = data_routes().layer(middleware::from_fn_with_state(
        state.clone(),
        real_tenant_guard,
    ));
    let demo = data_routes()
        .route("/reset", post(reset_demo))
        .route("/sample/extract", post(sample_extract))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            demo_tenant_guard,
        ));
    Router::new()
        .route("/session", post(start_real_session))
        .route("/demo/session", post(start_demo_session))
        .nest("/demo", demo)
        .merge(real)
}

fn data_routes() -> Router<AppState> {
    Router::new()
        .route("/license", post(verify_license))
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

async fn start_real_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let existing = tenant_from_cookie(&state, &headers, TenantKind::Real);
    let id = if let Some(id) = existing {
        let found: Option<(String,)> =
            sqlx::query_as("SELECT id FROM tenants WHERE id=? AND kind='real'")
                .bind(&id)
                .fetch_optional(&state.pool)
                .await?;
        found.map(|_| id)
    } else {
        None
    };
    let id = match id {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO tenants (id,kind,created_at) VALUES (?,'real',?)")
                .bind(&id)
                .bind(Utc::now().to_rfc3339())
                .execute(&state.pool)
                .await?;
            id
        }
    };
    session_response(&state, &headers, TenantKind::Real, &id, 31_536_000)
}

async fn start_demo_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    cleanup_expired_demos(&state).await?;
    let now = Utc::now().to_rfc3339();
    let existing = tenant_from_cookie(&state, &headers, TenantKind::Demo);
    let id = if let Some(id) = existing {
        sqlx::query_scalar::<_, String>(
            "SELECT id FROM tenants WHERE id=? AND kind='demo' AND expires_at>?",
        )
        .bind(&id)
        .bind(&now)
        .fetch_optional(&state.pool)
        .await?
    } else {
        None
    };
    let id = match id {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO tenants (id,kind,created_at,expires_at) VALUES (?,'demo',?,?)",
            )
            .bind(&id)
            .bind(&now)
            .bind((Utc::now() + Duration::hours(24)).to_rfc3339())
            .execute(&state.pool)
            .await?;
            seed_demo(&state, &id).await?;
            id
        }
    };
    session_response(&state, &headers, TenantKind::Demo, &id, 86_400)
}

fn session_response(
    state: &AppState,
    headers: &HeaderMap,
    kind: TenantKind,
    id: &str,
    max_age: u64,
) -> Result<Response, ApiError> {
    let signed = sign(state, &format!("{}:{id}", kind.as_str()));
    let cookie = cookie_header(kind.cookie(), &signed, max_age, forwarded_https(headers));
    let mut response = Json(json!({"ok":true, "workspace":kind.as_str()})).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not start the workspace".into(),
            )
        })?,
    );
    Ok(response)
}

async fn real_tenant_guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    tenant_guard(&state, request, next, TenantKind::Real).await
}

async fn demo_tenant_guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    tenant_guard(&state, request, next, TenantKind::Demo).await
}

async fn tenant_guard(
    state: &AppState,
    mut request: Request<Body>,
    next: Next,
    kind: TenantKind,
) -> Response {
    let Some(id) = tenant_from_cookie(state, request.headers(), kind) else {
        return ApiError(
            StatusCode::UNAUTHORIZED,
            "Start a workspace session before using the inbox".into(),
        )
        .into_response();
    };
    let now = Utc::now().to_rfc3339();
    let exists = if kind == TenantKind::Demo {
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM tenants WHERE id=? AND kind='demo' AND expires_at>?",
        )
        .bind(&id)
        .bind(now)
        .fetch_one(&state.pool)
        .await
    } else {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tenants WHERE id=? AND kind='real'")
            .bind(&id)
            .fetch_one(&state.pool)
            .await
    };
    if !matches!(exists, Ok(count) if count == 1) {
        return ApiError(
            StatusCode::UNAUTHORIZED,
            "This workspace expired. Reload to start a new one".into(),
        )
        .into_response();
    }
    let pro = kind == TenantKind::Demo || valid_pro_cookie(state, request.headers(), &id);
    request.extensions_mut().insert(Tenant { id, kind, pro });
    next.run(request).await
}

fn tenant_from_cookie(state: &AppState, headers: &HeaderMap, kind: TenantKind) -> Option<String> {
    let signed = cookie_value(headers, kind.cookie())?;
    let payload = verify(state, signed)?;
    let id = payload.strip_prefix(&format!("{}:", kind.as_str()))?;
    Uuid::parse_str(id).ok()?;
    Some(id.to_owned())
}

fn valid_pro_cookie(state: &AppState, headers: &HeaderMap, tenant_id: &str) -> bool {
    let Some(payload) = cookie_value(headers, "cdi_pro").and_then(|value| verify(state, value))
    else {
        return false;
    };
    let mut parts = payload.split(':');
    let valid = parts.next() == Some("pro")
        && parts.next() == Some(tenant_id)
        && parts
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .is_some_and(|expires| expires > unix_time());
    valid && parts.next().is_none()
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(key, value)| (key == name).then_some(value))
}

fn sign(state: &AppState, payload: &str) -> String {
    let encoded = URL_SAFE_NO_PAD.encode(payload.as_bytes());
    let mut mac = HmacSha256::new_from_slice(&state.session_secret).expect("HMAC key");
    mac.update(encoded.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{encoded}.{signature}")
}

fn verify(state: &AppState, signed: &str) -> Option<String> {
    let (payload, signature) = signed.split_once('.')?;
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    let mut mac = HmacSha256::new_from_slice(&state.session_secret).ok()?;
    mac.update(payload.as_bytes());
    mac.verify_slice(&signature).ok()?;
    String::from_utf8(URL_SAFE_NO_PAD.decode(payload).ok()?).ok()
}

fn cookie_header(name: &str, value: &str, max_age: u64, secure: bool) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        if secure { "; Secure" } else { "" }
    )
}

fn forwarded_https(headers: &HeaderMap) -> bool {
    headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("https"))
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Deserialize)]
struct LicenseInput {
    license: String,
}

async fn verify_license(
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
    headers: HeaderMap,
    Json(input): Json<LicenseInput>,
) -> Result<Response, ApiError> {
    if tenant.kind == TenantKind::Demo {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Licenses are not used in the sample workspace".into(),
        ));
    }
    let token = input.license.trim();
    if token.is_empty() || token.len() > 4096 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Enter the license token from your receipt".into(),
        ));
    }
    let endpoint = format!(
        "{}/api/v1/products/change-diff-inbox/verify",
        state.billing_base.trim_end_matches('/')
    );
    let response = reqwest::Client::new()
        .get(endpoint)
        .query(&[("license", token)])
        .send()
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::SERVICE_UNAVAILABLE,
                "The license service is unavailable. Free monitoring still works".into(),
            )
        })?;
    let verdict: Value = response.json().await.map_err(|_| {
        ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The license service returned an unreadable response".into(),
        )
    })?;
    let valid = verdict["valid"].as_bool().unwrap_or(false);
    let max_age = 86_400;
    let value = if valid {
        sign(
            &state,
            &format!("pro:{}:{}", tenant.id, unix_time() + max_age),
        )
    } else {
        String::new()
    };
    let cookie = cookie_header(
        "cdi_pro",
        &value,
        if valid { max_age } else { 0 },
        forwarded_https(&headers),
    );
    let mut result = Json(json!({
        "valid": valid,
        "reason": verdict["reason"].as_str().unwrap_or("invalid")
    }))
    .into_response();
    result.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| {
            ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not save the license result".into(),
            )
        })?,
    );
    Ok(result)
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

fn enforce_free_schedule(tenant: &Tenant, interval: i64) -> Result<(), ApiError> {
    if !tenant.pro && interval < 1440 {
        return Err(ApiError(
            StatusCode::PAYMENT_REQUIRED,
            "The free tier supports daily or weekly checks. A valid Pro license enables shorter schedules".into(),
        ));
    }
    Ok(())
}

async fn list_sources(
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
) -> Result<Json<Vec<Source>>, ApiError> {
    Ok(Json(
        sqlx::query_as("SELECT * FROM sources WHERE tenant_id=? ORDER BY created_at DESC")
            .bind(&tenant.id)
            .fetch_all(&state.pool)
            .await?,
    ))
}

async fn create_source(
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
    Json(input): Json<SourceInput>,
) -> Result<(StatusCode, Json<Source>), ApiError> {
    let (selector, mode, threshold, interval) = validate(&input)?;
    enforce_free_schedule(&tenant, interval)?;
    if !tenant.pro {
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM sources WHERE tenant_id=?")
            .bind(&tenant.id)
            .fetch_one(&state.pool)
            .await?;
        if count >= 5 {
            return Err(ApiError(
                StatusCode::PAYMENT_REQUIRED,
                "The free tier supports five sources. A valid Pro license enables more".into(),
            ));
        }
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO sources (id,tenant_id,name,url,selector,extract_mode,threshold,interval_minutes,next_check,created_at) VALUES (?,?,?,?,?,?,?,?,?,?)")
        .bind(&id).bind(&tenant.id).bind(input.name.trim()).bind(input.url.trim()).bind(selector).bind(mode).bind(threshold).bind(interval).bind(&now).bind(&now)
        .execute(&state.pool).await?;
    let source = sqlx::query_as("SELECT * FROM sources WHERE id=? AND tenant_id=?")
        .bind(id)
        .bind(&tenant.id)
        .fetch_one(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(source)))
}

async fn update_source(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
    Json(input): Json<SourceInput>,
) -> Result<Json<Source>, ApiError> {
    let (selector, mode, threshold, interval) = validate(&input)?;
    enforce_free_schedule(&tenant, interval)?;
    let existing = sqlx::query_as::<_, Source>("SELECT * FROM sources WHERE id=? AND tenant_id=?")
        .bind(&id)
        .bind(&tenant.id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Source not found".into()))?;
    let extraction_changed = existing.url != input.url.trim()
        || existing.selector != selector
        || existing.extract_mode != mode;
    let result = sqlx::query("UPDATE sources SET name=?,url=?,selector=?,extract_mode=?,threshold=?,interval_minutes=?,next_check=?, baseline=CASE WHEN ? THEN NULL ELSE baseline END, last_status=CASE WHEN ? THEN 'new' ELSE last_status END WHERE id=? AND tenant_id=?")
        .bind(input.name.trim()).bind(input.url.trim()).bind(selector).bind(mode).bind(threshold).bind(interval)
        .bind((Utc::now() + Duration::minutes(interval)).to_rfc3339()).bind(extraction_changed).bind(extraction_changed).bind(&id).bind(&tenant.id).execute(&state.pool).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "Source not found".into()));
    }
    Ok(Json(
        sqlx::query_as("SELECT * FROM sources WHERE id=? AND tenant_id=?")
            .bind(id)
            .bind(&tenant.id)
            .fetch_one(&state.pool)
            .await?,
    ))
}

async fn delete_source(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
) -> Result<StatusCode, ApiError> {
    let mut transaction = state.pool.begin().await?;
    sqlx::query("DELETE FROM changes WHERE source_id=? AND tenant_id=?")
        .bind(&id)
        .bind(&tenant.id)
        .execute(&mut *transaction)
        .await?;
    let result = sqlx::query("DELETE FROM sources WHERE id=? AND tenant_id=?")
        .bind(id)
        .bind(&tenant.id)
        .execute(&mut *transaction)
        .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "Source not found".into()));
    }
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn check_source(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
) -> Result<Json<crate::models::CheckResult>, ApiError> {
    if tenant.kind == TenantKind::Demo {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Sample sources do not request external pages".into(),
        ));
    }
    watcher::check_source(&state.pool, &tenant.id, &id)
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
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
    Query(query): Query<ChangeQuery>,
) -> Result<Json<Vec<Change>>, ApiError> {
    let review_state = query.state.unwrap_or_else(|| "all".into());
    if !["all", "unread", "reviewed", "archived"].contains(&review_state.as_str()) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Choose a supported review state".into(),
        ));
    }
    let source = query.source.unwrap_or_else(|| "all".into());
    let changes = sqlx::query_as::<_, Change>("SELECT c.*,s.name source_name,s.url source_url,s.selector FROM changes c JOIN sources s ON s.id=c.source_id AND s.tenant_id=c.tenant_id WHERE c.tenant_id=? AND (?='all' OR c.review_state=?) AND (?='all' OR c.source_id=?) ORDER BY c.created_at DESC LIMIT 250")
        .bind(&tenant.id).bind(&review_state).bind(&review_state).bind(&source).bind(&source).fetch_all(&state.pool).await?;
    Ok(Json(changes))
}

async fn review_change(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
    Json(input): Json<ReviewInput>,
) -> Result<Json<Value>, ApiError> {
    if let Some(ref review_state) = input.review_state {
        if !["unread", "reviewed", "archived"].contains(&review_state.as_str()) {
            return Err(ApiError(
                StatusCode::BAD_REQUEST,
                "Unsupported review state".into(),
            ));
        }
    }
    let useful = input.useful.map(i64::from);
    let result = sqlx::query("UPDATE changes SET review_state=COALESCE(?,review_state), useful=COALESCE(?,useful) WHERE id=? AND tenant_id=?")
        .bind(input.review_state).bind(useful).bind(id).bind(&tenant.id).execute(&state.pool).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "Change not found".into()));
    }
    Ok(Json(json!({"ok":true})))
}

async fn stats(
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
) -> Result<Json<Stats>, ApiError> {
    let result = sqlx::query_as::<_, Stats>("SELECT (SELECT count(*) FROM sources WHERE tenant_id=?) sources, (SELECT count(*) FROM changes WHERE tenant_id=? AND review_state='unread') unread, (SELECT count(*) FROM changes WHERE tenant_id=? AND useful=1) useful, (SELECT count(*) FROM changes WHERE tenant_id=? AND useful IS NOT NULL) rated")
        .bind(&tenant.id).bind(&tenant.id).bind(&tenant.id).bind(&tenant.id)
        .fetch_one(&state.pool).await?;
    Ok(Json(result))
}

async fn reset_demo(
    State(state): State<AppState>,
    Extension(tenant): Extension<Tenant>,
) -> Result<Json<Value>, ApiError> {
    let mut transaction = state.pool.begin().await?;
    sqlx::query("DELETE FROM changes WHERE tenant_id=?")
        .bind(&tenant.id)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM sources WHERE tenant_id=?")
        .bind(&tenant.id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    seed_demo(&state, &tenant.id).await?;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
struct SampleExtractInput {
    html: String,
    selector: String,
    mode: String,
    threshold: Option<f64>,
    previous: Option<String>,
    robots: Option<String>,
    path: Option<String>,
}

async fn sample_extract(Json(input): Json<SampleExtractInput>) -> Result<Json<Value>, ApiError> {
    if input.html.len() > 2_000_000 {
        return Err(ApiError(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Source is larger than the 2 MB safety limit".into(),
        ));
    }
    if let Some(robots) = input.robots.as_deref() {
        if !watcher::robots_allows(robots, input.path.as_deref().unwrap_or("/")) {
            return Err(ApiError(
                StatusCode::FORBIDDEN,
                "Blocked by this site's robots.txt".into(),
            ));
        }
    }
    let extracted = watcher::extract(&input.html, &input.selector, &input.mode)
        .map_err(|error| ApiError(StatusCode::BAD_REQUEST, error.to_string()))?;
    if extracted.len() > 250_000 {
        return Err(ApiError(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Selected content is larger than the 250 KB extraction limit".into(),
        ));
    }
    let previous = input.previous.unwrap_or_default();
    let ratio = watcher::change_ratio(&previous, &extracted);
    let threshold = input.threshold.unwrap_or(0.03);
    if !(0.0..=1.0).contains(&threshold) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Threshold must be between 0 and 100%".into(),
        ));
    }
    Ok(Json(json!({
        "extracted": extracted,
        "ratio": ratio,
        "outcome": if ratio < threshold { "noise" } else { "changed" },
        "summary": watcher::summarize(&previous, &extracted)
    })))
}

async fn cleanup_expired_demos(state: &AppState) -> Result<(), ApiError> {
    let expired: Vec<String> =
        sqlx::query_scalar("SELECT id FROM tenants WHERE kind='demo' AND expires_at<=? LIMIT 100")
            .bind(Utc::now().to_rfc3339())
            .fetch_all(&state.pool)
            .await?;
    for id in expired {
        let mut transaction = state.pool.begin().await?;
        sqlx::query("DELETE FROM changes WHERE tenant_id=?")
            .bind(&id)
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM sources WHERE tenant_id=?")
            .bind(&id)
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM tenants WHERE id=?")
            .bind(&id)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
    }
    Ok(())
}

async fn seed_demo(state: &AppState, tenant_id: &str) -> Result<(), ApiError> {
    struct Sample {
        id: &'static str,
        name: &'static str,
        url: &'static str,
        selector: &'static str,
        mode: &'static str,
        old_html: &'static str,
        new_html: &'static str,
        review_state: &'static str,
        useful: Option<i64>,
        minutes_ago: i64,
    }
    let samples = [
        Sample {
            id: "vendor-limits",
            name: "Northstar API limits",
            url: "https://docs.northstar.example/api/limits",
            selector: "#limits",
            mode: "table",
            old_html: "<table id='limits'><tr><th>Plan</th><th>Requests</th></tr><tr><td>Standard</td><td>1,000 per minute</td></tr><tr><td>Pro</td><td>5,000 per minute</td></tr></table>",
            new_html: "<table id='limits'><tr><th>Plan</th><th>Requests</th></tr><tr><td>Standard</td><td>1,500 per minute</td></tr><tr><td>Pro</td><td>7,500 per minute</td></tr></table>",
            review_state: "unread",
            useful: None,
            minutes_ago: 18,
        },
        Sample {
            id: "webhook-signature",
            name: "Acme webhook guide",
            url: "https://developers.acme.example/webhooks",
            selector: ".signature-example",
            mode: "code",
            old_html: "<pre class='signature-example'><code>verify(payload, signature)</code></pre>",
            new_html: "<pre class='signature-example'><code>verify(timestamp + '.' + payload, signature, 'sha256')</code></pre>",
            review_state: "reviewed",
            useful: Some(1),
            minutes_ago: 185,
        },
        Sample {
            id: "service-status",
            name: "Orbit service status",
            url: "https://status.orbit.example/",
            selector: "script[type='application/ld+json']",
            mode: "jsonld",
            old_html: "<script type='application/ld+json'>{\"status\":\"operational\",\"region\":\"eu-west\"}</script>",
            new_html: "<script type='application/ld+json'>{\"status\":\"maintenance\",\"region\":\"eu-west\",\"ends\":\"18:00 UTC\"}</script>",
            review_state: "archived",
            useful: Some(0),
            minutes_ago: 1440,
        },
    ];
    let now = Utc::now();
    let mut transaction = state.pool.begin().await?;
    for sample in samples {
        let source_id = format!("{tenant_id}-{}", sample.id);
        let old =
            watcher::extract(sample.old_html, sample.selector, sample.mode).map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Sample extraction failed".into(),
                )
            })?;
        let current =
            watcher::extract(sample.new_html, sample.selector, sample.mode).map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Sample extraction failed".into(),
                )
            })?;
        let ratio = watcher::change_ratio(&old, &current);
        let summary = watcher::summarize(&old, &current);
        let created = (now - Duration::minutes(sample.minutes_ago)).to_rfc3339();
        sqlx::query("INSERT INTO sources (id,tenant_id,name,url,selector,extract_mode,threshold,interval_minutes,enabled,baseline,last_checked,last_status,next_check,created_at) VALUES (?,?,?,?,?,?,?,?,0,?,?,'changed',?,?)")
            .bind(&source_id).bind(tenant_id).bind(sample.name).bind(sample.url).bind(sample.selector).bind(sample.mode).bind(0.03_f64).bind(1440_i64).bind(&current).bind(&created).bind((now + Duration::days(1)).to_rfc3339()).bind(&created)
            .execute(&mut *transaction).await?;
        sqlx::query("INSERT INTO changes (id,tenant_id,source_id,previous_text,current_text,change_ratio,summary,review_state,useful,created_at) VALUES (?,?,?,?,?,?,?,?,?,?)")
            .bind(format!("{source_id}-change")).bind(tenant_id).bind(&source_id).bind(old).bind(current).bind(ratio).bind(summary).bind(sample.review_state).bind(sample.useful).bind(created)
            .execute(&mut *transaction).await?;
    }
    transaction.commit().await?;
    Ok(())
}
