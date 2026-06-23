use async_trait::async_trait;
use thiserror::Error;

use crate::models::{TavilyExtractRequest, TavilySearchRequest};

pub type KagiResult<T> = Result<T, KagiError>;

#[derive(Debug, Error)]
pub enum KagiError {
    #[error("Kagi upstream error: {0}")]
    Upstream(String),
}

#[async_trait]
pub trait KagiClient: Send + Sync + 'static {
    async fn search(
        &self,
        api_key: &str,
        request: &TavilySearchRequest,
    ) -> KagiResult<kagi_openapi_rust::models::Search200Response>;

    async fn extract(
        &self,
        api_key: &str,
        request: &TavilyExtractRequest,
    ) -> KagiResult<kagi_openapi_rust::models::ExtractResponse>;
}

#[derive(Clone)]
pub struct KagiOpenApiClient;

impl KagiOpenApiClient {
    pub fn new() -> Self {
        Self
    }

    fn configuration(api_key: &str) -> kagi_openapi_rust::apis::configuration::Configuration {
        let mut configuration = kagi_openapi_rust::apis::configuration::Configuration::new();
        configuration.bearer_access_token = Some(api_key.to_string());
        configuration
    }
}

impl Default for KagiOpenApiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KagiClient for KagiOpenApiClient {
    async fn search(
        &self,
        api_key: &str,
        request: &TavilySearchRequest,
    ) -> KagiResult<kagi_openapi_rust::models::Search200Response> {
        let mut kagi_request = kagi_openapi_rust::models::SearchRequest::new(request.query.clone());
        kagi_request.limit = request
            .max_results
            .map(|max_results| max_results.clamp(1, 1024) as i32);
        kagi_request.workflow = match request.topic.as_deref() {
            Some("news") => Some(kagi_openapi_rust::models::search_request::Workflow::News),
            _ => Some(kagi_openapi_rust::models::search_request::Workflow::Search),
        };
        kagi_request.format = Some(kagi_openapi_rust::models::search_request::Format::Json);

        let configuration = Self::configuration(api_key);
        kagi_openapi_rust::apis::search_api::search(&configuration, kagi_request)
            .await
            .map_err(|err| KagiError::Upstream(err.to_string()))
    }

    async fn extract(
        &self,
        api_key: &str,
        request: &TavilyExtractRequest,
    ) -> KagiResult<kagi_openapi_rust::models::ExtractResponse> {
        let pages = request
            .url_list()
            .into_iter()
            .map(kagi_openapi_rust::models::PageInput::new)
            .collect::<Vec<_>>();
        let mut kagi_request = kagi_openapi_rust::models::ExtractRequest::new(pages);
        kagi_request.format = Some(kagi_openapi_rust::models::extract_request::Format::Json);

        let configuration = Self::configuration(api_key);
        kagi_openapi_rust::apis::extract_api::extract_content(&configuration, kagi_request)
            .await
            .map_err(|err| KagiError::Upstream(err.to_string()))
    }
}
