use std::{sync::Arc, time::Instant};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use crate::{
    adapter::{
        extract_response_from_kagi, search_response_from_kagi, validate_api_key,
        validate_extract_request,
    },
    kagi::KagiClient,
    models::{ErrorResponse, TavilyExtractRequest, TavilySearchRequest},
};

#[derive(Clone)]
pub struct AppState {
    kagi: Arc<dyn KagiClient>,
}

impl AppState {
    pub fn new(kagi: Arc<dyn KagiClient>) -> Self {
        Self { kagi }
    }
}

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/search", post(search))
        .route("/extract", post(extract))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TavilySearchRequest>,
) -> Result<Json<crate::models::TavilySearchResponse>, AppError> {
    let api_key = request_api_key(&headers, request.api_key.as_deref());
    validate_api_key(api_key.as_deref()).map_err(AppError::BadRequest)?;
    let started = Instant::now();
    let response = state
        .kagi
        .search(api_key.as_deref().unwrap_or_default(), &request)
        .await?;
    Ok(Json(search_response_from_kagi(
        &request,
        response,
        started.elapsed(),
    )))
}

async fn extract(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TavilyExtractRequest>,
) -> Result<Json<crate::models::TavilyExtractResponse>, AppError> {
    let api_key = request_api_key(&headers, request.api_key.as_deref());
    validate_extract_request(&request, api_key.as_deref()).map_err(AppError::BadRequest)?;
    let started = Instant::now();
    let response = state
        .kagi
        .extract(api_key.as_deref().unwrap_or_default(), &request)
        .await?;
    Ok(Json(extract_response_from_kagi(
        response,
        started.elapsed(),
    )))
}

fn request_api_key(headers: &HeaderMap, body_api_key: Option<&str>) -> Option<String> {
    bearer_token(headers).or_else(|| body_api_key.map(str::to_string))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    header
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .map(str::to_string)
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(ErrorResponse),
    Upstream(String),
}

impl From<crate::kagi::KagiError> for AppError {
    fn from(value: crate::kagi::KagiError) -> Self {
        Self::Upstream(value.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(error) => (StatusCode::BAD_REQUEST, Json(error)).into_response(),
            AppError::Upstream(error) => {
                (StatusCode::BAD_GATEWAY, Json(ErrorResponse { error })).into_response()
            }
        }
    }
}
