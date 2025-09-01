use super::CacheEntry;
use crate::api::ApiType;
use crate::error::{Result, WarpError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

/// SQLite-based cache storage implementation
#[derive(Debug)]
pub struct SqliteStorage {
    db_path: PathBuf,
}

impl SqliteStorage {
    /// Create a new SQLite storage instance
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path_buf = db_path.as_ref().to_path_buf();
        let is_new_file = !db_path_buf.exists();

        let storage = Self {
            db_path: db_path_buf.clone(),
        };

        // Initialize database in a blocking context
        let result = tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path_buf)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            // Set secure permissions on newly created database file
            if is_new_file {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = std::fs::Permissions::from_mode(0o600);
                    std::fs::set_permissions(&db_path_buf, permissions).map_err(|e| {
                        WarpError::Other(format!("Failed to set cache database permissions: {}", e))
                    })?;
                }
            }

            // Initialize schema
            Self::initialize_schema(&conn)?;
            Ok(())
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn database initialization: {}", e)))?;

        result?;

        Ok(storage)
    }

    /// Initialize the database schema (internal method)
    fn initialize_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS cache_entries (
                key TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                api_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                last_accessed TEXT NOT NULL
            )
            "#,
            [],
        )
        .map_err(|e| WarpError::Other(format!("Failed to initialize cache schema: {}", e)))?;

        // Create indices for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_expires_at ON cache_entries(expires_at)",
            [],
        )
        .map_err(|e| WarpError::Other(format!("Failed to create expires_at index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_api_type ON cache_entries(api_type)",
            [],
        )
        .map_err(|e| WarpError::Other(format!("Failed to create api_type index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_last_accessed ON cache_entries(last_accessed)",
            [],
        )
        .map_err(|e| WarpError::Other(format!("Failed to create last_accessed index: {}", e)))?;

        Ok(())
    }

    /// Get a cache entry by key
    pub async fn get(&self, key: &str) -> Result<Option<CacheEntry>> {
        let db_path = self.db_path.clone();
        let key_owned = key.to_string();

        tokio::task::spawn_blocking(move || -> Result<Option<CacheEntry>> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let now = Utc::now().to_rfc3339();

            // Update last_accessed timestamp
            let _updated = conn
                .execute(
                    "UPDATE cache_entries SET last_accessed = ?1 WHERE key = ?2",
                    params![now, &key_owned],
                )
                .map_err(|e| WarpError::Other(format!("Failed to update last_accessed: {}", e)))?;

            let mut stmt = conn
                .prepare(
                    r#"
                SELECT key, data, created_at, expires_at, api_type, size
                FROM cache_entries
                WHERE key = ?1
                "#,
                )
                .map_err(|e| WarpError::Other(format!("Failed to prepare get statement: {}", e)))?;

            let entry = stmt
                .query_row(params![&key_owned], |row| {
                    let api_type_str: String = row.get(4)?;
                    let api_type = api_type_str.parse::<ApiType>().map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            4,
                            "api_type".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?;

                    Ok(CacheEntry {
                        key: row.get(0)?,
                        data: row.get(1)?,
                        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                            .map_err(|_| {
                                rusqlite::Error::InvalidColumnType(
                                    2,
                                    "created_at".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            })?
                            .with_timezone(&Utc),
                        expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                            .map_err(|_| {
                                rusqlite::Error::InvalidColumnType(
                                    3,
                                    "expires_at".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            })?
                            .with_timezone(&Utc),
                        api_type,
                        size: row.get::<_, i64>(5)? as u64,
                    })
                })
                .optional()
                .map_err(|e| WarpError::Other(format!("Failed to get cache entry: {}", e)))?;

            Ok(entry)
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn get operation: {}", e)))?
    }

    /// Store a cache entry
    pub async fn put(&mut self, entry: CacheEntry) -> Result<()> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let now = Utc::now().to_rfc3339();

            conn.execute(
                r#"
                INSERT OR REPLACE INTO cache_entries
                (key, data, created_at, expires_at, api_type, size, last_accessed)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    entry.key,
                    entry.data,
                    entry.created_at.to_rfc3339(),
                    entry.expires_at.to_rfc3339(),
                    entry.api_type.as_str(),
                    entry.size as i64,
                    now,
                ],
            )
            .map_err(|e| WarpError::Other(format!("Failed to store cache entry: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn put operation: {}", e)))?
    }

    /// Remove a cache entry by key
    pub async fn remove(&mut self, key: &str) -> Result<bool> {
        let db_path = self.db_path.clone();
        let key_owned = key.to_string();

        tokio::task::spawn_blocking(move || -> Result<bool> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let affected = conn
                .execute(
                    "DELETE FROM cache_entries WHERE key = ?1",
                    params![key_owned],
                )
                .map_err(|e| WarpError::Other(format!("Failed to remove cache entry: {}", e)))?;

            Ok(affected > 0)
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn remove operation: {}", e)))?
    }

    /// Clear all cache entries
    pub async fn clear(&mut self) -> Result<()> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            conn.execute("DELETE FROM cache_entries", [])
                .map_err(|e| WarpError::Other(format!("Failed to clear cache: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn clear operation: {}", e)))?
    }

    /// Clear cache entries by API type
    pub async fn clear_by_api_type(&mut self, api_type: ApiType) -> Result<()> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            conn.execute(
                "DELETE FROM cache_entries WHERE api_type = ?1",
                params![api_type.as_str()],
            )
            .map_err(|e| WarpError::Other(format!("Failed to clear cache by API type: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| {
            WarpError::Other(format!(
                "Failed to spawn clear_by_api_type operation: {}",
                e
            ))
        })?
    }

    /// Get total number of entries
    pub async fn count_entries(&self) -> Result<u64> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<u64> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM cache_entries", [], |row| row.get(0))
                .map_err(|e| WarpError::Other(format!("Failed to count entries: {}", e)))?;

            Ok(count as u64)
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn count_entries operation: {}", e)))?
    }

    /// Get total cache size in bytes
    pub async fn get_total_size(&self) -> Result<u64> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<u64> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let size: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(size), 0) FROM cache_entries",
                    [],
                    |row| row.get(0),
                )
                .map_err(|e| WarpError::Other(format!("Failed to get total size: {}", e)))?;

            Ok(size as u64)
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn get_total_size operation: {}", e)))?
    }

    /// Count expired entries
    pub async fn count_expired_entries(&self) -> Result<u64> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<u64> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let now = Utc::now().to_rfc3339();
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM cache_entries WHERE expires_at < ?1",
                    params![now],
                    |row| row.get(0),
                )
                .map_err(|e| WarpError::Other(format!("Failed to count expired entries: {}", e)))?;

            Ok(count as u64)
        })
        .await
        .map_err(|e| {
            WarpError::Other(format!(
                "Failed to spawn count_expired_entries operation: {}",
                e
            ))
        })?
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&mut self) -> Result<u64> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<u64> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let now = Utc::now().to_rfc3339();
            let affected = conn
                .execute(
                    "DELETE FROM cache_entries WHERE expires_at < ?1",
                    params![now],
                )
                .map_err(|e| {
                    WarpError::Other(format!("Failed to cleanup expired entries: {}", e))
                })?;

            Ok(affected as u64)
        })
        .await
        .map_err(|e| {
            WarpError::Other(format!("Failed to spawn cleanup_expired operation: {}", e))
        })?
    }

    /// Evict least recently used entries to free up space
    pub async fn evict_lru(&mut self, space_to_free: u64) -> Result<()> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            // Get entries sorted by last_accessed (oldest first) and collect keys to delete
            let mut stmt = conn
                .prepare(
                    r#"
                SELECT key, size FROM cache_entries
                ORDER BY last_accessed ASC
                "#,
                )
                .map_err(|e| WarpError::Other(format!("Failed to prepare LRU query: {}", e)))?;

            let mut entries = Vec::new();
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
                })
                .map_err(|e| WarpError::Other(format!("Failed to execute LRU query: {}", e)))?;

            for row in rows {
                entries.push(
                    row.map_err(|e| WarpError::Other(format!("Failed to process LRU row: {}", e)))?,
                );
            }

            let mut freed_space = 0u64;
            let mut keys_to_delete = Vec::new();

            for (key, size) in entries {
                keys_to_delete.push(key);
                freed_space += size;

                if freed_space >= space_to_free {
                    break;
                }
            }

            // Delete the selected entries
            for key in keys_to_delete {
                conn.execute("DELETE FROM cache_entries WHERE key = ?1", params![key])
                    .map_err(|e| WarpError::Other(format!("Failed to delete LRU entry: {}", e)))?;
            }

            Ok(())
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn evict_lru operation: {}", e)))?
    }

    /// Get cache entries by API type
    pub async fn get_entries_by_api_type(&self, api_type: ApiType) -> Result<Vec<CacheEntry>> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<Vec<CacheEntry>> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            let mut stmt = conn
                .prepare(
                    r#"
            SELECT key, data, created_at, expires_at, api_type, size
            FROM cache_entries
            WHERE api_type = ?1
            ORDER BY created_at DESC
            "#,
                )
                .map_err(|e| {
                    WarpError::Other(format!("Failed to prepare API type query: {}", e))
                })?;

            let mut entries = Vec::new();
            let rows = stmt
                .query_map(params![api_type.as_str()], |row| {
                    let api_type_str: String = row.get(4)?;
                    let api_type = api_type_str.parse::<ApiType>().map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            4,
                            "api_type".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?;

                    Ok(CacheEntry {
                        key: row.get(0)?,
                        data: row.get(1)?,
                        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                            .map_err(|_| {
                                rusqlite::Error::InvalidColumnType(
                                    2,
                                    "created_at".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            })?
                            .with_timezone(&Utc),
                        expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                            .map_err(|_| {
                                rusqlite::Error::InvalidColumnType(
                                    3,
                                    "expires_at".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            })?
                            .with_timezone(&Utc),
                        api_type,
                        size: row.get::<_, i64>(5)? as u64,
                    })
                })
                .map_err(|e| {
                    WarpError::Other(format!("Failed to execute API type query: {}", e))
                })?;

            for row in rows {
                entries.push(row.map_err(|e| {
                    WarpError::Other(format!("Failed to process entry row: {}", e))
                })?);
            }

            Ok(entries)
        })
        .await
        .map_err(|e| {
            WarpError::Other(format!(
                "Failed to spawn get_entries_by_api_type operation: {}",
                e
            ))
        })?
    }

    /// Optimize database (vacuum and analyze)
    pub async fn optimize(&mut self) -> Result<()> {
        // Remove expired entries first
        self.cleanup_expired().await?;

        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let conn = Connection::open(&db_path)
                .map_err(|e| WarpError::Other(format!("Failed to open cache database: {}", e)))?;

            // Vacuum to reclaim space
            conn.execute("VACUUM", [])
                .map_err(|e| WarpError::Other(format!("Failed to vacuum database: {}", e)))?;

            // Analyze to update statistics
            conn.execute("ANALYZE", [])
                .map_err(|e| WarpError::Other(format!("Failed to analyze database: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| WarpError::Other(format!("Failed to spawn optimize operation: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::NamedTempFile;

    async fn create_test_storage() -> (SqliteStorage, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = SqliteStorage::new(temp_file.path()).await.unwrap();
        (storage, temp_file)
    }

    fn create_test_entry(key: &str, data: &[u8], api_type: ApiType) -> CacheEntry {
        let now = Utc::now();
        CacheEntry {
            key: key.to_string(),
            data: data.to_vec(),
            created_at: now,
            expires_at: now + Duration::hours(1),
            api_type,
            size: data.len() as u64,
        }
    }

    #[tokio::test]
    async fn test_storage_put_and_get() {
        let (mut storage, _temp_file) = create_test_storage().await;

        let entry = create_test_entry("test_key", b"test_data", ApiType::Nlic);
        storage.put(entry.clone()).await.unwrap();

        let retrieved = storage.get("test_key").await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.key, entry.key);
        assert_eq!(retrieved.data, entry.data);
        assert_eq!(retrieved.api_type, entry.api_type);
    }

    #[tokio::test]
    async fn test_storage_remove() {
        let (mut storage, _temp_file) = create_test_storage().await;

        let entry = create_test_entry("test_key", b"test_data", ApiType::Nlic);
        storage.put(entry).await.unwrap();

        let removed = storage.remove("test_key").await.unwrap();
        assert!(removed);

        let retrieved = storage.get("test_key").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_storage_clear_by_api_type() {
        let (mut storage, _temp_file) = create_test_storage().await;

        let entry1 = create_test_entry("nlic_key", b"nlic_data", ApiType::Nlic);
        let entry2 = create_test_entry("elis_key", b"elis_data", ApiType::Elis);

        storage.put(entry1).await.unwrap();
        storage.put(entry2).await.unwrap();

        storage.clear_by_api_type(ApiType::Nlic).await.unwrap();

        let nlic_entry = storage.get("nlic_key").await.unwrap();
        let elis_entry = storage.get("elis_key").await.unwrap();

        assert!(nlic_entry.is_none());
        assert!(elis_entry.is_some());
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let (mut storage, _temp_file) = create_test_storage().await;

        let entry1 = create_test_entry("key1", b"data1", ApiType::Nlic);
        let entry2 = create_test_entry("key2", b"data2", ApiType::Elis);

        storage.put(entry1).await.unwrap();
        storage.put(entry2).await.unwrap();

        let count = storage.count_entries().await.unwrap();
        let size = storage.get_total_size().await.unwrap();

        assert_eq!(count, 2);
        assert_eq!(size, 10); // "data1" + "data2" = 5 + 5 = 10 bytes
    }

    #[tokio::test]
    async fn test_expired_cleanup() {
        let (mut storage, _temp_file) = create_test_storage().await;

        let now = Utc::now();
        let mut entry = create_test_entry("expired_key", b"expired_data", ApiType::Nlic);
        entry.expires_at = now - Duration::hours(1); // Already expired

        storage.put(entry).await.unwrap();

        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        let retrieved = storage.get("expired_key").await.unwrap();
        assert!(retrieved.is_none());
    }
}
