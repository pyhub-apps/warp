use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use warp::api::{ApiClientFactory, ApiType};
use warp::api::client::ClientConfig;
use warp::api::types::{ResponseType, UnifiedSearchRequest};
use warp::cache::CacheConfig;
use warp::metrics::get_global_metrics;

// Mock test data
fn create_test_config(enable_cache: bool) -> ClientConfig {
    let cache_config = if enable_cache {
        Some(CacheConfig {
            max_size: 1000,
            ttl: std::time::Duration::from_secs(300),
            cleanup_interval: std::time::Duration::from_secs(60),
        })
    } else {
        None
    };

    ClientConfig {
        nlic: Some(warp::api::nlic::NlicConfig {
            api_key: "test_key".to_string(),
            base_url: Some("http://test.example.com".to_string()),
            timeout: Some(std::time::Duration::from_secs(10)),
        }),
        cache: cache_config,
        ..Default::default()
    }
}

/// Benchmark single API search without cache
fn bench_single_search_no_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config(false);
    
    c.bench_function("single_search_no_cache", |b| {
        b.iter(|| {
            let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).unwrap();
            let request = UnifiedSearchRequest {
                query: black_box("민법".to_string()),
                page_no: 1,
                page_size: 10,
                response_type: ResponseType::Json,
                ..Default::default()
            };
            
            // Note: This will fail without real API key, but measures setup overhead
            let _ = rt.block_on(client.search(request));
        });
    });
}

/// Benchmark search with cache enabled
fn bench_single_search_with_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config(true);
    
    c.bench_function("single_search_with_cache", |b| {
        b.iter(|| {
            let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).unwrap();
            let request = UnifiedSearchRequest {
                query: black_box("민법".to_string()),
                page_no: 1,
                page_size: 10,
                response_type: ResponseType::Json,
                ..Default::default()
            };
            
            let _ = rt.block_on(client.search(request));
        });
    });
}

/// Benchmark client creation performance across different configurations
fn bench_client_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("client_creation");
    
    for cache_enabled in [false, true] {
        let config = create_test_config(cache_enabled);
        let name = if cache_enabled { "with_cache" } else { "no_cache" };
        
        group.bench_with_input(
            BenchmarkId::new("api_client_factory", name),
            &config,
            |b, config| {
                b.iter(|| {
                    // Create new client for each iteration to test factory performance
                    let client = ApiClientFactory::create(ApiType::Nlic, config).unwrap();
                    let request = UnifiedSearchRequest {
                        query: black_box("test".to_string()),
                        page_no: 1,
                        page_size: 5,
                        response_type: ResponseType::Json,
                        ..Default::default()
                    };
                    
                    let _ = rt.block_on(client.search(request));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark cache and metrics operations
fn bench_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config(true);
    
    c.bench_function("cache_operations", |b| {
        b.iter(|| {
            // Test metrics collection overhead
            let metrics = get_global_metrics();
            metrics.record_request(black_box("test_api"));
            metrics.record_success(black_box("test_api"));
            
            let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).unwrap();
            let request = UnifiedSearchRequest {
                query: black_box("cache_test".to_string()),
                page_no: black_box(1),
                page_size: black_box(10),
                response_type: ResponseType::Json,
                ..Default::default()
            };
            
            // Measure cache key generation and lookup overhead
            let _ = rt.block_on(client.search(request));
        });
    });
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = create_test_config(false);
    
    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            // Test memory allocation patterns in API client creation
            for i in 0..black_box(10) {
                let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).unwrap();
                let request = UnifiedSearchRequest {
                    query: black_box(format!("query_{}", i)),
                    page_no: 1,
                    page_size: 5,
                    response_type: ResponseType::Json,
                    ..Default::default()
                };
                
                let _ = rt.block_on(client.search(request));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_single_search_no_cache,
    bench_single_search_with_cache,
    bench_client_creation,
    bench_cache_operations,
    bench_memory_patterns
);
criterion_main!(benches);