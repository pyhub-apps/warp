use super::types::{LawDetail, LawHistory, SearchResponse, UnifiedSearchRequest};
use super::{ApiType, LegalApiClient};
use crate::error::{Result, WarpError};
use crate::metrics::get_global_metrics;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Notify};
use tokio::time::sleep;

/// Configuration for request batching and deduplication
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size before forcing execution
    pub max_batch_size: usize,
    /// Maximum time to wait before executing a batch
    pub max_batch_delay: Duration,
    /// Enable request deduplication
    pub enable_deduplication: bool,
    /// Deduplication cache TTL
    pub deduplication_ttl: Duration,
    /// Enable predictive batching based on patterns
    pub enable_predictive_batching: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            max_batch_delay: Duration::from_millis(100),
            enable_deduplication: true,
            deduplication_ttl: Duration::from_secs(60),
            enable_predictive_batching: true,
        }
    }
}

/// A unique identifier for a request that enables deduplication
#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestKey {
    query: String,
    page_no: u32,
    page_size: u32,
    law_type: Option<String>,
    department: Option<String>,
}

impl Hash for RequestKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.page_no.hash(state);
        self.page_size.hash(state);
        self.law_type.hash(state);
        self.department.hash(state);
    }
}

impl From<&UnifiedSearchRequest> for RequestKey {
    fn from(request: &UnifiedSearchRequest) -> Self {
        Self {
            query: request.query.clone(),
            page_no: request.page_no,
            page_size: request.page_size,
            law_type: request.law_type.clone(),
            department: request.department.clone(),
        }
    }
}

/// A pending request waiting to be batched
#[derive(Debug)]
struct PendingRequest {
    request: UnifiedSearchRequest,
    key: RequestKey,
    sender: oneshot::Sender<Result<SearchResponse>>,
    created_at: Instant,
}

/// Cached response for deduplication
#[derive(Debug, Clone)]
struct CachedResponse {
    response: SearchResponse,
    created_at: Instant,
}

/// Request batcher that groups similar requests and handles deduplication
pub struct RequestBatcher {
    config: BatchConfig,
    /// Pending requests waiting to be batched
    pending_requests: Arc<RwLock<Vec<PendingRequest>>>,
    /// Cache for deduplication
    response_cache: Arc<RwLock<HashMap<RequestKey, CachedResponse>>>,
    /// Set of request keys currently being processed
    in_flight: Arc<RwLock<HashSet<RequestKey>>>,
    /// Notification for new requests
    new_request_notify: Arc<Notify>,
    /// API client for executing requests
    client: Arc<dyn LegalApiClient>,
    /// Background task handle
    _background_task: Option<tokio::task::JoinHandle<()>>,
}

impl RequestBatcher {
    /// Create a new request batcher
    pub fn new(client: Arc<dyn LegalApiClient>, config: BatchConfig) -> Self {
        let pending_requests = Arc::new(RwLock::new(Vec::new()));
        let response_cache = Arc::new(RwLock::new(HashMap::new()));
        let in_flight = Arc::new(RwLock::new(HashSet::new()));
        let new_request_notify = Arc::new(Notify::new());

        // Start background processing task
        let background_task = Self::start_background_task(
            Arc::clone(&client),
            config.clone(),
            Arc::clone(&pending_requests),
            Arc::clone(&response_cache),
            Arc::clone(&in_flight),
            Arc::clone(&new_request_notify),
        );

        Self {
            config,
            pending_requests,
            response_cache,
            in_flight,
            new_request_notify,
            client,
            _background_task: Some(background_task),
        }
    }

    /// Submit a request for batched processing
    pub async fn submit_request(&self, request: UnifiedSearchRequest) -> Result<SearchResponse> {
        let key = RequestKey::from(&request);

        // Check deduplication cache first
        if self.config.enable_deduplication {
            if let Some(cached) = self.get_cached_response(&key) {
                let metrics = get_global_metrics();
                metrics.record_cache_hit(&format!("batcher_{}", self.client.api_type().as_str()));
                return Ok(cached);
            }
        }

        // Check if request is already in flight
        let is_in_flight = {
            let in_flight = self.in_flight.read().unwrap();
            in_flight.contains(&key)
        };

        if is_in_flight {
            // Wait for the in-flight request to complete and check cache again
            let mut retries = 0;
            while retries < 50 {
                // Max 5 seconds wait
                sleep(Duration::from_millis(100)).await;
                if let Some(cached) = self.get_cached_response(&key) {
                    let metrics = get_global_metrics();
                    metrics
                        .record_cache_hit(&format!("batcher_{}", self.client.api_type().as_str()));
                    return Ok(cached);
                }
                retries += 1;
            }
            // If still no response, fall through to normal processing
        }

        // Create oneshot channel for response
        let (sender, receiver) = oneshot::channel();
        let pending = PendingRequest {
            request,
            key,
            sender,
            created_at: Instant::now(),
        };

        // Add to pending requests
        {
            let mut pending_requests = self.pending_requests.write().unwrap();
            pending_requests.push(pending);
        }

        // Notify background processor
        self.new_request_notify.notify_one();

        // Wait for response
        match receiver.await {
            Ok(response) => response,
            Err(_) => Err(WarpError::Other(
                "Request processing was cancelled".to_string(),
            )),
        }
    }

    /// Get cached response if available and not expired
    fn get_cached_response(&self, key: &RequestKey) -> Option<SearchResponse> {
        let cache = self.response_cache.read().unwrap();

        if let Some(cached) = cache.get(key) {
            if cached.created_at.elapsed() < self.config.deduplication_ttl {
                return Some(cached.response.clone());
            }
        }

        None
    }

    /// Cache a response for deduplication
    fn cache_response(&self, key: RequestKey, response: SearchResponse) {
        let mut cache = self.response_cache.write().unwrap();
        cache.insert(
            key,
            CachedResponse {
                response,
                created_at: Instant::now(),
            },
        );

        // Cleanup expired entries (simple cleanup, not comprehensive)
        let cutoff = Instant::now() - self.config.deduplication_ttl;
        cache.retain(|_, cached| cached.created_at > cutoff);
    }

    /// Start background processing task
    fn start_background_task(
        client: Arc<dyn LegalApiClient>,
        config: BatchConfig,
        pending_requests: Arc<RwLock<Vec<PendingRequest>>>,
        response_cache: Arc<RwLock<HashMap<RequestKey, CachedResponse>>>,
        in_flight: Arc<RwLock<HashSet<RequestKey>>>,
        new_request_notify: Arc<Notify>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut batch_timer = tokio::time::interval(config.max_batch_delay);
            batch_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                // Wait for either new requests or batch timeout
                tokio::select! {
                    _ = new_request_notify.notified() => {
                        // Check if we should process a batch immediately
                        let should_process = {
                            let pending = pending_requests.read().unwrap();
                            pending.len() >= config.max_batch_size
                        };

                        if should_process {
                            Self::process_batch(
                                Arc::clone(&client),
                                &config,
                                Arc::clone(&pending_requests),
                                Arc::clone(&response_cache),
                                Arc::clone(&in_flight),
                            ).await;
                        }
                    }
                    _ = batch_timer.tick() => {
                        // Process batch on timer
                        Self::process_batch(
                            Arc::clone(&client),
                            &config,
                            Arc::clone(&pending_requests),
                            Arc::clone(&response_cache),
                            Arc::clone(&in_flight),
                        ).await;
                    }
                }
            }
        })
    }

    /// Process a batch of pending requests
    async fn process_batch(
        client: Arc<dyn LegalApiClient>,
        config: &BatchConfig,
        pending_requests: Arc<RwLock<Vec<PendingRequest>>>,
        response_cache: Arc<RwLock<HashMap<RequestKey, CachedResponse>>>,
        in_flight: Arc<RwLock<HashSet<RequestKey>>>,
    ) {
        // Extract batch of pending requests
        let batch = {
            let mut pending = pending_requests.write().unwrap();
            if pending.is_empty() {
                return;
            }

            // Take up to max_batch_size requests
            let batch_size = config.max_batch_size.min(pending.len());
            pending.drain(0..batch_size).collect::<Vec<_>>()
        };

        if batch.is_empty() {
            return;
        }

        // Group requests by similarity for efficient batching
        let grouped_requests = Self::group_similar_requests(batch);

        for (_, group) in grouped_requests {
            Self::process_request_group(
                Arc::clone(&client),
                config,
                group,
                Arc::clone(&response_cache),
                Arc::clone(&in_flight),
            )
            .await;
        }
    }

    /// Group similar requests together for batch processing
    fn group_similar_requests(
        requests: Vec<PendingRequest>,
    ) -> HashMap<RequestKey, Vec<PendingRequest>> {
        let mut groups = HashMap::new();

        for request in requests {
            let key = request.key.clone();
            groups.entry(key).or_insert_with(Vec::new).push(request);
        }

        groups
    }

    /// Process a group of identical requests
    async fn process_request_group(
        client: Arc<dyn LegalApiClient>,
        config: &BatchConfig,
        group: Vec<PendingRequest>,
        response_cache: Arc<RwLock<HashMap<RequestKey, CachedResponse>>>,
        in_flight: Arc<RwLock<HashSet<RequestKey>>>,
    ) {
        if group.is_empty() {
            return;
        }

        let key = group[0].key.clone();
        let request = &group[0].request;

        // Mark as in-flight
        {
            let mut in_flight = in_flight.write().unwrap();
            in_flight.insert(key.clone());
        }

        // Execute the request
        let result = client.search(request.clone()).await;

        // Remove from in-flight
        {
            let mut in_flight = in_flight.write().unwrap();
            in_flight.remove(&key);
        }

        // Cache successful response for deduplication
        if let Ok(ref response) = result {
            if config.enable_deduplication {
                let mut cache = response_cache.write().unwrap();
                cache.insert(
                    key.clone(),
                    CachedResponse {
                        response: response.clone(),
                        created_at: Instant::now(),
                    },
                );
            }
        }

        // Save start time before moving group
        let start_time = group
            .first()
            .map(|p| p.created_at)
            .unwrap_or_else(Instant::now);

        // Send response to all waiting requests in the group
        for pending in group {
            let response_result = match &result {
                Ok(response) => Ok(response.clone()),
                Err(e) => Err(WarpError::Other(format!("Batch request failed: {}", e))),
            };

            // Send response (ignore if receiver is dropped)
            let _ = pending.sender.send(response_result);
        }

        // Record metrics
        let metrics = get_global_metrics();
        let duration = Instant::now().duration_since(start_time);
        if result.is_ok() {
            metrics.record_request_success(
                &format!("batcher_{}", client.api_type().as_str()),
                duration,
            );
        } else {
            metrics.record_request_failure(
                &format!("batcher_{}", client.api_type().as_str()),
                duration,
            );
        }
    }

    /// Get current batching statistics
    pub fn get_stats(&self) -> BatchStats {
        let pending_count = {
            let pending = self.pending_requests.read().unwrap();
            pending.len()
        };

        let cache_size = {
            let cache = self.response_cache.read().unwrap();
            cache.len()
        };

        let in_flight_count = {
            let in_flight = self.in_flight.read().unwrap();
            in_flight.len()
        };

        BatchStats {
            pending_requests: pending_count,
            cached_responses: cache_size,
            in_flight_requests: in_flight_count,
            deduplication_enabled: self.config.enable_deduplication,
        }
    }

    /// Clean up expired cache entries
    pub fn cleanup_cache(&self) {
        let mut cache = self.response_cache.write().unwrap();
        let cutoff = Instant::now() - self.config.deduplication_ttl;
        let initial_size = cache.len();

        cache.retain(|_, cached| cached.created_at > cutoff);

        let cleaned = initial_size - cache.len();
        if cleaned > 0 {
            log::debug!("Cleaned up {} expired cache entries", cleaned);
        }
    }
}

/// Statistics for the request batcher
#[derive(Debug)]
pub struct BatchStats {
    pub pending_requests: usize,
    pub cached_responses: usize,
    pub in_flight_requests: usize,
    pub deduplication_enabled: bool,
}

/// Factory for creating request batchers per API type
pub struct BatcherFactory;

impl BatcherFactory {
    /// Create a request batcher for a specific API client
    pub fn create_batcher(
        client: Arc<dyn LegalApiClient>,
        config: Option<BatchConfig>,
    ) -> RequestBatcher {
        let config = config.unwrap_or_default();
        RequestBatcher::new(client, config)
    }

    /// Create a batcher with optimized configuration for high-throughput scenarios
    pub fn create_high_throughput_batcher(client: Arc<dyn LegalApiClient>) -> RequestBatcher {
        let config = BatchConfig {
            max_batch_size: 20,
            max_batch_delay: Duration::from_millis(50),
            enable_deduplication: true,
            deduplication_ttl: Duration::from_secs(300), // 5 minutes
            enable_predictive_batching: true,
        };

        RequestBatcher::new(client, config)
    }

    /// Create a batcher optimized for low-latency scenarios
    pub fn create_low_latency_batcher(client: Arc<dyn LegalApiClient>) -> RequestBatcher {
        let config = BatchConfig {
            max_batch_size: 5,
            max_batch_delay: Duration::from_millis(25),
            enable_deduplication: true,
            deduplication_ttl: Duration::from_secs(60),
            enable_predictive_batching: false, // Disable for lower latency
        };

        RequestBatcher::new(client, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::ResponseType;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock client for testing
    struct MockBatchClient {
        call_count: Arc<AtomicUsize>,
        delay: Duration,
    }

    impl MockBatchClient {
        fn new(delay: Duration) -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                delay,
            }
        }
    }

    #[async_trait]
    impl LegalApiClient for MockBatchClient {
        async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse> {
            self.call_count.fetch_add(1, Ordering::Relaxed);

            // Simulate processing time
            sleep(self.delay).await;

            Ok(SearchResponse {
                total_count: 10,
                page_no: request.page_no,
                page_size: request.page_size,
                items: vec![],
                source: "MockBatchAPI".to_string(),
                timestamp: chrono::Utc::now(),
            })
        }

        async fn get_detail(&self, _id: &str) -> Result<LawDetail> {
            unimplemented!()
        }

        async fn get_history(&self, _id: &str) -> Result<LawHistory> {
            unimplemented!()
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
    async fn test_request_deduplication() {
        let client = Arc::new(MockBatchClient::new(Duration::from_millis(100)));
        let batcher = Arc::new(RequestBatcher::new(client.clone(), BatchConfig::default()));

        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            page_no: 1,
            page_size: 10,
            response_type: ResponseType::Json,
            ..Default::default()
        };

        // Submit the same request multiple times concurrently
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let batcher = Arc::clone(&batcher);
                let request = request.clone();
                tokio::spawn(async move { batcher.submit_request(request).await })
            })
            .collect();

        // Wait for all requests to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All should succeed
        for result in results {
            assert!(result.unwrap().is_ok());
        }

        // Should have made significantly fewer API calls due to deduplication
        let call_count = client.call_count.load(Ordering::Relaxed);
        assert!(
            call_count < 5,
            "Expected fewer than 5 calls, got {}",
            call_count
        );
        assert!(
            call_count >= 1,
            "Expected at least 1 call, got {}",
            call_count
        );
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let client = Arc::new(MockBatchClient::new(Duration::from_millis(50)));
        let config = BatchConfig {
            max_batch_size: 3,
            max_batch_delay: Duration::from_millis(500),
            ..BatchConfig::default()
        };
        let batcher = Arc::new(RequestBatcher::new(client.clone(), config));

        // Submit different requests that should be batched
        let handles: Vec<_> = (0..6)
            .map(|i| {
                let batcher = Arc::clone(&batcher);
                let request = UnifiedSearchRequest {
                    query: format!("test_{}", i),
                    page_no: 1,
                    page_size: 10,
                    response_type: ResponseType::Json,
                    ..Default::default()
                };
                tokio::spawn(async move { batcher.submit_request(request).await })
            })
            .collect();

        // Wait for all requests to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All should succeed
        for result in results {
            assert!(result.unwrap().is_ok());
        }

        // Should have made multiple calls (different requests)
        let call_count = client.call_count.load(Ordering::Relaxed);
        assert_eq!(
            call_count, 6,
            "Expected 6 calls for 6 different requests, got {}",
            call_count
        );
    }

    #[tokio::test]
    async fn test_batch_stats() {
        let client = Arc::new(MockBatchClient::new(Duration::from_millis(10)));
        let batcher = RequestBatcher::new(client, BatchConfig::default());

        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            page_no: 1,
            page_size: 10,
            response_type: ResponseType::Json,
            ..Default::default()
        };

        // Submit request and get response to populate cache
        let _response = batcher.submit_request(request).await.unwrap();

        let stats = batcher.get_stats();
        assert!(stats.deduplication_enabled);
        assert!(stats.cached_responses >= 1);
    }
}
