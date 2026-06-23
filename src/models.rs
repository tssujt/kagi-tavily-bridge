use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TavilySearchRequest {
    #[serde(default)]
    pub api_key: Option<String>,
    pub query: String,
    #[serde(default)]
    pub max_results: Option<usize>,
    #[serde(default)]
    pub include_images: bool,
    #[serde(default)]
    pub include_answer: Option<serde_json::Value>,
    #[serde(default)]
    pub search_depth: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TavilySearchResponse {
    pub query: String,
    pub answer: Option<String>,
    pub images: Vec<String>,
    pub results: Vec<TavilySearchResult>,
    pub response_time: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TavilySearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub score: f64,
    pub raw_content: Option<String>,
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TavilyExtractRequest {
    #[serde(default)]
    pub api_key: Option<String>,
    pub urls: TavilyUrls,
    #[serde(default)]
    pub include_images: bool,
    #[serde(default)]
    pub extract_depth: Option<String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub chunks_per_source: Option<usize>,
}

impl TavilyExtractRequest {
    pub fn url_list(&self) -> Vec<String> {
        match &self.urls {
            TavilyUrls::One(url) => vec![url.clone()],
            TavilyUrls::Many(urls) => urls.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TavilyUrls {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TavilyExtractResponse {
    pub results: Vec<TavilyExtractResult>,
    pub failed_results: Vec<TavilyFailedResult>,
    pub failed_urls: Vec<String>,
    pub response_time: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TavilyExtractResult {
    pub url: String,
    pub title: String,
    pub content: String,
    pub raw_content: String,
    pub images: Vec<String>,
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TavilyFailedResult {
    pub url: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ErrorResponse {
    pub error: String,
}
