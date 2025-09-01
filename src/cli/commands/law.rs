use crate::api::{ApiType, ApiClientFactory};
use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::types::{UnifiedSearchRequest, ResponseType};
use crate::cli::args::{LawArgs, LawCommand};
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;

/// Execute law command
pub async fn execute(args: LawArgs, format: OutputFormat) -> Result<()> {
    // Load configuration
    let config = Config::load()?;
    let api_key = config.get_nlic_api_key()
        .ok_or(WarpError::NoApiKey)?;
    
    // Create API client
    let client_config = ClientConfig {
        api_key,
        ..Default::default()
    };
    
    let client = ApiClientFactory::create(ApiType::Nlic, client_config)?;
    
    // Extract common args before match
    let law_type = args.law_type.clone();
    let department = args.department.clone();
    
    // Handle direct query or subcommand
    match args.command {
        Some(LawCommand::Search { query, page, size }) => {
            search_laws(client.as_ref(), query, page, size, law_type, department, format).await
        }
        Some(LawCommand::Detail { id }) => {
            get_law_detail(client.as_ref(), id, format).await
        }
        Some(LawCommand::History { id }) => {
            get_law_history(client.as_ref(), id, format).await
        }
        None => {
            // Direct query without subcommand
            if let Some(query) = args.query {
                search_laws(client.as_ref(), query, args.page, args.size, law_type, department, format).await
            } else {
                Err(WarpError::InvalidInput("No search query provided. Use 'warp law <query>' or 'warp law search <query>'".to_string()))
            }
        }
    }
}

async fn search_laws(
    client: &dyn LegalApiClient,
    query: String,
    page: u32,
    size: u32,
    law_type: Option<String>,
    department: Option<String>,
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
        law_type,
        department,
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

async fn get_law_detail(
    client: &dyn LegalApiClient,
    id: String,
    format: OutputFormat,
) -> Result<()> {
    let detail = client.get_detail(&id).await?;
    let output = output::format_law_detail(&detail, format)?;
    println!("{}", output);
    Ok(())
}

async fn get_law_history(
    client: &dyn LegalApiClient,
    id: String,
    format: OutputFormat,
) -> Result<()> {
    let history = client.get_history(&id).await?;
    let output = output::format_law_history(&history, format)?;
    println!("{}", output);
    Ok(())
}