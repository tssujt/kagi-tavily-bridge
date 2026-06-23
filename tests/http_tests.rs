use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use kagi_tavily_bridge::{
    http::{app_router, AppState},
    kagi::{KagiClient, KagiResult},
};
use std::sync::Arc;
use tower::ServiceExt;

#[derive(Clone)]
struct MockKagiClient;

#[async_trait]
impl KagiClient for MockKagiClient {
    async fn search(
        &self,
        _api_key: &str,
        _request: &kagi_tavily_bridge::models::TavilySearchRequest,
    ) -> KagiResult<kagi_openapi_rust::models::Search200Response> {
        let mut result = kagi_openapi_rust::models::SearchResult::new(
            "https://example.com/result".to_string(),
            "Result".to_string(),
        );
        result.snippet = Some("Search content".to_string());
        let mut data = kagi_openapi_rust::models::Search200ResponseData::new();
        data.search = Some(vec![result]);
        Ok(kagi_openapi_rust::models::Search200Response {
            meta: None,
            data: Some(Box::new(data)),
        })
    }

    async fn extract(
        &self,
        _api_key: &str,
        _request: &kagi_tavily_bridge::models::TavilyExtractRequest,
    ) -> KagiResult<kagi_openapi_rust::models::ExtractResponse> {
        let mut page =
            kagi_openapi_rust::models::PageOutput::new("https://example.com/doc".to_string());
        page.markdown = Some(Some("Extracted content".to_string()));
        Ok(kagi_openapi_rust::models::ExtractResponse::new(
            kagi_openapi_rust::models::Meta::new(),
            vec![page],
        ))
    }
}

#[tokio::test]
async fn search_endpoint_returns_hermes_fields() {
    let app = app_router(AppState::new(Arc::new(MockKagiClient)));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"api_key":"dummy","query":"hello","max_results":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["results"][0]["title"], "Result");
    assert_eq!(value["results"][0]["url"], "https://example.com/result");
    assert_eq!(value["results"][0]["content"], "Search content");
}

#[tokio::test]
async fn extract_endpoint_returns_hermes_fields() {
    let app = app_router(AppState::new(Arc::new(MockKagiClient)));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/extract")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"api_key":"dummy","urls":["https://example.com/doc"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["results"][0]["url"], "https://example.com/doc");
    assert_eq!(value["results"][0]["title"], "");
    assert_eq!(value["results"][0]["content"], "Extracted content");
    assert_eq!(value["results"][0]["raw_content"], "Extracted content");
    assert_eq!(value["failed_results"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn search_endpoint_requires_api_key() {
    let app = app_router(AppState::new(Arc::new(MockKagiClient)));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"query":"hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        value["error"],
        "Authorization bearer token or api_key is required"
    );
}

#[tokio::test]
async fn search_endpoint_accepts_authorization_bearer() {
    let app = app_router(AppState::new(Arc::new(MockKagiClient)));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .header("authorization", "Bearer kg_test")
                .body(Body::from(r#"{"query":"hello","max_results":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
