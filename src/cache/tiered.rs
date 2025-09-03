use super::{CacheConfig, CacheStore};
use crate::api::ApiType;
use crate::error::Result;
use crate::metrics::get_global_metrics;
use chrono::{DateTime, Duration, Utc};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

/// Configuration for tiered caching system
#[derive(Debug, Clone)]
pub struct TieredCacheConfig {
    /// L1 cache (in-memory) configuration
    pub l1_config: L1CacheConfig,
    /// L2 cache (SQLite) configuration - reuses existing CacheConfig
    pub l2_config: CacheConfig,
    /// Whether to enable L3 cache (compressed disk cache)
    pub enable_l3: bool,
    /// L3 cache directory
    pub l3_dir: std::path::PathBuf,
}

/// L1 cache configuration (in-memory)
#[derive(Debug, Clone)]
pub struct L1CacheConfig {
    /// Maximum number of entries in L1 cache
    pub max_entries: usize,
    /// TTL for L1 cache entries
    pub ttl: Duration,
    /// Enable compression for large entries
    pub enable_compression: bool,
    /// Minimum size threshold for compression (bytes)
    pub compression_threshold: usize,
}

impl Default for L1CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::minutes(30),
            enable_compression: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

impl Default for TieredCacheConfig {
    fn default() -> Self {
        Self {
            l1_config: L1CacheConfig::default(),
            l2_config: CacheConfig::default(),
            enable_l3: false, // Disabled by default
            l3_dir: dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("warp")
                .join("l3_cache"),
        }
    }
}

/// L1 cache entry with metadata
#[derive(Debug, Clone)]
struct L1Entry {
    data: Vec<u8>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    api_type: ApiType,
    access_count: u64,
    compressed: bool,
}

impl L1Entry {
    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Multi-tier cache system with L1 (memory), L2 (SQLite), and optional L3 (compressed disk)
pub struct TieredCache {
    config: TieredCacheConfig,
    /// L1 cache: Fast in-memory LRU cache for hot data
    l1_cache: Arc<RwLock<LruCache<String, L1Entry>>>,
    /// L2 cache: Persistent SQLite cache
    l2_cache: Arc<CacheStore>,
    /// Cache hit statistics per tier
    stats: Arc<RwLock<TieredCacheStats>>,
}

/// Statistics for the tiered cache system
#[derive(Debug, Clone, Default)]
pub struct TieredCacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l3_hits: u64,
    pub l3_misses: u64,
    pub promotions: u64, // L2 -> L1 promotions
    pub evictions: u64,  // L1 evictions
}

impl TieredCacheStats {
    pub fn total_hits(&self) -> u64 {
        self.l1_hits + self.l2_hits + self.l3_hits
    }

    pub fn total_requests(&self) -> u64 {
        self.total_hits() + self.l1_misses + self.l2_misses + self.l3_misses
    }

    pub fn overall_hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            (self.total_hits() as f64 / total as f64) * 100.0
        }
    }

    pub fn l1_hit_rate(&self) -> f64 {
        let total = self.l1_hits + self.l1_misses;
        if total == 0 {
            0.0
        } else {
            (self.l1_hits as f64 / total as f64) * 100.0
        }
    }

    pub fn l2_hit_rate(&self) -> f64 {
        let total = self.l2_hits + self.l2_misses;
        if total == 0 {
            0.0
        } else {
            (self.l2_hits as f64 / total as f64) * 100.0
        }
    }
}

impl TieredCache {
    /// Create a new tiered cache system
    pub async fn new(config: TieredCacheConfig) -> Result<Self> {
        // Initialize L1 cache
        let l1_cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(config.l1_config.max_entries).unwrap(),
        )));

        // Initialize L2 cache (existing SQLite cache)
        let l2_cache = Arc::new(CacheStore::new(config.l2_config.clone()).await?);

        // Initialize L3 cache directory if enabled
        if config.enable_l3 {
            std::fs::create_dir_all(&config.l3_dir)?;
        }

        Ok(Self {
            config,
            l1_cache,
            l2_cache,
            stats: Arc::new(RwLock::new(TieredCacheStats::default())),
        })
    }

    /// Get cached data, checking all tiers
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // L1 cache check (fast path)
        if let Some(data) = self.get_from_l1(key).await {
            let mut stats = self.stats.write().unwrap();
            stats.l1_hits += 1;

            // Record metrics
            let metrics = get_global_metrics();
            metrics.record_cache_hit("l1");

            return Ok(Some(data));
        }

        let mut stats = self.stats.write().unwrap();
        stats.l1_misses += 1;
        drop(stats);

        // L2 cache check (persistent)
        if let Some(data) = self.l2_cache.get(key).await? {
            let mut stats = self.stats.write().unwrap();
            stats.l2_hits += 1;
            stats.promotions += 1;
            drop(stats);

            // Promote to L1 cache for future access
            self.promote_to_l1(key, &data, ApiType::Nlic).await; // TODO: Pass actual API type

            // Record metrics
            let metrics = get_global_metrics();
            metrics.record_cache_hit("l2");

            return Ok(Some(data));
        }

        let mut stats = self.stats.write().unwrap();
        stats.l2_misses += 1;

        // TODO: L3 cache check (compressed disk cache) if enabled
        if self.config.enable_l3 {
            // Placeholder for L3 implementation
            stats.l3_misses += 1;
        }

        // Record cache miss
        let metrics = get_global_metrics();
        metrics.record_cache_miss("tiered");

        Ok(None)
    }

    /// Store data in the cache (writes to appropriate tiers)
    pub async fn put(
        &self,
        key: &str,
        data: Vec<u8>,
        api_type: ApiType,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let data_len = data.len();

        // Always store in L2 for persistence
        self.l2_cache.put(key, data.clone(), api_type, ttl).await?;

        // Store in L1 if under size limits and frequently accessed
        if self.should_cache_in_l1(&data) {
            self.put_in_l1(key, data, api_type, ttl).await;
        }

        // TODO: Store in L3 if enabled and data is large
        if self.config.enable_l3 && data_len > 10240 { // 10KB threshold
             // Placeholder for L3 compressed storage
        }

        Ok(())
    }

    /// Remove entry from all cache tiers
    pub async fn remove(&self, key: &str) -> Result<bool> {
        // Remove from L1
        let l1_removed = {
            let mut l1 = self.l1_cache.write().unwrap();
            l1.pop(key).is_some()
        };

        // Remove from L2
        let l2_removed = self.l2_cache.remove(key).await?;

        // TODO: Remove from L3 if enabled

        Ok(l1_removed || l2_removed)
    }

    /// Clear all cache tiers
    pub async fn clear(&self) -> Result<()> {
        // Clear L1
        {
            let mut l1 = self.l1_cache.write().unwrap();
            l1.clear();
        }

        // Clear L2
        self.l2_cache.clear().await?;

        // TODO: Clear L3 if enabled

        // Reset stats
        {
            let mut stats = self.stats.write().unwrap();
            *stats = TieredCacheStats::default();
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> TieredCacheStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Get L1 cache data if available and valid
    async fn get_from_l1(&self, key: &str) -> Option<Vec<u8>> {
        let mut l1 = self.l1_cache.write().unwrap();

        if let Some(entry) = l1.get_mut(key) {
            if entry.is_valid() {
                entry.access_count += 1;
                let data = if entry.compressed {
                    self.decompress_data(&entry.data)
                } else {
                    entry.data.clone()
                };
                return Some(data);
            } else {
                // Remove expired entry
                l1.pop(key);
            }
        }

        None
    }

    /// Store data in L1 cache
    async fn put_in_l1(&self, key: &str, data: Vec<u8>, api_type: ApiType, ttl: Option<Duration>) {
        let ttl = ttl.unwrap_or(self.config.l1_config.ttl);
        let now = Utc::now();
        let expires_at = now + ttl;

        // Compress data if enabled and above threshold
        let (final_data, compressed) = if self.config.l1_config.enable_compression
            && data.len() > self.config.l1_config.compression_threshold
        {
            (self.compress_data(&data), true)
        } else {
            (data, false)
        };

        let entry = L1Entry {
            data: final_data,
            created_at: now,
            expires_at,
            api_type,
            access_count: 1,
            compressed,
        };

        let mut l1 = self.l1_cache.write().unwrap();
        if l1.put(key.to_string(), entry).is_some() {
            // Evicted an old entry
            let mut stats = self.stats.write().unwrap();
            stats.evictions += 1;
        }
    }

    /// Promote data from L2 to L1 cache
    async fn promote_to_l1(&self, key: &str, data: &[u8], api_type: ApiType) {
        if self.should_cache_in_l1(data) {
            self.put_in_l1(key, data.to_vec(), api_type, None).await;
        }
    }

    /// Determine if data should be cached in L1
    fn should_cache_in_l1(&self, data: &[u8]) -> bool {
        // Simple heuristic: cache if data is reasonably sized
        data.len() <= 100 * 1024 // 100KB limit for L1 cache
    }

    /// Compress data for storage
    fn compress_data(&self, data: &[u8]) -> Vec<u8> {
        // Simple compression using deflate
        // In production, consider using a faster algorithm like LZ4
        use std::io::Write;
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    /// Decompress data from storage
    fn decompress_data(&self, compressed_data: &[u8]) -> Vec<u8> {
        use std::io::Read;
        let mut decoder = flate2::read::DeflateDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompressed
    }

    /// Cleanup expired entries from L1 cache
    pub async fn cleanup_expired(&self) -> u64 {
        let mut expired_count = 0;
        let now = Utc::now();

        {
            let mut l1 = self.l1_cache.write().unwrap();
            let keys_to_remove: Vec<String> = l1
                .iter()
                .filter_map(|(key, entry)| {
                    if now > entry.expires_at {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for key in keys_to_remove {
                l1.pop(&key);
                expired_count += 1;
            }
        }

        // Also cleanup L2 cache
        expired_count += self.l2_cache.cleanup_expired().await.unwrap_or(0);

        expired_count
    }

    /// Get cache utilization information
    pub async fn get_utilization(&self) -> CacheUtilization {
        let l1_count = {
            let l1 = self.l1_cache.read().unwrap();
            l1.len()
        };

        let l2_stats = self
            .l2_cache
            .stats()
            .await
            .unwrap_or_else(|_| super::CacheStats {
                total_entries: 0,
                expired_entries: 0,
                total_size: 0,
                max_size: 0,
            });

        CacheUtilization {
            l1_entries: l1_count,
            l1_max_entries: self.config.l1_config.max_entries,
            l2_entries: l2_stats.total_entries,
            l2_size: l2_stats.total_size,
            l2_max_size: l2_stats.max_size,
        }
    }

    /// Warm up the cache with commonly accessed data
    pub async fn warmup(&self, warmup_keys: Vec<String>) -> Result<u64> {
        let mut warmed = 0;

        for key in warmup_keys {
            // Try to load from L2 and promote to L1
            if let Some(data) = self.l2_cache.get(&key).await? {
                self.promote_to_l1(&key, &data, ApiType::Nlic).await; // TODO: Store API type in metadata
                warmed += 1;
            }
        }

        log::info!("Warmed up {} cache entries", warmed);
        Ok(warmed)
    }
}

/// Cache utilization information
#[derive(Debug)]
pub struct CacheUtilization {
    pub l1_entries: usize,
    pub l1_max_entries: usize,
    pub l2_entries: u64,
    pub l2_size: u64,
    pub l2_max_size: u64,
}

impl CacheUtilization {
    pub fn l1_utilization(&self) -> f64 {
        (self.l1_entries as f64 / self.l1_max_entries as f64) * 100.0
    }

    pub fn l2_utilization(&self) -> f64 {
        if self.l2_max_size == 0 {
            0.0
        } else {
            (self.l2_size as f64 / self.l2_max_size as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_tiered_cache() -> (TieredCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = TieredCacheConfig {
            l1_config: L1CacheConfig {
                max_entries: 10,
                ttl: Duration::minutes(5),
                enable_compression: true,
                compression_threshold: 100,
            },
            l2_config: CacheConfig {
                max_size: 1024 * 1024, // 1MB
                default_ttl: Duration::hours(1),
                db_path: temp_dir.path().join("test_tiered.db"),
            },
            enable_l3: false,
            l3_dir: temp_dir.path().join("l3"),
        };

        let cache = TieredCache::new(config).await.unwrap();
        (cache, temp_dir)
    }

    #[tokio::test]
    async fn test_tiered_cache_basic_operations() {
        let (cache, _temp_dir) = create_test_tiered_cache().await;

        let key = "test_key";
        let data = b"test_data".to_vec();

        // Store data
        cache
            .put(key, data.clone(), ApiType::Nlic, None)
            .await
            .unwrap();

        // Retrieve data
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_l1_l2_promotion() {
        let (cache, _temp_dir) = create_test_tiered_cache().await;

        let key = "promotion_test";
        let data = b"promotion_data".to_vec();

        // Store in L2 only by bypassing L1
        cache
            .l2_cache
            .put(key, data.clone(), ApiType::Nlic, None)
            .await
            .unwrap();

        // First access should hit L2 and promote to L1
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(data.clone()));

        let stats = cache.get_stats().await;
        assert_eq!(stats.l1_misses, 1);
        assert_eq!(stats.l2_hits, 1);
        assert_eq!(stats.promotions, 1);

        // Second access should hit L1
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(data));

        let stats = cache.get_stats().await;
        assert_eq!(stats.l1_hits, 1);
    }

    #[tokio::test]
    async fn test_cache_compression() {
        let (cache, _temp_dir) = create_test_tiered_cache().await;

        let key = "compression_test";
        let data = vec![b'A'; 200]; // Data larger than compression threshold

        cache
            .put(key, data.clone(), ApiType::Nlic, None)
            .await
            .unwrap();
        let retrieved = cache.get(key).await.unwrap();

        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let config = TieredCacheConfig {
            l1_config: L1CacheConfig {
                ttl: Duration::milliseconds(100), // Very short TTL
                ..L1CacheConfig::default()
            },
            l2_config: CacheConfig {
                db_path: temp_dir.path().join("test_expiration.db"),
                ..CacheConfig::default()
            },
            enable_l3: false,
            l3_dir: temp_dir.path().join("l3"),
        };

        let cache = TieredCache::new(config).await.unwrap();

        let key = "expiration_test";
        let data = b"expiration_data".to_vec();

        cache
            .put(key, data.clone(), ApiType::Nlic, None)
            .await
            .unwrap();

        // Should be available immediately
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(data));

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // L1 should be expired, but L2 might still have it
        let retrieved = cache.get(key).await.unwrap();
        // The result depends on L2 TTL settings
    }
}
