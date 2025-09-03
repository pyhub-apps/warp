use super::types::*;
use super::ApiType;
use crate::cache::CacheStore;
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Core trait for Korean legal API clients
///
/// Defines a unified interface for accessing different Korean government legal databases.
/// All API clients implement this trait to provide consistent functionality across
/// different data sources.
///
/// # Examples
///
/// ```no_run
/// use warp::api::{ApiClientFactory, ApiType, LegalApiClient};
/// use warp::api::types::UnifiedSearchRequest;
/// use warp::config::Config;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::load()?;
/// let client = ApiClientFactory::create(ApiType::Nlic, config.to_client_config())?;
///
/// // Search for documents
/// let request = UnifiedSearchRequest {
///     query: "환경보호".to_string(),
///     ..Default::default()
/// };
/// let response = client.search(request).await?;
///
/// // Get detailed information
/// if let Some(laws) = &response.laws {
///     if let Some(first_law) = laws.first() {
///         let detail = client.get_detail(&first_law.law_id).await?;
///         println!("Law: {}", detail.law_name);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait LegalApiClient: Send + Sync {
    /// Search for legal documents using the unified request format
    ///
    /// Performs a search across the API's document collection using the provided
    /// search criteria. Results are returned in a standardized format that can
    /// be processed consistently across different API types.
    ///
    /// # Arguments
    ///
    /// * `request` - Unified search request containing query parameters
    ///
    /// # Returns
    ///
    /// Returns a `SearchResponse` containing matching documents, pagination info,
    /// and metadata about the search results.
    ///
    /// # Errors
    ///
    /// * Network connectivity issues
    /// * API rate limiting
    /// * Invalid query parameters
    /// * Service unavailability
    async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse>;

    /// Retrieve detailed information about a specific legal document
    ///
    /// Fetches comprehensive details for a document identified by its unique ID,
    /// including full text content, metadata, and revision information.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the legal document
    ///
    /// # Returns
    ///
    /// Returns `LawDetail` containing comprehensive document information
    /// including content, metadata, and formatting information.
    async fn get_detail(&self, id: &str) -> Result<LawDetail>;

    /// Get revision history for a specific legal document
    ///
    /// Retrieves the complete revision history for a document, including
    /// amendment dates, change descriptions, and version information.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the legal document
    ///
    /// # Returns
    ///
    /// Returns `LawHistory` containing chronological revision information
    /// and change tracking details.
    async fn get_history(&self, id: &str) -> Result<LawHistory>;

    /// Get the API type for this client
    ///
    /// Returns the specific API type that this client connects to,
    /// useful for routing and debugging purposes.
    #[allow(dead_code)]
    fn api_type(&self) -> ApiType;

    /// Get the base URL for this API
    ///
    /// Returns the base URL endpoint that this client uses for API requests.
    /// Useful for configuration validation and debugging.
    #[allow(dead_code)]
    fn base_url(&self) -> &str;

    /// Check if the client is properly configured
    ///
    /// Validates that all required configuration parameters are present
    /// and valid, including API keys, endpoints, and network settings.
    ///
    /// # Returns
    ///
    /// Returns `true` if the client is ready for use, `false` if configuration
    /// issues need to be resolved.
    #[allow(dead_code)]
    fn is_configured(&self) -> bool;
}

/// Configuration for Korean legal API clients
///
/// Contains all necessary parameters for initializing and configuring
/// API clients, including authentication, networking, caching, and
/// performance settings.
///
/// # Examples
///
/// ```
/// use warp::api::client::ClientConfig;
///
/// // Create default configuration
/// let mut config = ClientConfig::default();
/// config.api_key = "your-api-key".to_string();
/// config.timeout = 60; // 60 second timeout
/// config.max_retries = 5; // Retry up to 5 times
/// ```
///
/// ```no_run
/// use warp::api::client::ClientConfig;
/// use warp::cache::CacheStore;
/// use std::sync::Arc;
///
/// // Configuration with caching
/// let cache = Arc::new(CacheStore::new("cache_dir").unwrap());
/// let config = ClientConfig {
///     api_key: "your-key".to_string(),
///     timeout: 30,
///     cache: Some(cache),
///     bypass_cache: false,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API key for authentication with Korean legal databases
    ///
    /// Required for most API endpoints. Obtain from the respective
    /// government agency providing the legal data service.
    pub api_key: String,

    /// Request timeout in seconds
    ///
    /// Maximum time to wait for API responses before timing out.
    /// Recommended: 30-60 seconds for reliable connections.
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    ///
    /// Number of times to retry failed requests before giving up.
    /// Uses exponential backoff between retries.
    pub max_retries: u32,

    /// Base delay for exponential backoff (milliseconds)
    ///
    /// Initial delay before first retry. Each subsequent retry
    /// doubles this delay (exponential backoff strategy).
    pub retry_base_delay: u64,

    /// User agent string for HTTP requests
    ///
    /// Identifies the client software to the API server.
    /// Automatically includes version information.
    pub user_agent: String,

    /// Optional cache store for API responses
    ///
    /// When provided, enables automatic caching of API responses
    /// to improve performance and reduce API usage.
    pub cache: Option<Arc<CacheStore>>,

    /// Whether to bypass cache for requests
    ///
    /// When `true`, forces fresh API requests even if cached
    /// responses are available. Useful for real-time data needs.
    pub bypass_cache: bool,

    /// Whether to use benchmark-safe mode
    ///
    /// Disables background tasks and optimizations that might
    /// interfere with performance benchmarking.
    pub benchmark_mode: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            timeout: 30,
            max_retries: 3,
            retry_base_delay: 100,
            user_agent: format!("warp/{}", env!("CARGO_PKG_VERSION")),
            cache: None,
            bypass_cache: false,
            benchmark_mode: false,
        }
    }
}

/// Factory for creating Korean legal API clients
///
/// Provides a centralized way to create and configure API clients for
/// different Korean government legal databases. Handles the complexity
/// of client initialization and ensures proper configuration.
///
/// # Examples
///
/// ```no_run
/// use warp::api::{ApiClientFactory, ApiType};
/// use warp::api::client::ClientConfig;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create NLIC client
/// let config = ClientConfig {
///     api_key: "your-nlic-key".to_string(),
///     ..Default::default()
/// };
/// let nlic_client = ApiClientFactory::create(ApiType::Nlic, config.clone())?;
///
/// // Create ELIS client
/// let elis_client = ApiClientFactory::create(ApiType::Elis, config)?;
/// # Ok(())
/// # }
/// ```
///
/// ```no_run
/// use warp::api::{ApiClientFactory, ApiType};
/// use warp::config::Config;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Load configuration from file
/// let config = Config::load()?;
///
/// // Create multiple clients for unified search
/// let apis = vec![ApiType::Nlic, ApiType::Elis, ApiType::Prec];
/// let mut clients = Vec::new();
///
/// for api_type in apis {
///     let client = ApiClientFactory::create(api_type, config.to_client_config())?;
///     clients.push(client);
/// }
/// # Ok(())
/// # }
/// ```
pub struct ApiClientFactory;

impl ApiClientFactory {
    /// Create a new API client for the specified Korean legal database
    ///
    /// Initializes and configures an API client based on the provided type
    /// and configuration. Each client type connects to a specific Korean
    /// government legal database with its own endpoints and data format.
    ///
    /// # Arguments
    ///
    /// * `api_type` - The type of Korean legal API to connect to
    /// * `config` - Client configuration including authentication and networking settings
    ///
    /// # Returns
    ///
    /// Returns a boxed trait object implementing `LegalApiClient` that can
    /// be used to interact with the specified API.
    ///
    /// # Errors
    ///
    /// * `WarpError::Other` - For unsupported API types or configuration issues
    /// * Configuration validation errors for invalid parameters
    ///
    /// # Supported API Types
    ///
    /// * `ApiType::Nlic` - National Law Information Center (국가법령정보센터)
    /// * `ApiType::Elis` - Local Regulations Information System (자치법규정보시스템)
    /// * `ApiType::Prec` - Court Precedents Database (판례)
    /// * `ApiType::Admrul` - Administrative Rules Database (행정규칙)
    /// * `ApiType::Expc` - Legal Interpretation Cases (법령해석례)
    /// * `ApiType::All` - Unified search (implementation pending)
    pub fn create(api_type: ApiType, config: ClientConfig) -> Result<Box<dyn LegalApiClient>> {
        match api_type {
            ApiType::Nlic => Ok(Box::new(super::nlic::NlicClient::new(config))),
            ApiType::Elis => Ok(Box::new(super::elis::ElisClient::new(config))),
            ApiType::Prec => Ok(Box::new(super::prec::PrecClient::new(config))),
            ApiType::Admrul => Ok(Box::new(super::admrul::AdmrulClient::new(config))),
            ApiType::Expc => Ok(Box::new(super::expc::ExpcClient::new(config))),
            ApiType::All => {
                // TODO: Implement unified search across all APIs
                Err(crate::error::WarpError::Other(
                    "Unified search across all APIs is not yet implemented. Please use individual API types for now.".to_string(),
                ))
            }
        }
    }
}
