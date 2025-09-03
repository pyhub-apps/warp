use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::parallel::{ParallelConfig, ParallelExecutor};
use crate::api::types::{ResponseType, SearchResponse, UnifiedSearchRequest};
use crate::api::{ApiClientFactory, ApiType};
use crate::cache::{CacheConfig, CacheStore};
use crate::cli::args::SearchArgs;
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;
use crate::progress::{messages, ApiStage, EnhancedApiProgress, ProgressManager};
use chrono::Utc;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;

/// Execute unified search command across multiple APIs
pub async fn execute(
    args: SearchArgs,
    format: OutputFormat,
    quiet: bool,
    verbose: bool,
    no_cache: bool,
) -> Result<()> {
    // Create progress manager
    let progress_manager = Arc::new(ProgressManager::new(quiet, verbose));
    if args.query.trim().is_empty() {
        return Err(WarpError::InvalidInput(
            "Search query cannot be empty".to_string(),
        ));
    }

    // Load configuration
    let config = Config::load()?;

    // Determine which APIs to search
    let api_types = if let Some(apis) = &args.apis {
        parse_apis(apis)
    } else {
        parse_source(&args.source)
    };

    // Check if parallel search is requested
    if args.parallel && api_types.len() > 1 {
        return execute_parallel_search(args, format, quiet, verbose, api_types, config).await;
    }

    // Create search request
    let request = UnifiedSearchRequest {
        query: args.query.clone(),
        page_no: args.page,
        page_size: args.size,
        response_type: ResponseType::Json,
        ..Default::default()
    };

    // Execute searches in parallel
    let mut tasks = Vec::new();
    let total_apis = api_types.len();

    // Create multi-API progress bar
    let multi_progress = progress_manager.create_multi_api_progress(
        total_apis as u64,
        &format!("{}개 API에서 '{}' 검색 중", total_apis, args.query),
    );

    for api_type in api_types.into_iter() {
        let api_key = match api_type {
            ApiType::Nlic => config.get_nlic_api_key(),
            ApiType::Elis => config.get_elis_api_key(),
            ApiType::Prec => config.get_prec_api_key(),
            ApiType::Admrul => config.get_admrul_api_key(),
            ApiType::Expc => config.get_expc_api_key(),
            ApiType::All => continue, // Skip, this is handled by selecting all APIs
        };

        if let Some(api_key) = api_key {
            // Create cache store if cache is enabled and not bypassed
            let cache = if config.cache.enabled && !no_cache {
                let cache_config = config.cache.to_cache_config();
                Some(Arc::new(CacheStore::new(cache_config).await?))
            } else {
                None
            };

            let client_config = ClientConfig {
                api_key,
                cache,
                bypass_cache: no_cache,
                ..Default::default()
            };

            if let Ok(client) = ApiClientFactory::create(api_type, client_config) {
                let req = request.clone();
                let client: Arc<Box<dyn LegalApiClient>> = Arc::from(client);
                let pm = progress_manager.clone();
                let api_name = api_type.display_name().to_string();

                tasks.push(tokio::spawn(async move {
                    // Create enhanced progress for each API
                    let mut api_progress = EnhancedApiProgress::new(pm.clone(), &api_name);

                    // Stage 1: Connecting
                    api_progress.advance_stage(ApiStage::Connecting, "API 서버 연결 중");

                    // Stage 2: Searching
                    api_progress.advance_stage(ApiStage::Searching, "검색 요청 전송 중");

                    let result = client.search(req).await;

                    // Stage 3: Parsing
                    api_progress.advance_stage(ApiStage::Parsing, "응답 데이터 파싱 중");

                    match &result {
                        Ok(response) => {
                            let completion_msg = messages::search_complete_with_time(
                                &api_name,
                                response.items.len(),
                                api_progress.elapsed().as_millis() as u64,
                            );
                            api_progress.complete_success(&completion_msg);
                        }
                        Err(e) => {
                            api_progress.complete_error(&format!("검색 실패: {}", e));
                        }
                    }

                    (api_type, result)
                }));
            }
        }
    }

    if tasks.is_empty() {
        return Err(WarpError::NoApiKey);
    }

    // Collect results
    let results = join_all(tasks).await;
    let mut all_responses = Vec::new();
    let mut errors = Vec::new();
    let mut completed = 0;

    for result in results {
        completed += 1;
        if let Some(pb) = multi_progress.as_ref() {
            pb.set_position(completed as u64);
            pb.set_message(messages::multi_api_progress(completed, total_apis));
        }

        match result {
            Ok((api_type, Ok(response))) => {
                all_responses.push((api_type, response));
            }
            Ok((api_type, Err(e))) => {
                errors.push((api_type, e));
            }
            Err(e) => {
                eprintln!("Task execution error: {}", e);
            }
        }
    }

    // Finish progress bar
    if let Some(pb) = multi_progress.as_ref() {
        pb.finish_with_message(format!(
            "✅ 검색 완료: {}개 API에서 {}개 결과",
            all_responses.len(),
            all_responses
                .iter()
                .map(|(_, r)| r.items.len())
                .sum::<usize>()
        ));
    }

    // Handle results
    if all_responses.is_empty() {
        if !errors.is_empty() {
            eprintln!("Search failed for all APIs:");
            for (api_type, error) in errors {
                eprintln!("  {}: {}", api_type.display_name(), error);
            }
            return Err(WarpError::Other("All API searches failed".to_string()));
        }
        println!("No results found for your search query.");
        return Ok(());
    }

    // Merge responses
    let merged_response = merge_responses(all_responses);

    // Format and output
    let output = output::format_search_response(&merged_response, format)?;
    println!("{}", output);

    // Report any errors
    if !errors.is_empty() {
        eprintln!("\nNote: Some APIs returned errors:");
        for (api_type, error) in errors {
            eprintln!("  {}: {}", api_type.display_name(), error);
        }
    }

    Ok(())
}

/// Parse source string to determine which APIs to search
fn parse_source(source: &str) -> Vec<ApiType> {
    match source.to_lowercase().as_str() {
        "all" | "" => vec![
            ApiType::Nlic,
            ApiType::Elis,
            ApiType::Prec,
            ApiType::Admrul,
            ApiType::Expc,
        ],
        "nlic" | "law" => vec![ApiType::Nlic],
        "elis" | "ordinance" => vec![ApiType::Elis],
        "prec" | "precedent" => vec![ApiType::Prec],
        "admrul" | "administrative" => vec![ApiType::Admrul],
        "expc" | "interpretation" => vec![ApiType::Expc],
        sources => {
            // Parse comma-separated list
            sources
                .split(',')
                .filter_map(|s| s.trim().parse::<ApiType>().ok())
                .collect()
        }
    }
}

/// Merge multiple search responses into one
fn merge_responses(responses: Vec<(ApiType, SearchResponse)>) -> SearchResponse {
    let mut total_count = 0;
    let mut all_items = Vec::new();

    for (api_type, mut response) in responses {
        total_count += response.total_count;

        // Add source info to each item
        for item in &mut response.items {
            item.source = api_type.display_name().to_string();
        }

        all_items.extend(response.items);
    }

    // Sort by relevance (in this simple implementation, we keep the order)
    // In a more sophisticated implementation, we could score and sort by relevance

    SearchResponse {
        total_count,
        page_no: 1, // Always 1 for merged results
        page_size: all_items.len() as u32,
        items: all_items,
        source: "통합검색".to_string(),
        timestamp: Utc::now(),
    }
}

/// Parse APIs string (comma-separated) to ApiType vector
fn parse_apis(apis: &str) -> Vec<ApiType> {
    apis.split(',')
        .filter_map(|api| {
            let api = api.trim().to_lowercase();
            match api.as_str() {
                "nlic" | "law" => Some(ApiType::Nlic),
                "elis" | "ordinance" => Some(ApiType::Elis),
                "prec" | "precedent" => Some(ApiType::Prec),
                "admrul" | "administrative" => Some(ApiType::Admrul),
                "expc" | "interpretation" => Some(ApiType::Expc),
                _ => None,
            }
        })
        .collect()
}

/// Execute parallel search with advanced optimization options
async fn execute_parallel_search(
    args: SearchArgs,
    format: OutputFormat,
    quiet: bool,
    verbose: bool,
    api_types: Vec<ApiType>,
    config: Config,
) -> Result<()> {
    let progress_manager = Arc::new(ProgressManager::new(quiet, verbose));

    if !quiet {
        println!(
            "🚀 병렬 검색 시작: {} APIs, 최적화 옵션 활성화",
            api_types.len()
        );
        if args.batch {
            println!("📦 배치 처리: {}개씩 그룹화", args.batch_size);
        }
        if let Some(tier) = args.cache_tier {
            println!(
                "⚡ 캐싱: Tier {} ({})",
                tier,
                if tier == 2 { "고급" } else { "기본" }
            );
        }
        println!("🔗 최대 동시 연결: {}개", args.max_concurrent);
    }

    // Create enhanced cache if requested
    let cache_store = if args.no_cache {
        None
    } else if let Some(tier) = args.cache_tier {
        Some(create_enhanced_cache(tier).await?)
    } else {
        None
    };

    // Create parallel configuration
    let parallel_config = ParallelConfig {
        max_concurrent: args.max_concurrent as usize,
        request_timeout: Duration::from_secs(args.timeout as u64),
        fail_fast: false,
        batch_delay: Duration::from_millis(100),
    };

    // Create search request
    let request = UnifiedSearchRequest {
        query: args.query.clone(),
        page_no: args.page,
        page_size: args.size,
        response_type: ResponseType::Json,
        ..Default::default()
    };

    // Create API clients with optimization
    let mut clients = Vec::new();
    for api_type in &api_types {
        let client_config = create_optimized_client_config(api_type, &config, cache_store.clone())?;
        let client = ApiClientFactory::create(*api_type, client_config)?;
        clients.push((*api_type, Arc::from(client)));
    }

    // Execute parallel search
    let executor = ParallelExecutor::new(parallel_config);
    let start_time = std::time::Instant::now();

    progress_manager.create_multi_api_progress(api_types.len() as u64, &args.query);

    let result = executor.search_parallel(clients, request).await;
    let execution_time = start_time.elapsed();

    match result {
        Ok(parallel_result) => {
            if !quiet {
                println!(
                    "✅ 병렬 검색 완료: {:.2}초, {} APIs 성공",
                    execution_time.as_secs_f64(),
                    parallel_result.successes.len()
                );
            }

            let merged_response = merge_responses(parallel_result.successes);
            let formatted_output = output::format_search_response(&merged_response, format)?;
            if !quiet {
                println!("{}", formatted_output);
            }
            Ok(())
        }
        Err(err) => {
            if !quiet {
                eprintln!(
                    "❌ 병렬 검색 오류: {:.2}초 후 실패",
                    execution_time.as_secs_f64()
                );
            }
            Err(err)
        }
    }
}

/// Create enhanced cache based on tier level
async fn create_enhanced_cache(tier: u8) -> Result<Arc<CacheStore>> {
    match tier {
        1 => {
            // Basic cache
            let cache_config = CacheConfig::default();
            let cache = CacheStore::new(cache_config).await?;
            Ok(Arc::new(cache))
        }
        2 => {
            // Advanced cache with compression would be implemented here
            // For now, use the same as tier 1
            let cache_config = CacheConfig::default();
            let cache = CacheStore::new(cache_config).await?;
            Ok(Arc::new(cache))
        }
        _ => Err(WarpError::InvalidInput(
            "Cache tier must be 1 or 2".to_string(),
        )),
    }
}

/// Create optimized client configuration for parallel search
fn create_optimized_client_config(
    api_type: &ApiType,
    config: &Config,
    cache: Option<Arc<CacheStore>>,
) -> Result<ClientConfig> {
    let api_key_name = match api_type {
        ApiType::Nlic => "law.nlic.key",
        ApiType::Elis => "law.elis.key",
        ApiType::Prec => "law.prec.key",
        ApiType::Admrul => "law.admrul.key",
        ApiType::Expc => "law.expc.key",
        ApiType::All => "law.key", // fallback
    };

    let api_key = config
        .get_api_key(api_key_name)
        .or_else(|| config.get_api_key("law.key"))
        .ok_or_else(|| {
            WarpError::InvalidInput(format!("API key for {} not found", api_type.display_name()))
        })?;

    Ok(ClientConfig {
        api_key,
        timeout: 30,
        max_retries: 3,
        retry_base_delay: 100,
        user_agent: format!("warp-parallel/{}", env!("CARGO_PKG_VERSION")),
        cache,
        bypass_cache: false,
        benchmark_mode: false,
    })
}
