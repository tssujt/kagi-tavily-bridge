use std::time::Duration;

use crate::models::{
    ErrorResponse, TavilyExtractRequest, TavilyExtractResponse, TavilyExtractResult,
    TavilyFailedResult, TavilySearchRequest, TavilySearchResponse, TavilySearchResult,
};

const MAX_KAGI_EXTRACT_URLS: usize = 10;

pub fn validate_api_key(api_key: Option<&str>) -> Result<(), ErrorResponse> {
    if api_key.is_none_or(str::is_empty) {
        return Err(ErrorResponse {
            error: "Authorization bearer token or api_key is required".to_string(),
        });
    }
    Ok(())
}

pub fn validate_extract_request(
    request: &TavilyExtractRequest,
    api_key: Option<&str>,
) -> Result<(), ErrorResponse> {
    validate_api_key(api_key)?;

    let urls = request.url_list();
    if urls.is_empty() {
        return Err(ErrorResponse {
            error: "urls must contain at least one URL".to_string(),
        });
    }
    if urls.len() > MAX_KAGI_EXTRACT_URLS {
        return Err(ErrorResponse {
            error: "Kagi extract accepts at most 10 URLs per request".to_string(),
        });
    }
    if let Some(chunks_per_source) = request.chunks_per_source {
        if request.query.is_none() {
            return Err(ErrorResponse {
                error: "chunks_per_source requires query".to_string(),
            });
        }
        if !(1..=5).contains(&chunks_per_source) {
            return Err(ErrorResponse {
                error: "chunks_per_source must be between 1 and 5".to_string(),
            });
        }
    }
    Ok(())
}

pub fn search_response_from_kagi(
    request: &TavilySearchRequest,
    response: kagi_openapi_rust::models::Search200Response,
    elapsed: Duration,
) -> TavilySearchResponse {
    let max_results = request.max_results.unwrap_or(5);
    let mut kagi_results = response
        .data
        .and_then(|data| data.search.clone())
        .unwrap_or_default();
    kagi_results.truncate(max_results);

    let results = kagi_results
        .into_iter()
        .enumerate()
        .map(|(idx, result)| TavilySearchResult {
            title: result.title,
            url: result.url,
            content: result.snippet.unwrap_or_default(),
            score: score_for_position(idx),
            raw_content: None,
            favicon: None,
        })
        .collect::<Vec<_>>();

    TavilySearchResponse {
        query: request.query.clone(),
        answer: None,
        images: Vec::new(),
        results,
        response_time: seconds(elapsed),
    }
}

pub fn extract_response_from_kagi(
    response: kagi_openapi_rust::models::ExtractResponse,
    elapsed: Duration,
) -> TavilyExtractResponse {
    let mut results = Vec::new();
    let mut failed_results = Vec::new();

    for page in response.data {
        match (page.markdown.flatten(), page.error) {
            (Some(markdown), _) => results.push(TavilyExtractResult {
                url: page.url,
                title: String::new(),
                content: markdown.clone(),
                raw_content: markdown,
                images: Vec::new(),
                favicon: None,
            }),
            (None, Some(error)) => failed_results.push(TavilyFailedResult {
                url: page.url,
                error,
            }),
            (None, None) => failed_results.push(TavilyFailedResult {
                url: page.url,
                error: "No extracted content returned by Kagi".to_string(),
            }),
        }
    }

    let failed_urls = failed_results
        .iter()
        .map(|failed| failed.url.clone())
        .collect::<Vec<_>>();

    TavilyExtractResponse {
        results,
        failed_results,
        failed_urls,
        response_time: seconds(elapsed),
    }
}

fn score_for_position(idx: usize) -> f64 {
    1.0 / (idx as f64 + 1.0)
}

fn seconds(duration: Duration) -> f64 {
    (duration.as_millis() as f64) / 1000.0
}
