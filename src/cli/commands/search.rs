use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::types::{ResponseType, SearchResponse, UnifiedSearchRequest};
use crate::api::{ApiClientFactory, ApiType};
use crate::cache::CacheStore;
use crate::cli::args::SearchArgs;
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;
use crate::progress::{messages, ProgressManager};
use chrono::Utc;
use futures::future::join_all;
use std::sync::Arc;

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
    let api_types = parse_source(&args.source);

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

    for (_idx, api_type) in api_types.into_iter().enumerate() {
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
                    pm.show_message(&format!("{} 검색 시작...", api_name));
                    let result = client.search(req).await;
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
                progress_manager.show_message(&messages::search_complete(
                    api_type.display_name(),
                    response.items.len(),
                ));
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
                .filter_map(|s| ApiType::from_str(s.trim()))
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
