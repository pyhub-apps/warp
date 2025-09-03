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

    /// Enable parallel search across multiple APIs for faster results
    #[arg(
        long,
        help = "Enable parallel search across multiple APIs (3-5x faster)"
    )]
    pub parallel: bool,

    /// APIs to search when using parallel mode (comma-separated: nlic,elis,prec,admrul,expc)
    #[arg(long, help = "Specify APIs for parallel search (e.g., nlic,elis,prec)")]
    pub apis: Option<String>,

    /// Enable request batching for improved performance
    #[arg(long, help = "Enable request batching and deduplication")]
    pub batch: bool,

    /// Batch size for request batching (1-50)
    #[arg(
        long,
        default_value = "10",
        help = "Number of requests per batch (1-50)"
    )]
    pub batch_size: u32,

    /// Enable tiered caching (1=basic, 2=advanced with compression)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=2), help = "Cache tier: 1=basic, 2=advanced")]
    pub cache_tier: Option<u8>,

    /// Maximum concurrent connections for parallel search
    #[arg(
        long,
        default_value = "5",
        help = "Max concurrent API connections (1-20)"
    )]
    pub max_concurrent: u32,

    /// Request timeout in seconds
    #[arg(long, default_value = "30", help = "Request timeout in seconds")]
    pub timeout: u32,

    /// Disable cache for this search
    #[arg(long, help = "Bypass all caching for fresh results")]
    pub no_cache: bool,

    // Advanced filtering options
    /// Law type filter (법률, 시행령, 시행규칙, 고시, 훈령, etc.)
    #[arg(long, help = "Law type filter (comma-separated: 법률,시행령,시행규칙)")]
    pub law_type: Option<String>,

    /// Department filter (부처명)
    #[arg(long, help = "Department filter (comma-separated: 법무부,행안부)")]
    pub department: Option<String>,

    /// Date range start (YYYYMMDD format)
    #[arg(long, help = "Start date for filtering (YYYYMMDD)")]
    pub from: Option<String>,

    /// Date range end (YYYYMMDD format)
    #[arg(long, help = "End date for filtering (YYYYMMDD)")]
    pub to: Option<String>,

    /// Recent days filter (alternative to from/to)
    #[arg(long, help = "Filter by recent N days")]
    pub recent_days: Option<u32>,

    /// Status filter (시행중, 폐지, 일부개정, etc.)
    #[arg(long, help = "Status filter (시행중,폐지,일부개정)")]
    pub status: Option<String>,

    /// Region filter for local ordinances
    #[arg(long, help = "Region filter for local ordinances (서울,부산,대구)")]
    pub region: Option<String>,

    /// Court filter for precedents
    #[arg(long, help = "Court filter for precedents (대법원,고등법원)")]
    pub court: Option<String>,

    /// Case type filter for precedents
    #[arg(long, help = "Case type filter for precedents (민사,형사,행정)")]
    pub case_type: Option<String>,

    /// Sort order
    #[arg(
        long,
        default_value = "relevance",
        help = "Sort order: relevance, date_asc, date_desc, title_asc, title_desc"
    )]
    pub sort: String,

    /// Enable regex search mode
    #[arg(long, help = "Enable regular expression search")]
    pub regex: bool,

    /// Search only in title
    #[arg(long, help = "Search only in document titles")]
    pub title_only: bool,

    /// Minimum relevance score (0.0-1.0)
    #[arg(long, help = "Minimum relevance score filter (0.0-1.0)")]
    pub min_score: Option<f32>,
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

/// Performance metrics command arguments
#[derive(Args, Debug)]
pub struct MetricsArgs {
    #[command(subcommand)]
    pub command: MetricsCommand,
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

#[derive(Subcommand, Debug)]
pub enum MetricsCommand {
    /// Show current performance metrics dashboard
    Show {
        /// Time window for metrics aggregation
        #[arg(long, default_value = "5m", help = "Time window: 1m, 5m, 15m, 1h, 24h")]
        window: String,

        /// Show detailed metrics including percentiles and error analysis
        #[arg(long, help = "Display detailed metrics and analysis")]
        details: bool,

        /// Refresh interval for continuous monitoring
        #[arg(long, help = "Auto-refresh interval (e.g., 5s, 1m)")]
        refresh: Option<String>,
    },

    /// Show performance dashboard (alias for show)
    Dashboard {
        /// Time window for metrics aggregation
        #[arg(long, default_value = "5m", help = "Time window: 1m, 5m, 15m, 1h, 24h")]
        window: String,

        /// Show detailed metrics
        #[arg(long, help = "Display detailed metrics and analysis")]
        details: bool,

        /// Refresh interval for continuous monitoring
        #[arg(long, help = "Auto-refresh interval (e.g., 5s, 1m)")]
        refresh: Option<String>,
    },

    /// Show historical performance data
    History {
        /// Number of hours to look back
        #[arg(long, help = "Hours of history to show")]
        hours: Option<u32>,

        /// Number of days to look back
        #[arg(long, help = "Days of history to show")]
        days: Option<u32>,

        /// Filter by specific API (nlic, elis, prec, admrul, expc)
        #[arg(long, help = "Filter by API type")]
        api: Option<String>,
    },

    /// Show cache performance metrics
    Cache,

    /// Show connection pool status
    Pools,

    /// Show detailed latency analysis with percentiles
    Latency {
        /// Percentiles to display (comma-separated)
        #[arg(
            long,
            default_value = "50,90,95,99",
            help = "Percentiles to show (e.g., 50,90,95,99)"
        )]
        percentiles: String,
    },

    /// Generate performance report
    Report {
        /// Start date for report (YYYY-MM-DD)
        #[arg(long, help = "Report start date")]
        from: Option<String>,

        /// End date for report (YYYY-MM-DD)
        #[arg(long, help = "Report end date")]
        to: Option<String>,

        /// Report output format (text, json, csv)
        #[arg(long, default_value = "text", help = "Report output format")]
        output_format: String,
    },

    /// Reset all metrics data
    Reset {
        /// Force reset without confirmation
        #[arg(long, help = "Force reset without confirmation")]
        force: bool,
    },

    /// Enable metrics collection
    Enable,

    /// Disable metrics collection
    Disable,

    /// Clean up old metrics data
    Cleanup {
        /// Remove data older than specified days
        #[arg(long, default_value = "30", help = "Remove data older than N days")]
        older_than: u32,

        /// Force cleanup without confirmation
        #[arg(long, help = "Force cleanup without confirmation")]
        force: bool,
    },
}
