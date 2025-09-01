use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use crate::error::{Result, WarpError};
use super::{ApiType, LegalApiClient};
use super::client::ClientConfig;
use super::types::{UnifiedSearchRequest, SearchResponse, SearchItem, LawDetail, LawHistory};

const BASE_URL: &str = "https://www.elis.go.kr/api/";
const SEARCH_URL: &str = "https://www.elis.go.kr/api/search";

/// ELIS (자치법규정보시스템) API Client
pub struct ElisClient {
    config: ClientConfig,
    client: Client,
}

impl ElisClient {
    pub fn new(config: ClientConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .unwrap_or_default();
        
        Self { config, client }
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
                        last_error = Some(WarpError::ServerError(
                            format!("Server error: {}", response.status())
                        ));
                    } else {
                        return Err(WarpError::ApiError {
                            code: response.status().to_string(),
                            message: format!("API request failed with status {}", response.status()),
                            hint: None,
                        });
                    }
                }
                Err(e) => {
                    last_error = Some(WarpError::Network(e));
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            WarpError::Other("Request failed after all retries".to_string())
        }))
    }
    
    /// Parse ELIS search response
    fn parse_search_response(&self, raw: ElisSearchResponse, requested_page: u32) -> SearchResponse {
        let laws = if let Some(search_data) = raw.law_search {
            search_data.laws
        } else {
            raw.laws.unwrap_or_default()
        };
        
        let items = laws.into_iter().map(|law| {
            let mut metadata = HashMap::new();
            if let Some(ref region) = law.region {
                metadata.insert("region".to_string(), region.clone());
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
                source: "ELIS".to_string(),
                metadata,
            }
        }).collect();
        
        SearchResponse {
            total_count: raw.total_count.unwrap_or(0),
            page_no: requested_page,  // Use the requested page number
            page_size: raw.page_size.unwrap_or(50),
            items,
            source: "ELIS".to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl LegalApiClient for ElisClient {
    async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse> {
        if self.config.api_key.is_empty() {
            return Err(WarpError::NoApiKey);
        }
        
        // Calculate the starting position (offset) for the API
        let offset = ((request.page_no - 1) * request.page_size) + 1;
        
        let mut params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "law".to_string()),
            ("type", "JSON".to_string()),
            ("query", request.query.clone()),
            ("page", offset.to_string()),  // Use offset instead of page number
            ("display", request.page_size.to_string()),
        ];
        
        // Add optional parameters
        if let Some(region) = &request.region {
            params.push(("region", region.clone()));
        }
        if let Some(law_type) = &request.law_type {
            params.push(("lsKndCd", law_type.clone()));
        }
        
        let url = reqwest::Url::parse_with_params(SEARCH_URL, &params)
            .map_err(|e| WarpError::Parse(e.to_string()))?;
        
        
        let response = self.execute_with_retry(url.to_string()).await?;
        
        // Get response text for better error reporting
        let response_text = response.text().await
            .map_err(|e| WarpError::Network(e))?;
        
        // Check if response is HTML (common when API key is invalid)
        if response_text.starts_with("<") {
            return Err(WarpError::ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: "API returned HTML instead of JSON. This usually means the API key is invalid or the service is unavailable.".to_string(),
                hint: Some("Please check your API key with 'warp config get law.elis.key' and ensure it's valid.".to_string()),
            });
        }
        
        // Try to parse JSON
        let raw: ElisSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                WarpError::Parse(format!("Failed to parse ELIS response: {}", e))
            })?;
        
        Ok(self.parse_search_response(raw, request.page_no))
    }
    
    async fn get_detail(&self, id: &str) -> Result<LawDetail> {
        if self.config.api_key.is_empty() {
            return Err(WarpError::NoApiKey);
        }
        
        // ELIS detail API implementation
        // Note: The actual API endpoint and parameters may differ
        let params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "lawDetail".to_string()),
            ("type", "JSON".to_string()),
            ("MST", id.to_string()),
        ];
        
        let url = reqwest::Url::parse_with_params(BASE_URL, &params)
            .map_err(|e| WarpError::Parse(e.to_string()))?;
        
        let response = self.execute_with_retry(url.to_string()).await?;
        let response_text = response.text().await
            .map_err(|e| WarpError::Network(e))?;
        
        // Check if response is HTML
        if response_text.starts_with("<") {
            return Err(WarpError::ApiError {
                code: "INVALID_RESPONSE".to_string(),
                message: "API returned HTML instead of JSON.".to_string(),
                hint: Some("Please check your API key configuration.".to_string()),
            });
        }
        
        let raw: ElisDetailResponse = serde_json::from_str(&response_text)
            .map_err(|e| WarpError::Parse(format!("Failed to parse detail response: {}", e)))?;
        
        Ok(raw.into_law_detail())
    }
    
    async fn get_history(&self, id: &str) -> Result<LawHistory> {
        // ELIS doesn't have a separate history API
        // Return a simple history with current version only
        Ok(LawHistory {
            law_id: id.to_string(),
            law_name: String::new(),
            total_count: 0,
            entries: vec![],
        })
    }
    
    fn api_type(&self) -> ApiType {
        ApiType::Elis
    }
    
    fn base_url(&self) -> &str {
        BASE_URL
    }
    
    fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }
}

// ELIS-specific response structures
#[derive(Debug, Deserialize)]
struct ElisSearchResponse {
    #[serde(rename = "LawSearch")]
    law_search: Option<ElisSearchData>,
    // Fallback for direct structure
    #[serde(rename = "totalCnt")]
    total_count: Option<u32>,
    #[serde(rename = "page")]
    page_no: Option<u32>,
    #[serde(rename = "display")]
    page_size: Option<u32>,
    #[serde(rename = "law", default)]
    laws: Option<Vec<ElisLaw>>,
}

#[derive(Debug, Deserialize)]
struct ElisSearchData {
    #[serde(rename = "totalCnt")]
    #[allow(dead_code)]
    total_count: Option<u32>,
    #[serde(rename = "page")]
    #[allow(dead_code)]
    page_no: Option<u32>,
    #[serde(rename = "display")]
    #[allow(dead_code)]
    page_size: Option<u32>,
    #[serde(rename = "law", default)]
    laws: Vec<ElisLaw>,
}

#[derive(Debug, Deserialize)]
struct ElisLaw {
    #[serde(rename = "자치법규ID")]
    law_id: String,
    #[serde(rename = "자치법규명")]
    law_name: String,
    #[serde(rename = "자치단체명")]
    region: Option<String>,
    #[serde(rename = "자치법규종류")]
    law_type: Option<String>,
    #[serde(rename = "소관부서")]
    department: Option<String>,
    #[serde(rename = "시행일자")]
    enforcement_date: Option<String>,
    #[serde(rename = "개정일자")]
    revision_date: Option<String>,
    #[serde(rename = "자치법규일련번호")]
    law_no: Option<String>,
    #[serde(rename = "요약")]
    law_summary: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ElisDetailResponse {
    #[serde(rename = "법령")]
    law: ElisDetailContent,
}

#[derive(Debug, Deserialize)]
struct ElisDetailContent {
    #[serde(rename = "기본정보")]
    basic_info: ElisBasicInfo,
    #[serde(rename = "조문")]
    articles: Vec<ElisArticle>,
}

#[derive(Debug, Deserialize)]
struct ElisBasicInfo {
    #[serde(rename = "자치법규ID")]
    law_id: String,
    #[serde(rename = "자치법규명")]
    law_name: String,
    #[serde(rename = "자치단체명")]
    #[allow(dead_code)]
    region: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ElisArticle {
    #[serde(rename = "조문번호")]
    number: String,
    #[serde(rename = "조문제목")]
    title: Option<String>,
    #[serde(rename = "조문내용")]
    content: String,
}

impl ElisDetailResponse {
    fn into_law_detail(self) -> LawDetail {
        LawDetail {
            law_id: self.law.basic_info.law_id,
            law_name: self.law.basic_info.law_name,
            law_no: None,
            law_type: None,
            department: None,
            enforcement_date: None,
            revision_date: None,
            content: self.law.articles.iter()
                .map(|a| format!("{}: {}", a.number, a.content))
                .collect::<Vec<_>>()
                .join("\n\n"),
            articles: self.law.articles.into_iter()
                .map(|a| crate::api::types::Article {
                    number: a.number,
                    title: a.title,
                    content: a.content,
                    paragraphs: vec![],
                })
                .collect(),
            attachments: vec![],
            related_laws: vec![],
            metadata: HashMap::new(),
        }
    }
}