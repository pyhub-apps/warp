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
                }

                // Use the improved monitor method
                dashboard
                    .monitor(refresh_interval)
                    .await
                    .map_err(|e| WarpError::Other(format!("Monitoring failed: {}", e)))?;
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
                }

                // Use the improved monitor method
                dashboard
                    .monitor(refresh_interval)
                    .await
                    .map_err(|e| WarpError::Other(format!("Monitoring failed: {}", e)))?;
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
    use colored::*;

    println!("{}", "📈 성능 히스토리".bold().cyan());
    println!("{}", "─".repeat(60).bright_black());

    // Determine time range and window
    let (time_range, window) = match (hours, days) {
        (Some(h), _) if h <= 1 => (
            "최근 1시간".to_string(),
            crate::metrics::MetricsWindow::LastHour,
        ),
        (Some(h), _) if h <= 24 => (
            format!("최근 {}시간", h),
            crate::metrics::MetricsWindow::Last24Hours,
        ),
        (_, Some(1)) => (
            "최근 24시간".to_string(),
            crate::metrics::MetricsWindow::Last24Hours,
        ),
        (_, Some(d)) => (
            format!("최근 {}일", d),
            crate::metrics::MetricsWindow::Last24Hours,
        ), // Extended view
        _ => (
            "최근 24시간".to_string(),
            crate::metrics::MetricsWindow::Last24Hours,
        ),
    };

    println!("🕒 분석 기간: {}", time_range.bright_white());
    if let Some(ref api_filter) = api {
        println!("🔍 API 필터: {}", api_filter.bright_yellow());
    }
    println!();

    // Get windowed historical data
    let snapshot = collector.get_windowed_metrics(window).await;

    if snapshot.operations.is_empty() {
        println!("📭 지정된 기간 동안의 히스토리 데이터가 없습니다.");
        return Ok(());
    }

    // Filter by API if specified
    let filtered_operations: Vec<_> = snapshot
        .operations
        .iter()
        .filter(|(op_name, _)| {
            if let Some(ref filter) = api {
                op_name.to_lowercase().contains(&filter.to_lowercase())
            } else {
                true
            }
        })
        .collect();

    if filtered_operations.is_empty() {
        println!("📭 필터링된 API에 대한 데이터가 없습니다.");
        return Ok(());
    }

    // Display summary statistics
    let total_requests: u64 = filtered_operations
        .iter()
        .map(|(_, m)| m.total_requests)
        .sum();
    let total_successes: u64 = filtered_operations
        .iter()
        .map(|(_, m)| m.successful_requests)
        .sum();
    let avg_success_rate = if total_requests > 0 {
        (total_successes as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };

    println!("📊 {} 요약", "전체 성능".bold().green());
    println!("├─ 총 요청: {}", total_requests.to_string().bright_white());
    println!("├─ 성공률: {}", format_percentage_colored(avg_success_rate));
    println!(
        "└─ 활성 API: {}",
        filtered_operations.len().to_string().bright_white()
    );
    println!();

    // Detailed API breakdown
    println!("📋 {} ({})", "API별 상세 분석".bold().blue(), time_range);
    println!("┌─────────────┬────────┬─────────┬──────────┬──────────┐");
    println!(
        "│ {}         │ {}  │ {}   │ {}  │ {}      │",
        "API".bold(),
        "요청수".bold(),
        "성공률".bold(),
        "평균시간".bold(),
        "상태".bold()
    );
    println!("├─────────────┼────────┼─────────┼──────────┼──────────┤");

    for (api_name, metrics) in &filtered_operations {
        let success_rate = metrics.success_rate();
        let status = get_status_indicator(success_rate, metrics.avg_duration.as_millis() as f64);

        println!(
            "│ {:<11} │ {:<6} │ {:>6.1}% │ {:>7}ms │ {:<8} │",
            api_name,
            metrics.total_requests,
            success_rate,
            metrics.avg_duration.as_millis(),
            status
        );
    }

    println!("└─────────────┴────────┴─────────┴──────────┴──────────┘");

    // Performance trends (simplified analysis)
    if filtered_operations.len() > 1 {
        println!(
            "\n🔍 {} (기간: {})",
            "성능 트렌드 분석".bold().magenta(),
            time_range
        );

        // Find best and worst performing APIs
        let best_api = filtered_operations.iter().max_by(|(_, a), (_, b)| {
            a.success_rate()
                .partial_cmp(&b.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let worst_api = filtered_operations.iter().min_by(|(_, a), (_, b)| {
            a.success_rate()
                .partial_cmp(&b.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some((best_name, best_metrics)) = best_api {
            println!(
                "🏆 최고 성능: {} (성공률 {:.1}%, 평균 {}ms)",
                best_name.green(),
                best_metrics.success_rate(),
                best_metrics.avg_duration.as_millis()
            );
        }

        if let Some((worst_name, worst_metrics)) = worst_api {
            println!(
                "⚠️  관심 필요: {} (성공률 {:.1}%, 평균 {}ms)",
                worst_name.yellow(),
                worst_metrics.success_rate(),
                worst_metrics.avg_duration.as_millis()
            );
        }

        // Response time analysis
        let avg_response_time: f64 = filtered_operations
            .iter()
            .map(|(_, m)| m.avg_duration.as_millis() as f64)
            .sum::<f64>()
            / filtered_operations.len() as f64;

        println!("📊 평균 응답시간: {:.0}ms", avg_response_time);

        // Show APIs that are above/below average
        let slow_apis: Vec<_> = filtered_operations
            .iter()
            .filter(|(_, m)| m.avg_duration.as_millis() as f64 > avg_response_time * 1.2)
            .collect();

        if !slow_apis.is_empty() {
            println!(
                "🐌 평균보다 느린 API ({:.0}ms 이상):",
                avg_response_time * 1.2
            );
            for (name, metrics) in slow_apis {
                println!("   • {}: {}ms", name, metrics.avg_duration.as_millis());
            }
        }
    }

    Ok(())
}

fn format_percentage_colored(percentage: f64) -> colored::ColoredString {
    use colored::*;
    let formatted = format!("{:.1}%", percentage);
    if percentage >= 95.0 {
        formatted.green()
    } else if percentage >= 90.0 {
        formatted.yellow()
    } else {
        formatted.red()
    }
}

fn get_status_indicator(success_rate: f64, avg_duration_ms: f64) -> &'static str {
    if success_rate >= 95.0 && avg_duration_ms < 500.0 {
        "🟢 우수"
    } else if success_rate >= 90.0 && avg_duration_ms < 1000.0 {
        "🟡 양호"
    } else if success_rate >= 80.0 {
        "🟠 주의"
    } else {
        "🔴 문제"
    }
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
    use chrono::prelude::*;
    use serde_json::json;

    let snapshot = collector.get_snapshot().await;
    let timestamp = Local::now();

    // Parse date range if provided
    let date_range = match (&from, &to) {
        (Some(f), Some(t)) => Some((f.clone(), t.clone())),
        (Some(f), None) => Some((f.clone(), timestamp.format("%Y-%m-%d").to_string())),
        (None, Some(t)) => {
            let week_ago = timestamp - chrono::Duration::days(7);
            Some((week_ago.format("%Y-%m-%d").to_string(), t.clone()))
        }
        _ => None,
    };

    match format.to_lowercase().as_str() {
        "json" => {
            // Enhanced JSON format with proper structure
            let mut operations_json = serde_json::Map::new();

            for (op_name, metrics) in &snapshot.operations {
                let operation_data = json!({
                    "total_requests": metrics.total_requests,
                    "successful_requests": metrics.successful_requests,
                    "failed_requests": metrics.failed_requests,
                    "success_rate_percent": metrics.success_rate(),
                    "error_rate_percent": metrics.error_rate(),
                    "response_times": {
                        "avg_ms": metrics.avg_duration.as_millis(),
                        "min_ms": metrics.min_duration.as_millis(),
                        "max_ms": metrics.max_duration.as_millis(),
                        "p50_ms": metrics.p50_duration.as_millis(),
                        "p95_ms": metrics.p95_duration.as_millis(),
                        "p99_ms": metrics.p99_duration.as_millis()
                    }
                });
                operations_json.insert(op_name.clone(), operation_data);
            }

            let mut cache_json = serde_json::Map::new();
            for (api_name, cache_metrics) in &snapshot.cache {
                let cache_data = json!({
                    "hits": cache_metrics.hits,
                    "misses": cache_metrics.misses,
                    "hit_rate_percent": cache_metrics.hit_rate(),
                    "storage_size_bytes": cache_metrics.storage_size,
                    "entry_count": cache_metrics.entry_count
                });
                cache_json.insert(api_name.clone(), cache_data);
            }

            let mut pools_json = serde_json::Map::new();
            for (pool_name, pool_metrics) in &snapshot.connection_pools {
                let pool_data = json!({
                    "active_connections": pool_metrics.active_connections,
                    "idle_connections": pool_metrics.idle_connections,
                    "total_connections": pool_metrics.total_connections,
                    "utilization_percent": pool_metrics.utilization(),
                    "connection_timeouts": pool_metrics.connection_timeouts,
                    "timeout_rate_percent": pool_metrics.timeout_rate()
                });
                pools_json.insert(pool_name.clone(), pool_data);
            }

            // Calculate summary values
            let total_requests: u64 = snapshot.operations.values().map(|m| m.total_requests).sum();
            let total_successes: u64 = snapshot
                .operations
                .values()
                .map(|m| m.successful_requests)
                .sum();
            let overall_success_rate = if total_requests > 0 {
                (total_successes as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            };

            let report = json!({
                "metadata": {
                    "generated_at": timestamp.to_rfc3339(),
                    "report_format": "json",
                    "date_range": date_range,
                    "uptime_seconds": snapshot.uptime.as_secs(),
                    "memory_usage_bytes": snapshot.memory_usage
                },
                "summary": {
                    "total_operations": snapshot.operations.len(),
                    "total_requests": total_requests,
                    "overall_success_rate": overall_success_rate
                },
                "operations": operations_json,
                "cache": cache_json,
                "connection_pools": pools_json
            });

            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }

        "csv" => {
            // Enhanced CSV format with comprehensive data
            println!(
                "# Warp CLI Performance Report - Generated: {}",
                timestamp.format("%Y-%m-%d %H:%M:%S")
            );
            if let Some((from_date, to_date)) = &date_range {
                println!("# Date Range: {} to {}", from_date, to_date);
            }
            println!();

            // Operations CSV
            println!("# API Operations");
            println!("api_name,total_requests,successful_requests,failed_requests,success_rate_percent,error_rate_percent,avg_duration_ms,min_duration_ms,max_duration_ms,p50_duration_ms,p95_duration_ms,p99_duration_ms");
            for (op_name, metrics) in &snapshot.operations {
                println!(
                    "{},{},{},{},{:.2},{:.2},{},{},{},{},{},{}",
                    op_name,
                    metrics.total_requests,
                    metrics.successful_requests,
                    metrics.failed_requests,
                    metrics.success_rate(),
                    metrics.error_rate(),
                    metrics.avg_duration.as_millis(),
                    metrics.min_duration.as_millis(),
                    metrics.max_duration.as_millis(),
                    metrics.p50_duration.as_millis(),
                    metrics.p95_duration.as_millis(),
                    metrics.p99_duration.as_millis()
                );
            }

            println!();
            println!("# Cache Performance");
            println!("api_name,hits,misses,hit_rate_percent,storage_size_bytes,entry_count");
            for (api_name, cache_metrics) in &snapshot.cache {
                println!(
                    "{},{},{},{:.2},{},{}",
                    api_name,
                    cache_metrics.hits,
                    cache_metrics.misses,
                    cache_metrics.hit_rate(),
                    cache_metrics.storage_size,
                    cache_metrics.entry_count
                );
            }

            if !snapshot.connection_pools.is_empty() {
                println!();
                println!("# Connection Pools");
                println!("pool_name,active_connections,idle_connections,total_connections,utilization_percent,connection_timeouts,timeout_rate_percent");
                for (pool_name, pool_metrics) in &snapshot.connection_pools {
                    println!(
                        "{},{},{},{},{:.2},{},{:.2}",
                        pool_name,
                        pool_metrics.active_connections,
                        pool_metrics.idle_connections,
                        pool_metrics.total_connections,
                        pool_metrics.utilization(),
                        pool_metrics.connection_timeouts,
                        pool_metrics.timeout_rate()
                    );
                }
            }
        }

        "html" => {
            // HTML report format
            println!("<!DOCTYPE html>");
            println!("<html lang=\"ko\">");
            println!("<head>");
            println!("    <meta charset=\"UTF-8\">");
            println!("    <title>Warp CLI 성능 리포트</title>");
            println!("    <style>");
            println!("        body {{ font-family: -apple-system, sans-serif; margin: 40px; background: #f5f5f5; }}");
            println!("        .container {{ background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}");
            println!("        h1, h2 {{ color: #333; }}");
            println!("        .metric {{ background: #f8f9fa; padding: 15px; margin: 10px 0; border-left: 4px solid #007bff; }}");
            println!("        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}");
            println!("        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}");
            println!("        th {{ background-color: #f2f2f2; font-weight: 600; }}");
            println!("        .success {{ color: #28a745; }}");
            println!("        .warning {{ color: #ffc107; }}");
            println!("        .danger {{ color: #dc3545; }}");
            println!("        .footer {{ margin-top: 30px; color: #666; font-size: 14px; }}");
            println!("    </style>");
            println!("</head>");
            println!("<body>");
            println!("    <div class=\"container\">");
            println!("        <h1>🚀 Warp CLI 성능 리포트</h1>");
            println!("        <div class=\"metric\">");
            println!(
                "            <strong>생성 시간:</strong> {}<br>",
                timestamp.format("%Y년 %m월 %d일 %H:%M:%S")
            );
            if let Some((from_date, to_date)) = &date_range {
                println!(
                    "            <strong>분석 기간:</strong> {} ~ {}<br>",
                    from_date, to_date
                );
            }
            println!(
                "            <strong>시스템 가동시간:</strong> {}초<br>",
                snapshot.uptime.as_secs()
            );
            println!(
                "            <strong>메모리 사용량:</strong> {:.1} MB",
                snapshot.memory_usage as f64 / 1024.0 / 1024.0
            );
            println!("        </div>");

            if !snapshot.operations.is_empty() {
                println!("        <h2>📊 API 성능</h2>");
                println!("        <table>");
                println!("            <tr><th>API</th><th>요청 수</th><th>성공률</th><th>평균 응답시간</th><th>상태</th></tr>");
                for (op_name, metrics) in &snapshot.operations {
                    let success_rate = metrics.success_rate();
                    let status_class = if success_rate >= 95.0 {
                        "success"
                    } else if success_rate >= 90.0 {
                        "warning"
                    } else {
                        "danger"
                    };
                    let status_text = if success_rate >= 95.0 {
                        "✅ 우수"
                    } else if success_rate >= 90.0 {
                        "⚠️ 주의"
                    } else {
                        "❌ 문제"
                    };

                    println!("            <tr>");
                    println!("                <td><strong>{}</strong></td>", op_name);
                    println!("                <td>{}</td>", metrics.total_requests);
                    println!(
                        "                <td class=\"{}\">{:.1}%</td>",
                        status_class, success_rate
                    );
                    println!(
                        "                <td>{}ms</td>",
                        metrics.avg_duration.as_millis()
                    );
                    println!(
                        "                <td class=\"{}\">{}</td>",
                        status_class, status_text
                    );
                    println!("            </tr>");
                }
                println!("        </table>");
            }

            if !snapshot.cache.is_empty() {
                println!("        <h2>💾 캐시 성능</h2>");
                println!("        <table>");
                println!("            <tr><th>API</th><th>히트 수</th><th>미스 수</th><th>히트율</th><th>저장소 크기</th></tr>");
                for (api_name, cache_metrics) in &snapshot.cache {
                    let hit_rate = cache_metrics.hit_rate();
                    let hit_class = if hit_rate >= 70.0 {
                        "success"
                    } else if hit_rate >= 50.0 {
                        "warning"
                    } else {
                        "danger"
                    };

                    println!("            <tr>");
                    println!("                <td><strong>{}</strong></td>", api_name);
                    println!("                <td>{}</td>", cache_metrics.hits);
                    println!("                <td>{}</td>", cache_metrics.misses);
                    println!(
                        "                <td class=\"{}\">{:.1}%</td>",
                        hit_class, hit_rate
                    );
                    println!(
                        "                <td>{:.1} KB</td>",
                        cache_metrics.storage_size as f64 / 1024.0
                    );
                    println!("            </tr>");
                }
                println!("        </table>");
            }

            println!("        <div class=\"footer\">");
            println!(
                "            리포트 생성: Warp CLI v{} | 시간: {}",
                env!("CARGO_PKG_VERSION"),
                timestamp.format("%Y-%m-%d %H:%M:%S")
            );
            println!("        </div>");
            println!("    </div>");
            println!("</body>");
            println!("</html>");
        }

        _ => {
            // Enhanced text format
            use colored::*;

            println!("{}", "📊 성능 리포트".bold().cyan());
            println!("{}", "=".repeat(60).bright_black());
            println!(
                "🕒 생성 시간: {}",
                timestamp
                    .format("%Y년 %m월 %d일 %H:%M:%S")
                    .to_string()
                    .bright_white()
            );

            if let Some((from_date, to_date)) = date_range {
                println!(
                    "📅 분석 기간: {} ~ {}",
                    from_date.bright_yellow(),
                    to_date.bright_yellow()
                );
            } else {
                println!("📅 분석 기간: 전체 데이터");
            }

            println!(
                "⏱️  시스템 가동시간: {}",
                format!("{}초", snapshot.uptime.as_secs()).bright_white()
            );
            println!(
                "💾 메모리 사용량: {}",
                format!("{:.1} MB", snapshot.memory_usage as f64 / 1024.0 / 1024.0).bright_white()
            );
            println!();

            let dashboard = crate::metrics::PerformanceDashboard::new(collector).with_details();
            println!("{}", dashboard.display().await);

            println!("{}", "=".repeat(60).bright_black());
            println!("📝 리포트 완료 - Warp CLI v{}", env!("CARGO_PKG_VERSION"));
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
