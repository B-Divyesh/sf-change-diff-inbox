use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use change_diff_inbox::app;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    app(pool, "frontend/dist")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
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

    let payload = json!({"name":"Rust releases","url":"https://www.rust-lang.org/","selector":"main","extract_mode":"selector","threshold":0.05,"interval_minutes":60});
    let created = router
        .clone()
        .oneshot(
            Request::post("/api/sources")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let source = json_body(created).await;
    let id = source["id"].as_str().unwrap();

    let listed = router
        .clone()
        .oneshot(Request::get("/api/sources").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(json_body(listed).await.as_array().unwrap().len(), 1);
    let stats = router
        .clone()
        .oneshot(Request::get("/api/stats").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(json_body(stats).await["sources"], 1);
    let changes = router
        .clone()
        .oneshot(Request::get("/api/changes").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(json_body(changes).await.as_array().unwrap().is_empty());

    let updated = json!({"name":"Rust home","url":"https://www.rust-lang.org/","selector":"body","extract_mode":"selector","threshold":0.08,"interval_minutes":120});
    let response = router
        .clone()
        .oneshot(
            Request::put(format!("/api/sources/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(updated.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let deleted = router
        .clone()
        .oneshot(
            Request::delete(format!("/api/sources/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn validation_errors_are_actionable() {
    let router = test_app().await;
    let payload = json!({"name":"x","url":"file:///secret","interval_minutes":1});
    let response = router
        .oneshot(
            Request::post("/api/sources")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(json_body(response).await["error"]
        .as_str()
        .unwrap()
        .contains("Name"));
}

#[tokio::test]
async fn check_and_review_routes_report_state() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    sqlx::query("INSERT INTO sources (id,name,url,selector,created_at) VALUES ('source-1','Vendor plans','https://example.com','h1','2026-08-27T00:00:00Z')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO changes (id,source_id,previous_text,current_text,change_ratio,summary,created_at) VALUES ('change-1','source-1','ten','twelve',0.5,'Price changed','2026-08-27T00:00:00Z')")
        .execute(&pool).await.unwrap();
    let router = app(pool, "frontend/dist");

    let reviewed = router
        .clone()
        .oneshot(
            Request::patch("/api/changes/change-1")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"review_state":"reviewed","useful":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reviewed.status(), StatusCode::OK);

    let changes = router
        .clone()
        .oneshot(
            Request::get("/api/changes?state=reviewed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = json_body(changes).await;
    assert_eq!(body[0]["useful"], 1);

    let missing = router
        .oneshot(
            Request::post("/api/sources/missing/check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
}
