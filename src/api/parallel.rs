use super::types::*;
use super::{ApiType, LegalApiClient};
use crate::error::{Result, WarpError};
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use std::time::Duration;

/// Configuration for parallel API operations
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of concurrent requests
    pub max_concurrent: usize,
    /// Timeout for individual requests
    pub request_timeout: Duration,
    /// Whether to fail fast on first error or collect all results
    pub fail_fast: bool,
    /// Minimum delay between batches to avoid overwhelming the server
    pub batch_delay: Duration,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            request_timeout: Duration::from_secs(30),
            fail_fast: false,
            batch_delay: Duration::from_millis(100),
        }
    }
}

/// Result of a parallel search operation
#[derive(Debug)]
pub struct ParallelSearchResult {
    /// Successful responses indexed by API type
    pub successes: Vec<(ApiType, SearchResponse)>,
    /// Failed requests with their errors
    pub failures: Vec<(ApiType, WarpError)>,
    /// Total execution time
    pub execution_time: Duration,
}

impl ParallelSearchResult {
    /// Merge all successful responses into a single unified response
    pub fn merge_responses(&self) -> Option<SearchResponse> {
        if self.successes.is_empty() {
            return None;
        }

        let mut merged_items = Vec::new();
        let mut total_count = 0u32;
        let mut page_no = 1u32;
        let mut page_size = 10u32;

        // Collect all items from successful responses
        for (_api_type, response) in &self.successes {
            merged_items.extend(response.items.iter().cloned());
            total_count = total_count.saturating_add(response.total_count);
            page_no = response.page_no; // Use last response's page info
            page_size = response.page_size;
        }

        // Sort items by relevance (you might want to implement custom scoring)
        // For now, we'll just maintain order and add source diversity

        Some(SearchResponse {
            total_count,
            page_no,
            page_size,
            items: merged_items,
            source: format!("Unified ({})", self.successes.len()),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get summary statistics of the parallel operation
    pub fn get_summary(&self) -> ParallelSummary {
        ParallelSummary {
            total_apis: self.successes.len() + self.failures.len(),
            successful_apis: self.successes.len(),
            failed_apis: self.failures.len(),
            total_items: self.successes.iter().map(|(_, r)| r.items.len()).sum(),
            execution_time: self.execution_time,
        }
    }
}

#[derive(Debug)]
pub struct ParallelSummary {
    pub total_apis: usize,
    pub successful_apis: usize,
    pub failed_apis: usize,
    pub total_items: usize,
    pub execution_time: Duration,
}

/// Parallel API operation executor
pub struct ParallelExecutor {
    config: ParallelConfig,
}

impl ParallelExecutor {
    /// Create a new parallel executor with custom configuration
    pub fn new(config: ParallelConfig) -> Self {
        Self { config }
    }

    /// Create a parallel executor with default configuration
    pub fn default() -> Self {
        Self {
            config: ParallelConfig::default(),
        }
    }

    /// Execute search requests across multiple APIs in parallel
    pub async fn search_parallel(
        &self,
        clients: Vec<(ApiType, Arc<dyn LegalApiClient>)>,
        request: UnifiedSearchRequest,
    ) -> Result<ParallelSearchResult> {
        if clients.is_empty() {
            return Err(WarpError::InvalidInput("No clients provided".to_string()));
        }

        let start_time = std::time::Instant::now();

        // Create futures for each API call
        let search_futures: Vec<_> = clients
            .into_iter()
            .map(|(api_type, client)| {
                let request = request.clone();
                async move {
                    // Add timeout to individual requests
                    let search_future = client.search(request);

                    match tokio::time::timeout(self.config.request_timeout, search_future).await {
                        Ok(result) => (api_type, result),
                        Err(_) => (
                            api_type,
                            Err(WarpError::Timeout(
                                self.config.request_timeout.as_millis() as u64
                            )),
                        ),
                    }
                }
            })
            .collect();

        // Execute with controlled concurrency
        let results: Vec<(ApiType, Result<SearchResponse>)> = stream::iter(search_futures)
            .buffer_unordered(self.config.max_concurrent)
            .collect()
            .await;

        // Add delay between batches if configured
        if self.config.batch_delay > Duration::ZERO {
            tokio::time::sleep(self.config.batch_delay).await;
        }

        let execution_time = start_time.elapsed();

        // Separate successes and failures
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for (api_type, result) in results {
            match result {
                Ok(response) => successes.push((api_type, response)),
                Err(error) => failures.push((api_type, error)),
            }
        }

        // If fail_fast is enabled and we have failures, return early
        if self.config.fail_fast && !failures.is_empty() {
            return Err(failures.into_iter().next().unwrap().1);
        }

        Ok(ParallelSearchResult {
            successes,
            failures,
            execution_time,
        })
    }

    /// Execute detail requests in parallel (for batch lookups)
    pub async fn get_details_parallel(
        &self,
        client: Arc<dyn LegalApiClient>,
        ids: Vec<String>,
    ) -> Result<Vec<(String, Result<LawDetail>)>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Create futures for each detail request
        let detail_futures: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let client = Arc::clone(&client);
                let id_clone = id.clone();
                async move {
                    let result = tokio::time::timeout(
                        self.config.request_timeout,
                        client.get_detail(&id_clone),
                    )
                    .await;

                    let detail_result = match result {
                        Ok(detail_result) => detail_result,
                        Err(_) => Err(WarpError::Timeout(
                            self.config.request_timeout.as_millis() as u64
                        )),
                    };

                    (id, detail_result)
                }
            })
            .collect();

        // Execute with controlled concurrency
        let results: Vec<(String, Result<LawDetail>)> = stream::iter(detail_futures)
            .buffer_unordered(self.config.max_concurrent)
            .collect()
            .await;

        Ok(results)
    }

    /// Execute a single request with retry logic
    pub async fn execute_with_retry<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
        T: Send,
    {
        const MAX_RETRIES: usize = 3;
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);

                    if attempt < MAX_RETRIES - 1 {
                        // Exponential backoff
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt as u32)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }
}

/// Helper function to create parallel search with default configuration
pub async fn search_all_apis(
    clients: Vec<(ApiType, Arc<dyn LegalApiClient>)>,
    request: UnifiedSearchRequest,
) -> Result<ParallelSearchResult> {
    let executor = ParallelExecutor::default();
    executor.search_parallel(clients, request).await
}

/// Helper function for rate-limited parallel requests
pub async fn search_with_rate_limit(
    clients: Vec<(ApiType, Arc<dyn LegalApiClient>)>,
    request: UnifiedSearchRequest,
    max_concurrent: usize,
    delay_between_requests: Duration,
) -> Result<ParallelSearchResult> {
    let config = ParallelConfig {
        max_concurrent,
        batch_delay: delay_between_requests,
        ..ParallelConfig::default()
    };

    let executor = ParallelExecutor::new(config);
    executor.search_parallel(clients, request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::ClientConfig;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock client for testing
    struct MockClient {
        delay: Duration,
        should_fail: bool,
        call_count: Arc<AtomicUsize>,
    }

    impl MockClient {
        fn new(delay: Duration, should_fail: bool) -> Self {
            Self {
                delay,
                should_fail,
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl LegalApiClient for MockClient {
        async fn search(&self, _request: UnifiedSearchRequest) -> Result<SearchResponse> {
            self.call_count.fetch_add(1, Ordering::Relaxed);

            tokio::time::sleep(self.delay).await;

            if self.should_fail {
                Err(WarpError::ApiError {
                    code: "TEST_ERROR".to_string(),
                    message: "Mock error".to_string(),
                    hint: None,
                })
            } else {
                Ok(SearchResponse {
                    total_count: 10,
                    page_no: 1,
                    page_size: 10,
                    items: vec![],
                    source: "Mock".to_string(),
                    timestamp: chrono::Utc::now(),
                })
            }
        }

        async fn get_detail(&self, _id: &str) -> Result<LawDetail> {
            unimplemented!("Mock detail not implemented")
        }

        async fn get_history(&self, _id: &str) -> Result<LawHistory> {
            unimplemented!("Mock history not implemented")
        }

        fn api_type(&self) -> ApiType {
            ApiType::Nlic
        }
        fn base_url(&self) -> &str {
            "http://mock"
        }
        fn is_configured(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_parallel_search_success() {
        let clients = vec![
            (
                ApiType::Nlic,
                Arc::new(MockClient::new(Duration::from_millis(100), false))
                    as Arc<dyn LegalApiClient>,
            ),
            (
                ApiType::Elis,
                Arc::new(MockClient::new(Duration::from_millis(150), false))
                    as Arc<dyn LegalApiClient>,
            ),
        ];

        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            ..Default::default()
        };

        let executor = ParallelExecutor::default();
        let result = executor.search_parallel(clients, request).await.unwrap();

        assert_eq!(result.successes.len(), 2);
        assert_eq!(result.failures.len(), 0);
        assert!(result.execution_time < Duration::from_millis(300)); // Should be faster than sequential
    }

    #[tokio::test]
    async fn test_parallel_search_with_failures() {
        let clients = vec![
            (
                ApiType::Nlic,
                Arc::new(MockClient::new(Duration::from_millis(50), false))
                    as Arc<dyn LegalApiClient>,
            ),
            (
                ApiType::Elis,
                Arc::new(MockClient::new(Duration::from_millis(50), true))
                    as Arc<dyn LegalApiClient>,
            ),
        ];

        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            ..Default::default()
        };

        let executor = ParallelExecutor::default();
        let result = executor.search_parallel(clients, request).await.unwrap();

        assert_eq!(result.successes.len(), 1);
        assert_eq!(result.failures.len(), 1);
    }
}
