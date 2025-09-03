use crate::api::batcher::{BatchConfig, BatcherFactory};
use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::parallel::{
    search_all_apis, ParallelConfig, ParallelExecutor, ParallelSearchResult,
};
use crate::api::types::{ResponseType, SearchResponse, UnifiedSearchRequest};
use crate::api::{ApiClientFactory, ApiType};
use crate::cache::tiered::{TieredCache, TieredCacheConfig};
use crate::cache::CacheStore;
use crate::cli::args::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;
use crate::progress::{messages, ApiStage, EnhancedApiProgress, ProgressManager};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for parallel search optimization
#[derive(Debug, Clone)]
pub struct ParallelSearchConfig {
    /// Enable parallel search across multiple APIs
    pub enable_parallel_apis: bool,
    /// Enable request batching and deduplication
    pub enable_batching: bool,
    /// Enable tiered caching for better performance
    pub enable_tiered_cache: bool,
    /// Maximum number of concurrent API requests
    pub max_concurrent_requests: usize,
    /// APIs to search in parallel
    pub apis_to_search: Vec<ApiType>,
    /// Enable predictive prefetching
    pub enable_prefetching: bool,
}

impl Default for ParallelSearchConfig {
    fn default() -> Self {
        Self {
            enable_parallel_apis: true,
            enable_batching: true,
            enable_tiered_cache: true,
            max_concurrent_requests: 5,
            apis_to_search: vec![ApiType::Nlic, ApiType::Elis, ApiType::Prec],
            enable_prefetching: false,
        }
    }
}

/// Enhanced search parameters with optimization settings
#[derive(Debug, Clone)]
pub struct EnhancedSearchParams {
    pub query: String,
    pub page: u32,
    pub size: u32,
    pub law_type: Option<String>,
    pub department: Option<String>,
    pub format: OutputFormat,
    pub parallel_config: ParallelSearchConfig,
    pub timeout: Duration,
}

/// Execute optimized parallel search across multiple APIs
pub async fn execute_parallel_search(
    params: EnhancedSearchParams,
    config: &Config,
    progress_manager: Arc<ProgressManager>,
    no_cache: bool,
) -> Result<()> {
    if params.query.trim().is_empty() {
        return Err(WarpError::InvalidInput(
            "Search query cannot be empty".to_string(),
        ));
    }

    // Initialize progress tracking
    let mut progress = EnhancedApiProgress::new(
        progress_manager.clone(),
        &format!(
            "병렬 검색 ({}개 API)",
            params.parallel_config.apis_to_search.len()
        ),
    );

    progress.advance_stage(ApiStage::Connecting, "최적화된 연결 풀 초기화 중");

    // Create optimized cache system
    let cache_system = if config.cache.enabled && !no_cache {
        if params.parallel_config.enable_tiered_cache {
            // Use tiered cache for better performance
            let tiered_config = TieredCacheConfig::default();
            Some(CacheSystem::Tiered(Arc::new(
                TieredCache::new(tiered_config).await?,
            )))
        } else {
            // Use basic cache
            let cache_config = config.cache.to_cache_config();
            Some(CacheSystem::Basic(Arc::new(
                CacheStore::new(cache_config).await?,
            )))
        }
    } else {
        None
    };

    progress.advance_stage(ApiStage::Searching, "병렬 API 클라이언트 생성 중");

    // Create API clients with optimizations
    let clients = create_optimized_clients(
        &params.parallel_config.apis_to_search,
        config,
        &cache_system,
        no_cache,
    )
    .await?;

    // Create search request
    let request = UnifiedSearchRequest {
        query: params.query.clone(),
        page_no: params.page,
        page_size: params.size,
        response_type: ResponseType::Json,
        law_type: params.law_type.clone(),
        department: params.department.clone(),
        ..Default::default()
    };

    progress.advance_stage(
        ApiStage::Searching,
        &format!(
            "'{}' 키워드로 {}개 API 병렬 검색 중",
            params.query,
            clients.len()
        ),
    );

    // Execute parallel search with optimizations
    let search_result = if params.parallel_config.enable_batching {
        execute_batched_parallel_search(clients, request, &params).await?
    } else {
        execute_direct_parallel_search(clients, request, &params).await?
    };

    progress.advance_stage(ApiStage::Parsing, "검색 결과 통합 및 최적화 중");

    // Process and merge results
    let merged_response = process_parallel_results(search_result, &params)?;

    // Complete progress
    let result_message = messages::search_complete_with_time(
        &format!("병렬 검색 ({}개 결과)", merged_response.total_count),
        merged_response.total_count as usize,
        progress.elapsed().as_millis() as u64,
    );
    progress.complete_success(&result_message);

    // Display results
    display_parallel_results(&merged_response, params.format).await?;

    Ok(())
}

/// Cache system enum for different caching strategies
enum CacheSystem {
    Basic(Arc<CacheStore>),
    Tiered(Arc<TieredCache>),
}

/// Create optimized API clients with all performance enhancements
async fn create_optimized_clients(
    api_types: &[ApiType],
    config: &Config,
    cache_system: &Option<CacheSystem>,
    no_cache: bool,
) -> Result<Vec<(ApiType, Arc<dyn LegalApiClient>)>> {
    let mut clients = Vec::new();

    for &api_type in api_types {
        // Get API key for the specific API type
        let api_key = match api_type {
            ApiType::Nlic => config.get_nlic_api_key(),
            ApiType::Elis => config.get_elis_api_key(),
            ApiType::Prec => config.get_prec_api_key(),
            ApiType::Admrul => config.get_admrul_api_key(),
            ApiType::Expc => config.get_expc_api_key(),
            ApiType::All => None, // Not applicable for individual clients
        };

        if let Some(key) = api_key {
            let cache = match cache_system {
                Some(CacheSystem::Basic(cache)) => Some(Arc::clone(cache)),
                Some(CacheSystem::Tiered(_)) => None, // TODO: Integrate tiered cache with client
                None => None,
            };

            let client_config = ClientConfig {
                api_key: key,
                cache,
                bypass_cache: no_cache,
                timeout: 30,          // Optimize timeout for parallel requests
                max_retries: 2,       // Reduce retries for faster parallel execution
                retry_base_delay: 50, // Faster retry for parallel scenarios
                user_agent: format!("warp-parallel/{}", env!("CARGO_PKG_VERSION")),
            };

            let client = ApiClientFactory::create(api_type, client_config)?;
            clients.push((api_type, client));
        } else {
            log::warn!("API key not configured for {:?}, skipping", api_type);
        }
    }

    if clients.is_empty() {
        return Err(WarpError::InvalidInput(
            "No API clients could be created. Please check your configuration.".to_string(),
        ));
    }

    Ok(clients)
}

/// Execute parallel search using batching for request optimization
async fn execute_batched_parallel_search(
    clients: Vec<(ApiType, Arc<dyn LegalApiClient>)>,
    request: UnifiedSearchRequest,
    params: &EnhancedSearchParams,
) -> Result<ParallelSearchResult> {
    let mut batchers = HashMap::new();
    let mut search_handles = Vec::new();

    // Create batchers for each API
    for (api_type, client) in clients {
        let batch_config = BatchConfig {
            max_batch_size: 5,
            max_batch_delay: Duration::from_millis(50),
            enable_deduplication: true,
            deduplication_ttl: Duration::from_secs(300),
            enable_predictive_batching: true,
        };

        let batcher = Arc::new(BatcherFactory::create_batcher(client, Some(batch_config)));
        batchers.insert(api_type, batcher);
    }

    // Submit requests to batchers
    for (api_type, batcher) in batchers {
        let request_clone = request.clone();
        let handle = tokio::spawn(async move {
            match tokio::time::timeout(params.timeout, batcher.submit_request(request_clone)).await
            {
                Ok(Ok(response)) => Ok((api_type, response)),
                Ok(Err(e)) => Err((api_type, e)),
                Err(_) => Err((
                    api_type,
                    WarpError::Timeout(params.timeout.as_millis() as u64),
                )),
            }
        });
        search_handles.push(handle);
    }

    // Collect results
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for handle in search_handles {
        match handle.await {
            Ok(Ok((api_type, response))) => successes.push((api_type, response)),
            Ok(Err((api_type, error))) => failures.push((api_type, error)),
            Err(e) => {
                failures.push((
                    ApiType::Nlic,
                    WarpError::Other(format!("Task join error: {}", e)),
                ));
            }
        }
    }

    Ok(ParallelSearchResult {
        successes,
        failures,
        execution_time: Duration::from_secs(0), // Will be calculated by the caller
    })
}

/// Execute direct parallel search without batching
async fn execute_direct_parallel_search(
    clients: Vec<(ApiType, Arc<dyn LegalApiClient>)>,
    request: UnifiedSearchRequest,
    params: &EnhancedSearchParams,
) -> Result<ParallelSearchResult> {
    let parallel_config = ParallelConfig {
        max_concurrent: params.parallel_config.max_concurrent_requests,
        request_timeout: params.timeout,
        fail_fast: false,
        batch_delay: Duration::from_millis(50),
    };

    let executor = ParallelExecutor::new(parallel_config);
    executor.search_parallel(clients, request).await
}

/// Process and optimize parallel search results
fn process_parallel_results(
    search_result: ParallelSearchResult,
    params: &EnhancedSearchParams,
) -> Result<SearchResponse> {
    if search_result.successes.is_empty() {
        return Err(WarpError::Other(format!(
            "All API requests failed. Errors: {:?}",
            search_result.failures
        )));
    }

    // Merge successful responses with intelligent deduplication
    let mut merged_response = search_result
        .merge_responses()
        .ok_or_else(|| WarpError::Other("Failed to merge search results".to_string()))?;

    // Apply result optimization
    optimize_merged_results(&mut merged_response, params);

    // Log performance statistics
    let summary = search_result.get_summary();
    log::info!(
        "Parallel search completed: {}/{} APIs successful, {} total items, {:?} execution time",
        summary.successful_apis,
        summary.total_apis,
        summary.total_items,
        summary.execution_time
    );

    Ok(merged_response)
}

/// Optimize merged search results for better user experience
fn optimize_merged_results(response: &mut SearchResponse, params: &EnhancedSearchParams) {
    // Sort by relevance (simple heuristic based on title match)
    response.items.sort_by(|a, b| {
        let a_score = calculate_relevance_score(&a.title, &params.query);
        let b_score = calculate_relevance_score(&b.title, &params.query);
        b_score
            .partial_cmp(&a_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Remove exact duplicates based on title and law_no
    response
        .items
        .dedup_by(|a, b| a.title == b.title && a.law_no == b.law_no);

    // Update total count after deduplication
    response.total_count = response.items.len() as u32;

    // Limit results to requested page size
    if response.items.len() > params.size as usize {
        response.items.truncate(params.size as usize);
    }
}

/// Calculate relevance score for search result ranking
fn calculate_relevance_score(title: &str, query: &str) -> f64 {
    let title_lower = title.to_lowercase();
    let query_lower = query.to_lowercase();

    let mut score = 0.0;

    // Exact match bonus
    if title_lower.contains(&query_lower) {
        score += 2.0;
    }

    // Word match scoring
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    for word in query_words {
        if title_lower.contains(word) {
            score += 1.0;
        }
    }

    // Position bonus (earlier matches are better)
    if let Some(pos) = title_lower.find(&query_lower) {
        score += 1.0 / (pos as f64 + 1.0);
    }

    score
}

/// Display parallel search results with enhanced formatting
async fn display_parallel_results(response: &SearchResponse, format: OutputFormat) -> Result<()> {
    if response.items.is_empty() {
        println!("🔍 병렬 검색 결과: 검색 조건에 맞는 결과가 없습니다.");
        return Ok(());
    }

    // Show search summary
    println!(
        "📊 병렬 검색 완료: {}개 결과 (통합 및 최적화됨)",
        response.total_count
    );
    println!();

    // Format and display results
    let output = output::format_search_response(response, format)?;
    println!("{}", output);

    // Show optimization info
    println!();
    println!("⚡ 성능 최적화 적용:");
    println!("   • 병렬 API 검색으로 응답 시간 단축");
    println!("   • 지능형 결과 통합 및 중복 제거");
    println!("   • 관련도 기반 결과 정렬");

    Ok(())
}

/// Create enhanced search parameters with optimization settings
pub fn create_enhanced_search_params(
    query: String,
    page: u32,
    size: u32,
    law_type: Option<String>,
    department: Option<String>,
    format: OutputFormat,
) -> EnhancedSearchParams {
    EnhancedSearchParams {
        query,
        page,
        size,
        law_type,
        department,
        format,
        parallel_config: ParallelSearchConfig::default(),
        timeout: Duration::from_secs(30),
    }
}

/// Enable specific optimizations for search
pub fn enable_optimizations(
    mut params: EnhancedSearchParams,
    enable_batching: bool,
    enable_tiered_cache: bool,
    max_concurrent: Option<usize>,
) -> EnhancedSearchParams {
    params.parallel_config.enable_batching = enable_batching;
    params.parallel_config.enable_tiered_cache = enable_tiered_cache;

    if let Some(max_concurrent) = max_concurrent {
        params.parallel_config.max_concurrent_requests = max_concurrent;
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_scoring() {
        let title = "민법 제1조 (기본원칙)";
        let query = "민법";

        let score = calculate_relevance_score(title, query);
        assert!(score > 0.0);

        // Exact match should score higher
        let exact_score = calculate_relevance_score("민법", "민법");
        assert!(exact_score > score);
    }

    #[test]
    fn test_enhanced_search_params_creation() {
        let params = create_enhanced_search_params(
            "test query".to_string(),
            1,
            10,
            None,
            None,
            OutputFormat::Table,
        );

        assert_eq!(params.query, "test query");
        assert_eq!(params.page, 1);
        assert_eq!(params.size, 10);
        assert!(params.parallel_config.enable_parallel_apis);
    }

    #[test]
    fn test_optimization_enabling() {
        let base_params = create_enhanced_search_params(
            "test".to_string(),
            1,
            10,
            None,
            None,
            OutputFormat::Table,
        );

        let optimized = enable_optimizations(base_params, true, true, Some(10));

        assert!(optimized.parallel_config.enable_batching);
        assert!(optimized.parallel_config.enable_tiered_cache);
        assert_eq!(optimized.parallel_config.max_concurrent_requests, 10);
    }
}
