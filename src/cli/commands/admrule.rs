use crate::progress::{ProgressManager};
use std::sync::Arc;
use crate::api::{ApiClientFactory, ApiType};
use crate::api::client::ClientConfig;
use crate::api::types::UnifiedSearchRequest;
use crate::cli::args::AdmruleArgs;
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::Result;
use crate::output::formatter::Formatter;
use crate::cache::CacheStore;

/// Execute admrule (administrative rule) command
pub async fn execute(args: AdmruleArgs, format: OutputFormat, quiet: bool, verbose: bool, no_cache: bool) -> Result<()> {
    let config = Config::load()?;
    
    // Check for API key
    let api_key = config.get_admrul_api_key()
        .ok_or(crate::error::WarpError::NoApiKey)?;
    
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
    
    let client = ApiClientFactory::create(ApiType::Admrul, client_config)?;
    let formatter = Formatter::new(format);
    
    // Handle search query
    if let Some(query) = args.query {
        let request = UnifiedSearchRequest {
            query,
            page_no: args.page,
            page_size: args.size,
            ..Default::default()
        };
        
        let response = client.search(request).await?;
        let output = formatter.format_search(&response)?;
        println!("{}", output);
    } else {
        println!("Usage: warp admrule <QUERY>");
        println!("\nExample:");
        println!("  warp admrule \"개인정보\"");
    }
    
    Ok(())
}