use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use openrtb_validator::app;
use serde_json::Value;
use tower::ServiceExt;

async fn send(method: &str, uri: &str, body: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    let response = app().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

#[tokio::test]
async fn health_returns_200() {
    let (status, json) = send("GET", "/health", "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json, serde_json::json!({ "status": "ok" }));
}

#[tokio::test]
async fn valid_request_returns_200_with_valid_true() {
    let body = r#"{
        "id": "req-1",
        "imp": [
            { "id": "imp-1", "banner": { "w": 300, "h": 250 }, "bidfloor": 0.5, "bidfloorcur": "USD" }
        ],
        "site": { "id": "site-1", "domain": "example.com" },
        "cur": ["USD"]
    }"#;
    let (status, json) = send("POST", "/validate", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["valid"], true);
    assert_eq!(json["errors"], serde_json::json!([]));
}

#[tokio::test]
async fn empty_id_and_imp_returns_422() {
    let (status, json) = send("POST", "/validate", r#"{"id":"","imp":[]}"#).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json["valid"], false);
    let codes: Vec<&str> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"EmptyId"));
    assert!(codes.contains(&"EmptyImp"));
}

#[tokio::test]
async fn warnings_only_request_returns_200_with_nonempty_errors() {
    let body = r#"{
        "id": "req-1",
        "imp": [
            { "id": "imp-1", "banner": { "w": 300, "h": 250 }, "bidfloorcur": "ZZZ" }
        ],
        "site": { "id": "site-1" }
    }"#;
    let (status, json) = send("POST", "/validate", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["valid"], true);
    let errors = json["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert!(errors.iter().all(|e| e["severity"] == "warning"));
}

#[tokio::test]
async fn non_json_body_returns_400() {
    let (status, json) = send("POST", "/validate", "not json").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["valid"], false);
    assert_eq!(json["errors"][0]["code"], "ParseError");
}
