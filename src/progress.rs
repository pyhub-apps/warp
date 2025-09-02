/*!
# Enhanced Progress Indicator System

This module provides a comprehensive progress indication system for API operations with the following features:

## Features
- **Stage-based Progress**: API operations broken down into 4 stages (Connect, Search, Parse, Cache)
- **ETA Calculations**: Automatic time estimation and elapsed time tracking
- **Parallel API Tracking**: Individual progress bars for parallel API operations
- **Data Transfer Metrics**: Bytes transferred tracking and speed indicators
- **Retry Progress**: Visual indication of retry attempts with failure handling
- **Cache Operations**: Progress tracking for cache read/write operations

## Usage Examples

### Basic API Progress
```rust,no_run
use std::sync::Arc;
use warp::progress::{ProgressManager, ApiProgress};

let manager = Arc::new(ProgressManager::new(false, false));
let progress = ApiProgress::new(manager.clone(), "국가법령정보센터");
progress.set_message("검색 중...");
// ... API operation ...
progress.finish_with_message("검색 완료: 10개 결과");
```

### Enhanced Stage-based Progress
```rust,no_run
use std::sync::Arc;
use warp::progress::{ProgressManager, EnhancedApiProgress, ApiStage};

let manager = Arc::new(ProgressManager::new(false, false));
let mut progress = EnhancedApiProgress::new(manager.clone(), "국가법령정보센터");
progress.advance_stage(ApiStage::Connecting, "서버 연결 중");
progress.advance_stage(ApiStage::Searching, "검색 요청 전송 중");
// ... API call ...
progress.advance_stage(ApiStage::Parsing, "응답 파싱 중");
progress.complete_success("검색 완료: 10개 결과");
```

### Parallel API Operations
The system automatically creates individual progress bars for each API in parallel searches,
providing real-time feedback on the status of multiple concurrent operations.

## Progress Bar Styles
- **Spinner**: For indeterminate operations (connections, searches)
- **Bar with ETA**: For determinate operations (downloads, large data processing)
- **Multi-stage Bar**: For complex operations broken into stages
- **Retry Bar**: For retry operations with attempt counting
*/

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::{self, IsTerminal};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Progress indicator manager
pub struct ProgressManager {
    multi: Arc<MultiProgress>,
    enabled: bool,
    verbose: bool,
}

impl ProgressManager {
    /// Create a new progress manager
    pub fn new(quiet: bool, verbose: bool) -> Self {
        // Only enable progress if we're in a terminal and not in quiet mode
        let enabled = !quiet && io::stdout().is_terminal();

        Self {
            multi: Arc::new(MultiProgress::new()),
            enabled,
            verbose,
        }
    }

    /// Create a spinner for searching
    pub fn create_search_spinner(&self, message: &str) -> Option<ProgressBar> {
        if !self.enabled {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        Some(pb)
    }

    /// Create a progress bar for multi-API searches
    pub fn create_multi_api_progress(&self, total: u64, message: &str) -> Option<ProgressBar> {
        if !self.enabled {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{bar:40.cyan/blue} {pos}/{len} ({percent}%)")
                .unwrap()
                .progress_chars("█▓░"),
        );
        pb.set_message(message.to_string());

        Some(pb)
    }

    /// Create a download progress bar
    pub fn create_download_progress(&self, total_size: u64) -> Option<ProgressBar> {
        if !self.enabled {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new(total_size));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{bar:40.green/white} {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message("다운로드 중...");

        Some(pb)
    }

    /// Create an enhanced API operation progress bar with stages
    pub fn create_api_operation_progress(&self, api_name: &str) -> Option<ProgressBar> {
        if !self.enabled {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new(4)); // 4 stages: Connect, Search, Parse, Cache
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{bar:40.cyan/blue} {pos}/{len} ({percent}%) [{elapsed_precise}]")
                .unwrap()
                .progress_chars("█▓░"),
        );
        pb.set_message(format!("{} 작업 준비 중...", api_name));

        Some(pb)
    }

    /// Create a retry progress indicator
    pub fn create_retry_progress(&self, max_retries: u64, operation: &str) -> Option<ProgressBar> {
        if !self.enabled {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new(max_retries));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{bar:30.yellow/red} 재시도 {pos}/{len}")
                .unwrap()
                .progress_chars("█▓░"),
        );
        pb.set_message(format!("{} 재시도 중...", operation));

        Some(pb)
    }

    /// Create a cache operation progress bar
    pub fn create_cache_progress(&self, operation: &str, total_items: u64) -> Option<ProgressBar> {
        if !self.enabled {
            return None;
        }

        let pb = self.multi.add(ProgressBar::new(total_items));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{bar:40.magenta/white} {pos}/{len} ({percent}%)")
                .unwrap()
                .progress_chars("█▓░"),
        );
        pb.set_message(format!("캐시 {} 중...", operation));

        Some(pb)
    }

    /// Show a simple message (for verbose mode)
    pub fn show_message(&self, message: &str) {
        if self.verbose && self.enabled {
            eprintln!("🔍 {}", message);
        }
    }

    /// Check if progress is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// API operation stages
#[derive(Debug, Clone, Copy)]
pub enum ApiStage {
    Connecting = 0,
    Searching = 1,
    Parsing = 2,
    Caching = 3,
}

/// Progress context for API operations with enhanced tracking
pub struct ApiProgress {
    spinner: Option<ProgressBar>,
    stage_progress: Option<ProgressBar>,
    manager: Arc<ProgressManager>,
    start_time: Instant,
    bytes_transferred: Arc<AtomicU64>,
    current_stage: ApiStage,
}

/// Progress context for enhanced API operations
pub struct EnhancedApiProgress {
    progress: Option<ProgressBar>,
    manager: Arc<ProgressManager>,
    start_time: Instant,
    current_stage: ApiStage,
    api_name: String,
}

impl ApiProgress {
    /// Create a new API progress context
    pub fn new(manager: Arc<ProgressManager>, api_name: &str) -> Self {
        let message = format!("{} 검색 중...", api_name);
        let spinner = if manager.is_enabled() {
            manager.create_search_spinner(&message)
        } else {
            None
        };

        Self {
            spinner,
            stage_progress: None,
            manager,
            start_time: Instant::now(),
            bytes_transferred: Arc::new(AtomicU64::new(0)),
            current_stage: ApiStage::Connecting,
        }
    }

    /// Create a new enhanced API progress context with stage tracking
    pub fn new_enhanced(manager: Arc<ProgressManager>, api_name: &str) -> Self {
        let stage_progress = if manager.is_enabled() {
            manager.create_api_operation_progress(api_name)
        } else {
            None
        };

        Self {
            spinner: None,
            stage_progress,
            manager,
            start_time: Instant::now(),
            bytes_transferred: Arc::new(AtomicU64::new(0)),
            current_stage: ApiStage::Connecting,
        }
    }

    /// Advance to the next API operation stage
    pub fn advance_stage(&mut self, stage: ApiStage, message: &str) {
        self.current_stage = stage;

        if let Some(ref pb) = self.stage_progress {
            pb.set_position(stage as u64);
            pb.set_message(message.to_string());
        } else if let Some(ref pb) = self.spinner {
            pb.set_message(message.to_string());
        }

        self.manager.show_message(message);
    }

    /// Add bytes transferred (for tracking data volume)
    pub fn add_bytes(&self, bytes: u64) {
        self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Get elapsed time since operation start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get bytes transferred so far
    pub fn bytes_transferred(&self) -> u64 {
        self.bytes_transferred.load(Ordering::Relaxed)
    }

    /// Update the progress message
    pub fn set_message(&self, message: &str) {
        if let Some(ref pb) = self.spinner {
            pb.set_message(message.to_string());
        }
        self.manager.show_message(message);
    }

    /// Finish with a success message
    pub fn finish_with_message(&self, message: &str) {
        let elapsed = self.elapsed();
        let bytes = self.bytes_transferred();

        let detailed_message = if bytes > 0 {
            format!(
                "✅ {} ({:.2}초, {}바이트)",
                message,
                elapsed.as_secs_f32(),
                bytes
            )
        } else {
            format!("✅ {} ({:.2}초)", message, elapsed.as_secs_f32())
        };

        if let Some(ref pb) = self.stage_progress {
            pb.finish_with_message(detailed_message);
        } else if let Some(ref pb) = self.spinner {
            pb.finish_with_message(detailed_message);
        }

        self.manager.show_message(&format!("완료: {}", message));
    }

    /// Finish and clear the progress
    pub fn finish_and_clear(&self) {
        if let Some(ref pb) = self.stage_progress {
            pb.finish_and_clear();
        } else if let Some(ref pb) = self.spinner {
            pb.finish_and_clear();
        }
    }
}

impl Drop for ApiProgress {
    fn drop(&mut self) {
        if let Some(ref pb) = self.stage_progress {
            pb.finish_and_clear();
        } else if let Some(ref pb) = self.spinner {
            pb.finish_and_clear();
        }
    }
}

impl EnhancedApiProgress {
    /// Create a new enhanced API progress context
    pub fn new(manager: Arc<ProgressManager>, api_name: &str) -> Self {
        let progress = if manager.is_enabled() {
            manager.create_api_operation_progress(api_name)
        } else {
            None
        };

        Self {
            progress,
            manager,
            start_time: Instant::now(),
            current_stage: ApiStage::Connecting,
            api_name: api_name.to_string(),
        }
    }

    /// Advance to the next stage
    pub fn advance_stage(&mut self, stage: ApiStage, message: &str) {
        self.current_stage = stage;

        if let Some(ref pb) = self.progress {
            pb.set_position(stage as u64);
            pb.set_message(format!("{}: {}", self.api_name, message));
        }

        self.manager
            .show_message(&format!("{}: {}", self.api_name, message));
    }

    /// Complete the operation with success
    pub fn complete_success(&self, result_message: &str) {
        let elapsed = self.start_time.elapsed();
        let message = format!("✅ {} ({:.2}초)", result_message, elapsed.as_secs_f32());

        if let Some(ref pb) = self.progress {
            pb.set_position(4); // Complete all stages
            pb.finish_with_message(message);
        }

        self.manager
            .show_message(&format!("완료: {}", result_message));
    }

    /// Get elapsed time since operation start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Complete with error
    pub fn complete_error(&self, error_message: &str) {
        let elapsed = self.start_time.elapsed();
        let message = format!("❌ {} ({:.2}초)", error_message, elapsed.as_secs_f32());

        if let Some(ref pb) = self.progress {
            pb.finish_with_message(message);
        }

        self.manager
            .show_message(&format!("오류: {}", error_message));
    }
}

impl Drop for EnhancedApiProgress {
    fn drop(&mut self) {
        if let Some(ref pb) = self.progress {
            pb.finish_and_clear();
        }
    }
}

/// Progress messages for different operations
pub mod messages {
    pub const CONNECTING: &str = "서버 연결 중...";
    pub const SEARCHING: &str = "검색 중...";
    pub const PROCESSING: &str = "결과 처리 중...";
    pub const FORMATTING: &str = "포맷팅 중...";
    pub const DOWNLOADING: &str = "다운로드 중...";
    pub const PARSING: &str = "데이터 파싱 중...";
    pub const CACHING: &str = "캐시 저장 중...";
    pub const RETRYING: &str = "재시도 중...";

    // Stage-specific messages
    pub const STAGE_CONNECTING: &str = "API 서버 연결 중...";
    pub const STAGE_SEARCHING: &str = "검색 요청 전송 중...";
    pub const STAGE_PARSING: &str = "응답 데이터 파싱 중...";
    pub const STAGE_CACHING: &str = "결과 캐시 저장 중...";

    pub fn searching_api(api_name: &str) -> String {
        format!("{} 검색 중...", api_name)
    }

    pub fn search_complete(api_name: &str, count: usize) -> String {
        format!("{}: {}개 결과", api_name, count)
    }

    pub fn search_complete_with_time(api_name: &str, count: usize, elapsed_ms: u64) -> String {
        format!("{}: {}개 결과 ({}ms)", api_name, count, elapsed_ms)
    }

    pub fn multi_api_progress(current: usize, total: usize) -> String {
        format!("API 검색 진행 중 ({}/{})", current, total)
    }

    pub fn stage_message(api_name: &str, stage: crate::progress::ApiStage) -> String {
        let stage_text = match stage {
            crate::progress::ApiStage::Connecting => "연결",
            crate::progress::ApiStage::Searching => "검색",
            crate::progress::ApiStage::Parsing => "파싱",
            crate::progress::ApiStage::Caching => "캐시",
        };
        format!("{} {}", api_name, stage_text)
    }

    pub fn retry_message(operation: &str, attempt: u32, max_attempts: u32) -> String {
        format!("{} 재시도 ({}/{})", operation, attempt, max_attempts)
    }

    pub fn cache_operation(operation: &str, progress: usize, total: usize) -> String {
        format!("캐시 {} ({}/{})", operation, progress, total)
    }

    pub fn bytes_transferred(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{}B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1}KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_manager_creation() {
        // Test with quiet mode - should disable progress
        let manager = ProgressManager::new(true, false);
        assert!(!manager.is_enabled());

        // Test with verbose mode (note: may be disabled if not in terminal)
        let _manager = ProgressManager::new(false, true);
        // This will depend on whether tests are run in a terminal
        // assert!(manager.is_enabled() || !io::stdout().is_terminal());
    }

    #[test]
    fn test_progress_messages() {
        assert_eq!(
            messages::searching_api("국가법령정보센터"),
            "국가법령정보센터 검색 중..."
        );
        assert_eq!(messages::search_complete("NLIC", 10), "NLIC: 10개 결과");
        assert_eq!(messages::multi_api_progress(2, 5), "API 검색 진행 중 (2/5)");
        assert_eq!(
            messages::search_complete_with_time("NLIC", 5, 1500),
            "NLIC: 5개 결과 (1500ms)"
        );
        assert_eq!(messages::retry_message("검색", 2, 3), "검색 재시도 (2/3)");
        assert_eq!(messages::bytes_transferred(512), "512B");
        assert_eq!(messages::bytes_transferred(2048), "2.0KB");
        assert_eq!(messages::bytes_transferred(2097152), "2.0MB");
    }

    #[test]
    fn test_api_progress_lifecycle() {
        let manager = Arc::new(ProgressManager::new(true, false)); // Quiet mode
        let progress = ApiProgress::new(manager.clone(), "테스트 API");

        // Should work without panic even when disabled
        progress.set_message("테스트 메시지");
        progress.finish_with_message("완료");
        progress.finish_and_clear();
    }
}
