use async_trait::async_trait;
use chrono::Utc;
use log::{debug, info, warn};
use reqwest::{Client, Response};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use super::client::ClientConfig;
use super::deserializers::single_or_vec;
use super::http_client::{create_custom_client, create_custom_client_for_benchmarks};
use super::types::*;
use super::{ApiType, LegalApiClient};
use crate::cache::key::CacheKeyGenerator;
use crate::error::{Result, WarpError};
use crate::metrics::{get_global_metrics, OperationTimer};

const BASE_URL: &str = "https://www.law.go.kr/DRF/lawService.do";
const SEARCH_URL: &str = "https://www.law.go.kr/DRF/lawSearch.do";

/// NLIC (National Law Information Center) API client
pub struct NlicClient {
    config: ClientConfig,
    http_client: Client,
}

impl NlicClient {
    /// Create a new NLIC client with optimized HTTP client
    pub fn new(config: ClientConfig) -> Self {
        // Use appropriate HTTP client based on benchmark mode
        let http_client = if config.benchmark_mode {
            create_custom_client_for_benchmarks(config.timeout, &config.user_agent)
        } else {
            create_custom_client(config.timeout, &config.user_agent)
        };

        Self {
            config,
            http_client,
        }
    }

    /// Check cache for cached response
    async fn check_cache(&self, cache_key: &str) -> Result<Option<SearchResponse>> {
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Checking cache for key: {}", cache_key);
                if let Some(cached_data) = cache.get(cache_key).await? {
                    debug!("Cache hit for key: {}", cache_key);
                    match serde_json::from_slice::<SearchResponse>(&cached_data) {
                        Ok(response) => {
                            info!("Successfully retrieved cached search response");
                            return Ok(Some(response));
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize cached response: {}, removing from cache",
                                e
                            );
                            let _ = cache.remove(cache_key).await;
                        }
                    }
                } else {
                    debug!("Cache miss for key: {}", cache_key);
                }
            } else {
                debug!("Cache bypassed for key: {}", cache_key);
            }
        }
        Ok(None)
    }

    /// Store response in cache
    async fn store_in_cache(&self, cache_key: &str, response: &SearchResponse) -> Result<()> {
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Storing response in cache for key: {}", cache_key);
                match serde_json::to_vec(response) {
                    Ok(serialized) => {
                        if let Err(e) = cache
                            .put(cache_key, serialized, self.api_type(), None)
                            .await
                        {
                            warn!("Failed to store response in cache: {}", e);
                        } else {
                            info!("Successfully cached search response");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to serialize response for caching: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Execute request with retry logic
    async fn execute_with_retry(&self, url: String) -> Result<Response> {
        let mut last_error = None;
        let mut retry_delay = Duration::from_millis(self.config.retry_base_delay);

        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                sleep(retry_delay).await;
                retry_delay *= 2; // Exponential backoff
            }

            match self.http_client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().as_u16() == 429 {
                        last_error = Some(WarpError::RateLimit);
                    } else if response.status().is_server_error() {
                        last_error = Some(WarpError::ServerError(format!(
                            "Server returned status {}",
                            response.status()
                        )));
                    } else {
                        return Err(WarpError::ApiError {
                            code: response.status().to_string(),
                            message: format!(
                                "API request failed with status {}",
                                response.status()
                            ),
                            hint: None,
                        });
                    }
                }
                Err(e) => {
                    last_error = Some(WarpError::Network(e));
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| WarpError::Other("Request failed after all retries".to_string())))
    }

    /// Parse NLIC search response
    fn parse_search_response(
        &self,
        raw: NlicSearchResponse,
        requested_page: u32,
    ) -> SearchResponse {
        // Handle nested structure or direct structure
        let (laws, total_count, _page_no, page_size) = if let Some(search_data) = raw.law_search {
            // New nested structure - parse strings to numbers
            (
                search_data.laws,
                search_data
                    .total_count
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0),
                search_data
                    .page_no
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1),
                search_data
                    .page_size
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(50),
            )
        } else {
            // Fallback to direct structure
            (
                raw.laws.unwrap_or_default(),
                raw.total_count.unwrap_or(0),
                raw.page_no.unwrap_or(1),
                raw.page_size.unwrap_or(50),
            )
        };

        let items = laws
            .into_iter()
            .map(|law| {
                let mut metadata = HashMap::new();
                if let Some(ref detail) = law.law_detail_link {
                    metadata.insert("detail_link".to_string(), detail.clone());
                }
                if let Some(ref full) = law.law_full_link {
                    metadata.insert("full_link".to_string(), full.clone());
                }

                SearchItem {
                    id: law.law_id,
                    title: law.law_name,
                    law_no: law.law_no,
                    law_type: law.law_type,
                    department: law.department,
                    enforcement_date: law.enforcement_date,
                    revision_date: law.revision_date,
                    summary: law.law_summary,
                    source: "NLIC".to_string(),
                    metadata,
                }
            })
            .collect();

        SearchResponse {
            total_count,
            page_no: requested_page, // Use the requested page number, not the API's offset
            page_size,
            items,
            source: "NLIC".to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl LegalApiClient for NlicClient {
    async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse> {
        // Start performance monitoring
        let timer = OperationTimer::start("nlic_search".to_string(), get_global_metrics());
        let metrics = get_global_metrics();

        if self.config.api_key.is_empty() {
            timer.finish_failure();
            return Err(WarpError::NoApiKey);
        }

        // Generate cache key for this request
        let cache_key = CacheKeyGenerator::nlic_key(
            "search",
            Some(&request.query),
            request.law_type.as_deref(),
            Some(request.page_no),
            Some(request.page_size),
        );

        // Check cache first
        if let Some(cached_response) = self.check_cache(&cache_key).await? {
            metrics.record_cache_hit("nlic");
            timer.finish_success();
            return Ok(cached_response);
        }

        metrics.record_cache_miss("nlic");

        // Calculate the starting position (offset) for the API
        // The API seems to expect an offset rather than a page number
        // For page 1 with size 10, offset should be 1
        // For page 2 with size 10, offset should be 11
        let offset = ((request.page_no - 1) * request.page_size) + 1;

        let mut params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "law".to_string()),
            ("type", "JSON".to_string()),
            ("query", request.query.clone()),
            ("page", offset.to_string()), // Use offset instead of page number
            ("display", request.page_size.to_string()),
        ];

        // Add optional parameters
        if let Some(law_type) = &request.law_type {
            params.push(("MST", law_type.clone()));
        }
        if let Some(department) = &request.department {
            params.push(("ORG", department.clone()));
        }
        if let Some(date_from) = &request.date_from {
            params.push(("efYd", date_from.clone()));
        }

        // Build URL with query parameters
        let url = reqwest::Url::parse_with_params(SEARCH_URL, &params)
            .map_err(|e| WarpError::Parse(e.to_string()))?;

        let response = self.execute_with_retry(url.to_string()).await?;

        // Check response status and content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let is_html = content_type.contains("text/html");

        // Get response text for better error reporting
        let response_text = response.text().await.map_err(WarpError::Network)?;

        // Check if response is HTML (common when API key is invalid)
        if is_html || response_text.starts_with("<") {
            timer.finish_failure();
            return Err(WarpError::ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: "API returned HTML instead of JSON. This usually means the API key is invalid or the service is unavailable.".to_string(),
                hint: Some("Please check your API key with 'warp config get law.nlic.key' and ensure it's valid.".to_string()),
            });
        }

        // Check if response is empty
        if response_text.trim().is_empty() {
            timer.finish_failure();
            return Err(WarpError::ApiError {
                code: "EMPTY_RESPONSE".to_string(),
                message: "API returned an empty response.".to_string(),
                hint: Some(
                    "This might indicate an invalid API key or server issue. Try again later."
                        .to_string(),
                ),
            });
        }

        // Try to parse JSON
        let raw: NlicSearchResponse = match serde_json::from_str(&response_text) {
            Ok(parsed) => parsed,
            Err(e) => {
                timer.finish_failure();
                // Try to provide more context about the error
                if response_text.contains("error") || response_text.contains("Error") {
                    return Err(WarpError::ApiError {
                        code: "API_ERROR".to_string(),
                        message: format!(
                            "API returned an error: {}",
                            response_text.chars().take(200).collect::<String>()
                        ),
                        hint: Some("Check your API key and request parameters.".to_string()),
                    });
                } else {
                    return Err(WarpError::Parse(format!(
                        "Failed to parse API response as JSON: {}. Response starts with: {}",
                        e,
                        response_text.chars().take(100).collect::<String>()
                    )));
                }
            }
        };

        let response = self.parse_search_response(raw, request.page_no);

        // Store in cache
        if let Err(e) = self.store_in_cache(&cache_key, &response).await {
            warn!("Failed to cache response: {}", e);
        }

        timer.finish_success();
        Ok(response)
    }

    async fn get_detail(&self, id: &str) -> Result<LawDetail> {
        if self.config.api_key.is_empty() {
            return Err(WarpError::NoApiKey);
        }

        // Generate cache key for detail request
        let cache_key = format!("{}:detail:{}", self.api_type().as_str(), id);

        // Check cache for detail response
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Checking cache for detail key: {}", cache_key);
                if let Some(cached_data) = cache.get(&cache_key).await? {
                    debug!("Cache hit for detail key: {}", cache_key);
                    match serde_json::from_slice::<LawDetail>(&cached_data) {
                        Ok(detail) => {
                            info!("Successfully retrieved cached law detail");
                            return Ok(detail);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize cached detail: {}, removing from cache",
                                e
                            );
                            let _ = cache.remove(&cache_key).await;
                        }
                    }
                } else {
                    debug!("Cache miss for detail key: {}", cache_key);
                }
            }
        }

        let params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "law".to_string()),
            ("type", "JSON".to_string()),
            ("MST", id.to_string()),
            ("JO_YN", "Y".to_string()), // Include articles
        ];

        let url = reqwest::Url::parse_with_params(BASE_URL, &params)
            .map_err(|e| WarpError::Parse(e.to_string()))?;

        let response = self.execute_with_retry(url.to_string()).await?;

        // Get response text for better error reporting
        let response_text = response.text().await.map_err(WarpError::Network)?;

        // Check if response is HTML or empty
        if response_text.starts_with("<") {
            return Err(WarpError::ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: "API returned HTML instead of JSON.".to_string(),
                hint: Some("Please check your API key configuration.".to_string()),
            });
        }

        let raw: NlicDetailResponse = serde_json::from_str(&response_text)
            .map_err(|e| WarpError::Parse(format!("Failed to parse detail response: {}", e)))?;

        // Convert NLIC response to unified format
        let detail = raw.law.into_law_detail();

        // Store detail in cache
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Storing detail in cache for key: {}", cache_key);
                match serde_json::to_vec(&detail) {
                    Ok(serialized) => {
                        if let Err(e) = cache
                            .put(&cache_key, serialized, self.api_type(), None)
                            .await
                        {
                            warn!("Failed to store detail in cache: {}", e);
                        } else {
                            info!("Successfully cached law detail");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to serialize detail for caching: {}", e);
                    }
                }
            }
        }

        Ok(detail)
    }

    async fn get_history(&self, id: &str) -> Result<LawHistory> {
        if self.config.api_key.is_empty() {
            return Err(WarpError::NoApiKey);
        }

        // Generate cache key for history request
        let cache_key = format!("{}:history:{}", self.api_type().as_str(), id);

        // Check cache for history response
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Checking cache for history key: {}", cache_key);
                if let Some(cached_data) = cache.get(&cache_key).await? {
                    debug!("Cache hit for history key: {}", cache_key);
                    match serde_json::from_slice::<LawHistory>(&cached_data) {
                        Ok(history) => {
                            info!("Successfully retrieved cached law history");
                            return Ok(history);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize cached history: {}, removing from cache",
                                e
                            );
                            let _ = cache.remove(&cache_key).await;
                        }
                    }
                } else {
                    debug!("Cache miss for history key: {}", cache_key);
                }
            }
        }

        let params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "lsHstry".to_string()),
            ("type", "JSON".to_string()),
            ("MST", id.to_string()),
        ];

        let url = reqwest::Url::parse_with_params(BASE_URL, &params)
            .map_err(|e| WarpError::Parse(e.to_string()))?;

        let response = self.execute_with_retry(url.to_string()).await?;

        // Get response text for better error reporting
        let response_text = response.text().await.map_err(WarpError::Network)?;

        // Check if response is HTML or empty
        if response_text.starts_with("<") {
            return Err(WarpError::ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: "API returned HTML instead of JSON.".to_string(),
                hint: Some("Please check your API key configuration.".to_string()),
            });
        }

        let raw: NlicHistoryResponse = serde_json::from_str(&response_text)
            .map_err(|e| WarpError::Parse(format!("Failed to parse history response: {}", e)))?;

        let history = raw.into_law_history();

        // Store history in cache
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Storing history in cache for key: {}", cache_key);
                match serde_json::to_vec(&history) {
                    Ok(serialized) => {
                        if let Err(e) = cache
                            .put(&cache_key, serialized, self.api_type(), None)
                            .await
                        {
                            warn!("Failed to store history in cache: {}", e);
                        } else {
                            info!("Successfully cached law history");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to serialize history for caching: {}", e);
                    }
                }
            }
        }

        Ok(history)
    }

    fn api_type(&self) -> ApiType {
        ApiType::Nlic
    }

    fn base_url(&self) -> &str {
        BASE_URL
    }

    fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }
}

// NLIC-specific response structures
// The actual API returns a nested structure: { "LawSearch": { "law": [...] } }
#[derive(Debug, Deserialize)]
struct NlicSearchResponse {
    #[serde(rename = "LawSearch")]
    law_search: Option<NlicSearchData>,
    // Fallback for direct structure (older API format)
    #[serde(rename = "totalCnt")]
    total_count: Option<u32>,
    #[serde(rename = "page")]
    page_no: Option<u32>,
    #[serde(rename = "display")]
    page_size: Option<u32>,
    #[serde(
        rename = "law",
        default,
        deserialize_with = "super::deserializers::single_or_vec_or_null"
    )]
    laws: Option<Vec<NlicLaw>>,
}

#[derive(Debug, Deserialize)]
struct NlicSearchData {
    #[serde(rename = "totalCnt")]
    total_count: Option<String>, // API returns string
    #[serde(rename = "page")]
    page_no: Option<String>, // API returns string
    #[serde(rename = "display")]
    page_size: Option<String>, // API returns string
    #[serde(rename = "law", default, deserialize_with = "single_or_vec")]
    laws: Vec<NlicLaw>,
}

#[derive(Debug, Deserialize)]
struct NlicLaw {
    #[serde(rename = "법령ID")]
    law_id: String,
    #[serde(rename = "법령명한글")]
    law_name: String,
    #[serde(rename = "법령일련번호")]
    law_no: Option<String>,
    #[serde(rename = "법종구분명")]
    law_type: Option<String>,
    #[serde(rename = "소관부처명")]
    department: Option<String>,
    #[serde(rename = "시행일자")]
    enforcement_date: Option<String>,
    #[serde(rename = "개정일자")]
    revision_date: Option<String>,
    #[serde(rename = "법령요약내용")]
    law_summary: Option<String>,
    #[serde(rename = "법령상세링크")]
    law_detail_link: Option<String>,
    #[serde(rename = "법령원문링크")]
    law_full_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NlicDetailResponse {
    #[serde(rename = "법령")]
    law: NlicDetailContent,
}

#[derive(Debug, Deserialize)]
struct NlicDetailContent {
    #[serde(rename = "법령ID")]
    law_id: String,
    #[serde(rename = "법령명한글")]
    law_name: String,
    #[serde(rename = "법령일련번호")]
    law_no: Option<String>,
    #[serde(rename = "법종구분명")]
    law_type: Option<String>,
    #[serde(rename = "소관부처명")]
    department: Option<String>,
    #[serde(rename = "시행일자")]
    enforcement_date: Option<String>,
    #[serde(rename = "개정일자")]
    revision_date: Option<String>,
    #[serde(rename = "조문", default)]
    articles: Vec<NlicArticle>,
}

#[derive(Debug, Deserialize)]
struct NlicArticle {
    #[serde(rename = "조문키")]
    #[allow(dead_code)]
    article_key: String,
    #[serde(rename = "조문번호")]
    article_number: String,
    #[serde(rename = "조문제목")]
    article_title: Option<String>,
    #[serde(rename = "조문내용")]
    article_content: String,
}

impl NlicDetailContent {
    fn into_law_detail(self) -> LawDetail {
        let articles = self
            .articles
            .into_iter()
            .map(|a| {
                Article {
                    number: a.article_number,
                    title: a.article_title,
                    content: a.article_content,
                    paragraphs: vec![], // TODO: Parse paragraphs from content
                }
            })
            .collect();

        LawDetail {
            law_id: self.law_id,
            law_name: self.law_name,
            law_no: self.law_no,
            law_type: self.law_type,
            department: self.department,
            enforcement_date: self.enforcement_date,
            revision_date: self.revision_date,
            content: String::new(), // TODO: Combine articles into full content
            articles,
            attachments: vec![],
            related_laws: vec![],
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct NlicHistoryResponse {
    #[serde(rename = "법령연혁")]
    history: Vec<NlicHistoryEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NlicHistoryEntry {
    #[serde(rename = "법령ID")]
    law_id: String,
    #[serde(rename = "법령명")]
    law_name: String,
    #[serde(rename = "개정구분")]
    revision_type: String,
    #[serde(rename = "공포일자")]
    announcement_date: String,
    #[serde(rename = "시행일자")]
    enforcement_date: Option<String>,
    #[serde(rename = "개정이유")]
    reason: Option<String>,
}

impl NlicHistoryResponse {
    fn into_law_history(self) -> LawHistory {
        let mut entries = vec![];
        for (idx, entry) in self.history.into_iter().enumerate() {
            entries.push(HistoryEntry {
                revision_no: idx as u32 + 1,
                revision_date: entry.announcement_date,
                enforcement_date: entry.enforcement_date,
                revision_type: entry.revision_type,
                reason: entry.reason,
                changed_articles: vec![], // TODO: Parse changed articles
            });
        }

        LawHistory {
            law_id: entries.first().map(|_| String::new()).unwrap_or_default(),
            law_name: String::new(), // Will be filled from first entry
            total_count: entries.len() as u32,
            entries,
        }
    }
}
