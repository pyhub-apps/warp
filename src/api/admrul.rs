use async_trait::async_trait;
use chrono::Utc;
use log::{debug, info, warn};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use super::client::ClientConfig;
use super::deserializers::{single_or_vec, single_or_vec_or_null};
use super::types::{LawDetail, LawHistory, SearchItem, SearchResponse, UnifiedSearchRequest};
use super::{ApiType, LegalApiClient};
use crate::cache::key::CacheKeyGenerator;
use crate::error::{Result, WarpError};

const BASE_URL: &str = "https://www.law.go.kr/DRF/lawSearch.do";
const DETAIL_URL: &str = "https://www.law.go.kr/DRF/lawService.do";

/// ADMRUL (행정규칙) API Client
pub struct AdmrulClient {
    config: ClientConfig,
    client: Client,
}

impl AdmrulClient {
    pub fn new(config: ClientConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// Check cache for cached search response
    async fn check_search_cache(&self, cache_key: &str) -> Result<Option<SearchResponse>> {
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Checking cache for ADMRUL search key: {}", cache_key);
                if let Some(cached_data) = cache.get(cache_key).await? {
                    debug!("Cache hit for ADMRUL search key: {}", cache_key);
                    match serde_json::from_slice::<SearchResponse>(&cached_data) {
                        Ok(response) => {
                            info!("Successfully retrieved cached ADMRUL search response");
                            return Ok(Some(response));
                        }
                        Err(e) => {
                            warn!("Failed to deserialize cached ADMRUL search response: {}, removing from cache", e);
                            let _ = cache.remove(cache_key).await;
                        }
                    }
                } else {
                    debug!("Cache miss for ADMRUL search key: {}", cache_key);
                }
            }
        }
        Ok(None)
    }

    /// Store search response in cache
    async fn store_search_in_cache(
        &self,
        cache_key: &str,
        response: &SearchResponse,
    ) -> Result<()> {
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!(
                    "Storing ADMRUL search response in cache for key: {}",
                    cache_key
                );
                match serde_json::to_vec(response) {
                    Ok(serialized) => {
                        if let Err(e) = cache
                            .put(cache_key, serialized, self.api_type(), None)
                            .await
                        {
                            warn!("Failed to store ADMRUL search response in cache: {}", e);
                        } else {
                            info!("Successfully cached ADMRUL search response");
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to serialize ADMRUL search response for caching: {}",
                            e
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Execute HTTP request with retry logic
    async fn execute_with_retry(&self, url: String) -> Result<reqwest::Response> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                let delay = Duration::from_secs(2_u64.pow(attempt));
                sleep(delay).await;
            }

            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() {
                        last_error = Some(WarpError::ServerError(format!(
                            "Server error: {}",
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

    /// Parse ADMRUL search response
    fn parse_search_response(
        &self,
        raw: AdmrulSearchResponse,
        requested_page: u32,
    ) -> SearchResponse {
        let (rules, total_count, _page_no, page_size) = if let Some(search_data) = raw.admrul_search
        {
            (
                search_data.rules,
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
            (
                raw.rules.unwrap_or_default(),
                raw.total_count.unwrap_or(0),
                raw.page_no.unwrap_or(1),
                raw.page_size.unwrap_or(50),
            )
        };

        let items = rules
            .into_iter()
            .map(|rule| {
                let mut metadata = HashMap::new();
                if let Some(ref rule_type) = rule.rule_type {
                    metadata.insert("rule_type".to_string(), rule_type.clone());
                }
                if let Some(ref status) = rule.status {
                    metadata.insert("status".to_string(), status.clone());
                }

                SearchItem {
                    id: rule.rule_id,
                    title: rule.rule_name,
                    law_no: rule.rule_no,
                    law_type: rule.rule_type,
                    department: rule.department,
                    enforcement_date: rule.enforcement_date,
                    revision_date: rule.revision_date,
                    summary: rule.rule_summary,
                    source: "ADMRUL".to_string(),
                    metadata,
                }
            })
            .collect();

        SearchResponse {
            total_count,
            page_no: requested_page, // Use the requested page number
            page_size,
            items,
            source: "ADMRUL".to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl LegalApiClient for AdmrulClient {
    async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse> {
        if self.config.api_key.is_empty() {
            return Err(WarpError::NoApiKey);
        }

        // Generate cache key for this ADMRUL search request
        let cache_key = CacheKeyGenerator::admrul_key(
            "search",
            Some(&request.query),
            request.department.as_deref(),
            None, // rule_type not used in this request
            Some(request.page_no),
            Some(request.page_size),
        );

        // Check cache first
        if let Some(cached_response) = self.check_search_cache(&cache_key).await? {
            return Ok(cached_response);
        }

        // Calculate the starting position (offset) for the API
        let offset = ((request.page_no - 1) * request.page_size) + 1;

        let mut params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "admrul".to_string()),
            ("type", "JSON".to_string()),
            ("query", request.query.clone()),
            ("page", offset.to_string()), // Use offset instead of page number
            ("display", request.page_size.to_string()),
        ];

        // Add optional parameters
        if let Some(department) = &request.department {
            params.push(("org", department.clone()));
        }
        if let Some(date_from) = &request.date_from {
            params.push(("fromDate", date_from.clone()));
        }
        if let Some(date_to) = &request.date_to {
            params.push(("toDate", date_to.clone()));
        }

        let url = reqwest::Url::parse_with_params(BASE_URL, &params)
            .map_err(|e| WarpError::Parse(e.to_string()))?;

        let response = self.execute_with_retry(url.to_string()).await?;

        // Get response text for better error reporting
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let is_html = content_type.contains("text/html");

        let response_text = response.text().await.map_err(WarpError::Network)?;

        // Check if response is HTML (common when API key is invalid)
        if is_html || response_text.starts_with("<") {
            return Err(WarpError::ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: "API returned HTML instead of JSON. This usually means the API key is invalid or the service is unavailable.".to_string(),
                hint: Some("Please check your API key with 'warp config get law.key' and ensure it's valid.".to_string()),
            });
        }

        // Check if response is empty
        if response_text.trim().is_empty() {
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
        let raw: AdmrulSearchResponse = serde_json::from_str(&response_text).map_err(|e| {
            if response_text.contains("error") || response_text.contains("Error") {
                WarpError::ApiError {
                    code: "API_ERROR".to_string(),
                    message: format!(
                        "API returned an error: {}",
                        response_text.chars().take(200).collect::<String>()
                    ),
                    hint: Some("Check your API key and request parameters.".to_string()),
                }
            } else {
                WarpError::Parse(format!(
                    "Failed to parse administrative rule API response: {}",
                    e
                ))
            }
        })?;

        let response = self.parse_search_response(raw, request.page_no);

        // Store in cache
        if let Err(e) = self.store_search_in_cache(&cache_key, &response).await {
            warn!("Failed to cache ADMRUL search response: {}", e);
        }

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
                debug!("Checking cache for ADMRUL detail key: {}", cache_key);
                if let Some(cached_data) = cache.get(&cache_key).await? {
                    debug!("Cache hit for ADMRUL detail key: {}", cache_key);
                    match serde_json::from_slice::<LawDetail>(&cached_data) {
                        Ok(detail) => {
                            info!("Successfully retrieved cached ADMRUL law detail");
                            return Ok(detail);
                        }
                        Err(e) => {
                            warn!("Failed to deserialize cached ADMRUL detail: {}, removing from cache", e);
                            let _ = cache.remove(&cache_key).await;
                        }
                    }
                } else {
                    debug!("Cache miss for ADMRUL detail key: {}", cache_key);
                }
            }
        }

        let params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "admrul".to_string()),
            ("type", "JSON".to_string()),
            ("ID", id.to_string()),
        ];

        let url = reqwest::Url::parse_with_params(DETAIL_URL, &params)
            .map_err(|e| WarpError::Parse(e.to_string()))?;

        let response = self.execute_with_retry(url.to_string()).await?;
        let response_text = response.text().await.map_err(WarpError::Network)?;

        // Check if response is HTML
        if response_text.starts_with("<") {
            return Err(WarpError::ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: "API returned HTML instead of JSON.".to_string(),
                hint: Some("Please check your API key configuration.".to_string()),
            });
        }

        let raw: AdmrulDetailResponse = serde_json::from_str(&response_text).map_err(|e| {
            WarpError::Parse(format!("Failed to parse administrative rule detail: {}", e))
        })?;

        let detail = raw.into_law_detail();

        // Store detail in cache
        if let Some(ref cache) = self.config.cache {
            if !self.config.bypass_cache {
                debug!("Storing ADMRUL detail in cache for key: {}", cache_key);
                match serde_json::to_vec(&detail) {
                    Ok(serialized) => {
                        if let Err(e) = cache
                            .put(&cache_key, serialized, self.api_type(), None)
                            .await
                        {
                            warn!("Failed to store ADMRUL detail in cache: {}", e);
                        } else {
                            info!("Successfully cached ADMRUL law detail");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to serialize ADMRUL detail for caching: {}", e);
                    }
                }
            }
        }

        Ok(detail)
    }

    async fn get_history(&self, _id: &str) -> Result<LawHistory> {
        // Administrative rules don't have history in the same way laws do
        // Return empty history
        Ok(LawHistory {
            law_id: _id.to_string(),
            law_name: String::new(),
            total_count: 0,
            entries: vec![],
        })
    }

    fn api_type(&self) -> ApiType {
        ApiType::Admrul
    }

    fn base_url(&self) -> &str {
        BASE_URL
    }

    fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }
}

// ADMRUL-specific response structures
#[derive(Debug, Deserialize)]
struct AdmrulSearchResponse {
    #[serde(rename = "AdmrulSearch")]
    admrul_search: Option<AdmrulSearchData>,
    // Fallback for direct structure
    #[serde(rename = "totalCnt")]
    total_count: Option<u32>,
    #[serde(rename = "page")]
    page_no: Option<u32>,
    #[serde(rename = "display")]
    page_size: Option<u32>,
    #[serde(rename = "admrul", default, deserialize_with = "single_or_vec_or_null")]
    rules: Option<Vec<AdmrulRule>>,
}

#[derive(Debug, Deserialize)]
struct AdmrulSearchData {
    #[serde(rename = "totalCnt")]
    total_count: Option<String>,
    #[serde(rename = "page")]
    page_no: Option<String>,
    #[serde(rename = "display")]
    page_size: Option<String>,
    #[serde(rename = "admrul", default, deserialize_with = "single_or_vec")]
    rules: Vec<AdmrulRule>,
}

#[derive(Debug, Deserialize)]
struct AdmrulRule {
    #[serde(rename = "행정규칙ID")]
    rule_id: String,
    #[serde(rename = "행정규칙명")]
    rule_name: String,
    #[serde(rename = "행정규칙일련번호")]
    rule_no: Option<String>,
    #[serde(rename = "행정규칙종류")]
    rule_type: Option<String>,
    #[serde(rename = "소관부처")]
    department: Option<String>,
    #[serde(rename = "시행일자")]
    enforcement_date: Option<String>,
    #[serde(rename = "개정일자")]
    revision_date: Option<String>,
    #[serde(rename = "현행연혁")]
    status: Option<String>,
    #[serde(rename = "행정규칙요약")]
    rule_summary: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdmrulDetailResponse {
    #[serde(rename = "AdmrulService")]
    admrul: AdmrulDetailContent,
}

#[derive(Debug, Deserialize)]
struct AdmrulDetailContent {
    #[serde(rename = "행정규칙정보")]
    rule_info: AdmrulDetailInfo,
}

#[derive(Debug, Deserialize)]
struct AdmrulDetailInfo {
    #[serde(rename = "행정규칙ID")]
    rule_id: String,
    #[serde(rename = "행정규칙명")]
    rule_name: String,
    #[serde(rename = "행정규칙일련번호")]
    rule_no: Option<String>,
    #[serde(rename = "행정규칙종류")]
    rule_type: Option<String>,
    #[serde(rename = "소관부처")]
    department: Option<String>,
    #[serde(rename = "시행일자")]
    enforcement_date: Option<String>,
    #[serde(rename = "개정일자")]
    revision_date: Option<String>,
    #[serde(rename = "행정규칙내용")]
    rule_content: Option<String>,
    #[serde(rename = "개정이유")]
    revision_reason: Option<String>,
}

impl AdmrulDetailResponse {
    fn into_law_detail(self) -> LawDetail {
        let info = self.admrul.rule_info;

        let mut content = String::new();
        if let Some(rule_content) = &info.rule_content {
            content.push_str(rule_content);
        }
        if let Some(reason) = &info.revision_reason {
            content.push_str("\n\n【개정이유】\n");
            content.push_str(reason);
        }

        LawDetail {
            law_id: info.rule_id,
            law_name: info.rule_name,
            law_no: info.rule_no,
            law_type: info.rule_type,
            department: info.department,
            enforcement_date: info.enforcement_date,
            revision_date: info.revision_date,
            content,
            articles: vec![],
            attachments: vec![],
            related_laws: vec![],
            metadata: HashMap::new(),
        }
    }
}
