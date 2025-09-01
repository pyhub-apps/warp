use clap::{Args, Subcommand};

/// Cache management arguments
#[derive(Args, Debug)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommand,
}

#[derive(Subcommand, Debug)]
pub enum CacheCommand {
    /// Show cache status and statistics
    Status,
    
    /// Clear all cached data
    Clear {
        /// Clear only specific API cache (nlic, elis, prec, admrul, expc)
        #[arg(short, long)]
        api: Option<String>,
        
        /// Force clear without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show cache configuration
    Config,
    
    /// Enable cache
    Enable,
    
    /// Disable cache
    Disable,
}