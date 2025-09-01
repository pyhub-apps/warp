use crate::api::{ApiType, ApiClientFactory};
use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::types::{UnifiedSearchRequest, ResponseType};
use crate::cli::args::{LawArgs, LawCommand};
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;
use crate::progress::{ProgressManager, ApiProgress};
use std::sync::Arc;

/// Execute law command
pub async fn execute(args: LawArgs, format: OutputFormat, quiet: bool, verbose: bool) -> Result<()> {
    // Create progress manager
    let progress_manager = Arc::new(ProgressManager::new(quiet, verbose));
    
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
            search_laws(client.as_ref(), query, page, size, law_type, department, format, progress_manager).await
        }
        Some(LawCommand::Detail { id }) => {
            get_law_detail(client.as_ref(), id, format, progress_manager).await
        }
        Some(LawCommand::History { id }) => {
            get_law_history(client.as_ref(), id, format, progress_manager).await
        }
        None => {
            // Direct query without subcommand
            if let Some(query) = args.query {
                search_laws(client.as_ref(), query, args.page, args.size, law_type, department, format, progress_manager).await
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
    progress_manager: Arc<ProgressManager>,
) -> Result<()> {
    if query.trim().is_empty() {
        return Err(WarpError::InvalidInput("Search query cannot be empty".to_string()));
    }
    
    let request = UnifiedSearchRequest {
        query: query.clone(),
        page_no: page,
        page_size: size,
        response_type: ResponseType::Json,
        law_type,
        department,
        ..Default::default()
    };
    
    // Show progress while searching
    let progress = ApiProgress::new(progress_manager.clone(), "국가법령정보센터");
    progress.set_message(&format!("'{}' 검색 중...", query));
    
    let response = client.search(request).await?;
    
    progress.finish_with_message(&format!("검색 완료: {}개 결과", response.total_count));
    
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
    progress_manager: Arc<ProgressManager>,
) -> Result<()> {
    // Show progress while fetching details
    let progress = ApiProgress::new(progress_manager.clone(), "국가법령정보센터");
    progress.set_message(&format!("법령 상세 정보 조회 중... (ID: {})", id));
    
    let detail = client.get_detail(&id).await?;
    
    progress.finish_and_clear();
    let output = output::format_law_detail(&detail, format)?;
    println!("{}", output);
    Ok(())
}

async fn get_law_history(
    client: &dyn LegalApiClient,
    id: String,
    format: OutputFormat,
    progress_manager: Arc<ProgressManager>,
) -> Result<()> {
    // Show progress while fetching history
    let progress = ApiProgress::new(progress_manager.clone(), "국가법령정보센터");
    progress.set_message(&format!("법령 개정 이력 조회 중... (ID: {})", id));
    
    let history = client.get_history(&id).await?;
    
    progress.finish_and_clear();
    let output = output::format_law_history(&history, format)?;
    println!("{}", output);
    Ok(())
}