use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::{self, IsTerminal};
use std::sync::Arc;
use std::time::Duration;

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
                .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"])
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
                .progress_chars("█▓░")
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
                .template("{msg}\n{bar:40.green/white} {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("=>-")
        );
        pb.set_message("다운로드 중...");
        
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

/// Progress context for API operations
pub struct ApiProgress {
    spinner: Option<ProgressBar>,
    manager: Arc<ProgressManager>,
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
        
        Self { spinner, manager }
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
        if let Some(ref pb) = self.spinner {
            pb.finish_with_message(format!("✅ {}", message));
        }
        self.manager.show_message(&format!("완료: {}", message));
    }
    
    /// Finish and clear the progress
    pub fn finish_and_clear(&self) {
        if let Some(ref pb) = self.spinner {
            pb.finish_and_clear();
        }
    }
}

impl Drop for ApiProgress {
    fn drop(&mut self) {
        if let Some(ref pb) = self.spinner {
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
    
    pub fn searching_api(api_name: &str) -> String {
        format!("{} 검색 중...", api_name)
    }
    
    pub fn search_complete(api_name: &str, count: usize) -> String {
        format!("{}: {}개 결과", api_name, count)
    }
    
    pub fn multi_api_progress(current: usize, total: usize) -> String {
        format!("API 검색 진행 중 ({}/{})", current, total)
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
        let manager = ProgressManager::new(false, true);
        // This will depend on whether tests are run in a terminal
        // assert!(manager.is_enabled() || !io::stdout().is_terminal());
    }
    
    #[test]
    fn test_progress_messages() {
        assert_eq!(messages::searching_api("국가법령정보센터"), "국가법령정보센터 검색 중...");
        assert_eq!(messages::search_complete("NLIC", 10), "NLIC: 10개 결과");
        assert_eq!(messages::multi_api_progress(2, 5), "API 검색 진행 중 (2/5)");
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