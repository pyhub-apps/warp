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

    // Create search request with filters
    let request = create_search_request(&args);

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

    // Merge responses with post-processing
    let merged_response = merge_responses_with_filtering(all_responses, &args);

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

/// Merge multiple search responses with advanced filtering and processing
fn merge_responses_with_filtering(
    responses: Vec<(ApiType, SearchResponse)>,
    args: &SearchArgs,
) -> SearchResponse {
    let mut all_items = Vec::new();

    for (api_type, mut response) in responses {
        // Add source info to each item
        for item in &mut response.items {
            item.source = api_type.display_name().to_string();
        }

        all_items.extend(response.items);
    }

    // Apply client-side filters
    let filtered_items = apply_client_side_filters(all_items, args);

    // Sort results based on specified order
    let sorted_items = sort_search_results(filtered_items, &args.sort);

    SearchResponse {
        total_count: sorted_items.len() as u32, // Update count after filtering
        page_no: 1,                             // Always 1 for merged results
        page_size: sorted_items.len() as u32,
        items: sorted_items,
        source: "통합검색".to_string(),
        timestamp: Utc::now(),
    }
}

/// Apply client-side filters that can't be handled by APIs
fn apply_client_side_filters(
    mut items: Vec<crate::api::types::SearchItem>,
    args: &SearchArgs,
) -> Vec<crate::api::types::SearchItem> {
    use regex::Regex;

    // Apply regex search if enabled
    if args.regex {
        if let Ok(regex) = Regex::new(&args.query) {
            items.retain(|item| {
                regex.is_match(&item.title)
                    || item.summary.as_ref().is_some_and(|s| regex.is_match(s))
            });
        }
    }

    // Apply title-only search if enabled
    if args.title_only {
        let query_lower = args.query.to_lowercase();
        items.retain(|item| item.title.to_lowercase().contains(&query_lower));
    }

    // Apply minimum score filter if specified
    if let Some(min_score) = args.min_score {
        // For now, we'll use a simple relevance heuristic based on query match
        let query_terms: Vec<&str> = args.query.split_whitespace().collect();
        items.retain(|item| {
            let title_lower = item.title.to_lowercase();
            let summary_lower = item
                .summary
                .as_ref()
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            let match_count = query_terms
                .iter()
                .filter(|term| {
                    let term_lower = term.to_lowercase();
                    title_lower.contains(&term_lower) || summary_lower.contains(&term_lower)
                })
                .count();

            let relevance_score = match_count as f32 / query_terms.len() as f32;
            relevance_score >= min_score
        });
    }

    // Apply law type filter (additional filtering for comma-separated values)
    if let Some(ref law_types) = args.law_type {
        let allowed_types: Vec<&str> = law_types.split(',').map(|s| s.trim()).collect();
        items.retain(|item| {
            item.law_type
                .as_ref()
                .is_some_and(|lt| allowed_types.iter().any(|allowed| lt.contains(allowed)))
        });
    }

    // Apply department filter (additional filtering for comma-separated values)
    if let Some(ref departments) = args.department {
        let allowed_departments: Vec<&str> = departments.split(',').map(|s| s.trim()).collect();
        items.retain(|item| {
            item.department.as_ref().is_some_and(|dept| {
                allowed_departments
                    .iter()
                    .any(|allowed| dept.contains(allowed))
            })
        });
    }

    items
}

/// Sort search results based on specified sort order
fn sort_search_results(
    mut items: Vec<crate::api::types::SearchItem>,
    sort_order: &str,
) -> Vec<crate::api::types::SearchItem> {
    match sort_order {
        "date_asc" => {
            items.sort_by(|a, b| {
                a.enforcement_date
                    .cmp(&b.enforcement_date)
                    .then_with(|| a.revision_date.cmp(&b.revision_date))
            });
        }
        "date_desc" => {
            items.sort_by(|a, b| {
                b.enforcement_date
                    .cmp(&a.enforcement_date)
                    .then_with(|| b.revision_date.cmp(&a.revision_date))
            });
        }
        "title_asc" => {
            items.sort_by(|a, b| a.title.cmp(&b.title));
        }
        "title_desc" => {
            items.sort_by(|a, b| b.title.cmp(&a.title));
        }
        "relevance" => {
            // Keep original order for relevance (APIs return results by relevance)
            // Could implement more sophisticated relevance scoring here
        }
        _ => {
            // Default to relevance for unknown sort orders
        }
    }

    items
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

    // Create search request with filters
    let request = create_search_request(&args);

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

            let merged_response = merge_responses_with_filtering(parallel_result.successes, &args);
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

/// Create UnifiedSearchRequest with filters from SearchArgs
fn create_search_request(args: &SearchArgs) -> UnifiedSearchRequest {
    use crate::api::types::SortOrder;
    use std::collections::HashMap;

    // Determine sort order
    let sort_order = match args.sort.as_str() {
        "date_asc" => Some(SortOrder::DateAsc),
        "date_desc" => Some(SortOrder::DateDesc),
        "title_asc" => Some(SortOrder::TitleAsc),
        "title_desc" => Some(SortOrder::TitleDesc),
        "relevance" => Some(SortOrder::Relevance),
        _ => Some(SortOrder::Relevance), // Default to relevance
    };

    // Prepare extra parameters for advanced filtering
    let mut extras = HashMap::new();

    // Add regex search mode if enabled
    if args.regex {
        extras.insert("regex_search".to_string(), "true".to_string());
    }

    // Add title-only search mode if enabled
    if args.title_only {
        extras.insert("title_only".to_string(), "true".to_string());
    }

    // Add minimum score filter if specified
    if let Some(min_score) = args.min_score {
        extras.insert("min_score".to_string(), min_score.to_string());
    }

    // Add case type filter for precedents
    if let Some(ref case_type) = args.case_type {
        extras.insert("case_type".to_string(), case_type.clone());
    }

    // Add court filter for precedents
    if let Some(ref court) = args.court {
        extras.insert("court".to_string(), court.clone());
    }

    // Add status filter
    if let Some(ref status) = args.status {
        extras.insert("status".to_string(), status.clone());
    }

    // Handle date range filters
    let (date_from, date_to) = if let Some(recent_days) = args.recent_days {
        // Calculate date range for recent days filter
        use chrono::{Duration as ChronoDuration, Utc};
        let end_date = Utc::now();
        let start_date = end_date - ChronoDuration::days(recent_days as i64);
        (
            Some(start_date.format("%Y%m%d").to_string()),
            Some(end_date.format("%Y%m%d").to_string()),
        )
    } else {
        (args.from.clone(), args.to.clone())
    };

    UnifiedSearchRequest {
        query: args.query.clone(),
        page_no: args.page,
        page_size: args.size,
        response_type: ResponseType::Json,
        region: args.region.clone(),
        law_type: args.law_type.clone(),
        department: args.department.clone(),
        date_from,
        date_to,
        sort: sort_order,
        extras,
    }
}
