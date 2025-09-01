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

const BASE_URL: &str = "https://www.law.go.kr/DRF/lawSearch.do";
const DETAIL_URL: &str = "https://www.law.go.kr/DRF/lawService.do";

/// PREC (판례) API Client
pub struct PrecClient {
    config: ClientConfig,
    client: Client,
}

impl PrecClient {
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
    
    /// Parse PREC search response
    fn parse_search_response(&self, raw: PrecSearchResponse) -> SearchResponse {
        let (cases, total_count, page_no, page_size) = if let Some(search_data) = raw.prec_search {
            (
                search_data.cases,
                search_data.total_count.and_then(|s| s.parse::<u32>().ok()).unwrap_or(0),
                search_data.page_no.and_then(|s| s.parse::<u32>().ok()).unwrap_or(1),
                search_data.page_size.and_then(|s| s.parse::<u32>().ok()).unwrap_or(50),
            )
        } else {
            (
                raw.cases.unwrap_or_default(),
                raw.total_count.unwrap_or(0),
                raw.page_no.unwrap_or(1),
                raw.page_size.unwrap_or(50),
            )
        };
        
        let items = cases.into_iter().map(|case| {
            let mut metadata = HashMap::new();
            if let Some(ref court) = case.court_name {
                metadata.insert("court".to_string(), court.clone());
            }
            if let Some(ref case_type) = case.case_type {
                metadata.insert("case_type".to_string(), case_type.clone());
            }
            if let Some(ref judgment_date) = case.judgment_date {
                metadata.insert("judgment_date".to_string(), judgment_date.clone());
            }
            
            SearchItem {
                id: case.case_id,
                title: case.case_name,
                law_no: case.case_number,
                law_type: case.case_type,
                department: case.court_name,
                enforcement_date: case.judgment_date.clone(),
                revision_date: None,
                summary: case.case_summary,
                source: "PREC".to_string(),
                metadata,
            }
        }).collect();
        
        SearchResponse {
            total_count,
            page_no,
            page_size,
            items,
            source: "PREC".to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl LegalApiClient for PrecClient {
    async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse> {
        if self.config.api_key.is_empty() {
            return Err(WarpError::NoApiKey);
        }
        
        let mut params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "prec".to_string()),
            ("type", "JSON".to_string()),
            ("query", request.query.clone()),
            ("page", request.page_no.to_string()),
            ("display", request.page_size.to_string()),
        ];
        
        // Add optional parameters for precedent search
        if let Some(court) = request.extras.get("court") {
            params.push(("court", court.clone()));
        }
        if let Some(case_type) = request.extras.get("case_type") {
            params.push(("caseType", case_type.clone()));
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
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        
        let is_html = content_type.contains("text/html");
        
        let response_text = response.text().await
            .map_err(|e| WarpError::Network(e))?;
        
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
                hint: Some("This might indicate an invalid API key or server issue. Try again later.".to_string()),
            });
        }
        
        // Try to parse JSON
        let raw: PrecSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                if response_text.contains("error") || response_text.contains("Error") {
                    WarpError::ApiError {
                        code: "API_ERROR".to_string(),
                        message: format!("API returned an error: {}", response_text.chars().take(200).collect::<String>()),
                        hint: Some("Check your API key and request parameters.".to_string()),
                    }
                } else {
                    WarpError::Parse(format!("Failed to parse precedent API response: {}", e))
                }
            })?;
        
        Ok(self.parse_search_response(raw))
    }
    
    async fn get_detail(&self, id: &str) -> Result<LawDetail> {
        if self.config.api_key.is_empty() {
            return Err(WarpError::NoApiKey);
        }
        
        let params = vec![
            ("OC", self.config.api_key.clone()),
            ("target", "prec".to_string()),
            ("type", "JSON".to_string()),
            ("ID", id.to_string()),
        ];
        
        let url = reqwest::Url::parse_with_params(DETAIL_URL, &params)
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
        
        let raw: PrecDetailResponse = serde_json::from_str(&response_text)
            .map_err(|e| WarpError::Parse(format!("Failed to parse precedent detail: {}", e)))?;
        
        Ok(raw.into_law_detail())
    }
    
    async fn get_history(&self, _id: &str) -> Result<LawHistory> {
        // Precedents don't have history in the same way laws do
        // Return empty history
        Ok(LawHistory {
            law_id: _id.to_string(),
            law_name: String::new(),
            total_count: 0,
            entries: vec![],
        })
    }
    
    fn api_type(&self) -> ApiType {
        ApiType::Prec
    }
    
    fn base_url(&self) -> &str {
        BASE_URL
    }
    
    fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }
}

// PREC-specific response structures
#[derive(Debug, Deserialize)]
struct PrecSearchResponse {
    #[serde(rename = "PrecSearch")]
    prec_search: Option<PrecSearchData>,
    // Fallback for direct structure
    #[serde(rename = "totalCnt")]
    total_count: Option<u32>,
    #[serde(rename = "page")]
    page_no: Option<u32>,
    #[serde(rename = "display")]
    page_size: Option<u32>,
    #[serde(rename = "prec", default)]
    cases: Option<Vec<PrecCase>>,
}

#[derive(Debug, Deserialize)]
struct PrecSearchData {
    #[serde(rename = "totalCnt")]
    total_count: Option<String>,
    #[serde(rename = "page")]
    page_no: Option<String>,
    #[serde(rename = "display")]
    page_size: Option<String>,
    #[serde(rename = "prec", default)]
    cases: Vec<PrecCase>,
}

#[derive(Debug, Deserialize)]
struct PrecCase {
    #[serde(rename = "판례일련번호")]
    case_id: String,
    #[serde(rename = "사건명")]
    case_name: String,
    #[serde(rename = "사건번호")]
    case_number: Option<String>,
    #[serde(rename = "선고일자")]
    judgment_date: Option<String>,
    #[serde(rename = "법원명")]
    court_name: Option<String>,
    #[serde(rename = "사건종류명")]
    case_type: Option<String>,
    #[serde(rename = "판시사항")]
    case_summary: Option<String>,
    #[serde(rename = "판결요지")]
    #[allow(dead_code)]
    judgment_summary: Option<String>,
    #[serde(rename = "참조조문")]
    #[allow(dead_code)]
    reference_laws: Option<String>,
    #[serde(rename = "참조판례")]
    #[allow(dead_code)]
    reference_cases: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PrecDetailResponse {
    #[serde(rename = "PrecService")]
    prec: PrecDetailContent,
}

#[derive(Debug, Deserialize)]
struct PrecDetailContent {
    #[serde(rename = "판례정보")]
    case_info: PrecDetailInfo,
}

#[derive(Debug, Deserialize)]
struct PrecDetailInfo {
    #[serde(rename = "판례일련번호")]
    case_id: String,
    #[serde(rename = "사건명")]
    case_name: String,
    #[serde(rename = "사건번호")]
    case_number: Option<String>,
    #[serde(rename = "선고일자")]
    judgment_date: Option<String>,
    #[serde(rename = "법원명")]
    court_name: Option<String>,
    #[serde(rename = "사건종류명")]
    case_type: Option<String>,
    #[serde(rename = "판시사항")]
    case_holding: Option<String>,
    #[serde(rename = "판결요지")]
    judgment_summary: Option<String>,
    #[serde(rename = "참조조문")]
    reference_laws: Option<String>,
    #[serde(rename = "참조판례")]
    reference_cases: Option<String>,
    #[serde(rename = "판례내용")]
    case_content: Option<String>,
}

impl PrecDetailResponse {
    fn into_law_detail(self) -> LawDetail {
        let info = self.prec.case_info;
        
        let mut content = String::new();
        if let Some(holding) = &info.case_holding {
            content.push_str("【판시사항】\n");
            content.push_str(holding);
            content.push_str("\n\n");
        }
        if let Some(summary) = &info.judgment_summary {
            content.push_str("【판결요지】\n");
            content.push_str(summary);
            content.push_str("\n\n");
        }
        if let Some(full_content) = &info.case_content {
            content.push_str("【판례내용】\n");
            content.push_str(full_content);
        }
        
        let mut metadata = HashMap::new();
        if let Some(ref_laws) = info.reference_laws {
            metadata.insert("reference_laws".to_string(), ref_laws);
        }
        if let Some(ref_cases) = info.reference_cases {
            metadata.insert("reference_cases".to_string(), ref_cases);
        }
        
        LawDetail {
            law_id: info.case_id,
            law_name: info.case_name,
            law_no: info.case_number,
            law_type: info.case_type,
            department: info.court_name,
            enforcement_date: info.judgment_date,
            revision_date: None,
            content,
            articles: vec![],
            attachments: vec![],
            related_laws: vec![],
            metadata,
        }
    }
}