use super::{MetricsCollector, MetricsSnapshot, MetricsWindow};
use colored::*;
use std::fmt::Write as FmtWrite;
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

        // Header
        writeln!(&mut output, "{}", "📊 Performance Dashboard".bold().blue()).unwrap();
        writeln!(&mut output, "{}", "─".repeat(60)).unwrap();

        // System overview
        self.render_system_overview(&mut output, &snapshot);

        // API Operations
        self.render_operations_summary(&mut output, &snapshot);

        // Cache Performance
        self.render_cache_summary(&mut output, &snapshot);

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
        if let Some((op_name, metrics)) = snapshot
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
        writeln!(output, "🖥️  {}", "System Overview".bold()).unwrap();
        writeln!(output, "   Uptime: {}", format_duration(snapshot.uptime)).unwrap();
        writeln!(output, "   Memory: {}", format_bytes(snapshot.memory_usage)).unwrap();
        writeln!(output, "   Window: {:?}", self.window).unwrap();
        writeln!(output).unwrap();
    }

    fn render_operations_summary(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        writeln!(output, "🔄 {}", "API Operations".bold()).unwrap();

        if snapshot.operations.is_empty() {
            writeln!(output, "   No operations recorded").unwrap();
            writeln!(output).unwrap();
            return;
        }

        // Summary statistics
        let total_requests: u64 = snapshot.operations.values().map(|o| o.total_requests).sum();
        let total_successes: u64 = snapshot
            .operations
            .values()
            .map(|o| o.successful_requests)
            .sum();
        let avg_success_rate = if total_requests > 0 {
            (total_successes as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        writeln!(
            output,
            "   Total Requests: {}",
            total_requests.to_string().green()
        )
        .unwrap();
        writeln!(
            output,
            "   Success Rate: {}",
            format_percentage(avg_success_rate)
        )
        .unwrap();

        // Top operations
        let top_ops = self.collector.get_top_operations(3);
        if !top_ops.is_empty() {
            writeln!(output, "   Top Operations:").unwrap();
            for (name, metrics) in top_ops {
                let status_color = if metrics.error_rate() > 10.0 {
                    "red"
                } else {
                    "green"
                };
                writeln!(
                    output,
                    "     • {} ({} req, {}ms avg, {})",
                    name,
                    metrics.total_requests,
                    metrics.avg_duration.as_millis(),
                    format_percentage(metrics.success_rate()).color(status_color)
                )
                .unwrap();
            }
        }
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

    fn render_connection_pools(&self, output: &mut String, snapshot: &MetricsSnapshot) {
        writeln!(output, "🔗 {}", "Connection Pools".bold()).unwrap();

        if snapshot.connection_pools.is_empty() {
            writeln!(output, "   No connection pool data available").unwrap();
            writeln!(output).unwrap();
            return;
        }

        for (pool_name, pool_metrics) in &snapshot.connection_pools {
            let util_color = if pool_metrics.utilization() > 80.0 {
                "red"
            } else if pool_metrics.utilization() > 60.0 {
                "yellow"
            } else {
                "green"
            };

            writeln!(
                output,
                "   {}: {} utilization ({}/{} active)",
                pool_name,
                format_percentage(pool_metrics.utilization()).color(util_color),
                pool_metrics.active_connections,
                pool_metrics.total_connections
            )
            .unwrap();

            if pool_metrics.connection_timeouts > 0 {
                writeln!(
                    output,
                    "     Timeouts: {} ({:.1}%)",
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

    /// Start continuous monitoring mode
    pub async fn monitor(&self, interval: Duration) -> tokio::task::JoinHandle<()> {
        let collector = Arc::clone(&self.collector);
        let window = self.window;
        let show_details = self.show_details;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            loop {
                interval.tick().await;

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

        assert!(output.contains("Performance Dashboard"));
        assert!(output.contains("System Overview"));
        assert!(output.contains("API Operations"));
    }
}
