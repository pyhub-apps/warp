use super::{
    CacheMetrics, ConnectionPoolMetrics, MetricsSnapshot, MetricsWindow, OperationMetrics,
};
use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Thread-safe metrics collector for tracking application performance
pub struct MetricsCollector {
    /// Operation-specific metrics (API calls, searches, etc.)
    operations: RwLock<HashMap<String, OperationStats>>,
    /// Cache performance metrics per API
    caches: RwLock<HashMap<String, CacheMetrics>>,
    /// Connection pool metrics per endpoint
    connection_pools: RwLock<HashMap<String, ConnectionPoolMetrics>>,
    /// Application start time
    start_time: Instant,
}

/// Internal operation statistics with timing data
#[derive(Debug)]
struct OperationStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    /// Rolling window of request durations for percentile calculations
    durations: VecDeque<Duration>,
    total_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
    last_updated: Instant,
}

impl Default for OperationStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            durations: VecDeque::new(),
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            last_updated: Instant::now(),
        }
    }
}

impl OperationStats {
    /// Add a successful request duration
    fn add_success(&mut self, duration: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.add_duration(duration);
    }

    /// Add a failed request duration
    fn add_failure(&mut self, duration: Duration) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.add_duration(duration);
    }

    /// Add duration to statistics
    fn add_duration(&mut self, duration: Duration) {
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
        self.last_updated = Instant::now();

        // Keep a rolling window of durations for percentile calculations
        // Limit to 10,000 entries to prevent unbounded memory growth
        self.durations.push_back(duration);
        if self.durations.len() > 10000 {
            self.durations.pop_front();
        }
    }

    /// Convert to public OperationMetrics with calculated percentiles
    fn to_operation_metrics(&self) -> OperationMetrics {
        let avg_duration = if self.total_requests > 0 {
            self.total_duration / self.total_requests as u32
        } else {
            Duration::ZERO
        };

        let (p50, p95, p99) = self.calculate_percentiles();

        OperationMetrics {
            total_requests: self.total_requests,
            successful_requests: self.successful_requests,
            failed_requests: self.failed_requests,
            total_duration: self.total_duration,
            min_duration: if self.min_duration == Duration::MAX {
                Duration::ZERO
            } else {
                self.min_duration
            },
            max_duration: self.max_duration,
            avg_duration,
            p50_duration: p50,
            p95_duration: p95,
            p99_duration: p99,
        }
    }

    /// Calculate percentiles from duration data
    fn calculate_percentiles(&self) -> (Duration, Duration, Duration) {
        if self.durations.is_empty() {
            return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
        }

        let mut sorted_durations: Vec<Duration> = self.durations.iter().cloned().collect();
        sorted_durations.sort();

        let len = sorted_durations.len();
        let p50_idx = (len as f64 * 0.50) as usize;
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;

        let p50 = sorted_durations
            .get(p50_idx.saturating_sub(1))
            .cloned()
            .unwrap_or(Duration::ZERO);
        let p95 = sorted_durations
            .get(p95_idx.saturating_sub(1))
            .cloned()
            .unwrap_or(Duration::ZERO);
        let p99 = sorted_durations
            .get(p99_idx.saturating_sub(1))
            .cloned()
            .unwrap_or(Duration::ZERO);

        (p50, p95, p99)
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
            caches: RwLock::new(HashMap::new()),
            connection_pools: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    /// Record a successful request
    pub fn record_request_success(&self, operation: &str, duration: Duration) {
        let mut operations = self.operations.write().unwrap();
        let stats = operations.entry(operation.to_string()).or_default();
        stats.add_success(duration);
    }

    /// Record a failed request
    pub fn record_request_failure(&self, operation: &str, duration: Duration) {
        let mut operations = self.operations.write().unwrap();
        let stats = operations.entry(operation.to_string()).or_default();
        stats.add_failure(duration);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self, api: &str) {
        let mut caches = self.caches.write().unwrap();
        let metrics = caches.entry(api.to_string()).or_default();
        metrics.hits += 1;
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self, api: &str) {
        let mut caches = self.caches.write().unwrap();
        let metrics = caches.entry(api.to_string()).or_default();
        metrics.misses += 1;
    }

    /// Record cache eviction
    pub fn record_cache_eviction(&self, api: &str) {
        let mut caches = self.caches.write().unwrap();
        let metrics = caches.entry(api.to_string()).or_default();
        metrics.evictions += 1;
    }

    /// Update cache storage metrics
    pub fn update_cache_storage(&self, api: &str, size: u64, entry_count: u64) {
        let mut caches = self.caches.write().unwrap();
        let metrics = caches.entry(api.to_string()).or_default();
        metrics.storage_size = size;
        metrics.entry_count = entry_count;
    }

    /// Update connection pool metrics
    pub fn update_connection_pool(&self, pool_name: &str, active: u32, idle: u32, total: u32) {
        let mut pools = self.connection_pools.write().unwrap();
        let metrics = pools.entry(pool_name.to_string()).or_default();
        metrics.active_connections = active;
        metrics.idle_connections = idle;
        metrics.total_connections = total;
    }

    /// Record connection acquisition
    pub fn record_connection_acquire(&self, pool_name: &str, success: bool, timeout: bool) {
        let mut pools = self.connection_pools.write().unwrap();
        let metrics = pools.entry(pool_name.to_string()).or_default();
        metrics.connection_acquires += 1;

        if timeout {
            metrics.connection_timeouts += 1;
        } else if !success {
            metrics.connection_errors += 1;
        }
    }

    /// Get current metrics snapshot
    pub async fn get_snapshot(&self) -> MetricsSnapshot {
        let operations = {
            let ops = self.operations.read().unwrap();
            ops.iter()
                .map(|(k, v)| (k.clone(), v.to_operation_metrics()))
                .collect()
        };

        let cache = {
            let caches = self.caches.read().unwrap();
            caches.clone()
        };

        let connection_pools = {
            let pools = self.connection_pools.read().unwrap();
            pools.clone()
        };

        MetricsSnapshot {
            timestamp: Instant::now(),
            operations,
            cache,
            connection_pools,
            memory_usage: self.get_memory_usage(),
            uptime: self.start_time.elapsed(),
        }
    }

    /// Get metrics for a specific time window
    pub async fn get_windowed_metrics(&self, _window: MetricsWindow) -> MetricsSnapshot {
        // For now, return current snapshot
        // TODO: Implement time-windowed metrics with historical data
        self.get_snapshot().await
    }

    /// Get operation metrics for a specific operation
    pub fn get_operation_metrics(&self, operation: &str) -> Option<OperationMetrics> {
        let operations = self.operations.read().unwrap();
        operations
            .get(operation)
            .map(|stats| stats.to_operation_metrics())
    }

    /// Get cache metrics for a specific API
    pub fn get_cache_metrics(&self, api: &str) -> Option<CacheMetrics> {
        let caches = self.caches.read().unwrap();
        caches.get(api).cloned()
    }

    /// Get top operations by request count
    pub fn get_top_operations(&self, limit: usize) -> Vec<(String, OperationMetrics)> {
        let operations = self.operations.read().unwrap();
        let mut ops: Vec<_> = operations
            .iter()
            .map(|(k, v)| (k.clone(), v.to_operation_metrics()))
            .collect();

        ops.sort_by(|a, b| b.1.total_requests.cmp(&a.1.total_requests));
        ops.truncate(limit);
        ops
    }

    /// Get slowest operations by average duration
    pub fn get_slowest_operations(&self, limit: usize) -> Vec<(String, OperationMetrics)> {
        let operations = self.operations.read().unwrap();
        let mut ops: Vec<_> = operations
            .iter()
            .map(|(k, v)| (k.clone(), v.to_operation_metrics()))
            .collect();

        ops.sort_by(|a, b| b.1.avg_duration.cmp(&a.1.avg_duration));
        ops.truncate(limit);
        ops
    }

    /// Reset all metrics
    pub fn reset(&self) {
        let mut operations = self.operations.write().unwrap();
        let mut caches = self.caches.write().unwrap();
        let mut pools = self.connection_pools.write().unwrap();

        operations.clear();
        caches.clear();
        pools.clear();
    }

    /// Get approximate memory usage of the collector itself
    fn get_memory_usage(&self) -> u64 {
        // Simple approximation - in production, use a proper memory profiler
        let operations = self.operations.read().unwrap();
        let ops_memory = operations.len() * 1024; // Rough estimate per operation

        let durations_memory: usize = operations
            .values()
            .map(|stats| stats.durations.len() * std::mem::size_of::<Duration>())
            .sum();

        (ops_memory + durations_memory) as u64
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_stats_success() {
        let mut stats = OperationStats::default();
        stats.add_success(Duration::from_millis(100));
        stats.add_success(Duration::from_millis(200));

        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 0);
    }

    #[test]
    fn test_operation_stats_failure() {
        let mut stats = OperationStats::default();
        stats.add_failure(Duration::from_millis(50));

        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 1);
    }

    #[test]
    fn test_percentile_calculation() {
        let mut stats = OperationStats::default();

        // Add some test durations
        for ms in [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000] {
            stats.add_duration(Duration::from_millis(ms));
        }

        let (p50, p95, _p99) = stats.calculate_percentiles();

        // With 10 samples, p50 should be around 500ms
        assert!(p50 >= Duration::from_millis(400) && p50 <= Duration::from_millis(600));
        // p95 should be around 950ms
        assert!(p95 >= Duration::from_millis(900) && p95 <= Duration::from_millis(1000));
    }

    #[tokio::test]
    async fn test_metrics_collector_basic_operations() {
        let collector = MetricsCollector::new();

        collector.record_request_success("test_op", Duration::from_millis(100));
        collector.record_request_failure("test_op", Duration::from_millis(200));
        collector.record_cache_hit("test_api");
        collector.record_cache_miss("test_api");

        let snapshot = collector.get_snapshot().await;

        assert!(snapshot.operations.contains_key("test_op"));
        assert!(snapshot.cache.contains_key("test_api"));

        let op_metrics = snapshot.operations.get("test_op").unwrap();
        assert_eq!(op_metrics.total_requests, 2);
        assert_eq!(op_metrics.successful_requests, 1);
        assert_eq!(op_metrics.failed_requests, 1);

        let cache_metrics = snapshot.cache.get("test_api").unwrap();
        assert_eq!(cache_metrics.hits, 1);
        assert_eq!(cache_metrics.misses, 1);
        assert_eq!(cache_metrics.hit_rate(), 50.0);
    }
}
