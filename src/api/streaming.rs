use super::types::*;
use super::{ApiType, LegalApiClient};
use crate::error::{Result, WarpError};
use futures::stream::{self, Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Configuration for streaming large result sets
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Size of each page/batch to fetch
    pub page_size: u32,
    /// Maximum number of concurrent page requests
    pub max_concurrent_pages: usize,
    /// Delay between page requests to avoid rate limiting
    pub page_delay: Duration,
    /// Maximum total items to fetch (0 = unlimited)
    pub max_items: u32,
    /// Buffer size for the stream
    pub buffer_size: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            page_size: 100,
            max_concurrent_pages: 3,
            page_delay: Duration::from_millis(200),
            max_items: 0, // unlimited
            buffer_size: 1000,
        }
    }
}

/// A streaming iterator over search results that fetches pages on-demand
pub struct SearchResultStream {
    client: Arc<dyn LegalApiClient>,
    base_request: UnifiedSearchRequest,
    config: StreamConfig,
    current_page: u32,
    total_count: Option<u32>,
    items_fetched: u32,
    finished: bool,
}

impl SearchResultStream {
    /// Create a new streaming search result iterator
    pub fn new(
        client: Arc<dyn LegalApiClient>,
        mut request: UnifiedSearchRequest,
        config: StreamConfig,
    ) -> Self {
        // Ensure page size matches config
        request.page_size = config.page_size;

        Self {
            client,
            base_request: request,
            config,
            current_page: 1,
            total_count: None,
            items_fetched: 0,
            finished: false,
        }
    }

    /// Convert to a stream of individual SearchItem
    pub fn into_item_stream(self) -> impl Stream<Item = Result<SearchItem>> {
        stream::unfold(self, |mut state| async move {
            if state.finished {
                return None;
            }

            // Check if we've reached the maximum items limit
            if state.config.max_items > 0 && state.items_fetched >= state.config.max_items {
                return None;
            }

            // Fetch next page
            match state.fetch_next_page().await {
                Ok(Some(response)) => {
                    // Update state
                    if state.total_count.is_none() {
                        state.total_count = Some(response.total_count);
                    }

                    state.items_fetched += response.items.len() as u32;
                    state.current_page += 1;

                    // Check if we should continue
                    if response.items.is_empty()
                        || (state.total_count.is_some()
                            && state.items_fetched >= state.total_count.unwrap())
                    {
                        state.finished = true;
                    }

                    // Return items from this page
                    let items: Vec<Result<SearchItem>> =
                        response.items.into_iter().map(Ok).collect();

                    Some((stream::iter(items), state))
                }
                Ok(None) => {
                    state.finished = true;
                    None
                }
                Err(e) => {
                    state.finished = true;
                    Some((stream::iter(vec![Err(e)]), state))
                }
            }
        })
        .flatten()
    }

    /// Convert to a stream of pages (SearchResponse)
    pub fn into_page_stream(self) -> impl Stream<Item = Result<SearchResponse>> {
        stream::unfold(self, |mut state| async move {
            if state.finished {
                return None;
            }

            match state.fetch_next_page().await {
                Ok(Some(response)) => {
                    // Update state
                    if state.total_count.is_none() {
                        state.total_count = Some(response.total_count);
                    }

                    state.items_fetched += response.items.len() as u32;
                    state.current_page += 1;

                    // Check if we should continue
                    if response.items.is_empty()
                        || (state.total_count.is_some()
                            && state.items_fetched >= state.total_count.unwrap())
                    {
                        state.finished = true;
                    }

                    // Add delay between requests
                    if state.config.page_delay > Duration::ZERO && !state.finished {
                        tokio::time::sleep(state.config.page_delay).await;
                    }

                    Some((Ok(response), state))
                }
                Ok(None) => {
                    state.finished = true;
                    None
                }
                Err(e) => {
                    state.finished = true;
                    Some((Err(e), state))
                }
            }
        })
    }

    async fn fetch_next_page(&self) -> Result<Option<SearchResponse>> {
        // Check if we've reached the maximum items limit
        if self.config.max_items > 0 && self.items_fetched >= self.config.max_items {
            return Ok(None);
        }

        let mut request = self.base_request.clone();
        request.page_no = self.current_page;
        request.page_size = self.config.page_size;

        match self.client.search(request).await {
            Ok(response) => {
                if response.items.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(response))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Get current streaming statistics
    pub fn get_stats(&self) -> StreamStats {
        StreamStats {
            current_page: self.current_page,
            items_fetched: self.items_fetched,
            total_count: self.total_count,
            finished: self.finished,
        }
    }
}

/// Statistics about the streaming operation
#[derive(Debug, Clone)]
pub struct StreamStats {
    pub current_page: u32,
    pub items_fetched: u32,
    pub total_count: Option<u32>,
    pub finished: bool,
}

impl StreamStats {
    /// Get progress as a percentage (0.0 - 1.0)
    pub fn progress(&self) -> Option<f64> {
        self.total_count.map(|total| {
            if total == 0 {
                1.0
            } else {
                (self.items_fetched as f64) / (total as f64)
            }
        })
    }
}

/// Parallel streaming for multiple APIs
pub struct ParallelSearchStream {
    streams: Vec<Pin<Box<dyn Stream<Item = Result<SearchItem>> + Send>>>,
    config: StreamConfig,
}

impl ParallelSearchStream {
    /// Create parallel streams for multiple API clients
    pub fn new(
        clients: Vec<(ApiType, Arc<dyn LegalApiClient>)>,
        request: UnifiedSearchRequest,
        config: StreamConfig,
    ) -> Self {
        let streams = clients
            .into_iter()
            .map(|(_api_type, client)| {
                let stream = SearchResultStream::new(client, request.clone(), config.clone());
                Box::pin(stream.into_item_stream())
                    as Pin<Box<dyn Stream<Item = Result<SearchItem>> + Send>>
            })
            .collect();

        Self { streams, config }
    }

    /// Merge all streams into a single stream with round-robin fairness
    pub fn merge_fair(self) -> impl Stream<Item = Result<SearchItem>> {
        // Use select_all for fair merging
        stream::select_all(self.streams)
    }

    /// Merge all streams with buffering for better performance  
    pub fn merge_buffered(self) -> impl Stream<Item = Result<SearchItem>> {
        // Note: buffered() requires items to be futures, not results
        // For now, just use the fair merge
        stream::select_all(self.streams)
    }
}

/// Helper function to create a memory-efficient search stream
pub fn stream_search_results(
    client: Arc<dyn LegalApiClient>,
    request: UnifiedSearchRequest,
) -> impl Stream<Item = Result<SearchItem>> {
    let config = StreamConfig::default();
    SearchResultStream::new(client, request, config).into_item_stream()
}

/// Helper function for streaming all results from multiple APIs
pub fn stream_all_apis(
    clients: Vec<(ApiType, Arc<dyn LegalApiClient>)>,
    request: UnifiedSearchRequest,
) -> impl Stream<Item = Result<SearchItem>> {
    let config = StreamConfig::default();
    ParallelSearchStream::new(clients, request, config).merge_fair()
}

/// Utility to collect stream into batches for processing
pub fn batch_stream<T>(
    stream: impl Stream<Item = T>,
    batch_size: usize,
) -> impl Stream<Item = Vec<T>> {
    stream.ready_chunks(batch_size)
}

/// Rate-limited stream wrapper
pub fn rate_limit_stream<T>(
    stream: impl Stream<Item = T>,
    rate_limit: Duration,
) -> impl Stream<Item = T> {
    // Create a rate-limited stream using zip with an interval
    let intervals = stream::repeat(()).then(move |_| async move {
        tokio::time::sleep(rate_limit).await;
    });

    stream.zip(intervals).map(|(item, _)| item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::ClientConfig;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock client for testing streaming
    struct MockStreamClient {
        total_items: u32,
        page_size: u32,
        delay: Duration,
        call_count: Arc<AtomicUsize>,
    }

    impl MockStreamClient {
        fn new(total_items: u32, page_size: u32, delay: Duration) -> Self {
            Self {
                total_items,
                page_size,
                delay,
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait]
    impl LegalApiClient for MockStreamClient {
        async fn search(&self, request: UnifiedSearchRequest) -> Result<SearchResponse> {
            self.call_count.fetch_add(1, Ordering::Relaxed);

            // Simulate network delay
            tokio::time::sleep(self.delay).await;

            let start_idx = ((request.page_no - 1) * request.page_size) as usize;
            let end_idx = std::cmp::min(
                start_idx + request.page_size as usize,
                self.total_items as usize,
            );

            let items: Vec<SearchItem> = (start_idx..end_idx)
                .map(|i| SearchItem {
                    id: format!("item_{}", i),
                    title: format!("Mock Item {}", i),
                    law_no: Some(format!("LAW{:04}", i)),
                    law_type: Some("Mock Law".to_string()),
                    department: Some("Mock Department".to_string()),
                    enforcement_date: None,
                    revision_date: None,
                    summary: None,
                    source: "MockAPI".to_string(),
                    metadata: std::collections::HashMap::new(),
                })
                .collect();

            Ok(SearchResponse {
                total_count: self.total_items,
                page_no: request.page_no,
                page_size: request.page_size,
                items,
                source: "MockAPI".to_string(),
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
    async fn test_search_result_stream() {
        let client = Arc::new(MockStreamClient::new(250, 50, Duration::from_millis(10)));
        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            page_size: 50,
            ..Default::default()
        };

        let config = StreamConfig {
            page_size: 50,
            max_items: 100, // Limit to first 100 items
            ..StreamConfig::default()
        };

        let stream = SearchResultStream::new(client.clone(), request, config);
        let items: Vec<Result<SearchItem>> = stream.into_item_stream().collect().await;

        assert_eq!(items.len(), 100);
        assert!(items.iter().all(|item| item.is_ok()));

        // Should have made 2 API calls (100 items / 50 per page)
        assert_eq!(client.call_count.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_parallel_search_stream() {
        let clients = vec![
            (
                ApiType::Nlic,
                Arc::new(MockStreamClient::new(100, 25, Duration::from_millis(5)))
                    as Arc<dyn LegalApiClient>,
            ),
            (
                ApiType::Elis,
                Arc::new(MockStreamClient::new(150, 25, Duration::from_millis(5)))
                    as Arc<dyn LegalApiClient>,
            ),
        ];

        let request = UnifiedSearchRequest {
            query: "test".to_string(),
            page_size: 25,
            ..Default::default()
        };

        let config = StreamConfig {
            page_size: 25,
            max_items: 50, // 50 items from each API
            ..StreamConfig::default()
        };

        let stream = ParallelSearchStream::new(clients, request, config);
        let items: Vec<Result<SearchItem>> = stream.merge_fair().collect().await;

        assert_eq!(items.len(), 100); // 50 items from each API
        assert!(items.iter().all(|item| item.is_ok()));
    }

    #[tokio::test]
    async fn test_stream_batching() {
        let items = stream::iter(0..100);
        let batches: Vec<Vec<i32>> = batch_stream(items, 10).collect().await;

        assert_eq!(batches.len(), 10);
        assert_eq!(batches[0].len(), 10);
        assert_eq!(batches[9].len(), 10);
    }
}
