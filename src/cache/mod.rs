use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use crate::api::ApiType;
use crate::error::Result;

pub mod key;
pub mod storage;

use storage::SqliteStorage;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size in bytes (default: 100MB)
    pub max_size: u64,
    /// Default TTL for cache entries (default: 24 hours)
    pub default_ttl: Duration,
    /// Database file path
    pub db_path: PathBuf,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 100 * 1024 * 1024, // 100MB
            default_ttl: Duration::hours(24),
            db_path: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("warp")
                .join("cache.db"),
        }
    }
}

/// Cache entry metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub api_type: ApiType,
    pub size: u64,
}

impl CacheEntry {
    /// Check if the cache entry is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the cache entry is still valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Main cache store for API responses
#[derive(Debug)]
pub struct CacheStore {
    storage: Arc<RwLock<SqliteStorage>>,
    config: CacheConfig,
}

impl CacheStore {
    /// Create a new cache store
    pub async fn new(config: CacheConfig) -> Result<Self> {
        // Ensure cache directory exists
        if let Some(parent) = config.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let storage = SqliteStorage::new(&config.db_path).await?;
        
        Ok(Self {
            storage: Arc::new(RwLock::new(storage)),
            config,
        })
    }

    /// Get cached data by key
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let storage = self.storage.read().await;
        
        match storage.get(key).await? {
            Some(entry) => {
                if entry.is_valid() {
                    Ok(Some(entry.data))
                } else {
                    // Entry is expired, remove it
                    drop(storage);
                    let mut storage = self.storage.write().await;
                    storage.remove(key).await?;
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Store data in cache
    pub async fn put(
        &self,
        key: &str,
        data: Vec<u8>,
        api_type: ApiType,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let now = Utc::now();
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        let expires_at = now + ttl;
        
        let size = data.len() as u64;
        let entry = CacheEntry {
            key: key.to_string(),
            data,
            created_at: now,
            expires_at,
            api_type,
            size,
        };

        let mut storage = self.storage.write().await;
        
        // Check if we need to evict entries to make space
        let current_size = storage.get_total_size().await?;
        if current_size + entry.size > self.config.max_size {
            self.evict_lru(&mut storage, entry.size).await?;
        }

        storage.put(entry).await?;
        Ok(())
    }

    /// Remove entry from cache
    pub async fn remove(&self, key: &str) -> Result<bool> {
        let mut storage = self.storage.write().await;
        storage.remove(key).await
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear().await
    }

    /// Clear cache entries for specific API type
    pub async fn clear_api(&self, api_type: ApiType) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear_by_api_type(api_type).await
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats> {
        let storage = self.storage.read().await;
        
        let total_entries = storage.count_entries().await?;
        let total_size = storage.get_total_size().await?;
        let expired_entries = storage.count_expired_entries().await?;

        Ok(CacheStats {
            total_entries,
            expired_entries,
            total_size,
            max_size: self.config.max_size,
        })
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let mut storage = self.storage.write().await;
        storage.cleanup_expired().await
    }

    /// Evict least recently used entries to make space
    async fn evict_lru(&self, storage: &mut SqliteStorage, needed_space: u64) -> Result<()> {
        let current_size = storage.get_total_size().await?;
        let target_size = self.config.max_size - needed_space;
        
        if current_size <= target_size {
            return Ok(());
        }

        let space_to_free = current_size - target_size;
        storage.evict_lru(space_to_free).await?;
        
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: u64,
    pub expired_entries: u64,
    pub total_size: u64,
    pub max_size: u64,
}

impl CacheStats {
    pub fn utilization_percent(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.total_size as f64 / self.max_size as f64) * 100.0
        }
    }

    pub fn expired_percent(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            (self.expired_entries as f64 / self.total_entries as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_cache() -> (CacheStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            max_size: 1024, // 1KB for testing
            default_ttl: Duration::minutes(5),
            db_path: temp_dir.path().join("test_cache.db"),
        };
        
        let cache = CacheStore::new(config).await.unwrap();
        (cache, temp_dir)
    }

    #[tokio::test]
    async fn test_cache_put_and_get() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let key = "test_key";
        let data = b"test_data".to_vec();
        
        cache.put(key, data.clone(), ApiType::Nlic, None).await.unwrap();
        
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let key = "test_key";
        let data = b"test_data".to_vec();
        
        // Set TTL to 1 millisecond
        cache.put(key, data, ApiType::Nlic, Some(Duration::milliseconds(1))).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            max_size: 1024,
            default_ttl: Duration::minutes(5),
            db_path: temp_dir.path().join("test_permissions.db"),
        };
        
        let _cache = CacheStore::new(config.clone()).await.unwrap();
        
        // On Unix systems, verify file permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            let metadata = std::fs::metadata(&config.db_path).unwrap();
            let mode = metadata.permissions().mode() & 0o777;
            
            // Database file should have 0600 permissions (owner read/write only)
            assert_eq!(mode, 0o600, "Cache database should have 0600 permissions, got {:o}", mode);
        }
    }
}