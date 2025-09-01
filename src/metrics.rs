use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Performance metrics collector for API operations
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    // Request metrics
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,

    // Timing metrics (in milliseconds)
    total_request_time_ms: AtomicU64,
    min_request_time_ms: AtomicU64,
    max_request_time_ms: AtomicU64,

    // Memory metrics
    peak_memory_usage: AtomicUsize,
    current_memory_usage: AtomicUsize,

    // Cache metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,

    // Per-API metrics
    api_metrics: Arc<RwLock<HashMap<String, ApiMetrics>>>,
}

#[derive(Debug, Default, Clone)]
pub struct ApiMetrics {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub total_time_ms: u64,
    pub min_time_ms: u64,
    pub max_time_ms: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful request
    pub fn record_request_success(&self, api_name: &str, duration: Duration) {
        let duration_ms = duration.as_millis() as u64;

        // Update global metrics
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_request_time_ms
            .fetch_add(duration_ms, Ordering::Relaxed);

        // Update min/max timing
        self.update_min_time(duration_ms);
        self.update_max_time(duration_ms);

        // Update per-API metrics
        self.update_api_metrics(api_name, |metrics| {
            metrics.requests += 1;
            metrics.successes += 1;
            metrics.total_time_ms += duration_ms;
            metrics.min_time_ms = if metrics.min_time_ms == 0 {
                duration_ms
            } else {
                metrics.min_time_ms.min(duration_ms)
            };
            metrics.max_time_ms = metrics.max_time_ms.max(duration_ms);
        });
    }

    /// Record a failed request
    pub fn record_request_failure(&self, api_name: &str, duration: Duration) {
        let duration_ms = duration.as_millis() as u64;

        // Update global metrics
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.total_request_time_ms
            .fetch_add(duration_ms, Ordering::Relaxed);

        // Update per-API metrics
        self.update_api_metrics(api_name, |metrics| {
            metrics.requests += 1;
            metrics.failures += 1;
            metrics.total_time_ms += duration_ms;
        });
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self, api_name: &str) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);

        self.update_api_metrics(api_name, |metrics| {
            metrics.cache_hits += 1;
        });
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self, api_name: &str) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        self.update_api_metrics(api_name, |metrics| {
            metrics.cache_misses += 1;
        });
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, current: usize) {
        self.current_memory_usage.store(current, Ordering::Relaxed);

        // Update peak if necessary
        let current_peak = self.peak_memory_usage.load(Ordering::Relaxed);
        if current > current_peak {
            self.peak_memory_usage.store(current, Ordering::Relaxed);
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> MetricsSnapshot {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.successful_requests.load(Ordering::Relaxed);
        let failed_requests = self.failed_requests.load(Ordering::Relaxed);
        let total_time_ms = self.total_request_time_ms.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);

        let avg_response_time = if total_requests > 0 {
            total_time_ms / total_requests
        } else {
            0
        };

        let success_rate = if total_requests > 0 {
            (successful_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            (cache_hits as f64 / (cache_hits + cache_misses) as f64) * 100.0
        } else {
            0.0
        };

        let api_stats = self
            .api_metrics
            .read()
            .unwrap()
            .iter()
            .map(|(name, metrics)| (name.clone(), metrics.clone()))
            .collect();

        MetricsSnapshot {
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms: avg_response_time,
            min_response_time_ms: self.min_request_time_ms.load(Ordering::Relaxed),
            max_response_time_ms: self.max_request_time_ms.load(Ordering::Relaxed),
            success_rate,
            cache_hit_rate,
            peak_memory_usage: self.peak_memory_usage.load(Ordering::Relaxed),
            current_memory_usage: self.current_memory_usage.load(Ordering::Relaxed),
            api_stats,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.total_request_time_ms.store(0, Ordering::Relaxed);
        self.min_request_time_ms.store(0, Ordering::Relaxed);
        self.max_request_time_ms.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.peak_memory_usage.store(0, Ordering::Relaxed);
        self.current_memory_usage.store(0, Ordering::Relaxed);
        self.api_metrics.write().unwrap().clear();
    }

    fn update_min_time(&self, duration_ms: u64) {
        let current = self.min_request_time_ms.load(Ordering::Relaxed);
        if current == 0 || duration_ms < current {
            self.min_request_time_ms
                .store(duration_ms, Ordering::Relaxed);
        }
    }

    fn update_max_time(&self, duration_ms: u64) {
        let current = self.max_request_time_ms.load(Ordering::Relaxed);
        if duration_ms > current {
            self.max_request_time_ms
                .store(duration_ms, Ordering::Relaxed);
        }
    }

    fn update_api_metrics<F>(&self, api_name: &str, update_fn: F)
    where
        F: FnOnce(&mut ApiMetrics),
    {
        let mut api_metrics = self.api_metrics.write().unwrap();
        let metrics = api_metrics.entry(api_name.to_string()).or_default();
        update_fn(metrics);
    }
}

/// Snapshot of current metrics
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: u64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub peak_memory_usage: usize,
    pub current_memory_usage: usize,
    pub api_stats: HashMap<String, ApiMetrics>,
}

impl MetricsSnapshot {
    /// Format metrics as a human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("=== Performance Metrics ===\n"));
        output.push_str(&format!("Total Requests: {}\n", self.total_requests));
        output.push_str(&format!("Success Rate: {:.1}%\n", self.success_rate));
        output.push_str(&format!("Cache Hit Rate: {:.1}%\n", self.cache_hit_rate));
        output.push_str(&format!("\nTiming:\n"));
        output.push_str(&format!("  Average: {}ms\n", self.avg_response_time_ms));
        output.push_str(&format!("  Min: {}ms\n", self.min_response_time_ms));
        output.push_str(&format!("  Max: {}ms\n", self.max_response_time_ms));
        output.push_str(&format!("\nMemory:\n"));
        output.push_str(&format!("  Current: {} bytes\n", self.current_memory_usage));
        output.push_str(&format!("  Peak: {} bytes\n", self.peak_memory_usage));

        if !self.api_stats.is_empty() {
            output.push_str(&format!("\nPer-API Stats:\n"));
            for (api, stats) in &self.api_stats {
                let avg_time = if stats.requests > 0 {
                    stats.total_time_ms / stats.requests
                } else {
                    0
                };
                output.push_str(&format!(
                    "  {}: {} req, {:.1}% success, {}ms avg\n",
                    api,
                    stats.requests,
                    if stats.requests > 0 {
                        (stats.successes as f64 / stats.requests as f64) * 100.0
                    } else {
                        0.0
                    },
                    avg_time
                ));
            }
        }

        output
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    start: Instant,
    api_name: String,
    metrics: Arc<PerformanceMetrics>,
}

impl OperationTimer {
    /// Start timing an operation
    pub fn start(api_name: String, metrics: Arc<PerformanceMetrics>) -> Self {
        Self {
            start: Instant::now(),
            api_name,
            metrics,
        }
    }

    /// Finish timing and record success
    pub fn finish_success(self) {
        let duration = self.start.elapsed();
        self.metrics
            .record_request_success(&self.api_name, duration);
    }

    /// Finish timing and record failure
    pub fn finish_failure(self) {
        let duration = self.start.elapsed();
        self.metrics
            .record_request_failure(&self.api_name, duration);
    }
}

/// Global metrics instance
use once_cell::sync::Lazy;
static GLOBAL_METRICS: Lazy<Arc<PerformanceMetrics>> =
    Lazy::new(|| Arc::new(PerformanceMetrics::new()));

/// Get the global metrics instance
pub fn get_global_metrics() -> Arc<PerformanceMetrics> {
    GLOBAL_METRICS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_recording() {
        let metrics = PerformanceMetrics::new();

        metrics.record_request_success("test_api", Duration::from_millis(100));
        metrics.record_request_failure("test_api", Duration::from_millis(200));
        metrics.record_cache_hit("test_api");
        metrics.record_cache_miss("test_api");

        let stats = metrics.get_stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.success_rate, 50.0);
        assert_eq!(stats.cache_hit_rate, 50.0);
    }

    #[test]
    fn test_operation_timer() {
        let metrics = Arc::new(PerformanceMetrics::new());

        let timer = OperationTimer::start("test".to_string(), metrics.clone());
        std::thread::sleep(Duration::from_millis(10));
        timer.finish_success();

        let stats = metrics.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert!(stats.avg_response_time_ms >= 10);
    }
}
