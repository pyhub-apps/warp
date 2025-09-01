use chrono::Duration;
use std::path::PathBuf;
use warp::api::ApiType;
use warp::cache::key::CacheKeyGenerator;
use warp::cache::{CacheConfig, CacheStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create cache configuration
    let config = CacheConfig {
        max_size: 50 * 1024 * 1024,       // 50MB
        default_ttl: Duration::hours(12), // 12 hours
        db_path: PathBuf::from("example_cache.db"),
    };

    // Initialize cache store
    let cache = CacheStore::new(config).await?;

    // Example 1: Cache NLIC API response
    let nlic_key =
        CacheKeyGenerator::nlic_key("/api/search", Some("민법"), Some("law"), Some(1), Some(10));

    println!("Generated NLIC cache key: {}", nlic_key);

    // Store some mock data
    let mock_response = r#"{"results": [{"title": "민법", "content": "..."}]}"#;
    cache
        .put(
            &nlic_key,
            mock_response.as_bytes().to_vec(),
            ApiType::Nlic,
            None, // Use default TTL
        )
        .await?;

    // Retrieve cached data
    if let Some(cached_data) = cache.get(&nlic_key).await? {
        let cached_json = String::from_utf8(cached_data)?;
        println!("Retrieved from cache: {}", cached_json);
    }

    // Example 2: Cache ELIS API response
    let elis_key = CacheKeyGenerator::elis_key(
        "/api/ordinance",
        Some("서울특별시"),
        Some("서울특별시"),
        Some("행정"),
        Some(1),
        Some(20),
    );

    println!("Generated ELIS cache key: {}", elis_key);

    let mock_ordinance = r#"{"ordinances": [{"title": "서울특별시 조례", "region": "서울"}]}"#;
    cache
        .put(
            &elis_key,
            mock_ordinance.as_bytes().to_vec(),
            ApiType::Elis,
            Some(Duration::hours(6)), // Custom TTL
        )
        .await?;

    // Example 3: Cache statistics
    let stats = cache.stats().await?;
    println!("\nCache Statistics:");
    println!("  Total entries: {}", stats.total_entries);
    println!("  Total size: {} bytes", stats.total_size);
    println!("  Utilization: {:.2}%", stats.utilization_percent());
    println!("  Expired entries: {}", stats.expired_entries);

    // Example 4: Generate unified search key
    let unified_key = CacheKeyGenerator::unified_search_key(
        "법률",
        &[ApiType::Nlic, ApiType::Elis, ApiType::Prec],
        Some(1),
        Some(50),
    );
    println!("\nUnified search key: {}", unified_key);

    // Example 5: Key validation
    println!("\nKey validation:");
    println!(
        "  NLIC key valid: {}",
        CacheKeyGenerator::is_valid_key(&nlic_key)
    );
    println!(
        "  ELIS key valid: {}",
        CacheKeyGenerator::is_valid_key(&elis_key)
    );
    println!(
        "  Invalid key valid: {}",
        CacheKeyGenerator::is_valid_key("invalid:key")
    );

    // Example 6: Clear cache for specific API
    cache.clear_api(ApiType::Nlic).await?;
    println!("\nCleared NLIC cache entries");

    let stats_after_clear = cache.stats().await?;
    println!(
        "Entries after NLIC clear: {}",
        stats_after_clear.total_entries
    );

    // Cleanup
    cache.clear().await?;
    println!("Cache cleared successfully");

    Ok(())
}
