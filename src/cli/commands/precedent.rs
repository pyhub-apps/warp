use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::types::{ResponseType, UnifiedSearchRequest};
use crate::api::{ApiClientFactory, ApiType};
use crate::cache::CacheStore;
use crate::cli::args::{PrecedentArgs, PrecedentCommand};
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;
use std::collections::HashMap;
use std::sync::Arc;

/// Parameters for precedent search operation
struct SearchParams {
    query: String,
    page: u32,
    size: u32,
    court: Option<String>,
    case_type: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    format: OutputFormat,
}

/// Execute precedent command (판례)
pub async fn execute(
    args: PrecedentArgs,
    format: OutputFormat,
    _quiet: bool,
    _verbose: bool,
    no_cache: bool,
) -> Result<()> {
    // Load configuration
    let config = Config::load()?;
    let api_key = config.get_prec_api_key().ok_or(WarpError::NoApiKey)?;

    // Create cache store if cache is enabled and not bypassed
    let cache = if config.cache.enabled && !no_cache {
        let cache_config = config.cache.to_cache_config();
        Some(Arc::new(CacheStore::new(cache_config).await?))
    } else {
        None
    };

    // Create API client
    let client_config = ClientConfig {
        api_key,
        cache,
        bypass_cache: no_cache,
        ..Default::default()
    };

    let client = ApiClientFactory::create(ApiType::Prec, client_config)?;

    // Extract common args
    let court = args.court.clone();
    let case_type = args.case_type.clone();
    let date_from = args.date_from.clone();
    let date_to = args.date_to.clone();

    // Handle direct query or subcommand
    match args.command {
        Some(PrecedentCommand::Search { query, page, size }) => {
            let params = SearchParams {
                query,
                page,
                size,
                court,
                case_type,
                date_from,
                date_to,
                format,
            };
            search_precedents(client.as_ref(), params).await
        }
        Some(PrecedentCommand::Detail { id }) => {
            get_precedent_detail(client.as_ref(), id, format).await
        }
        None => {
            // Direct query without subcommand
            if let Some(query) = args.query {
                let params = SearchParams {
                    query,
                    page: args.page,
                    size: args.size,
                    court,
                    case_type,
                    date_from,
                    date_to,
                    format,
                };
                search_precedents(client.as_ref(), params).await
            } else {
                Err(WarpError::InvalidInput(
                    "No search query provided. Use 'warp precedent <query>' or 'warp precedent search <query>'".to_string()
                ))
            }
        }
    }
}

async fn search_precedents(client: &dyn LegalApiClient, params: SearchParams) -> Result<()> {
    if params.query.trim().is_empty() {
        return Err(WarpError::InvalidInput(
            "Search query cannot be empty".to_string(),
        ));
    }

    let mut extras = HashMap::new();
    if let Some(court) = params.court {
        extras.insert("court".to_string(), court);
    }
    if let Some(case_type) = params.case_type {
        extras.insert("case_type".to_string(), case_type);
    }

    let request = UnifiedSearchRequest {
        query: params.query,
        page_no: params.page,
        page_size: params.size,
        response_type: ResponseType::Json,
        date_from: params.date_from,
        date_to: params.date_to,
        extras,
        ..Default::default()
    };

    let response = client.search(request).await?;

    if response.items.is_empty() {
        println!("No precedents found for your search query.");
        return Ok(());
    }

    let output = output::format_search_response(&response, params.format)?;
    println!("{}", output);

    Ok(())
}

async fn get_precedent_detail(
    client: &dyn LegalApiClient,
    id: String,
    format: OutputFormat,
) -> Result<()> {
    let detail = client.get_detail(&id).await?;
    let output = output::format_law_detail(&detail, format)?;
    println!("{}", output);
    Ok(())
}
