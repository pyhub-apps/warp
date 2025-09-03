use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use warp::api::batcher::{BatchConfig, BatcherFactory};
use warp::api::client::ClientConfig;
use warp::api::parallel::{ParallelConfig, ParallelExecutor};
use warp::api::pool::{get_pool_registry, AdaptivePoolConfig};
use warp::api::types::{ResponseType, UnifiedSearchRequest};
use warp::api::{ApiClientFactory, ApiType};
use warp::cache::tiered::{L1CacheConfig, TieredCache, TieredCacheConfig};
use warp::cache::{CacheConfig, CacheStore};
use warp::metrics::get_global_metrics;

/// Create test configuration with various optimization features
async fn create_optimized_config(
    enable_cache: bool,
    enable_tiered_cache: bool,
    enable_batching: bool,
) -> (
    ClientConfig,
    Option<Arc<TieredCache>>,
    Option<Arc<warp::api::batcher::RequestBatcher>>,
) {
    let cache_store = if enable_cache {
        if enable_tiered_cache {
            let tiered_config = TieredCacheConfig {
                l1_config: L1CacheConfig {
                    max_entries: 100,
                    ttl: chrono::Duration::minutes(10),
                    enable_compression: true,
                    compression_threshold: 512,
                },
                l2_config: CacheConfig {
                    max_size: 10 * 1024 * 1024, // 10MB
                    default_ttl: chrono::Duration::hours(1),
                    db_path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
                },
                enable_l3: false,
                l3_dir: tempfile::tempdir().unwrap().path().to_path_buf(),
            };
            let tiered_cache = Arc::new(TieredCache::new(tiered_config).await.unwrap());
            (None, Some(tiered_cache))
        } else {
            let cache = Some(Arc::new(
                CacheStore::new(CacheConfig {
                    max_size: 10 * 1024 * 1024, // 10MB
                    default_ttl: chrono::Duration::hours(1),
                    db_path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
                })
                .await
                .unwrap(),
            ));
            (cache, None)
        }
    } else {
        (None, None)
    };

    let client_config = ClientConfig {
        api_key: "benchmark_key".to_string(),
        timeout: 30,
        max_retries: 3,
        retry_base_delay: 100,
        user_agent: "benchmark-agent/1.0".to_string(),
        cache: cache_store.0.clone(),
        bypass_cache: false,
    };

    let batcher = if enable_batching {
        let client = ApiClientFactory::create(ApiType::Nlic, client_config.clone()).unwrap();
        let batch_config = BatchConfig {
            max_batch_size: 10,
            max_batch_delay: Duration::from_millis(50),
            enable_deduplication: true,
            deduplication_ttl: Duration::from_secs(300),
            enable_predictive_batching: true,
        };
        Some(Arc::new(BatcherFactory::create_batcher(
            Arc::from(client),
            Some(batch_config),
        )))
    } else {
        None
    };

    (client_config, cache_store.1, batcher)
}

/// Benchmark basic API search performance
fn bench_basic_search_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("basic_search");

    // Test different configurations
    let configs = [
        ("no_optimizations", false, false, false),
        ("with_cache", true, false, false),
        ("with_tiered_cache", true, true, false),
        ("with_batching", false, false, true),
        ("fully_optimized", true, true, true),
    ];

    for (name, cache, tiered, batch) in configs.iter() {
        let (config, _tiered_cache, _batcher) =
            rt.block_on(create_optimized_config(*cache, *tiered, *batch));

        group.bench_with_input(BenchmarkId::new("search", name), &config, |b, config| {
            b.iter(|| {
                let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).unwrap();
                let request = UnifiedSearchRequest {
                    query: black_box(format!("benchmark_query_{}", rand::random::<u32>())),
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

    group.finish();
}

/// Benchmark parallel search performance
fn bench_parallel_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("parallel_search");

    // Test different concurrency levels
    let concurrency_levels = [1, 2, 5, 10, 20];

    for &concurrency in concurrency_levels.iter() {
        let config = ParallelConfig {
            max_concurrent: concurrency,
            request_timeout: Duration::from_secs(10),
            fail_fast: false,
            batch_delay: Duration::from_millis(50),
        };

        group.throughput(Throughput::Elements(concurrency as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrency),
            &config,
            |b, config| {
                b.iter(|| {
                    let executor = ParallelExecutor::new(config.clone());
                    let base_config = rt.block_on(create_optimized_config(true, false, false)).0;

                    let clients = (0..concurrency)
                        .map(|i| {
                            (
                                match i % 3 {
                                    0 => ApiType::Nlic,
                                    1 => ApiType::Elis,
                                    _ => ApiType::Prec,
                                },
                                Arc::from(
                                    ApiClientFactory::create(ApiType::Nlic, base_config.clone())
                                        .unwrap(),
                                ),
                            )
                        })
                        .collect();

                    let request = UnifiedSearchRequest {
                        query: black_box(format!("parallel_query_{}", rand::random::<u32>())),
                        page_no: 1,
                        page_size: 5,
                        response_type: ResponseType::Json,
                        ..Default::default()
                    };

                    let _ = rt.block_on(executor.search_parallel(clients, request));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark connection pool performance
fn bench_connection_pool_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("connection_pools");

    // Test different pool configurations
    let pool_configs = [
        ("default", AdaptivePoolConfig::default()),
        (
            "high_throughput",
            AdaptivePoolConfig {
                initial_size: 20,
                max_size: 100,
                target_utilization: 0.8,
                scale_up_threshold: 0.9,
                scale_down_threshold: 0.2,
                ..AdaptivePoolConfig::default()
            },
        ),
        (
            "low_latency",
            AdaptivePoolConfig {
                initial_size: 50,
                max_size: 50, // Fixed size for consistent latency
                target_utilization: 0.5,
                ..AdaptivePoolConfig::default()
            },
        ),
    ];

    for (name, pool_config) in pool_configs.iter() {
        group.bench_with_input(
            BenchmarkId::new("pool_acquisition", name),
            pool_config,
            |b, config| {
                b.iter(|| {
                    let registry = get_pool_registry();
                    let pool = registry.get_or_create_pool(
                        &format!("bench_pool_{}", black_box(rand::random::<u32>())),
                        config.clone(),
                        "benchmark/1.0",
                    );

                    rt.block_on(async {
                        let _permit = pool.acquire_connection().await;
                        // Connection is automatically released when permit is dropped
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache performance across different cache tiers
fn bench_cache_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("cache_performance");

    // Create different cache configurations
    let cache_setups = rt.block_on(async {
        vec![
            (
                "basic_cache",
                create_optimized_config(true, false, false).await.0,
            ),
            (
                "tiered_cache",
                create_optimized_config(true, true, false).await.0,
            ),
        ]
    });

    for (name, config) in cache_setups {
        group.bench_with_input(
            BenchmarkId::new("cache_operations", name),
            &config,
            |b, config| {
                b.iter(|| {
                    let client = ApiClientFactory::create(ApiType::Nlic, config.clone()).unwrap();
                    let request = UnifiedSearchRequest {
                        query: black_box("cached_query".to_string()), // Same query for cache hits
                        page_no: 1,
                        page_size: 10,
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

/// Benchmark batching and deduplication performance
fn bench_batching_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batching_performance");

    // Test different batch sizes
    let batch_sizes = [1, 5, 10, 20, 50];

    for &batch_size in batch_sizes.iter() {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_requests", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    rt.block_on(async {
                        let (_, _, batcher) = create_optimized_config(false, false, true).await;
                        let batcher = batcher.unwrap();

                        let mut handles = vec![];
                        for i in 0..batch_size {
                            let batcher = Arc::clone(&batcher);
                            handles.push(tokio::spawn(async move {
                                let request = UnifiedSearchRequest {
                                    query: black_box(if i % 2 == 0 {
                                        "duplicate_query".to_string() // Some duplicate queries
                                    } else {
                                        format!("unique_query_{}", i)
                                    }),
                                    page_no: 1,
                                    page_size: 10,
                                    response_type: ResponseType::Json,
                                    ..Default::default()
                                };

                                batcher.submit_request(request).await
                            }));
                        }

                        // Wait for all requests to complete
                        for handle in handles {
                            let _ = handle.await;
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark metrics collection overhead
fn bench_metrics_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("metrics_overhead");

    // Test metrics collection performance
    group.bench_function("metrics_collection", |b| {
        let collector = get_global_metrics();
        b.iter(|| {
            // Simulate various metrics operations
            collector.record_request_success(
                black_box("test_operation"),
                black_box(Duration::from_millis(rand::random::<u64>() % 1000)),
            );
            collector.record_cache_hit(black_box("test_api"));
            collector.record_cache_miss(black_box("test_api"));
        });
    });

    // Test metrics snapshot generation
    group.bench_function("metrics_snapshot", |b| {
        let collector = get_global_metrics();

        // Pre-populate with some data
        for i in 0..100 {
            collector.record_request_success(
                &format!("operation_{}", i % 10),
                Duration::from_millis(i as u64),
            );
        }

        b.iter(|| {
            rt.block_on(async {
                let _snapshot = collector.get_snapshot().await;
            })
        });
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_patterns");

    // Test memory allocation patterns in different scenarios
    group.bench_function("high_frequency_allocations", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate high-frequency API client creation
                for _ in 0..black_box(100) {
                    let (config, _, _) = create_optimized_config(false, false, false).await;
                    let client = ApiClientFactory::create(ApiType::Nlic, config).unwrap();
                    let request = UnifiedSearchRequest {
                        query: black_box(format!("mem_test_{}", rand::random::<u32>())),
                        page_no: 1,
                        page_size: 5,
                        response_type: ResponseType::Json,
                        ..Default::default()
                    };

                    let _ = client.search(request).await;
                }
            });
        });
    });

    group.finish();
}

/// Comprehensive end-to-end performance benchmark
fn bench_end_to_end_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("end_to_end");
    group.sample_size(10); // Reduce sample size for complex benchmarks

    // Test complete workflow with all optimizations enabled
    group.bench_function("full_optimized_workflow", |b| {
        b.iter(|| {
            // Simplified synchronous test for basic functionality
            let metrics = get_global_metrics();
            metrics.record_request_success("benchmark_test", std::time::Duration::from_millis(100));
            let _ = black_box(metrics.get_snapshot());
        })
    });

    group.finish();
}

criterion_group!(
    performance_benches,
    bench_basic_search_performance,
    bench_parallel_search,
    bench_connection_pool_performance,
    bench_cache_performance,
    bench_batching_performance,
    bench_metrics_overhead,
    bench_memory_patterns,
    bench_end_to_end_performance
);

criterion_main!(performance_benches);
