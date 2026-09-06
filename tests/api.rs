use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use base64::Engine;
use change_diff_inbox::app;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn test_app() -> Router {
    test_app_with_pool().await.0
}

async fn test_app_with_pool() -> (Router, sqlx::SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    (app(pool.clone(), "frontend/dist"), pool)
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn session(router: &Router, demo: bool, ip: &str) -> String {
    let path = if demo {
        "/api/demo/session"
    } else {
        "/api/session"
    };
    let response = router
        .clone()
        .oneshot(
            Request::post(path)
                .header("x-forwarded-for", ip)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

fn request(method: &str, uri: impl AsRef<str>, cookie: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri.as_ref())
        .header(header::COOKIE, cookie)
        .header("x-forwarded-for", "198.51.100.20");
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
        .unwrap()
}

fn source(name: &str) -> Value {
    json!({"name":name,"url":"https://example.com/","selector":"main","extract_mode":"selector","threshold":0.05,"interval_minutes":1440})
}

#[tokio::test]
async fn health_and_source_lifecycle() {
    let router = test_app().await;
    let health = router
        .clone()
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    assert_eq!(json_body(health).await["status"], "ok");

    let cookie = session(&router, false, "198.51.100.1").await;
    let created = router
        .clone()
        .oneshot(request(
            "POST",
            "/api/sources",
            &cookie,
            Some(source("Rust releases")),
        ))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let id = json_body(created).await["id"].as_str().unwrap().to_owned();

    let listed = router
        .clone()
        .oneshot(request("GET", "/api/sources", &cookie, None))
        .await
        .unwrap();
    assert_eq!(json_body(listed).await.as_array().unwrap().len(), 1);
    let updated = router
        .clone()
        .oneshot(request(
            "PUT",
            format!("/api/sources/{id}"),
            &cookie,
            Some(source("Rust home")),
        ))
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    let deleted = router
        .clone()
        .oneshot(request(
            "DELETE",
            format!("/api/sources/{id}"),
            &cookie,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn workspaces_cannot_read_or_change_each_others_records() {
    let router = test_app().await;
    let first = session(&router, false, "198.51.100.2").await;
    let second = session(&router, false, "198.51.100.3").await;
    let created = router
        .clone()
        .oneshot(request(
            "POST",
            "/api/sources",
            &first,
            Some(source("Private source")),
        ))
        .await
        .unwrap();
    let id = json_body(created).await["id"].as_str().unwrap().to_owned();
    let second_list = router
        .clone()
        .oneshot(request("GET", "/api/sources", &second, None))
        .await
        .unwrap();
    assert!(json_body(second_list).await.as_array().unwrap().is_empty());
    let cross_delete = router
        .clone()
        .oneshot(request(
            "DELETE",
            format!("/api/sources/{id}"),
            &second,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(cross_delete.status(), StatusCode::NOT_FOUND);
    let first_list = router
        .clone()
        .oneshot(request("GET", "/api/sources", &first, None))
        .await
        .unwrap();
    assert_eq!(json_body(first_list).await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn demo_is_seeded_resettable_and_separate() {
    let router = test_app().await;
    let real = session(&router, false, "198.51.100.4").await;
    let demo = session(&router, true, "198.51.100.5").await;
    let demo_sources = router
        .clone()
        .oneshot(request("GET", "/api/demo/sources", &demo, None))
        .await
        .unwrap();
    assert_eq!(json_body(demo_sources).await.as_array().unwrap().len(), 3);
    let real_sources = router
        .clone()
        .oneshot(request("GET", "/api/sources", &real, None))
        .await
        .unwrap();
    assert!(json_body(real_sources).await.as_array().unwrap().is_empty());
    let changes = router
        .clone()
        .oneshot(request("GET", "/api/demo/changes", &demo, None))
        .await
        .unwrap();
    let change_id = json_body(changes).await[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    router
        .clone()
        .oneshot(request(
            "PATCH",
            format!("/api/demo/changes/{change_id}"),
            &demo,
            Some(json!({"review_state":"archived"})),
        ))
        .await
        .unwrap();
    let reset = router
        .clone()
        .oneshot(request("POST", "/api/demo/reset", &demo, None))
        .await
        .unwrap();
    assert_eq!(reset.status(), StatusCode::OK);
    let after = router
        .oneshot(request(
            "GET",
            "/api/demo/changes?state=unread",
            &demo,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(json_body(after).await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn expired_demo_workspace_is_rejected_then_replaced_with_a_new_sample() {
    let (router, pool) = test_app_with_pool().await;
    let expired_cookie = session(&router, true, "198.51.100.45").await;
    let signed = expired_cookie.split_once('=').unwrap().1;
    let encoded = signed.split('.').next().unwrap();
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .unwrap();
    let tenant_id = String::from_utf8(payload)
        .unwrap()
        .strip_prefix("demo:")
        .unwrap()
        .to_owned();
    sqlx::query("UPDATE tenants SET expires_at='2000-01-01T00:00:00+00:00' WHERE id=?")
        .bind(&tenant_id)
        .execute(&pool)
        .await
        .unwrap();

    let rejected = router
        .clone()
        .oneshot(request("GET", "/api/demo/sources", &expired_cookie, None))
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);

    let replacement = router
        .clone()
        .oneshot(
            Request::post("/api/demo/session")
                .header(header::COOKIE, &expired_cookie)
                .header("x-forwarded-for", "198.51.100.45")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(replacement.status(), StatusCode::OK);
    let replacement_cookie = replacement
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    assert_ne!(replacement_cookie, expired_cookie);
    let replacement_sources = router
        .oneshot(request(
            "GET",
            "/api/demo/sources",
            &replacement_cookie,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        json_body(replacement_sources)
            .await
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[tokio::test]
async fn free_limits_are_enforced_by_the_api() {
    let router = test_app().await;
    let cookie = session(&router, false, "198.51.100.6").await;
    let short = json!({"name":"Fast source","url":"https://example.com/","selector":"main","extract_mode":"selector","threshold":0.05,"interval_minutes":15});
    let response = router
        .clone()
        .oneshot(request("POST", "/api/sources", &cookie, Some(short)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    for number in 1..=5 {
        let created = router
            .clone()
            .oneshot(request(
                "POST",
                "/api/sources",
                &cookie,
                Some(source(&format!("Source {number}"))),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
    }
    let sixth = router
        .oneshot(request(
            "POST",
            "/api/sources",
            &cookie,
            Some(source("Source 6")),
        ))
        .await
        .unwrap();
    assert_eq!(sixth.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn api_rate_limit_uses_forwarded_client_and_returns_retry_after() {
    let router = test_app().await;
    let cookie = session(&router, false, "198.51.100.7").await;
    let mut limited = None;
    for _ in 0..45 {
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/stats")
                    .header(header::COOKIE, &cookie)
                    .header("x-forwarded-for", "203.0.113.44, 10.0.0.8")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            limited = Some(response);
            break;
        }
    }
    let limited = limited.expect("request allowance should be enforced");
    assert!(limited.headers().contains_key(header::RETRY_AFTER));
}

#[tokio::test]
async fn validation_errors_are_actionable() {
    let router = test_app().await;
    let cookie = session(&router, false, "198.51.100.8").await;
    let response = router
        .oneshot(request(
            "POST",
            "/api/sources",
            &cookie,
            Some(json!({"name":"x","url":"file:///secret","interval_minutes":1})),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(json_body(response).await["error"]
        .as_str()
        .unwrap()
        .contains("Name"));
}
