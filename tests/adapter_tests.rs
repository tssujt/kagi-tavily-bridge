use kagi_tavily_bridge::{
    adapter::{extract_response_from_kagi, search_response_from_kagi, validate_extract_request},
    models::{TavilyExtractRequest, TavilySearchRequest, TavilyUrls},
};
use std::time::Duration;

#[test]
fn search_response_uses_hermes_tavily_result_shape() {
    let request = TavilySearchRequest {
        api_key: Some("dummy".to_string()),
        query: "rust async web framework".to_string(),
        max_results: Some(1),
        include_images: false,
        include_answer: None,
        search_depth: Some("basic".to_string()),
        topic: Some("general".to_string()),
    };
    let mut kagi_result = kagi_openapi_rust::models::SearchResult::new(
        "https://example.com/axum".to_string(),
        "Axum".to_string(),
    );
    kagi_result.snippet = Some("Ergonomic Rust web framework.".to_string());
    let mut data = kagi_openapi_rust::models::Search200ResponseData::new();
    data.search = Some(vec![kagi_result]);
    let kagi_response = kagi_openapi_rust::models::Search200Response {
        meta: None,
        data: Some(Box::new(data)),
    };

    let response = search_response_from_kagi(&request, kagi_response, Duration::from_millis(25));

    let value = serde_json::to_value(response).unwrap();
    assert_eq!(value["query"], "rust async web framework");
    assert_eq!(value["results"][0]["title"], "Axum");
    assert_eq!(value["results"][0]["url"], "https://example.com/axum");
    assert_eq!(
        value["results"][0]["content"],
        "Ergonomic Rust web framework."
    );
    assert_eq!(value["results"][0]["raw_content"], serde_json::Value::Null);
    assert_eq!(value["results"][0]["score"], 1.0);
}

#[test]
fn extract_response_uses_hermes_tavily_document_shape() {
    let mut page =
        kagi_openapi_rust::models::PageOutput::new("https://example.com/guide".to_string());
    page.markdown = Some(Some("# Guide\n\nUseful content.".to_string()));
    let kagi_response = kagi_openapi_rust::models::ExtractResponse::new(
        kagi_openapi_rust::models::Meta::new(),
        vec![page],
    );

    let response = extract_response_from_kagi(kagi_response, Duration::from_millis(33));

    let value = serde_json::to_value(response).unwrap();
    assert_eq!(value["results"][0]["url"], "https://example.com/guide");
    assert_eq!(value["results"][0]["title"], "");
    assert_eq!(value["results"][0]["content"], "# Guide\n\nUseful content.");
    assert_eq!(
        value["results"][0]["raw_content"],
        "# Guide\n\nUseful content."
    );
    assert_eq!(value["failed_results"].as_array().unwrap().len(), 0);
    assert_eq!(value["failed_urls"].as_array().unwrap().len(), 0);
}

#[test]
fn extract_validation_rejects_more_urls_than_kagi_accepts() {
    let urls = (0..11)
        .map(|idx| format!("https://example.com/{idx}"))
        .collect::<Vec<_>>();

    let request = TavilyExtractRequest {
        api_key: Some("dummy".to_string()),
        urls: TavilyUrls::Many(urls),
        include_images: false,
        extract_depth: None,
        query: None,
        chunks_per_source: None,
    };
    let error = validate_extract_request(&request, request.api_key.as_deref()).unwrap_err();

    assert_eq!(
        error.error,
        "Kagi extract accepts at most 10 URLs per request"
    );
}
