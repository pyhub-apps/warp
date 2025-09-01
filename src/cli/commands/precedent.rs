use crate::api::{ApiType, ApiClientFactory};
use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::types::{UnifiedSearchRequest, ResponseType};
use crate::cli::args::{PrecedentArgs, PrecedentCommand};
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;
use std::collections::HashMap;

/// Execute precedent command (판례)
pub async fn execute(args: PrecedentArgs, format: OutputFormat) -> Result<()> {
    // Load configuration
    let config = Config::load()?;
    let api_key = config.get_nlic_api_key()  // Use same key as NLIC for now
        .ok_or(WarpError::NoApiKey)?;
    
    // Create API client
    let client_config = ClientConfig {
        api_key,
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
            search_precedents(
                client.as_ref(), 
                query, 
                page, 
                size, 
                court, 
                case_type,
                date_from,
                date_to,
                format
            ).await
        }
        Some(PrecedentCommand::Detail { id }) => {
            get_precedent_detail(client.as_ref(), id, format).await
        }
        None => {
            // Direct query without subcommand
            if let Some(query) = args.query {
                search_precedents(
                    client.as_ref(), 
                    query, 
                    args.page, 
                    args.size, 
                    court, 
                    case_type,
                    date_from,
                    date_to,
                    format
                ).await
            } else {
                Err(WarpError::InvalidInput(
                    "No search query provided. Use 'warp precedent <query>' or 'warp precedent search <query>'".to_string()
                ))
            }
        }
    }
}

async fn search_precedents(
    client: &dyn LegalApiClient,
    query: String,
    page: u32,
    size: u32,
    court: Option<String>,
    case_type: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    if query.trim().is_empty() {
        return Err(WarpError::InvalidInput("Search query cannot be empty".to_string()));
    }
    
    let mut extras = HashMap::new();
    if let Some(court) = court {
        extras.insert("court".to_string(), court);
    }
    if let Some(case_type) = case_type {
        extras.insert("case_type".to_string(), case_type);
    }
    
    let request = UnifiedSearchRequest {
        query,
        page_no: page,
        page_size: size,
        response_type: ResponseType::Json,
        date_from,
        date_to,
        extras,
        ..Default::default()
    };
    
    let response = client.search(request).await?;
    
    if response.items.is_empty() {
        println!("No precedents found for your search query.");
        return Ok(());
    }
    
    let output = output::format_search_response(&response, format)?;
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