use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub mod collector;
pub mod dashboard;
pub mod storage;

pub use collector::MetricsCollector;
pub use dashboard::PerformanceDashboard;
pub use storage::MetricsStorage;

/// Global metrics instance for the application
static GLOBAL_METRICS: Lazy<Arc<MetricsCollector>> =
    Lazy::new(|| Arc::new(MetricsCollector::new()));

/// Get the global metrics collector
pub fn get_global_metrics() -> Arc<MetricsCollector> {
    GLOBAL_METRICS.clone()
}

/// Operation timer for measuring request latencies
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
    collector: Arc<MetricsCollector>,
}

impl OperationTimer {
    /// Start timing an operation
    pub fn start(operation: String, collector: Arc<MetricsCollector>) -> Self {
        Self {
            operation,
            start_time: Instant::now(),
            collector,
        }
    }

    /// Finish the timer successfully
    pub fn finish_success(self) {
        let duration = self.start_time.elapsed();
        self.collector
            .record_request_success(&self.operation, duration);
    }

    /// Finish the timer with failure
    pub fn finish_failure(self) {
        let duration = self.start_time.elapsed();
        self.collector
            .record_request_failure(&self.operation, duration);
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Performance metrics for a specific operation
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
}

impl Default for OperationMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            p50_duration: Duration::ZERO,
            p95_duration: Duration::ZERO,
            p99_duration: Duration::ZERO,
        }
    }
}

impl OperationMetrics {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Get error rate as percentage
    pub fn error_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }

    /// Get requests per second based on time window
    pub fn requests_per_second(&self, time_window: Duration) -> f64 {
        if time_window.is_zero() {
            0.0
        } else {
            self.total_requests as f64 / time_window.as_secs_f64()
        }
    }
}

/// Cache metrics for tracking hit/miss ratios
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub storage_size: u64,
    pub entry_count: u64,
}

impl CacheMetrics {
    /// Get cache hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get cache miss rate as percentage
    pub fn miss_rate(&self) -> f64 {
        100.0 - self.hit_rate()
    }
}

/// Connection pool metrics
#[derive(Debug, Clone, Default)]
pub struct ConnectionPoolMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub connection_acquires: u64,
    pub connection_timeouts: u64,
    pub connection_errors: u64,
}

impl ConnectionPoolMetrics {
    /// Get connection utilization as percentage
    pub fn utilization(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            (self.active_connections as f64 / self.total_connections as f64) * 100.0
        }
    }

    /// Get connection timeout rate as percentage
    pub fn timeout_rate(&self) -> f64 {
        if self.connection_acquires == 0 {
            0.0
        } else {
            (self.connection_timeouts as f64 / self.connection_acquires as f64) * 100.0
        }
    }
}

/// System-wide metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: Instant,
    pub operations: HashMap<String, OperationMetrics>,
    pub cache: HashMap<String, CacheMetrics>,
    pub connection_pools: HashMap<String, ConnectionPoolMetrics>,
    pub memory_usage: u64,
    pub uptime: Duration,
}

/// Metrics aggregation window
#[derive(Debug, Clone, Copy)]
pub enum MetricsWindow {
    LastMinute,
    Last5Minutes,
    Last15Minutes,
    LastHour,
    Last24Hours,
}

impl MetricsWindow {
    pub fn duration(&self) -> Duration {
        match self {
            MetricsWindow::LastMinute => Duration::from_secs(60),
            MetricsWindow::Last5Minutes => Duration::from_secs(300),
            MetricsWindow::Last15Minutes => Duration::from_secs(900),
            MetricsWindow::LastHour => Duration::from_secs(3600),
            MetricsWindow::Last24Hours => Duration::from_secs(86400),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_metrics_success_rate() {
        let metrics = OperationMetrics {
            total_requests: 100,
            successful_requests: 85,
            failed_requests: 15,
            ..Default::default()
        };

        assert_eq!(metrics.success_rate(), 85.0);
        assert_eq!(metrics.error_rate(), 15.0);
    }

    #[test]
    fn test_cache_metrics_hit_rate() {
        let metrics = CacheMetrics {
            hits: 80,
            misses: 20,
            ..Default::default()
        };

        assert_eq!(metrics.hit_rate(), 80.0);
        assert_eq!(metrics.miss_rate(), 20.0);
    }

    #[test]
    fn test_connection_pool_utilization() {
        let metrics = ConnectionPoolMetrics {
            active_connections: 8,
            idle_connections: 2,
            total_connections: 10,
            ..Default::default()
        };

        assert_eq!(metrics.utilization(), 80.0);
    }

    #[tokio::test]
    async fn test_operation_timer() {
        let collector = get_global_metrics();
        let timer = OperationTimer::start("test_operation".to_string(), collector.clone());

        tokio::time::sleep(Duration::from_millis(10)).await;
        timer.finish_success();

        let snapshot = collector.get_snapshot().await;
        assert!(snapshot.operations.contains_key("test_operation"));
    }
}
