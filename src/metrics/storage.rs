use super::{MetricsSnapshot, MetricsWindow};
use crate::error::{Result, WarpError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persistent storage for metrics data and historical analysis
pub struct MetricsStorage {
    db_path: PathBuf,
}

/// Stored metrics entry for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMetricsEntry {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_duration_ms: u64,
    pub p95_duration_ms: u64,
    pub p99_duration_ms: u64,
}

/// Cache metrics entry for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCacheEntry {
    pub timestamp: DateTime<Utc>,
    pub api: String,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub storage_size: u64,
    pub entry_count: u64,
}

/// Historical metrics query result
#[derive(Debug, Clone)]
pub struct HistoricalMetrics {
    pub entries: Vec<StoredMetricsEntry>,
    pub cache_entries: Vec<StoredCacheEntry>,
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
}

impl MetricsStorage {
    /// Create a new metrics storage
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let storage = Self { db_path };
        storage.init_database().await?;
        Ok(storage)
    }

    /// Initialize the database schema
    async fn init_database(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| WarpError::Other(format!("Failed to open metrics database: {}", e)))?;

        // Set secure permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&self.db_path)
                .map_err(|e| {
                    WarpError::Other(format!("Failed to get database permissions: {}", e))
                })?
                .permissions();
            permissions.set_mode(0o600);
            std::fs::set_permissions(&self.db_path, permissions).map_err(|e| {
                WarpError::Other(format!("Failed to set database permissions: {}", e))
            })?;
        }

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS operation_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME NOT NULL,
                operation TEXT NOT NULL,
                total_requests INTEGER NOT NULL,
                successful_requests INTEGER NOT NULL,
                failed_requests INTEGER NOT NULL,
                avg_duration_ms INTEGER NOT NULL,
                p95_duration_ms INTEGER NOT NULL,
                p99_duration_ms INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS cache_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME NOT NULL,
                api TEXT NOT NULL,
                hits INTEGER NOT NULL,
                misses INTEGER NOT NULL,
                hit_rate REAL NOT NULL,
                storage_size INTEGER NOT NULL,
                entry_count INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS system_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME NOT NULL,
                memory_usage INTEGER NOT NULL,
                uptime_seconds INTEGER NOT NULL,
                active_connections INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_operation_timestamp ON operation_metrics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_cache_timestamp ON cache_metrics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_system_timestamp ON system_metrics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_operation_name ON operation_metrics(operation);
            CREATE INDEX IF NOT EXISTS idx_cache_api ON cache_metrics(api);
            "#,
        )
        .map_err(|e| WarpError::Other(format!("Failed to create database schema: {}", e)))?;

        Ok(())
    }

    /// Store a metrics snapshot
    pub async fn store_snapshot(&self, snapshot: &MetricsSnapshot) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| WarpError::Other(format!("Failed to open database: {}", e)))?;

        let timestamp = Utc::now();

        // Store operation metrics
        for (operation, metrics) in &snapshot.operations {
            conn.execute(
                r#"
                INSERT INTO operation_metrics
                (timestamp, operation, total_requests, successful_requests, failed_requests,
                 avg_duration_ms, p95_duration_ms, p99_duration_ms)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    timestamp,
                    operation,
                    metrics.total_requests,
                    metrics.successful_requests,
                    metrics.failed_requests,
                    metrics.avg_duration.as_millis() as u64,
                    metrics.p95_duration.as_millis() as u64,
                    metrics.p99_duration.as_millis() as u64,
                ],
            )
            .map_err(|e| WarpError::Other(format!("Failed to store operation metrics: {}", e)))?;
        }

        // Store cache metrics
        for (api, cache_metrics) in &snapshot.cache {
            conn.execute(
                r#"
                INSERT INTO cache_metrics
                (timestamp, api, hits, misses, hit_rate, storage_size, entry_count)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    timestamp,
                    api,
                    cache_metrics.hits,
                    cache_metrics.misses,
                    cache_metrics.hit_rate(),
                    cache_metrics.storage_size,
                    cache_metrics.entry_count,
                ],
            )
            .map_err(|e| WarpError::Other(format!("Failed to store cache metrics: {}", e)))?;
        }

        // Store system metrics
        let total_connections: u32 = snapshot
            .connection_pools
            .values()
            .map(|p| p.total_connections)
            .sum();

        conn.execute(
            r#"
            INSERT INTO system_metrics
            (timestamp, memory_usage, uptime_seconds, active_connections)
            VALUES (?, ?, ?, ?)
            "#,
            params![
                timestamp,
                snapshot.memory_usage,
                snapshot.uptime.as_secs(),
                total_connections,
            ],
        )
        .map_err(|e| WarpError::Other(format!("Failed to store system metrics: {}", e)))?;

        Ok(())
    }

    /// Get historical metrics for a specific time window
    pub async fn get_historical_metrics(&self, window: MetricsWindow) -> Result<HistoricalMetrics> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| WarpError::Other(format!("Failed to open database: {}", e)))?;

        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::from_std(window.duration()).unwrap();

        // Get operation metrics
        let mut stmt = conn
            .prepare(
                r#"
            SELECT timestamp, operation, total_requests, successful_requests, failed_requests,
                   avg_duration_ms, p95_duration_ms, p99_duration_ms
            FROM operation_metrics
            WHERE timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
            )
            .map_err(|e| WarpError::Other(format!("Failed to prepare query: {}", e)))?;

        let entries: Result<Vec<StoredMetricsEntry>> = stmt
            .query_map(params![start_time, end_time], |row| {
                Ok(StoredMetricsEntry {
                    timestamp: row.get("timestamp")?,
                    operation: row.get("operation")?,
                    total_requests: row.get("total_requests")?,
                    successful_requests: row.get("successful_requests")?,
                    failed_requests: row.get("failed_requests")?,
                    avg_duration_ms: row.get("avg_duration_ms")?,
                    p95_duration_ms: row.get("p95_duration_ms")?,
                    p99_duration_ms: row.get("p99_duration_ms")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| WarpError::Other(format!("Failed to parse operation metrics: {}", e)));

        let entries = entries?;

        // Get cache metrics
        let mut cache_stmt = conn
            .prepare(
                r#"
            SELECT timestamp, api, hits, misses, hit_rate, storage_size, entry_count
            FROM cache_metrics
            WHERE timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
            )
            .map_err(|e| WarpError::Other(format!("Failed to prepare cache query: {}", e)))?;

        let cache_entries: Result<Vec<StoredCacheEntry>> = cache_stmt
            .query_map(params![start_time, end_time], |row| {
                Ok(StoredCacheEntry {
                    timestamp: row.get("timestamp")?,
                    api: row.get("api")?,
                    hits: row.get("hits")?,
                    misses: row.get("misses")?,
                    hit_rate: row.get("hit_rate")?,
                    storage_size: row.get("storage_size")?,
                    entry_count: row.get("entry_count")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| WarpError::Other(format!("Failed to parse cache metrics: {}", e)));

        let cache_entries = cache_entries?;

        Ok(HistoricalMetrics {
            entries,
            cache_entries,
            time_range: (start_time, end_time),
        })
    }

    /// Get performance trend for a specific operation
    pub async fn get_operation_trend(
        &self,
        operation: &str,
        window: MetricsWindow,
    ) -> Result<Vec<StoredMetricsEntry>> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| WarpError::Other(format!("Failed to open database: {}", e)))?;

        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::from_std(window.duration()).unwrap();

        let mut stmt = conn
            .prepare(
                r#"
            SELECT timestamp, operation, total_requests, successful_requests, failed_requests,
                   avg_duration_ms, p95_duration_ms, p99_duration_ms
            FROM operation_metrics
            WHERE operation = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp ASC
            "#,
            )
            .map_err(|e| WarpError::Other(format!("Failed to prepare trend query: {}", e)))?;

        let entries: Result<Vec<StoredMetricsEntry>> = stmt
            .query_map(params![operation, start_time, end_time], |row| {
                Ok(StoredMetricsEntry {
                    timestamp: row.get("timestamp")?,
                    operation: row.get("operation")?,
                    total_requests: row.get("total_requests")?,
                    successful_requests: row.get("successful_requests")?,
                    failed_requests: row.get("failed_requests")?,
                    avg_duration_ms: row.get("avg_duration_ms")?,
                    p95_duration_ms: row.get("p95_duration_ms")?,
                    p99_duration_ms: row.get("p99_duration_ms")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| WarpError::Other(format!("Failed to parse trend data: {}", e)));

        entries
    }

    /// Clean up old metrics data beyond retention period
    pub async fn cleanup_old_data(&self, retention_days: u32) -> Result<u64> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| WarpError::Other(format!("Failed to open database: {}", e)))?;

        let cutoff_time = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut total_deleted = 0;

        // Clean operation metrics
        let deleted = conn
            .execute(
                "DELETE FROM operation_metrics WHERE timestamp < ?",
                params![cutoff_time],
            )
            .map_err(|e| WarpError::Other(format!("Failed to clean operation metrics: {}", e)))?;
        total_deleted += deleted;

        // Clean cache metrics
        let deleted = conn
            .execute(
                "DELETE FROM cache_metrics WHERE timestamp < ?",
                params![cutoff_time],
            )
            .map_err(|e| WarpError::Other(format!("Failed to clean cache metrics: {}", e)))?;
        total_deleted += deleted;

        // Clean system metrics
        let deleted = conn
            .execute(
                "DELETE FROM system_metrics WHERE timestamp < ?",
                params![cutoff_time],
            )
            .map_err(|e| WarpError::Other(format!("Failed to clean system metrics: {}", e)))?;
        total_deleted += deleted;

        // Vacuum database to reclaim space
        conn.execute("VACUUM", [])
            .map_err(|e| WarpError::Other(format!("Failed to vacuum database: {}", e)))?;

        Ok(total_deleted as u64)
    }

    /// Get database statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| WarpError::Other(format!("Failed to open database: {}", e)))?;

        let file_size = std::fs::metadata(&self.db_path)
            .map_err(|e| WarpError::Other(format!("Failed to get file size: {}", e)))?
            .len();

        // Count total entries
        let operation_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM operation_metrics", [], |row| {
                row.get(0)
            })
            .map_err(|e| WarpError::Other(format!("Failed to count operations: {}", e)))?;

        let cache_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM cache_metrics", [], |row| row.get(0))
            .map_err(|e| WarpError::Other(format!("Failed to count cache entries: {}", e)))?;

        let system_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM system_metrics", [], |row| row.get(0))
            .map_err(|e| WarpError::Other(format!("Failed to count system entries: {}", e)))?;

        // Get date range
        let oldest_entry: Option<DateTime<Utc>> = conn
            .query_row("SELECT MIN(timestamp) FROM operation_metrics", [], |row| {
                row.get(0)
            })
            .ok();

        Ok(StorageStats {
            file_size,
            operation_entries: operation_count,
            cache_entries: cache_count,
            system_entries: system_count,
            oldest_entry,
        })
    }
}

/// Storage statistics
#[derive(Debug)]
pub struct StorageStats {
    pub file_size: u64,
    pub operation_entries: u64,
    pub cache_entries: u64,
    pub system_entries: u64,
    pub oldest_entry: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{CacheMetrics, OperationMetrics};
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    async fn create_test_storage() -> (MetricsStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_metrics.db");
        let storage = MetricsStorage::new(db_path).await.unwrap();
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_store_and_retrieve_metrics() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Create test snapshot
        let mut operations = HashMap::new();
        operations.insert(
            "test_op".to_string(),
            OperationMetrics {
                total_requests: 100,
                successful_requests: 95,
                failed_requests: 5,
                avg_duration: Duration::from_millis(150),
                p95_duration: Duration::from_millis(300),
                p99_duration: Duration::from_millis(500),
                ..Default::default()
            },
        );

        let mut cache = HashMap::new();
        cache.insert(
            "test_api".to_string(),
            CacheMetrics {
                hits: 80,
                misses: 20,
                ..Default::default()
            },
        );

        let snapshot = MetricsSnapshot {
            timestamp: Instant::now(),
            operations,
            cache,
            connection_pools: HashMap::new(),
            memory_usage: 1024 * 1024,
            uptime: Duration::from_secs(3600),
        };

        // Store snapshot
        storage.store_snapshot(&snapshot).await.unwrap();

        // Retrieve historical data
        let historical = storage
            .get_historical_metrics(MetricsWindow::LastHour)
            .await
            .unwrap();

        assert!(!historical.entries.is_empty());
        assert!(!historical.cache_entries.is_empty());

        let entry = &historical.entries[0];
        assert_eq!(entry.operation, "test_op");
        assert_eq!(entry.total_requests, 100);
        assert_eq!(entry.successful_requests, 95);
    }

    #[tokio::test]
    async fn test_cleanup_old_data() {
        let (storage, _temp_dir) = create_test_storage().await;

        // Store some test data
        let snapshot = MetricsSnapshot {
            timestamp: Instant::now(),
            operations: HashMap::new(),
            cache: HashMap::new(),
            connection_pools: HashMap::new(),
            memory_usage: 1024,
            uptime: Duration::from_secs(60),
        };

        storage.store_snapshot(&snapshot).await.unwrap();

        // Clean up data older than 0 days (should delete everything)
        let deleted = storage.cleanup_old_data(0).await.unwrap();
        assert!(deleted > 0);
    }
}
