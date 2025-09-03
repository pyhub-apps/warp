use clap_complete::Shell;
use rust_i18n::t;
use std::env;
use std::path::Path;

/// Detected shell information
#[derive(Debug, Clone)]
pub enum DetectedShell {
    /// Standard shells supported by clap_complete
    Supported(Shell),
    /// Windows Command Prompt (CMD)
    WindowsCmd,
    /// Unknown or unsupported shell
    Unknown(String),
}

impl DetectedShell {
    /// Get display name for the shell
    pub fn display_name(&self) -> &str {
        match self {
            DetectedShell::Supported(shell) => match shell {
                Shell::Bash => "Bash",
                Shell::Elvish => "Elvish",
                Shell::Fish => "Fish",
                Shell::PowerShell => "PowerShell",
                Shell::Zsh => "Zsh",
                _ => "Unknown",
            },
            DetectedShell::WindowsCmd => "Windows Command Prompt (CMD)",
            DetectedShell::Unknown(name) => name,
        }
    }
}

/// Detect the current shell
pub fn detect_current_shell() -> Option<DetectedShell> {
    // Windows platform detection
    #[cfg(windows)]
    {
        // Check COMSPEC first (usually points to cmd.exe)
        if let Ok(comspec) = env::var("COMSPEC") {
            let comspec_lower = comspec.to_lowercase();
            if comspec_lower.contains("cmd.exe") {
                // Check if we're actually in PowerShell by looking at parent process
                if is_running_in_powershell() {
                    return Some(DetectedShell::Supported(Shell::PowerShell));
                }
                return Some(DetectedShell::WindowsCmd);
            }
        }

        // Check for PowerShell specific variables
        if env::var("PSModulePath").is_ok() || env::var("POWERSHELL_DISTRIBUTION_CHANNEL").is_ok() {
            return Some(DetectedShell::Supported(Shell::PowerShell));
        }
    }

    // Unix-like platform detection
    #[cfg(unix)]
    {
        // First, try the SHELL environment variable
        if let Ok(shell_path) = env::var("SHELL") {
            if let Some(shell) = detect_from_path(&shell_path) {
                return Some(shell);
            }
        }

        // Try to detect from parent process (more reliable for subshells)
        if let Some(shell) = detect_from_parent_process() {
            return Some(shell);
        }
    }

    // Fallback: try to detect from any platform
    detect_from_environment()
}

/// Detect shell from a path string
fn detect_from_path(path: &str) -> Option<DetectedShell> {
    let path = Path::new(path);
    let shell_name = path.file_name()?.to_str()?;

    // Remove common extensions
    let shell_name = shell_name.trim_end_matches(".exe");

    match shell_name {
        "bash" | "sh" => Some(DetectedShell::Supported(Shell::Bash)),
        "zsh" => Some(DetectedShell::Supported(Shell::Zsh)),
        "fish" => Some(DetectedShell::Supported(Shell::Fish)),
        "pwsh" | "powershell" => Some(DetectedShell::Supported(Shell::PowerShell)),
        "elvish" => Some(DetectedShell::Supported(Shell::Elvish)),
        "cmd" => Some(DetectedShell::WindowsCmd),
        other => Some(DetectedShell::Unknown(other.to_string())),
    }
}

/// Detect shell from environment variables
fn detect_from_environment() -> Option<DetectedShell> {
    // Check for shell-specific environment variables
    if env::var("BASH_VERSION").is_ok() {
        return Some(DetectedShell::Supported(Shell::Bash));
    }

    if env::var("ZSH_VERSION").is_ok() || env::var("ZSH_NAME").is_ok() {
        return Some(DetectedShell::Supported(Shell::Zsh));
    }

    if env::var("FISH_VERSION").is_ok() {
        return Some(DetectedShell::Supported(Shell::Fish));
    }

    if env::var("PSModulePath").is_ok() {
        return Some(DetectedShell::Supported(Shell::PowerShell));
    }

    None
}

/// Check if running in PowerShell on Windows
#[cfg(windows)]
fn is_running_in_powershell() -> bool {
    // Check for PowerShell-specific environment variables
    env::var("PSModulePath").is_ok()
        || env::var("POWERSHELL_DISTRIBUTION_CHANNEL").is_ok()
        || env::var("PSVersionTable").is_ok()
}

/// Try to detect shell from parent process
#[cfg(unix)]
fn detect_from_parent_process() -> Option<DetectedShell> {
    use std::process::Command;

    // Try to get parent process ID
    let ppid = std::process::id();

    // Try using ps command to get parent process name
    let output = Command::new("ps")
        .args(["-p", &ppid.to_string(), "-o", "comm="])
        .output()
        .ok()?;

    if output.status.success() {
        let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return detect_from_path(&process_name);
    }

    None
}

#[cfg(not(unix))]
fn detect_from_parent_process() -> Option<DetectedShell> {
    None
}

/// Prompt user for confirmation
pub fn confirm_shell_selection(shell: &DetectedShell) -> bool {
    use std::io::{self, Write};

    println!(
        "🔍 {}: {}",
        t!("shell_completions.detected"),
        shell.display_name()
    );

    match shell {
        DetectedShell::WindowsCmd => {
            println!("⚠️  {}.", t!("shell_completions.cmd_warning"));
            println!("   {}.", t!("shell_completions.powershell_recommend"));
            println!();
            print!("{} (y/n): ", t!("shell_completions.generate_powershell"));
        }
        DetectedShell::Supported(_s) => {
            print!(
                "{}{} (y/n): ",
                shell.display_name(),
                t!("shell_completions.generate_for")
            );
        }
        DetectedShell::Unknown(name) => {
            println!("⚠️  {}: {}", t!("shell_completions.unknown_shell"), name);
            println!("   {}:", t!("shell_completions.select_manually"));
            println!("   warp completions bash");
            println!("   warp completions zsh");
            println!("   warp completions fish");
            println!("   warp completions powershell");
            return false;
        }
    }

    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Get shell to use for completion generation
pub fn get_completion_shell(detected: &DetectedShell) -> Option<Shell> {
    match detected {
        DetectedShell::Supported(shell) => Some(*shell),
        DetectedShell::WindowsCmd => {
            // Recommend PowerShell for CMD users
            Some(Shell::PowerShell)
        }
        DetectedShell::Unknown(_) => None,
    }
}
