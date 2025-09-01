use crate::progress::{ProgressManager};
use std::sync::Arc;
use crate::api::{ApiType, ApiClientFactory};
use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::types::{UnifiedSearchRequest, ResponseType};
use crate::cli::args::{OrdinanceArgs, OrdinanceCommand};
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;

/// Execute ordinance command (자치법규)
pub async fn execute(args: OrdinanceArgs, format: OutputFormat, quiet: bool, verbose: bool) -> Result<()> {
    // Load configuration
    let config = Config::load()?;
    let api_key = config.get_elis_api_key()
        .ok_or(WarpError::NoApiKey)?;
    
    // Create API client
    let client_config = ClientConfig {
        api_key,
        ..Default::default()
    };
    
    let client = ApiClientFactory::create(ApiType::Elis, client_config)?;
    
    // Extract common args before match
    let region = args.region.clone();
    let law_type = args.law_type.clone();
    
    // Handle direct query or subcommand
    match args.command {
        Some(OrdinanceCommand::Search { query, page, size }) => {
            search_ordinances(client.as_ref(), query, page, size, region, law_type, format).await
        }
        Some(OrdinanceCommand::Detail { id }) => {
            get_ordinance_detail(client.as_ref(), id, format).await
        }
        None => {
            // Direct query without subcommand
            if let Some(query) = args.query {
                search_ordinances(client.as_ref(), query, args.page, args.size, region, law_type, format).await
            } else {
                Err(WarpError::InvalidInput("No search query provided. Use 'warp ordinance <query>' or 'warp ordinance search <query>'".to_string()))
            }
        }
    }
}

async fn search_ordinances(
    client: &dyn LegalApiClient,
    query: String,
    page: u32,
    size: u32,
    region: Option<String>,
    law_type: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    if query.trim().is_empty() {
        return Err(WarpError::InvalidInput("Search query cannot be empty".to_string()));
    }
    
    let request = UnifiedSearchRequest {
        query,
        page_no: page,
        page_size: size,
        response_type: ResponseType::Json,
        region,
        law_type,
        ..Default::default()
    };
    
    let response = client.search(request).await?;
    
    if response.items.is_empty() {
        println!("No results found for your search query.");
        return Ok(());
    }
    
    let output = output::format_search_response(&response, format)?;
    println!("{}", output);
    
    Ok(())
}

async fn get_ordinance_detail(
    client: &dyn LegalApiClient,
    id: String,
    format: OutputFormat,
) -> Result<()> {
    let detail = client.get_detail(&id).await?;
    let output = output::format_law_detail(&detail, format)?;
    println!("{}", output);
    Ok(())
}