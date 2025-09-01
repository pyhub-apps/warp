use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use tokio::runtime::Runtime;
use warp::api::{ApiClientFactory, ApiType};
use warp::api::client::ClientConfig;
use warp::api::types::{ResponseType, UnifiedSearchRequest};
use warp::cache::CacheStore;
use warp::cache::CacheConfig;

// Mock test data
const TEST_QUERIES: &[&str] = &[
    "민법",
    "형법", 
    "상법",
    "헌법",
    "개인정보보호법"
];

/// Setup test client configuration
fn create_test_config(with_cache: bool) -> ClientConfig {
    let cache = if with_cache {
        // Create in-memory cache for testing
        let rt = Runtime::new().unwrap();
        let cache_config = CacheConfig {
            max_size: 10 * 1024 * 1024, // 10MB
            default_ttl: chrono::Duration::minutes(30),
            db_path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        };
        Some(std::sync::Arc::new(rt.block_on(async {
            CacheStore::new(cache_config).await.unwrap()
        })))
    } else {
        None
    };

    ClientConfig {
        api_key: "test_api_key".to_string(),
        timeout: 30,
        max_retries: 3,
        retry_base_delay: 100,
        user_agent: "warp-benchmark/1.0".to_string(),
        cache,
        bypass_cache: !with_cache,
    }
}

/// Benchmark single API search performance
fn bench_single_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config(false);
    
    c.bench_function("single_search_no_cache", |b| {
        b.to_async(&rt).iter(|| async {
            let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).await.unwrap();
            let request = UnifiedSearchRequest {
                query: black_box("민법".to_string()),
                page_no: 1,
                page_size: 10,
                response_type: ResponseType::Json,
                ..Default::default()
            };
            
            // Note: This will fail without real API key, but measures setup overhead
            let _ = client.search(request).await;
        });
    });
}

/// Benchmark search with cache
fn bench_cached_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config(true);
    
    c.bench_function("single_search_with_cache", |b| {
        b.to_async(&rt).iter(|| async {
            let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).await.unwrap();
            let request = UnifiedSearchRequest {
                query: black_box("민법".to_string()),
                page_no: 1,
                page_size: 10,
                response_type: ResponseType::Json,
                ..Default::default()
            };
            
            let _ = client.search(request).await;
        });
    });
}

/// Benchmark parallel searches
fn bench_parallel_searches(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_searches");
    
    for concurrent_count in [1, 2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent", concurrent_count),
            concurrent_count,
            |b, &concurrent_count| {
                b.to_async(&rt).iter(|| async {
                    let config = create_test_config(false);
                    let client = ApiClientFactory::create(ApiType::Nlic, config).await.unwrap();
                    
                    let mut handles = vec![];
                    for i in 0..concurrent_count {
                        let client = client.clone();
                        let query = TEST_QUERIES[i % TEST_QUERIES.len()];
                        
                        let handle = tokio::spawn(async move {
                            let request = UnifiedSearchRequest {
                                query: query.to_string(),
                                page_no: 1,
                                page_size: 10,
                                response_type: ResponseType::Json,
                                ..Default::default()
                            };
                            client.search(request).await
                        });
                        handles.push(handle);
                    }
                    
                    // Wait for all requests to complete
                    futures::future::try_join_all(handles).await.unwrap();
                });
            },
        );
    }
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config(false);
    
    c.bench_function("memory_allocation_pattern", |b| {
        b.to_async(&rt).iter(|| async {
            // Test creating and destroying clients
            for _ in 0..10 {
                let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).await.unwrap();
                drop(client);
            }
        });
    });
}

/// Benchmark cache key generation
fn bench_cache_operations(c: &mut Criterion) {
    use warp::cache::key::CacheKeyGenerator;
    
    c.bench_function("cache_key_generation", |b| {
        b.iter(|| {
            let key = CacheKeyGenerator::nlic_key(
                black_box("search"),
                black_box(Some("test query")),
                black_box(Some("law")),
                black_box(Some("department")),
                black_box(Some(1)),
                black_box(Some(10)),
            );
            black_box(key);
        });
    });
}

/// Benchmark JSON parsing performance
fn bench_json_parsing(c: &mut Criterion) {
    let sample_json = r#"{
        "totalCnt": 100,
        "currentPage": 1,
        "LawSearch": [
            {
                "법령ID": "001",
                "법령명_한글": "민법",
                "공포일자": "19581222",
                "소관부처명": "법무부"
            },
            {
                "법령ID": "002", 
                "법령명_한글": "형법",
                "공포일자": "19531018",
                "소관부처명": "법무부"
            }
        ]
    }"#;
    
    c.bench_function("json_parse_search_response", |b| {
        b.iter(|| {
            let _: Result<serde_json::Value, _> = serde_json::from_str(black_box(sample_json));
        });
    });
}

criterion_group!(
    benches,
    bench_single_search,
    bench_cached_search,
    bench_parallel_searches,
    bench_memory_allocation,
    bench_cache_operations,
    bench_json_parsing
);

criterion_main!(benches);