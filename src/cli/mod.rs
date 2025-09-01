pub mod commands;
pub mod args;
pub mod shell_detect;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// Korean Legal Information CLI
#[derive(Parser, Debug)]
#[command(
    name = "warp",
    about = "Korean Legal Information CLI - Search laws, ordinances, and legal documents from the terminal",
    version,
    author,
    long_about = None
)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable progress indicators
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable cache for this operation
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Output format
    #[arg(short, long, global = true, value_enum, default_value = "table")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    /// Table format (default)
    Table,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// CSV format
    Csv,
    /// HTML format
    Html,
    /// Simple HTML format
    HtmlSimple,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search and view laws (국가법령)
    #[command(alias = "l")]
    Law(args::LawArgs),
    
    /// Search and view local ordinances (자치법규)
    #[command(alias = "o")]
    Ordinance(args::OrdinanceArgs),
    
    /// Search precedents (판례)
    #[command(alias = "p")]
    Precedent(args::PrecedentArgs),
    
    /// Search administrative rules (행정규칙)
    #[command(alias = "a")]
    Admrule(args::AdmruleArgs),
    
    /// Search legal interpretations (법령해석례)
    #[command(alias = "i")]
    Interpretation(args::InterpretationArgs),
    
    /// Unified search across all sources
    #[command(alias = "s")]
    Search(args::SearchArgs),
    
    /// Manage configuration
    #[command(alias = "c")]
    Config(args::ConfigArgs),
    
    /// Manage cache
    Cache(args::CacheArgs),
    
    /// Show version information
    Version,
    
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for (auto-detect if not specified)
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
}

impl Cli {
    /// Generate shell completion scripts
    fn generate_completions(shell: Shell) {
        use clap::{Command, CommandFactory};
        use clap_complete::generate;
        use std::io;
        
        let mut cmd = Self::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
    }
    
    /// Run the CLI application
    pub async fn run() -> crate::error::Result<()> {
        let cli = Self::parse();
        
        // Set up logging
        if cli.verbose {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .init();
        } else {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
                .init();
        }
        
        let result = match cli.command {
            Commands::Law(args) => commands::law::execute(args, cli.format, cli.quiet, cli.verbose, cli.no_cache).await,
            Commands::Ordinance(args) => commands::ordinance::execute(args, cli.format, cli.quiet, cli.verbose, cli.no_cache).await,
            Commands::Precedent(args) => commands::precedent::execute(args, cli.format, cli.quiet, cli.verbose, cli.no_cache).await,
            Commands::Admrule(args) => commands::admrule::execute(args, cli.format, cli.quiet, cli.verbose, cli.no_cache).await,
            Commands::Interpretation(args) => commands::interpretation::execute(args, cli.format, cli.quiet, cli.verbose, cli.no_cache).await,
            Commands::Search(args) => commands::search::execute(args, cli.format, cli.quiet, cli.verbose, cli.no_cache).await,
            Commands::Config(args) => commands::config::execute(args).await,
            Commands::Cache(args) => commands::cache::execute(args).await,
            Commands::Version => {
                commands::version::execute();
                Ok(())
            }
            Commands::Completions { shell } => {
                use shell_detect::{detect_current_shell, confirm_shell_selection, get_completion_shell};
                
                let target_shell = if let Some(shell) = shell {
                    // User specified a shell explicitly
                    shell
                } else {
                    // Auto-detect current shell
                    match detect_current_shell() {
                        Some(detected) => {
                            // Ask user for confirmation
                            if confirm_shell_selection(&detected) {
                                // Get the appropriate shell for completion generation
                                match get_completion_shell(&detected) {
                                    Some(s) => s,
                                    None => {
                                        eprintln!("Unable to generate completions for the detected shell.");
                                        eprintln!("Please specify a shell manually:");
                                        eprintln!("  warp completions bash");
                                        eprintln!("  warp completions zsh");
                                        eprintln!("  warp completions fish");
                                        eprintln!("  warp completions powershell");
                                        return Ok(());
                                    }
                                }
                            } else {
                                // User declined
                                eprintln!("\nCompletion generation cancelled.");
                                eprintln!("You can manually generate completions with:");
                                eprintln!("  warp completions bash");
                                eprintln!("  warp completions zsh");
                                eprintln!("  warp completions fish");
                                eprintln!("  warp completions powershell");
                                return Ok(());
                            }
                        }
                        None => {
                            eprintln!("Unable to detect current shell.");
                            eprintln!("Please specify a shell:");
                            eprintln!("  warp completions bash");
                            eprintln!("  warp completions zsh");
                            eprintln!("  warp completions fish");
                            eprintln!("  warp completions powershell");
                            return Ok(());
                        }
                    }
                };
                
                Self::generate_completions(target_shell);
                Ok(())
            }
        };
        
        // Handle errors with better messaging
        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                use crate::error::WarpError;
                
                // Print main error message
                eprintln!("\n{}", e);
                
                // Print hint if available
                if let Some(hint) = e.hint() {
                    eprintln!("\n{}", hint);
                }
                
                // Add verbose suggestion for certain errors
                match &e {
                    WarpError::Parse(_) | WarpError::ApiError { .. } => {
                        if !cli.verbose {
                            eprintln!("\n💡 더 자세한 정보를 보려면 --verbose 옵션을 사용하세요");
                        }
                    }
                    _ => {}
                }
                
                Err(e)
            }
        }
    }
}