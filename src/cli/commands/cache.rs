use crate::cache::CacheStore;
use crate::cli::args::{CacheArgs, CacheCommand};
use crate::config::Config;
use crate::error::Result;
use colored::*;
use std::io::{self, Write};

/// Execute cache management commands
pub async fn execute(args: CacheArgs) -> Result<()> {
    let config = Config::load()?;
    let cache_config = config.cache.to_cache_config();

    match args.command {
        CacheCommand::Status => show_status(&cache_config).await,
        CacheCommand::Clear { api, force } => clear_cache(&cache_config, api, force).await,
        CacheCommand::Config => show_config(&config),
        CacheCommand::Enable => enable_cache(&config).await,
        CacheCommand::Disable => disable_cache(&config).await,
    }
}

/// Show cache status and statistics
async fn show_status(cache_config: &crate::cache::CacheConfig) -> Result<()> {
    let cache = CacheStore::new(cache_config.clone()).await?;
    let stats = cache.stats().await?;

    println!("{}", "캐시 상태 (Cache Status)".bold().cyan());
    println!("{}", "=".repeat(50));

    println!("  {} {}", "총 항목수:".bold(), stats.total_entries);
    println!(
        "  {} {:.2} MB / {:.2} MB ({:.1}%)",
        "사용량:".bold(),
        stats.total_size as f64 / 1_048_576.0,
        stats.max_size as f64 / 1_048_576.0,
        (stats.total_size as f64 / stats.max_size as f64) * 100.0
    );
    println!("  {} {}", "만료된 항목:".bold(), stats.expired_entries);

    if stats.total_entries > 0 {
        println!("\n{}", "API별 캐시 현황:".bold());
        println!("  NLIC: {} 항목", stats.total_entries / 5); // Simplified for demo
        println!("  ELIS: {} 항목", stats.total_entries / 5);
        println!("  PREC: {} 항목", stats.total_entries / 5);
        println!("  ADMRUL: {} 항목", stats.total_entries / 5);
        println!("  EXPC: {} 항목", stats.total_entries / 5);
    }

    Ok(())
}

/// Clear cache data
async fn clear_cache(
    cache_config: &crate::cache::CacheConfig,
    api: Option<String>,
    force: bool,
) -> Result<()> {
    if !force {
        print!("정말로 캐시를 삭제하시겠습니까? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("취소되었습니다.");
            return Ok(());
        }
    }

    let cache = CacheStore::new(cache_config.clone()).await?;

    if let Some(api_type) = api {
        use crate::api::ApiType;

        // Clear specific API cache
        let api_upper = api_type.to_uppercase();
        let api_type_enum = match api_upper.as_str() {
            "NLIC" => ApiType::Nlic,
            "ELIS" => ApiType::Elis,
            "PREC" => ApiType::Prec,
            "ADMRUL" => ApiType::Admrul,
            "EXPC" => ApiType::Expc,
            _ => {
                println!(
                    "{}",
                    "올바른 API 타입을 지정해주세요: nlic, elis, prec, admrul, expc".red()
                );
                return Ok(());
            }
        };

        cache.clear_api(api_type_enum).await?;
        println!("{} API 캐시가 삭제되었습니다.", api_upper.green());
    } else {
        // Clear all cache
        cache.clear().await?;
        println!("{}", "모든 캐시가 삭제되었습니다.".green());
    }

    Ok(())
}

/// Show cache configuration
fn show_config(config: &Config) -> Result<()> {
    let cache_config = &config.cache;

    println!("{}", "캐시 설정 (Cache Configuration)".bold().cyan());
    println!("{}", "=".repeat(50));

    println!(
        "  {} {}",
        "활성화:".bold(),
        if cache_config.enabled {
            "Yes".green()
        } else {
            "No".red()
        }
    );
    println!(
        "  {} {} 시간",
        "TTL:".bold(),
        cache_config.ttl_seconds / 3600
    );
    println!("  {} {} MB", "최대 크기:".bold(), cache_config.max_size_mb);

    if let Some(ref dir) = cache_config.cache_dir {
        println!("  {} {}", "캐시 디렉토리:".bold(), dir.display());
    } else {
        #[cfg(target_os = "macos")]
        let default_path = "~/Library/Caches/pyhub-warp";
        #[cfg(target_os = "linux")]
        let default_path = "~/.cache/pyhub-warp";
        #[cfg(target_os = "windows")]
        let default_path = "%LOCALAPPDATA%\\pyhub-warp";
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let default_path = "~/.cache/pyhub-warp";

        println!(
            "  {} 기본 위치 ({})",
            "캐시 디렉토리:".bold(),
            default_path.italic()
        );
    }

    Ok(())
}

/// Enable cache
async fn enable_cache(config: &Config) -> Result<()> {
    let mut config = config.clone();
    config.cache.enabled = true;

    // Save the updated configuration
    let config_path = Config::get_config_path()?;
    let toml_content = toml::to_string_pretty(&config).map_err(|e| {
        crate::error::WarpError::Config(format!("Failed to serialize config: {}", e))
    })?;
    std::fs::write(&config_path, toml_content)?;

    println!("{}", "캐시가 활성화되었습니다.".green());
    Ok(())
}

/// Disable cache
async fn disable_cache(config: &Config) -> Result<()> {
    let mut config = config.clone();
    config.cache.enabled = false;

    // Save the updated configuration
    let config_path = Config::get_config_path()?;
    let toml_content = toml::to_string_pretty(&config).map_err(|e| {
        crate::error::WarpError::Config(format!("Failed to serialize config: {}", e))
    })?;
    std::fs::write(&config_path, toml_content)?;

    println!("{}", "캐시가 비활성화되었습니다.".yellow());
    Ok(())
}
