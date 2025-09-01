use async_trait::async_trait;
use crate::error::Result;
use super::types::*;
use super::ApiType;

/// Trait for legal API clients
#[async_trait]
pub trait LegalApiClient: Send + Sync {
    /// Search for laws/documents
    async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse>;
    
    /// Get detailed information about a specific law
    async fn get_detail(&self, id: &str) -> Result<LawDetail>;
    
    /// Get revision history of a law
    async fn get_history(&self, id: &str) -> Result<LawHistory>;
    
    /// Get the API type
    #[allow(dead_code)]
    fn api_type(&self) -> ApiType;
    
    /// Get the base URL for this API
    #[allow(dead_code)]
    fn base_url(&self) -> &str;
    
    /// Check if the client is configured properly
    #[allow(dead_code)]
    fn is_configured(&self) -> bool;
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API key
    pub api_key: String,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay for exponential backoff (milliseconds)
    pub retry_base_delay: u64,
    /// User agent string
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            timeout: 30,
            max_retries: 3,
            retry_base_delay: 100,
            user_agent: format!("warp-cli/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Factory for creating API clients
pub struct ApiClientFactory;

impl ApiClientFactory {
    /// Create a new API client based on the API type
    pub fn create(api_type: ApiType, config: ClientConfig) -> Result<Box<dyn LegalApiClient>> {
        match api_type {
            ApiType::Nlic => {
                Ok(Box::new(super::nlic::NlicClient::new(config)))
            }
            ApiType::Elis => {
                Ok(Box::new(super::elis::ElisClient::new(config)))
            }
            ApiType::Prec => {
                Ok(Box::new(super::prec::PrecClient::new(config)))
            }
            ApiType::Admrul => {
                Ok(Box::new(super::admrul::AdmrulClient::new(config)))
            }
            ApiType::Expc => {
                Ok(Box::new(super::expc::ExpcClient::new(config)))
            }
            ApiType::All => {
                // TODO: Implement unified search across all APIs
                Err(crate::error::WarpError::Other(
                    "Unified search is not yet implemented".to_string()
                ))
            }
        }
    }
}