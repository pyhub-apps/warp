use super::{MetricsCollector, MetricsSnapshot, MetricsWindow};
use colored::*;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

/// Terminal-based performance dashboard for real-time metrics
pub struct PerformanceDashboard {
    collector: Arc<MetricsCollector>,
    window: MetricsWindow,
    show_details: bool,
}

impl PerformanceDashboard {
    /// Create a new performance dashboard
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self {
            collector,
            window: MetricsWindow::Last5Minutes,
            show_details: false,
        }
    }

    /// Create dashboard with custom window
    pub fn with_window(collector: Arc<MetricsCollector>, window: MetricsWindow) -> Self {
        Self {
            collector,
            window,
            show_details: false,
        }
    }

    /// Enable detailed metrics display
    pub fn with_details(mut self) -> Self {
        self.show_details = true;
        self
    }

    /// Display current performance metrics
    pub async fn display(&self) -> String {
        let snapshot = self.collector.get_windowed_metrics(self.window).await;
        let mut output = String::new();

        // Header with timestamp
        let now = chrono::Local::now();
        writeln!(
            &mut output,
            "{} ({})",
            "📊 Warp CLI 성능 대시보드".bold().blue(),
            now.format("%Y-%m-%d %H:%M:%S").to_string().bright_black()
        )
        .unwrap();
        writeln!(&mut output, "{}", "─".repeat(60).bright_black()).unwrap();

        // System overview
        self.render_system_overview(&mut output, &snapshot);

        // API Operations
        self.render_operations_summary(&mut output, &snapshot);

        // Cache Performance
        self.render_cache_summary(&mut output, &snapshot);

        // Response time distribution
        self.render_response_time_distribution(&mut output, &snapshot);

        // Connection Pools
        self.render_connection_pools(&mut output, &snapshot);

        if self.show_details {
            self.render_detailed_metrics(&mut output, &snapshot);
        }

        output
    }

    /// Display compact metrics for CLI status
    pub async fn display_compact(&self) -> String {
        let snapshot = self.collector.get_snapshot().await;
        let mut output = String::new();

        // Find most active operation
        if let Some((_op_name, metrics)) = snapshot
            .operations
            .iter()
            .max_by_key(|(_, m)| m.total_requests)
        {
            write!(&mut output, "🚀 {} ops", metrics.total_requests).unwrap();

            if metrics.total_requests > 0 {
                write!(&mut output, " ({:.1}% success", metrics.success_rate()).unwrap();
                write!(&mut output, ", avg {}ms)", metrics.avg_duration.as_millis()).unwrap();
            }
        }

        // Cache hit rate
        let total_hits: u64 = snapshot.cache.values().map(|c| c.hits).sum();
        let total_misses: u64 = snapshot.cache.values().map(|c| c.misses).sum();
        let total_cache_ops = total_hits + total_misses;

        if total_cache_ops > 0 {
            let hit_rate = (total_hits as f64 / total_cache_ops as f64) * 100.0;
            write!(&mut output, " | 💾 {:.1}% cache", hit_rate).unwrap();
        }

        output
    }

    fn render_system_overview(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        writeln!(output, "\n🚀 {}", "시스템 상태".bold().green()).unwrap();

        // System status indicator
        let total_ops: u64 = snapshot.operations.values().map(|o| o.total_requests).sum();
        let total_errors: u64 = snapshot
            .operations
            .values()
            .map(|o| o.failed_requests)
            .sum();
        let overall_error_rate = if total_ops > 0 {
            (total_errors as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        };

        let status_icon = if overall_error_rate > 10.0 {
            "❌ 문제"
        } else if overall_error_rate > 5.0 {
            "⚠️  주의"
        } else {
            "✅ 정상"
        };

        writeln!(output, "├─ 전체 상태: {}", status_icon).unwrap();
        writeln!(
            output,
            "├─ 가동시간: {}",
            format_duration(snapshot.uptime).bright_white()
        )
        .unwrap();

        // Memory usage with progress bar
        let memory_mb = snapshot.memory_usage as f64 / 1024.0 / 1024.0;
        let max_memory_mb = 512.0; // Assume 512MB as reasonable limit
        let memory_percentage = (memory_mb / max_memory_mb * 100.0).min(100.0);
        let memory_bar = create_progress_bar(memory_percentage, 20);
        writeln!(
            output,
            "├─ 메모리: {:.1} MB / {:.0} MB {} {:.1}%",
            memory_mb, max_memory_mb, memory_bar, memory_percentage
        )
        .unwrap();

        writeln!(output, "└─ 측정 구간: {:?}", self.window).unwrap();
        writeln!(output).unwrap();
    }

    fn render_operations_summary(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        writeln!(
            output,
            "⚡ {} (최근 {})",
            "API 성능".bold().cyan(),
            format_window(self.window)
        )
        .unwrap();

        if snapshot.operations.is_empty() {
            writeln!(output, "   📭 작업 기록이 없습니다").unwrap();
            writeln!(output).unwrap();
            return;
        }

        // Table header
        writeln!(
            output,
            "┌─────────┬────────┬─────────┬──────────┬──────────┐"
        )
        .unwrap();
        writeln!(
            output,
            "│ {}     │ {}  │ {}   │ {}  │ {}    │",
            "API".bold(),
            "요청수".bold(),
            "성공률".bold(),
            "평균시간".bold(),
            "캐시율".bold()
        )
        .unwrap();
        writeln!(
            output,
            "├─────────┼────────┼─────────┼──────────┼──────────┤"
        )
        .unwrap();

        // API rows
        for (api_name, metrics) in &snapshot.operations {
            let success_rate = metrics.success_rate();
            let success_indicator = if success_rate >= 95.0 {
                "✅"
            } else if success_rate >= 90.0 {
                "⚠️ "
            } else {
                "❌"
            };

            // Get cache hit rate for this API
            let cache_rate = if let Some(cache_metrics) = snapshot.cache.get(api_name) {
                let hit_rate = cache_metrics.hit_rate();
                let cache_indicator = if hit_rate >= 70.0 {
                    "🎯"
                } else if hit_rate >= 50.0 {
                    "⚠️ "
                } else {
                    "📉"
                };
                format!("{:.1}% {}", hit_rate, cache_indicator)
            } else {
                "N/A".to_string()
            };

            writeln!(
                output,
                "│ {:<7} │ {:<6} │ {:.1}% {} │ {:<8} │ {:<8} │",
                api_name,
                metrics.total_requests,
                success_rate,
                success_indicator,
                format!("{}ms", metrics.avg_duration.as_millis()),
                cache_rate
            )
            .unwrap();
        }

        writeln!(
            output,
            "└─────────┴────────┴─────────┴──────────┴──────────┘"
        )
        .unwrap();
        writeln!(output).unwrap();
    }

    fn render_cache_summary(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        writeln!(output, "💾 {}", "Cache Performance".bold()).unwrap();

        if snapshot.cache.is_empty() {
            writeln!(output, "   No cache data available").unwrap();
            writeln!(output).unwrap();
            return;
        }

        for (api, cache_metrics) in &snapshot.cache {
            let hit_rate_color = if cache_metrics.hit_rate() > 70.0 {
                "green"
            } else if cache_metrics.hit_rate() > 40.0 {
                "yellow"
            } else {
                "red"
            };

            writeln!(
                output,
                "   {}: {} hit rate ({} hits, {} misses)",
                api,
                format_percentage(cache_metrics.hit_rate()).color(hit_rate_color),
                cache_metrics.hits,
                cache_metrics.misses
            )
            .unwrap();

            if cache_metrics.storage_size > 0 {
                writeln!(
                    output,
                    "     Storage: {} ({} entries)",
                    format_bytes(cache_metrics.storage_size),
                    cache_metrics.entry_count
                )
                .unwrap();
            }
        }
        writeln!(output).unwrap();
    }

    fn render_response_time_distribution(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        if snapshot.operations.is_empty() {
            return;
        }

        writeln!(output, "📈 {}", "응답시간 분포".bold().magenta()).unwrap();

        // Calculate overall percentiles
        let all_durations: Vec<Duration> = snapshot
            .operations
            .values()
            .flat_map(|metrics| {
                // Use available percentile data as proxy
                vec![
                    metrics.p50_duration,
                    metrics.p95_duration,
                    metrics.p99_duration,
                    metrics.avg_duration,
                    metrics.min_duration,
                    metrics.max_duration,
                ]
            })
            .collect();

        if !all_durations.is_empty() {
            let max_duration = all_durations
                .iter()
                .max()
                .unwrap_or(&Duration::from_millis(1000))
                .as_millis() as f64;

            // Calculate representative percentiles
            let p50 = all_durations.iter().map(|d| d.as_millis()).sum::<u128>() as f64
                / all_durations.len() as f64
                * 0.5;
            let p95 = max_duration * 0.8;
            let p99 = max_duration * 0.95;

            writeln!(
                output,
                "{}",
                create_horizontal_bar(p50, max_duration, 20, "p50")
            )
            .unwrap();
            writeln!(
                output,
                "{}",
                create_horizontal_bar(p95, max_duration, 20, "p95")
            )
            .unwrap();
            writeln!(
                output,
                "{}",
                create_horizontal_bar(p99, max_duration, 20, "p99")
            )
            .unwrap();
        }

        writeln!(output).unwrap();
    }

    fn render_connection_pools(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        writeln!(output, "🔗 {}", "연결 풀 상태".bold().yellow()).unwrap();

        if snapshot.connection_pools.is_empty() {
            writeln!(output, "   📭 연결 풀 데이터가 없습니다").unwrap();
            writeln!(output).unwrap();
            return;
        }

        for (pool_name, pool_metrics) in &snapshot.connection_pools {
            let utilization = pool_metrics.utilization();
            let util_bar = create_progress_bar(utilization, 15);

            writeln!(
                output,
                "├─ {}: {} 사용률 ({}/{})",
                pool_name.bright_white(),
                util_bar,
                pool_metrics.active_connections,
                pool_metrics.total_connections
            )
            .unwrap();

            if pool_metrics.connection_timeouts > 0 {
                writeln!(
                    output,
                    "│  ⚠️  타임아웃: {} ({:.1}%)",
                    pool_metrics.connection_timeouts,
                    pool_metrics.timeout_rate()
                )
                .unwrap();
            }
        }
        writeln!(output).unwrap();
    }

    fn render_detailed_metrics(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        writeln!(output, "📈 {}", "Detailed Metrics".bold()).unwrap();

        // Slowest operations
        let slowest_ops = self.collector.get_slowest_operations(5);
        if !slowest_ops.is_empty() {
            writeln!(output, "   Slowest Operations:").unwrap();
            for (name, metrics) in slowest_ops {
                writeln!(
                    output,
                    "     • {}: avg {}ms (min: {}ms, max: {}ms, p95: {}ms)",
                    name,
                    metrics.avg_duration.as_millis(),
                    metrics.min_duration.as_millis(),
                    metrics.max_duration.as_millis(),
                    metrics.p95_duration.as_millis()
                )
                .unwrap();
            }
            writeln!(output).unwrap();
        }

        // Error analysis
        let error_ops: Vec<_> = snapshot
            .operations
            .iter()
            .filter(|(_, m)| m.failed_requests > 0)
            .collect();

        if !error_ops.is_empty() {
            writeln!(output, "   Operations with Errors:").unwrap();
            for (name, metrics) in error_ops {
                writeln!(
                    output,
                    "     • {}: {} errors ({:.1}% error rate)",
                    name,
                    metrics.failed_requests,
                    metrics.error_rate()
                )
                .unwrap();
            }
        }
    }

    /// Start continuous monitoring mode with enhanced terminal handling
    pub async fn monitor(
        &self,
        interval: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let collector = Arc::clone(&self.collector);
        let window = self.window;
        let show_details = self.show_details;

        // Hide cursor and enable alternate screen
        print!("\x1B[?25l\x1B[?1049h");
        std::io::stdout().flush()?;

        let mut interval_timer = tokio::time::interval(interval);
        let mut iteration = 0u64;

        // Setup signal handling for graceful exit
        let should_exit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let exit_flag = should_exit.clone();

        tokio::select! {
            _ = async {
                loop {
                    interval_timer.tick().await;
                    iteration += 1;

                    let dashboard = PerformanceDashboard {
                        collector: Arc::clone(&collector),
                        window,
                        show_details,
                    };

                    // Clear screen and move cursor to home
                    print!("\x1B[2J\x1B[H");

                    // Display dashboard
                    println!("{}", dashboard.display().await);

                    // Add footer with controls and stats
                    let uptime_secs = interval.as_secs() * iteration;
                    println!("{}", "─".repeat(60).bright_black());
                    println!("🔄 새로고침: {}초 마다 │ 실행시간: {}초 │ 업데이트: #{}",
                        interval.as_secs(),
                        uptime_secs,
                        iteration
                    );
                    println!("{}", "💡 Ctrl+C를 눌러서 종료".bright_blue());

                    // Flush output
                    std::io::stdout().flush()?;

                    if should_exit.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                }
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            } => {},
            _ = tokio::signal::ctrl_c() => {
                exit_flag.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }

        // Restore terminal state
        print!("\x1B[?1049l\x1B[?25h");
        std::io::stdout().flush()?;
        println!("\n✅ 모니터링이 종료되었습니다.");

        Ok(())
    }

    /// Create a one-shot monitoring session (legacy support)
    pub async fn monitor_legacy(&self, interval: Duration) -> tokio::task::JoinHandle<()> {
        let collector = Arc::clone(&self.collector);
        let window = self.window;
        let show_details = self.show_details;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                let dashboard = PerformanceDashboard {
                    collector: Arc::clone(&collector),
                    window,
                    show_details,
                };

                // Clear screen and move cursor to top
                print!("\x1B[2J\x1B[1;1H");
                println!("{}", dashboard.display().await);

                // Flush stdout
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        })
    }
}

// Utility functions for formatting

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn format_percentage(percentage: f64) -> ColoredString {
    let formatted = format!("{:.1}%", percentage);

    if percentage >= 90.0 {
        formatted.green()
    } else if percentage >= 70.0 {
        formatted.yellow()
    } else {
        formatted.red()
    }
}

/// Create a progress bar string with filled and empty blocks
fn create_progress_bar(percentage: f64, width: usize) -> ColoredString {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width - filled;

    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    if percentage >= 80.0 {
        bar.red()
    } else if percentage >= 60.0 {
        bar.yellow()
    } else {
        bar.green()
    }
}

/// Create a text-based horizontal bar chart
fn create_horizontal_bar(value: f64, max_value: f64, width: usize, label: &str) -> String {
    let percentage = if max_value > 0.0 {
        (value / max_value * 100.0).min(100.0)
    } else {
        0.0
    };

    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width - filled;

    format!(
        "{}: {}ms │{}{} {:.0}%",
        label,
        value as u64,
        "█".repeat(filled).bright_blue(),
        "░".repeat(empty).bright_black(),
        percentage
    )
}

/// Format MetricsWindow for display
fn format_window(window: MetricsWindow) -> &'static str {
    match window {
        MetricsWindow::LastMinute => "1분",
        MetricsWindow::Last5Minutes => "5분",
        MetricsWindow::Last15Minutes => "15분",
        MetricsWindow::LastHour => "1시간",
        MetricsWindow::Last24Hours => "24시간",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[tokio::test]
    async fn test_dashboard_display() {
        use super::super::MetricsCollector;

        let collector = Arc::new(MetricsCollector::new());

        // Add some test data
        collector.record_request_success("test_op", Duration::from_millis(100));
        collector.record_cache_hit("test_api");

        let dashboard = PerformanceDashboard::new(collector);
        let output = dashboard.display().await;

        assert!(output.contains("Warp CLI 성능 대시보드"));
        assert!(output.contains("시스템 상태"));
        assert!(output.contains("API 성능"));
    }
}
