use clap::{Args, Subcommand};

/// Law command arguments
#[derive(Args, Debug)]
pub struct LawArgs {
    #[command(subcommand)]
    pub command: Option<LawCommand>,
    
    /// Search query (can be used directly without subcommand)
    pub query: Option<String>,
    
    /// Page number
    #[arg(short, long, default_value = "1")]
    pub page: u32,
    
    /// Results per page
    #[arg(short = 's', long, default_value = "50")]
    pub size: u32,
    
    /// Law type filter
    #[arg(short = 't', long)]
    pub law_type: Option<String>,
    
    /// Department filter
    #[arg(short = 'd', long)]
    pub department: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum LawCommand {
    /// Search for laws
    Search {
        /// Search query
        query: String,
        
        /// Page number
        #[arg(short, long, default_value = "1")]
        page: u32,
        
        /// Results per page
        #[arg(short = 's', long, default_value = "50")]
        size: u32,
    },
    
    /// Get law details
    Detail {
        /// Law ID
        id: String,
    },
    
    /// Get law history
    History {
        /// Law ID
        id: String,
    },
}

/// Ordinance command arguments
#[derive(Args, Debug)]
pub struct OrdinanceArgs {
    #[command(subcommand)]
    pub command: Option<OrdinanceCommand>,
    
    /// Search query (can be used directly without subcommand)
    pub query: Option<String>,
    
    /// Page number
    #[arg(short, long, default_value = "1")]
    pub page: u32,
    
    /// Results per page
    #[arg(short = 's', long, default_value = "50")]
    pub size: u32,
    
    /// Region filter
    #[arg(short = 'r', long)]
    pub region: Option<String>,
    
    /// Law type filter
    #[arg(short = 't', long)]
    pub law_type: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum OrdinanceCommand {
    /// Search for ordinances
    Search {
        /// Search query
        query: String,
        
        /// Page number
        #[arg(short, long, default_value = "1")]
        page: u32,
        
        /// Results per page
        #[arg(short = 's', long, default_value = "50")]
        size: u32,
    },
    
    /// Get ordinance details
    Detail {
        /// Ordinance ID
        id: String,
    },
}

/// Precedent command arguments
#[derive(Args, Debug)]
pub struct PrecedentArgs {
    #[command(subcommand)]
    pub command: Option<PrecedentCommand>,
    
    /// Search query (can be used directly without subcommand)
    pub query: Option<String>,
    
    /// Page number
    #[arg(short, long, default_value = "1")]
    pub page: u32,
    
    /// Results per page
    #[arg(short = 's', long, default_value = "50")]
    pub size: u32,
    
    /// Court filter
    #[arg(short = 'c', long)]
    pub court: Option<String>,
    
    /// Case type filter
    #[arg(short = 't', long)]
    pub case_type: Option<String>,
    
    /// Date from (YYYYMMDD)
    #[arg(long)]
    pub date_from: Option<String>,
    
    /// Date to (YYYYMMDD)
    #[arg(long)]
    pub date_to: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum PrecedentCommand {
    /// Search for precedents
    Search {
        /// Search query
        query: String,
        
        /// Page number
        #[arg(short, long, default_value = "1")]
        page: u32,
        
        /// Results per page
        #[arg(short = 's', long, default_value = "50")]
        size: u32,
    },
    
    /// Get precedent details
    Detail {
        /// Precedent ID
        id: String,
    },
}

/// Administrative rule command arguments
#[derive(Args, Debug)]
pub struct AdmruleArgs {
    /// Search query
    pub query: Option<String>,
    
    /// Page number
    #[arg(short, long, default_value = "1")]
    pub page: u32,
    
    /// Results per page
    #[arg(short = 's', long, default_value = "50")]
    pub size: u32,
}

/// Legal interpretation command arguments
#[derive(Args, Debug)]
pub struct InterpretationArgs {
    /// Search query
    pub query: Option<String>,
    
    /// Page number
    #[arg(short, long, default_value = "1")]
    pub page: u32,
    
    /// Results per page
    #[arg(short = 's', long, default_value = "50")]
    pub size: u32,
}

/// Unified search command arguments
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
    
    /// Page number
    #[arg(short, long, default_value = "1")]
    pub page: u32,
    
    /// Results per page
    #[arg(short = 's', long, default_value = "50")]
    pub size: u32,
    
    /// Source to search (nlic, elis, all)
    #[arg(short = 'S', long, default_value = "all")]
    pub source: String,
}

/// Configuration command arguments
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

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

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., law.key)
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Show configuration file path
    Path,
    
    /// Initialize configuration
    Init,
}