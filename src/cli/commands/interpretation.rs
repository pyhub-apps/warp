use crate::api::{ApiClientFactory, ApiType};
use crate::api::client::ClientConfig;
use crate::api::types::UnifiedSearchRequest;
use crate::cli::args::InterpretationArgs;
use crate::cli::OutputFormat;
use crate::config::Config;
use crate::error::Result;
use crate::output::formatter::Formatter;

/// Execute interpretation (legal interpretation) command
pub async fn execute(args: InterpretationArgs, format: OutputFormat) -> Result<()> {
    let config = Config::load()?;
    
    // Check for API key (use NLIC key since EXPC uses the same API)
    let api_key = config.law.nlic.key.as_ref()
        .or(config.law.key.as_ref())
        .ok_or(crate::error::WarpError::NoApiKey)?;
    
    let client_config = ClientConfig {
        api_key: api_key.clone(),
        ..Default::default()
    };
    
    let client = ApiClientFactory::create(ApiType::Expc, client_config)?;
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
        println!("Usage: warp interpretation <QUERY>");
        println!("\nExample:");
        println!("  warp interpretation \"건축법\"");
    }
    
    Ok(())
}