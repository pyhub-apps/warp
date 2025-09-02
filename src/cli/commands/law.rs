use crate::api::client::{ClientConfig, LegalApiClient};
use crate::api::types::{ResponseType, UnifiedSearchRequest};
use crate::api::{ApiClientFactory, ApiType};
use crate::cache::CacheStore;
use crate::cli::args::{LawArgs, LawCommand};
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::{Result, WarpError};
use crate::output;
use crate::progress::{messages, ApiStage, EnhancedApiProgress, ProgressManager};
use std::sync::Arc;

/// Parameters for law search operation
struct SearchParams {
    query: String,
    page: u32,
    size: u32,
    law_type: Option<String>,
    department: Option<String>,
    format: OutputFormat,
}

/// Execute law command
pub async fn execute(
    args: LawArgs,
    format: OutputFormat,
    quiet: bool,
    verbose: bool,
    no_cache: bool,
) -> Result<()> {
    // Create progress manager
    let progress_manager = Arc::new(ProgressManager::new(quiet, verbose));

    // Load configuration
    let config = Config::load()?;
    let api_key = config.get_nlic_api_key().ok_or(WarpError::NoApiKey)?;

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

    let client = ApiClientFactory::create(ApiType::Nlic, client_config)?;

    // Extract common args before match
    let law_type = args.law_type.clone();
    let department = args.department.clone();

    // Handle direct query or subcommand
    match args.command {
        Some(LawCommand::Search { query, page, size }) => {
            let params = SearchParams {
                query,
                page,
                size,
                law_type,
                department,
                format,
            };
            search_laws(client.as_ref(), params, progress_manager).await
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
                let params = SearchParams {
                    query,
                    page: args.page,
                    size: args.size,
                    law_type,
                    department,
                    format,
                };
                search_laws(client.as_ref(), params, progress_manager).await
            } else {
                Err(WarpError::InvalidInput(
                    "No search query provided. Use 'warp law <query>' or 'warp law search <query>'"
                        .to_string(),
                ))
            }
        }
    }
}

async fn search_laws(
    client: &dyn LegalApiClient,
    params: SearchParams,
    progress_manager: Arc<ProgressManager>,
) -> Result<()> {
    if params.query.trim().is_empty() {
        return Err(WarpError::InvalidInput(
            "Search query cannot be empty".to_string(),
        ));
    }

    let request = UnifiedSearchRequest {
        query: params.query.clone(),
        page_no: params.page,
        page_size: params.size,
        response_type: ResponseType::Json,
        law_type: params.law_type,
        department: params.department,
        ..Default::default()
    };

    // Show enhanced progress with stages
    let mut progress = EnhancedApiProgress::new(progress_manager.clone(), "국가법령정보센터");

    // Stage 1: Connecting
    progress.advance_stage(
        ApiStage::Connecting,
        &format!("'{}' 검색을 위한 연결 중", params.query),
    );

    // Stage 2: Searching
    progress.advance_stage(
        ApiStage::Searching,
        &format!("'{}' 검색 요청 전송 중", params.query),
    );

    let response = client.search(request).await?;

    // Stage 3: Parsing
    progress.advance_stage(ApiStage::Parsing, "응답 데이터 파싱 중");

    // Stage 4: Complete
    let result_message = messages::search_complete_with_time(
        "국가법령정보센터",
        response.total_count as usize,
        progress.elapsed().as_millis() as u64,
    );
    progress.complete_success(&result_message);

    if response.items.is_empty() {
        println!("No results found for your search query.");
        return Ok(());
    }

    let output = output::format_search_response(&response, params.format)?;
    println!("{}", output);

    Ok(())
}

async fn get_law_detail(
    client: &dyn LegalApiClient,
    id: String,
    format: OutputFormat,
    progress_manager: Arc<ProgressManager>,
) -> Result<()> {
    // Show enhanced progress for detail retrieval
    let mut progress = EnhancedApiProgress::new(progress_manager.clone(), "국가법령정보센터");

    progress.advance_stage(
        ApiStage::Connecting,
        &format!("법령 상세 정보 연결 중 (ID: {})", id),
    );
    progress.advance_stage(ApiStage::Searching, "상세 정보 요청 전송 중");

    let detail = client.get_detail(&id).await?;

    progress.advance_stage(ApiStage::Parsing, "상세 정보 파싱 중");
    progress.complete_success("법령 상세 정보 조회 완료");
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
    // Show enhanced progress for history retrieval
    let mut progress = EnhancedApiProgress::new(progress_manager.clone(), "국가법령정보센터");

    progress.advance_stage(
        ApiStage::Connecting,
        &format!("법령 개정 이력 연결 중 (ID: {})", id),
    );
    progress.advance_stage(ApiStage::Searching, "개정 이력 요청 전송 중");

    let history = client.get_history(&id).await?;

    progress.advance_stage(ApiStage::Parsing, "개정 이력 파싱 중");
    progress.complete_success("법령 개정 이력 조회 완료");
    let output = output::format_law_history(&history, format)?;
    println!("{}", output);
    Ok(())
}
