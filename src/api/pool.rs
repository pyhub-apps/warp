use crate::metrics::{get_global_metrics, ConnectionPoolMetrics};
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Adaptive connection pool configuration
#[derive(Debug, Clone)]
pub struct AdaptivePoolConfig {
    /// Initial pool size
    pub initial_size: u32,
    /// Minimum pool size
    pub min_size: u32,
    /// Maximum pool size
    pub max_size: u32,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Idle timeout for connections
    pub idle_timeout: Duration,
    /// Pool resize evaluation interval
    pub resize_interval: Duration,
    /// Target utilization for pool sizing (0.0-1.0)
    pub target_utilization: f64,
    /// Utilization threshold for scaling up
    pub scale_up_threshold: f64,
    /// Utilization threshold for scaling down
    pub scale_down_threshold: f64,
    /// Disable background task for testing/benchmarking environments
    pub disable_background_task: bool,
}

impl Default for AdaptivePoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 10,
            min_size: 2,
            max_size: 50,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(60),
            resize_interval: Duration::from_secs(30),
            target_utilization: 0.7,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            disable_background_task: false,
        }
    }
}

/// Connection pool statistics for monitoring
#[derive(Debug, Clone)]
struct PoolStats {
    active_connections: u32,
    #[allow(dead_code)] // Used for monitoring/debugging purposes
    idle_connections: u32,
    total_acquires: u64,
    successful_acquires: u64,
    timeouts: u64,
    last_resize: Instant,
    utilization_history: Vec<(Instant, f64)>,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            active_connections: 0,
            idle_connections: 0,
            total_acquires: 0,
            successful_acquires: 0,
            timeouts: 0,
            last_resize: Instant::now(),
            utilization_history: Vec::new(),
        }
    }
}

/// Adaptive HTTP connection pool that scales based on load
pub struct AdaptiveConnectionPool {
    config: AdaptivePoolConfig,
    client: Client,
    /// Semaphore to limit concurrent connections
    connection_semaphore: Arc<Semaphore>,
    /// Pool statistics for adaptive sizing
    stats: Arc<RwLock<PoolStats>>,
    /// Pool name for metrics
    pool_name: String,
    /// Background task handle for pool management
    _background_task: Option<tokio::task::JoinHandle<()>>,
}

impl AdaptiveConnectionPool {
    /// Create a new adaptive connection pool
    pub fn new(pool_name: String, config: AdaptivePoolConfig, user_agent: &str) -> Self {
        // Create optimized HTTP client
        let client = ClientBuilder::new()
            .pool_max_idle_per_host(config.max_size as usize)
            .pool_idle_timeout(config.idle_timeout)
            .timeout(config.connection_timeout)
            .user_agent(user_agent)
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .http2_prior_knowledge()
            .use_rustls_tls()
            .build()
            .expect("Failed to create HTTP client");

        let connection_semaphore = Arc::new(Semaphore::new(config.initial_size as usize));
        let stats = Arc::new(RwLock::new(PoolStats::default()));

        // Conditionally start background pool management task
        let background_task = if !config.disable_background_task {
            Some(Self::start_background_task(
                pool_name.clone(),
                config.clone(),
                Arc::clone(&stats),
                Arc::clone(&connection_semaphore),
            ))
        } else {
            None
        };

        Self {
            config,
            client,
            connection_semaphore,
            stats,
            pool_name,
            _background_task: background_task,
        }
    }

    /// Acquire a connection permit (for tracking usage)
    pub async fn acquire_connection(&self) -> Result<ConnectionPermit<'_>, PoolError> {
        let start_time = Instant::now();

        // Record acquisition attempt
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_acquires += 1;
        }

        // Try to acquire connection permit with timeout
        let permit = match tokio::time::timeout(
            self.config.connection_timeout,
            self.connection_semaphore.acquire(),
        )
        .await
        {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                // Semaphore closed (should not happen in normal operation)
                self.record_failure(true);
                return Err(PoolError::PoolClosed);
            }
            Err(_) => {
                // Timeout
                self.record_failure(true);
                return Err(PoolError::AcquisitionTimeout);
            }
        };

        // Record successful acquisition
        {
            let mut stats = self.stats.write().unwrap();
            stats.successful_acquires += 1;
            stats.active_connections += 1;
        }

        // Report metrics to global collector
        let metrics = get_global_metrics();
        metrics.record_connection_acquire(&self.pool_name, true, false);

        Ok(ConnectionPermit {
            _permit: permit,
            pool_name: self.pool_name.clone(),
            stats: Arc::clone(&self.stats),
            acquired_at: start_time,
        })
    }

    /// Get the HTTP client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get current pool statistics
    pub fn get_stats(&self) -> ConnectionPoolMetrics {
        let stats = self.stats.read().unwrap();
        let current_permits = self.connection_semaphore.available_permits() as u32;
        let total_size =
            self.connection_semaphore.available_permits() as u32 + stats.active_connections;

        ConnectionPoolMetrics {
            active_connections: stats.active_connections,
            idle_connections: current_permits,
            total_connections: total_size,
            connection_acquires: stats.total_acquires,
            connection_timeouts: stats.timeouts,
            connection_errors: stats.total_acquires - stats.successful_acquires - stats.timeouts,
        }
    }

    fn record_failure(&self, timeout: bool) {
        let mut stats = self.stats.write().unwrap();
        if timeout {
            stats.timeouts += 1;
        }

        // Report to global metrics
        let metrics = get_global_metrics();
        metrics.record_connection_acquire(&self.pool_name, false, timeout);
    }

    /// Start background task for pool management
    fn start_background_task(
        pool_name: String,
        config: AdaptivePoolConfig,
        stats: Arc<RwLock<PoolStats>>,
        semaphore: Arc<Semaphore>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut resize_interval = tokio::time::interval(config.resize_interval);
            resize_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                resize_interval.tick().await;

                // Calculate current utilization and decide on resize
                let should_resize = {
                    let mut stats = stats.write().unwrap();
                    let current_size =
                        semaphore.available_permits() + stats.active_connections as usize;
                    let utilization = if current_size > 0 {
                        stats.active_connections as f64 / current_size as f64
                    } else {
                        0.0
                    };

                    // Add to utilization history
                    stats
                        .utilization_history
                        .push((Instant::now(), utilization));

                    // Keep only recent history (last 10 minutes)
                    let cutoff = Instant::now() - Duration::from_secs(600);
                    stats.utilization_history.retain(|(time, _)| *time > cutoff);

                    // Calculate average utilization over recent history
                    let avg_utilization = if stats.utilization_history.is_empty() {
                        utilization
                    } else {
                        stats
                            .utilization_history
                            .iter()
                            .map(|(_, u)| u)
                            .sum::<f64>()
                            / stats.utilization_history.len() as f64
                    };

                    // Decide on resize action

                    if avg_utilization > config.scale_up_threshold
                        && current_size < config.max_size as usize
                    {
                        Some(ResizeAction::ScaleUp)
                    } else if avg_utilization < config.scale_down_threshold
                        && current_size > config.min_size as usize
                    {
                        Some(ResizeAction::ScaleDown)
                    } else {
                        None
                    }
                };

                // Execute resize if needed
                if let Some(action) = should_resize {
                    match action {
                        ResizeAction::ScaleUp => {
                            semaphore.add_permits(1);
                            log::info!("Scaled up connection pool '{}' by 1", pool_name);
                        }
                        ResizeAction::ScaleDown => {
                            // Try to acquire and forget a permit to reduce pool size
                            if let Ok(permit) = semaphore.try_acquire() {
                                permit.forget();
                                log::info!("Scaled down connection pool '{}' by 1", pool_name);
                            }
                        }
                    }

                    let mut stats = stats.write().unwrap();
                    stats.last_resize = Instant::now();
                }

                // Update global metrics
                let current_stats = {
                    let stats = stats.read().unwrap();
                    let available = semaphore.available_permits() as u32;
                    (
                        stats.active_connections,
                        available,
                        stats.active_connections + available,
                    )
                };

                let metrics = get_global_metrics();
                metrics.update_connection_pool(
                    &pool_name,
                    current_stats.0, // active
                    current_stats.1, // idle
                    current_stats.2, // total
                );
            }
        })
    }
}

#[derive(Debug)]
enum ResizeAction {
    ScaleUp,
    ScaleDown,
}

/// Connection permit that tracks usage
pub struct ConnectionPermit<'a> {
    _permit: tokio::sync::SemaphorePermit<'a>,
    pool_name: String,
    stats: Arc<RwLock<PoolStats>>,
    acquired_at: Instant,
}

impl Drop for ConnectionPermit<'_> {
    fn drop(&mut self) {
        // Update stats when connection is released
        {
            let mut stats = self.stats.write().unwrap();
            stats.active_connections = stats.active_connections.saturating_sub(1);
        }

        // Record connection usage duration for metrics
        let duration = self.acquired_at.elapsed();
        log::debug!(
            "Released connection from pool '{}' after {:?}",
            self.pool_name,
            duration
        );
    }
}

/// Pool-specific errors
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Connection acquisition timed out")]
    AcquisitionTimeout,
    #[error("Connection pool is closed")]
    PoolClosed,
}

/// Global registry of connection pools
pub struct PoolRegistry {
    pools: RwLock<HashMap<String, Arc<AdaptiveConnectionPool>>>,
}

impl PoolRegistry {
    fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a connection pool
    pub fn get_or_create_pool(
        &self,
        pool_name: &str,
        config: AdaptivePoolConfig,
        user_agent: &str,
    ) -> Arc<AdaptiveConnectionPool> {
        // Try to get existing pool
        {
            let pools = self.pools.read().unwrap();
            if let Some(pool) = pools.get(pool_name) {
                return Arc::clone(pool);
            }
        }

        // Create new pool
        let mut pools = self.pools.write().unwrap();
        // Double-check in case another thread created it
        if let Some(pool) = pools.get(pool_name) {
            return Arc::clone(pool);
        }

        let pool = Arc::new(AdaptiveConnectionPool::new(
            pool_name.to_string(),
            config,
            user_agent,
        ));
        pools.insert(pool_name.to_string(), Arc::clone(&pool));
        pool
    }

    /// Get all pools for monitoring
    pub fn get_all_pools(&self) -> Vec<(String, Arc<AdaptiveConnectionPool>)> {
        let pools = self.pools.read().unwrap();
        pools
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect()
    }
}

/// Global pool registry
static POOL_REGISTRY: Lazy<PoolRegistry> = Lazy::new(PoolRegistry::new);

/// Get the global pool registry
pub fn get_pool_registry() -> &'static PoolRegistry {
    &POOL_REGISTRY
}

/// Create an optimized HTTP client with adaptive connection pooling
pub fn create_adaptive_client(pool_name: &str, timeout_secs: u64, user_agent: &str) -> Client {
    let config = AdaptivePoolConfig {
        connection_timeout: Duration::from_secs(timeout_secs),
        ..AdaptivePoolConfig::default()
    };

    let registry = get_pool_registry();
    let pool = registry.get_or_create_pool(pool_name, config, user_agent);
    pool.client().clone()
}

/// Create an adaptive HTTP client with background tasks disabled (for benchmarks/tests)
pub fn create_adaptive_client_for_benchmarks(
    pool_name: &str,
    timeout_secs: u64,
    user_agent: &str,
) -> Client {
    let config = AdaptivePoolConfig {
        connection_timeout: Duration::from_secs(timeout_secs),
        disable_background_task: true,
        ..AdaptivePoolConfig::default()
    };

    let registry = get_pool_registry();
    let pool = registry.get_or_create_pool(pool_name, config, user_agent);
    pool.client().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_pool_creation() {
        let config = AdaptivePoolConfig::default();
        let pool =
            AdaptiveConnectionPool::new("test_pool".to_string(), config.clone(), "test-agent/1.0");

        assert_eq!(pool.pool_name, "test_pool");

        // Test connection acquisition
        let permit = pool.acquire_connection().await.unwrap();
        let stats = pool.get_stats();
        assert_eq!(stats.active_connections, 1);

        drop(permit);
        // Give some time for the drop to update stats
        tokio::time::sleep(Duration::from_millis(10)).await;

        let stats = pool.get_stats();
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_pool_registry() {
        let registry = get_pool_registry();
        let config = AdaptivePoolConfig::default();

        let pool1 = registry.get_or_create_pool("test_registry", config.clone(), "test/1.0");
        let pool2 = registry.get_or_create_pool("test_registry", config.clone(), "test/1.0");

        // Should return the same pool instance
        assert!(Arc::ptr_eq(&pool1, &pool2));
    }

    #[tokio::test]
    async fn test_concurrent_acquisitions() {
        let config = AdaptivePoolConfig {
            initial_size: 2,
            max_size: 2,
            ..AdaptivePoolConfig::default()
        };

        let pool = Arc::new(AdaptiveConnectionPool::new(
            "concurrent_test".to_string(),
            config,
            "test-agent/1.0",
        ));

        let pool1 = Arc::clone(&pool);
        let pool2 = Arc::clone(&pool);
        let pool3 = Arc::clone(&pool);

        // Acquire all available connections
        let permit1 = pool1.acquire_connection().await.unwrap();
        let permit2 = pool2.acquire_connection().await.unwrap();

        // Third acquisition should timeout
        let start = Instant::now();
        let result = pool3.acquire_connection().await;
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(elapsed >= Duration::from_secs(30)); // Should timeout after configured duration

        drop(permit1);
        drop(permit2);
    }
}
