//! Integration tests for foxtive-axum.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use foxtive_axum::http::extractors::{ByteBody, JsonBody, StringBody};
use foxtive_axum::http::response::ext::StructResponseExt;
use foxtive_axum::http::HttpResult;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Debug, Serialize, Deserialize)]
struct TestPayload {
    name: String,
    value: i32,
}

fn test_router() -> Router {
    Router::new()
        .route("/json", post(json_handler))
        .route("/string", post(string_handler))
        .route("/bytes", post(bytes_handler))
        .route("/respond", get(respond_handler))
        .route("/respond-struct", get(respond_struct_handler))
}

async fn json_handler(body: JsonBody<TestPayload>) -> HttpResult {
    format!("{}:{}", body.name, body.value).respond()
}

async fn string_handler(body: StringBody) -> HttpResult {
    format!("received:{}", body.body()).respond()
}

async fn bytes_handler(body: ByteBody) -> HttpResult {
    format!("bytes:{}", body.len()).respond()
}

async fn respond_handler() -> HttpResult {
    "ok".respond()
}

async fn respond_struct_handler() -> HttpResult {
    let payload = TestPayload {
        name: "test".into(),
        value: 42,
    };
    payload.respond()
}

#[tokio::test]
async fn test_json_body_extractor() {
    let app = test_router();

    let body = serde_json::to_string(&TestPayload {
        name: "hello".into(),
        value: 99,
    })
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/json")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_string_body_extractor() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/string")
                .method("POST")
                .header("content-type", "text/plain")
                .body(Body::from("hello world"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_byte_body_extractor() {
    let app = test_router();

    let data = vec![1u8, 2, 3, 4, 5];

    let response = app
        .oneshot(
            Request::builder()
                .uri("/bytes")
                .method("POST")
                .header("content-type", "application/octet-stream")
                .body(Body::from(data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_respond_trait() {
    let app = test_router();

    let response = app
        .oneshot(Request::builder().uri("/respond").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_struct_response_ext() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/respond-struct")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Verify the response contains the struct data in the standard envelope
    let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(parsed["data"]["name"], "test");
    assert_eq!(parsed["data"]["value"], 42);
    assert!(parsed["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_404_fallback() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_method_not_allowed() {
    let app = test_router();

    // /respond only accepts GET, sending POST should get 405
    let response = app
        .oneshot(
            Request::builder()
                .uri("/respond")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_json_body_too_large() {
    let app = test_router();

    // Create a body larger than the default 2MB limit
    let large_body = "x".repeat(3 * 1024 * 1024); // 3MB

    let response = app
        .oneshot(
            Request::builder()
                .uri("/json")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(large_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
