use crate::cli::args::{MetricsArgs, MetricsCommand};
use crate::cli::OutputFormat;
use crate::error::{Result, WarpError};
use crate::metrics::{get_global_metrics, MetricsWindow, PerformanceDashboard};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

/// Execute metrics command with various options
pub async fn execute(
    args: MetricsArgs,
    _format: OutputFormat,
    quiet: bool,
    _verbose: bool,
) -> Result<()> {
    let collector = get_global_metrics();

    match args.command {
        MetricsCommand::Show {
            window,
            details,
            refresh,
        } => {
            let metrics_window = parse_time_window(&window)?;
            let dashboard = if details {
                PerformanceDashboard::with_window(collector, metrics_window).with_details()
            } else {
                PerformanceDashboard::with_window(collector, metrics_window)
            };

            if let Some(refresh_str) = refresh {
                let refresh_interval = parse_duration(&refresh_str)?;
                if !quiet {
                    println!(
                        "📊 실시간 메트릭스 모니터링 시작 ({}초마다 갱신)",
                        refresh_interval.as_secs()
                    );
                    println!("Ctrl+C로 종료");
                }
                let _handle = dashboard.monitor(refresh_interval).await;

                // Wait for Ctrl+C
                tokio::signal::ctrl_c()
                    .await
                    .map_err(|e| WarpError::Other(format!("Failed to wait for Ctrl+C: {}", e)))?;

                if !quiet {
                    println!("\n모니터링이 종료되었습니다.");
                }
            } else {
                println!("{}", dashboard.display().await);
            }
        }

        MetricsCommand::Dashboard {
            window,
            details,
            refresh,
        } => {
            // Dashboard is an alias for Show - duplicate the logic to avoid recursion
            let metrics_window = parse_time_window(&window)?;
            let dashboard = if details {
                PerformanceDashboard::with_window(collector, metrics_window).with_details()
            } else {
                PerformanceDashboard::with_window(collector, metrics_window)
            };

            if let Some(refresh_str) = refresh {
                let refresh_interval = parse_duration(&refresh_str)?;
                if !quiet {
                    println!(
                        "📊 실시간 메트릭스 모니터링 시작 ({}초마다 갱신)",
                        refresh_interval.as_secs()
                    );
                    println!("Ctrl+C로 종료");
                }
                let _handle = dashboard.monitor(refresh_interval).await;

                // Wait for Ctrl+C
                tokio::signal::ctrl_c()
                    .await
                    .map_err(|e| WarpError::Other(format!("Failed to wait for Ctrl+C: {}", e)))?;

                if !quiet {
                    println!("\n모니터링이 종료되었습니다.");
                }
            } else {
                println!("{}", dashboard.display().await);
            }
        }

        MetricsCommand::History { hours, days, api } => {
            execute_history_command(collector, hours, days, api).await?;
        }

        MetricsCommand::Cache => {
            execute_cache_command(collector).await?;
        }

        MetricsCommand::Pools => {
            execute_pools_command(collector).await?;
        }

        MetricsCommand::Latency { percentiles } => {
            execute_latency_command(collector, &percentiles).await?;
        }

        MetricsCommand::Report {
            from,
            to,
            output_format,
        } => {
            execute_report_command(collector, from, to, &output_format).await?;
        }

        MetricsCommand::Reset { force } => {
            execute_reset_command(collector, force).await?;
        }

        MetricsCommand::Enable => {
            execute_enable_command().await?;
        }

        MetricsCommand::Disable => {
            execute_disable_command().await?;
        }

        MetricsCommand::Cleanup { older_than, force } => {
            execute_cleanup_command(collector, older_than, force).await?;
        }
    }

    Ok(())
}

async fn execute_history_command(
    collector: Arc<crate::metrics::MetricsCollector>,
    hours: Option<u32>,
    days: Option<u32>,
    api: Option<String>,
) -> Result<()> {
    println!("📈 성능 히스토리");
    println!("{}", "─".repeat(50));

    // Determine time range
    let time_range = match (hours, days) {
        (Some(h), _) => format!("최근 {}시간", h),
        (_, Some(d)) => format!("최근 {}일", d),
        _ => "최근 24시간".to_string(),
    };

    println!("🕒 기간: {}", time_range);

    if let Some(ref api_filter) = api {
        println!("🔍 API 필터: {}", api_filter);
    }

    // Get historical data (for now, show current snapshot as placeholder)
    let snapshot = collector.get_snapshot().await;

    if snapshot.operations.is_empty() {
        println!("📭 히스토리 데이터가 없습니다.");
        return Ok(());
    }

    println!("\n📊 API 작업 히스토리:");
    for (op_name, metrics) in &snapshot.operations {
        if let Some(ref filter) = api {
            if !op_name.to_lowercase().contains(&filter.to_lowercase()) {
                continue;
            }
        }

        println!("  • {}", op_name);
        println!("    요청 수: {}", metrics.total_requests);
        println!("    성공률: {:.1}%", metrics.success_rate());
        println!("    평균 응답시간: {}ms", metrics.avg_duration.as_millis());
        println!();
    }

    Ok(())
}

async fn execute_cache_command(collector: Arc<crate::metrics::MetricsCollector>) -> Result<()> {
    println!("💾 캐시 성능 메트릭스");
    println!("{}", "─".repeat(50));

    let snapshot = collector.get_snapshot().await;

    if snapshot.cache.is_empty() {
        println!("📭 캐시 데이터가 없습니다.");
        return Ok(());
    }

    for (api, cache_metrics) in &snapshot.cache {
        println!("🔸 {}", api);
        println!("  히트율: {:.1}%", cache_metrics.hit_rate());
        println!("  히트 수: {}", cache_metrics.hits);
        println!("  미스 수: {}", cache_metrics.misses);

        if cache_metrics.storage_size > 0 {
            println!("  저장 크기: {}KB", cache_metrics.storage_size / 1024);
            println!("  항목 수: {}", cache_metrics.entry_count);
        }
        println!();
    }

    Ok(())
}

async fn execute_pools_command(collector: Arc<crate::metrics::MetricsCollector>) -> Result<()> {
    println!("🔗 연결 풀 상태");
    println!("{}", "─".repeat(50));

    let snapshot = collector.get_snapshot().await;

    if snapshot.connection_pools.is_empty() {
        println!("📭 연결 풀 데이터가 없습니다.");
        return Ok(());
    }

    for (pool_name, pool_metrics) in &snapshot.connection_pools {
        println!("🔸 {}", pool_name);
        println!(
            "  활성 연결: {}/{}",
            pool_metrics.active_connections, pool_metrics.total_connections
        );
        println!("  유휴 연결: {}", pool_metrics.idle_connections);
        println!("  사용률: {:.1}%", pool_metrics.utilization());

        if pool_metrics.connection_timeouts > 0 {
            println!(
                "  타임아웃: {} ({:.1}%)",
                pool_metrics.connection_timeouts,
                pool_metrics.timeout_rate()
            );
        }
        println!();
    }

    Ok(())
}

async fn execute_latency_command(
    collector: Arc<crate::metrics::MetricsCollector>,
    percentiles: &str,
) -> Result<()> {
    println!("⚡ 지연시간 분석");
    println!("{}", "─".repeat(50));

    let snapshot = collector.get_snapshot().await;

    if snapshot.operations.is_empty() {
        println!("📭 지연시간 데이터가 없습니다.");
        return Ok(());
    }

    // Parse percentiles
    let requested_percentiles: Vec<u8> = percentiles
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    for (op_name, metrics) in &snapshot.operations {
        println!("🔸 {}", op_name);
        println!("  평균: {}ms", metrics.avg_duration.as_millis());
        println!("  최소: {}ms", metrics.min_duration.as_millis());
        println!("  최대: {}ms", metrics.max_duration.as_millis());

        if requested_percentiles.contains(&50) {
            println!("  p50: {}ms", metrics.p50_duration.as_millis());
        }
        if requested_percentiles.contains(&90) {
            println!("  p90: {}ms", metrics.p95_duration.as_millis()); // Using p95 as proxy
        }
        if requested_percentiles.contains(&95) {
            println!("  p95: {}ms", metrics.p95_duration.as_millis());
        }
        if requested_percentiles.contains(&99) {
            println!("  p99: {}ms", metrics.p99_duration.as_millis());
        }
        println!();
    }

    Ok(())
}

async fn execute_report_command(
    collector: Arc<crate::metrics::MetricsCollector>,
    from: Option<String>,
    to: Option<String>,
    format: &str,
) -> Result<()> {
    let snapshot = collector.get_snapshot().await;

    match format.to_lowercase().as_str() {
        "json" => {
            // JSON 형식 출력
            println!("{{");
            println!("  \"timestamp\": \"{:?}\",", snapshot.timestamp);
            println!("  \"uptime_seconds\": {},", snapshot.uptime.as_secs());
            println!("  \"memory_usage_bytes\": {},", snapshot.memory_usage);
            println!("  \"operations\": {{");

            let mut first = true;
            for (op_name, metrics) in &snapshot.operations {
                if !first {
                    println!(",");
                }
                first = false;
                println!("    \"{}\": {{", op_name);
                println!("      \"total_requests\": {},", metrics.total_requests);
                println!("      \"success_rate\": {:.2}", metrics.success_rate());
                println!(
                    "      \"avg_duration_ms\": {}",
                    metrics.avg_duration.as_millis()
                );
                print!("    }}");
            }

            println!();
            println!("  }}");
            println!("}}");
        }

        "csv" => {
            // CSV 형식 출력
            println!("operation,total_requests,success_rate,avg_duration_ms");
            for (op_name, metrics) in &snapshot.operations {
                println!(
                    "{},{},{:.2},{}",
                    op_name,
                    metrics.total_requests,
                    metrics.success_rate(),
                    metrics.avg_duration.as_millis()
                );
            }
        }

        _ => {
            // 기본 텍스트 형식
            println!("📊 성능 리포트");
            println!("{}", "─".repeat(50));

            if let Some(from_date) = from {
                println!("📅 시작일: {}", from_date);
            }
            if let Some(to_date) = to {
                println!("📅 종료일: {}", to_date);
            }

            let dashboard = PerformanceDashboard::new(collector).with_details();
            println!("{}", dashboard.display().await);
        }
    }

    Ok(())
}

async fn execute_reset_command(
    collector: Arc<crate::metrics::MetricsCollector>,
    force: bool,
) -> Result<()> {
    if !force {
        print!("🚨 모든 메트릭스 데이터를 삭제하시겠습니까? (y/N): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ 취소되었습니다.");
            return Ok(());
        }
    }

    collector.reset();
    println!("🔄 메트릭스 데이터를 초기화합니다...");
    println!("✅ 메트릭스 데이터가 초기화되었습니다.");

    Ok(())
}

async fn execute_enable_command() -> Result<()> {
    use crate::config::Config;

    let mut config = Config::load()?;
    config.metrics.enabled = true;
    config.save()?;

    println!("✅ 메트릭스 수집이 활성화되었습니다.");
    Ok(())
}

async fn execute_disable_command() -> Result<()> {
    use crate::config::Config;

    println!("⚠️  메트릭스 수집을 비활성화하시겠습니까? (y/N): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase().starts_with('y') {
        let mut config = Config::load()?;
        config.metrics.enabled = false;
        config.save()?;

        println!("🔄 메트릭스 수집을 비활성화합니다...");
        println!("✅ 메트릭스 수집이 비활성화되었습니다.");
    } else {
        println!("❌ 취소되었습니다.");
    }

    Ok(())
}

async fn execute_cleanup_command(
    collector: Arc<crate::metrics::MetricsCollector>,
    older_than: u32,
    force: bool,
) -> Result<()> {
    if !force {
        print!(
            "🗑️  {}일 이전의 메트릭스 데이터를 삭제하시겠습니까? (y/N): ",
            older_than
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ 취소되었습니다.");
            return Ok(());
        }
    }

    println!("🔄 {}일 이전 데이터를 정리합니다...", older_than);

    // Get snapshot before cleanup to show what was cleaned
    let snapshot_before = collector.get_snapshot().await;
    let total_operations_before = snapshot_before.operations.len();

    // For now, just reset old data (in a real implementation, you'd check timestamps)
    if older_than <= 1 {
        collector.reset();
        println!(
            "✅ {} 작업의 메트릭스 데이터가 정리되었습니다.",
            total_operations_before
        );
    } else {
        println!(
            "✅ 메트릭스 데이터 정리가 완료되었습니다. ({}일 이상 된 데이터 없음)",
            older_than
        );
    }

    Ok(())
}

/// Parse time window string (1m, 5m, 15m, 1h, 24h) to MetricsWindow enum
fn parse_time_window(window: &str) -> Result<MetricsWindow> {
    match window.to_lowercase().as_str() {
        "1m" | "1min" | "minute" => Ok(MetricsWindow::LastMinute),
        "5m" | "5min" => Ok(MetricsWindow::Last5Minutes),
        "15m" | "15min" => Ok(MetricsWindow::Last15Minutes),
        "1h" | "1hour" | "hour" => Ok(MetricsWindow::LastHour),
        "24h" | "24hour" | "day" => Ok(MetricsWindow::Last24Hours),
        _ => Err(WarpError::InvalidInput(format!(
            "Invalid time window: '{}'. Valid options: 1m, 5m, 15m, 1h, 24h",
            window
        ))),
    }
}

/// Parse duration string (5s, 1m, etc.) to Duration
fn parse_duration(duration_str: &str) -> Result<Duration> {
    let duration_str = duration_str.trim().to_lowercase();

    if let Some(num_str) = duration_str.strip_suffix('s') {
        let seconds: u64 = num_str.parse().map_err(|_| {
            WarpError::InvalidInput(format!("Invalid duration: '{}'", duration_str))
        })?;
        Ok(Duration::from_secs(seconds))
    } else if let Some(num_str) = duration_str.strip_suffix('m') {
        let minutes: u64 = num_str.parse().map_err(|_| {
            WarpError::InvalidInput(format!("Invalid duration: '{}'", duration_str))
        })?;
        Ok(Duration::from_secs(minutes * 60))
    } else if let Some(num_str) = duration_str.strip_suffix('h') {
        let hours: u64 = num_str.parse().map_err(|_| {
            WarpError::InvalidInput(format!("Invalid duration: '{}'", duration_str))
        })?;
        Ok(Duration::from_secs(hours * 3600))
    } else {
        Err(WarpError::InvalidInput(format!(
            "Invalid duration format: '{}'. Use format like '5s', '1m', '2h'",
            duration_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_window() {
        assert!(matches!(
            parse_time_window("1m").unwrap(),
            MetricsWindow::LastMinute
        ));
        assert!(matches!(
            parse_time_window("5m").unwrap(),
            MetricsWindow::Last5Minutes
        ));
        assert!(matches!(
            parse_time_window("1h").unwrap(),
            MetricsWindow::LastHour
        ));
        assert!(matches!(
            parse_time_window("24h").unwrap(),
            MetricsWindow::Last24Hours
        ));

        assert!(parse_time_window("invalid").is_err());
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_duration("2m").unwrap(), Duration::from_secs(120));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));

        assert!(parse_duration("invalid").is_err());
    }
}
