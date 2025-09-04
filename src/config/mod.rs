use crate::error::{Result, WarpError};
use chrono::{Duration, Utc};
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR_NAME: &str = ".pyhub/warp";
const CONFIG_FILE_NAME: &str = "config.toml";
const LEGACY_CONFIG_FILE_NAME: &str = "config.yaml";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub law: LawConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,

    /// Filter presets
    #[serde(default)]
    pub filter_presets: HashMap<String, FilterPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LawConfig {
    /// Legacy API key (for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// NLIC configuration
    #[serde(default)]
    pub nlic: ApiConfig,

    /// ELIS configuration
    #[serde(default)]
    pub elis: ApiConfig,

    /// PREC configuration
    #[serde(default)]
    pub prec: ApiConfig,

    /// ADMRUL configuration
    #[serde(default)]
    pub admrul: ApiConfig,

    /// EXPC configuration
    #[serde(default)]
    pub expc: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    /// API-specific key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable/disable cache (default: true)
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// TTL in seconds (default: 86400 = 24 hours)
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,

    /// Maximum cache size in MB (default: 100)
    #[serde(default = "default_cache_max_size")]
    pub max_size_mb: u64,

    /// Cache directory path (default: user's cache directory + "warp")
    #[serde(default = "default_cache_dir", skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            ttl_seconds: default_cache_ttl(),
            max_size_mb: default_cache_max_size(),
            cache_dir: default_cache_dir(),
        }
    }
}

// Default value functions for cache configuration
fn default_cache_enabled() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    86400 // 24 hours in seconds
}

fn default_cache_max_size() -> u64 {
    100 // 100 MB
}

fn default_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|dir| dir.join("pyhub-warp"))
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable/disable metrics collection (default: true)
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,

    /// Maximum age of metrics data in days (default: 30)
    #[serde(default = "default_metrics_retention_days")]
    pub retention_days: u32,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            retention_days: default_metrics_retention_days(),
        }
    }
}

// Default value functions for metrics configuration
fn default_metrics_enabled() -> bool {
    true
}

fn default_metrics_retention_days() -> u32 {
    30
}

/// Filter preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    /// Preset name
    pub name: String,

    /// Search query
    pub query: Option<String>,

    /// Law type filter
    pub law_type: Option<String>,

    /// Department filter
    pub department: Option<String>,

    /// Status filter
    pub status: Option<String>,

    /// Region filter
    pub region: Option<String>,

    /// Date from
    pub from: Option<String>,

    /// Date to
    pub to: Option<String>,

    /// Recent days
    pub recent_days: Option<u32>,

    /// Enable regex
    #[serde(default)]
    pub regex: bool,

    /// Search only in title
    #[serde(default)]
    pub title_only: bool,

    /// Minimum score
    pub min_score: Option<f32>,

    /// Creation timestamp
    #[serde(default = "Utc::now")]
    pub created_at: chrono::DateTime<Utc>,
}

impl CacheConfig {
    /// Get the cache database file path
    pub fn get_cache_db_path(&self) -> Result<PathBuf> {
        let cache_dir = self
            .cache_dir
            .clone()
            .or_else(|| dirs::cache_dir().map(|dir| dir.join("pyhub-warp")))
            .ok_or_else(|| WarpError::Config("Could not determine cache directory".to_string()))?;

        // Ensure cache directory exists with secure permissions
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).map_err(|e| {
                WarpError::Config(format!("Failed to create cache directory: {}", e))
            })?;

            // Set directory permissions to 0700 on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = std::fs::Permissions::from_mode(0o700);
                std::fs::set_permissions(&cache_dir, permissions).map_err(|e| {
                    WarpError::Config(format!("Failed to set cache directory permissions: {}", e))
                })?;
            }
        }

        let db_path = cache_dir.join("cache.db");

        // Set secure permissions on database file if it exists
        if db_path.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = std::fs::Permissions::from_mode(0o600);
                std::fs::set_permissions(&db_path, permissions).map_err(|e| {
                    WarpError::Config(format!("Failed to set cache database permissions: {}", e))
                })?;
            }
        }

        Ok(db_path)
    }

    /// Convert to the cache module's CacheConfig
    pub fn to_cache_config(&self) -> crate::cache::CacheConfig {
        let db_path = self
            .get_cache_db_path()
            .unwrap_or_else(|_| PathBuf::from("cache.db"));

        crate::cache::CacheConfig {
            max_size: self.max_size_mb * 1024 * 1024, // Convert MB to bytes
            default_ttl: Duration::seconds(self.ttl_seconds as i64),
            db_path,
        }
    }

    /// Get cache TTL as chrono Duration
    pub fn get_ttl(&self) -> Duration {
        Duration::seconds(self.ttl_seconds as i64)
    }

    /// Get max size in bytes
    pub fn get_max_size_bytes(&self) -> u64 {
        self.max_size_mb * 1024 * 1024
    }
}

// Backward compatibility type aliases
pub type NlicConfig = ApiConfig;
pub type ElisConfig = ApiConfig;

impl Config {
    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| WarpError::Config("Could not determine home directory".to_string()))?;

        Ok(home_dir.join(CONFIG_DIR_NAME))
    }

    /// Get the full path to the configuration file (alias for config_file_path)
    pub fn get_config_path() -> Result<PathBuf> {
        Self::config_file_path()
    }

    /// Get the configuration file full path
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_path()?.join(CONFIG_FILE_NAME))
    }

    /// Get the legacy YAML configuration file path
    fn legacy_config_file_path() -> Result<PathBuf> {
        Ok(Self::config_path()?.join(LEGACY_CONFIG_FILE_NAME))
    }

    /// Migrate from YAML to TOML if legacy file exists
    fn migrate_yaml_to_toml() -> Result<()> {
        let legacy_file = Self::legacy_config_file_path()?;
        let new_file = Self::config_file_path()?;

        // Only migrate if YAML exists and TOML doesn't
        if legacy_file.exists() && !new_file.exists() {
            eprintln!("📦 Migrating configuration from YAML to TOML format...");

            // Read and parse YAML
            let yaml_contents = fs::read_to_string(&legacy_file)
                .map_err(|e| WarpError::Config(format!("Failed to read legacy config: {}", e)))?;

            let config: Self = serde_yaml::from_str(&yaml_contents)
                .map_err(|e| WarpError::Config(format!("Failed to parse legacy config: {}", e)))?;

            // Save as TOML
            let toml_str = toml::to_string_pretty(&config)
                .map_err(|e| WarpError::Config(format!("Failed to serialize to TOML: {}", e)))?;

            fs::write(&new_file, toml_str)
                .map_err(|e| WarpError::Config(format!("Failed to write TOML config: {}", e)))?;

            // Set permissions on new file
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = fs::Permissions::from_mode(0o600);
                fs::set_permissions(&new_file, permissions).map_err(|e| {
                    WarpError::Config(format!("Failed to set file permissions: {}", e))
                })?;
            }

            // Create backup of YAML file
            let backup_file = legacy_file.with_extension("yaml.backup");
            fs::rename(&legacy_file, &backup_file)
                .map_err(|e| WarpError::Config(format!("Failed to backup legacy config: {}", e)))?;

            eprintln!("✅ Configuration migrated successfully!");
            eprintln!("   Old YAML file backed up to: {}", backup_file.display());
        }

        Ok(())
    }

    /// Initialize configuration directory and file
    pub fn initialize() -> Result<()> {
        let config_dir = Self::config_path()?;

        // Create config directory with restricted permissions
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                WarpError::Config(format!("Failed to create config directory: {}", e))
            })?;

            // Set directory permissions to 0700 on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = fs::Permissions::from_mode(0o700);
                fs::set_permissions(&config_dir, permissions).map_err(|e| {
                    WarpError::Config(format!("Failed to set directory permissions: {}", e))
                })?;
            }
        }

        // Try to migrate from YAML to TOML if needed
        Self::migrate_yaml_to_toml()?;

        let config_file = Self::config_file_path()?;

        // Create default config file if it doesn't exist
        if !config_file.exists() {
            let default_config = Self::default();
            let toml_str = toml::to_string_pretty(&default_config)
                .map_err(|e| WarpError::Config(format!("Failed to serialize config: {}", e)))?;

            fs::write(&config_file, toml_str)
                .map_err(|e| WarpError::Config(format!("Failed to write config file: {}", e)))?;

            // Set file permissions to 0600 on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = fs::Permissions::from_mode(0o600);
                fs::set_permissions(&config_file, permissions).map_err(|e| {
                    WarpError::Config(format!("Failed to set file permissions: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        Self::initialize()?;

        let config_file = Self::config_file_path()?;
        let contents = fs::read_to_string(&config_file)
            .map_err(|e| WarpError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Self = toml::from_str(&contents)
            .map_err(|e| WarpError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        Self::initialize()?;

        let config_file = Self::config_file_path()?;
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| WarpError::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_file, toml_str)
            .map_err(|e| WarpError::Config(format!("Failed to write config file: {}", e)))?;

        // Set file permissions to 0600 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&config_file, permissions)
                .map_err(|e| WarpError::Config(format!("Failed to set file permissions: {}", e)))?;
        }

        Ok(())
    }

    /// Get NLIC API key (with backward compatibility)
    pub fn get_nlic_api_key(&self) -> Option<String> {
        self.law.nlic.key.clone().or_else(|| self.law.key.clone())
    }

    /// Get ELIS API key (with backward compatibility)
    pub fn get_elis_api_key(&self) -> Option<String> {
        self.law.elis.key.clone().or_else(|| self.law.key.clone())
    }

    /// Get PREC API key (with backward compatibility)
    pub fn get_prec_api_key(&self) -> Option<String> {
        self.law.prec.key.clone().or_else(|| self.law.key.clone())
    }

    /// Get ADMRUL API key (with backward compatibility)
    pub fn get_admrul_api_key(&self) -> Option<String> {
        self.law.admrul.key.clone().or_else(|| self.law.key.clone())
    }

    /// Get EXPC API key (with backward compatibility)
    pub fn get_expc_api_key(&self) -> Option<String> {
        self.law.expc.key.clone().or_else(|| self.law.key.clone())
    }

    /// Get API key for specific API type
    pub fn get_api_key(&self, api_type: &str) -> Option<String> {
        match api_type.to_lowercase().as_str() {
            "nlic" => self.get_nlic_api_key(),
            "elis" => self.get_elis_api_key(),
            "prec" => self.get_prec_api_key(),
            "admrul" => self.get_admrul_api_key(),
            "expc" => self.get_expc_api_key(),
            _ => self.law.key.clone(),
        }
    }

    /// Set a configuration value by key path
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "law.key" => {
                self.law.key = Some(value.to_string());
            }
            "law.nlic.key" => {
                self.law.nlic.key = Some(value.to_string());
            }
            "law.elis.key" => {
                self.law.elis.key = Some(value.to_string());
            }
            "law.prec.key" => {
                self.law.prec.key = Some(value.to_string());
            }
            "law.admrul.key" => {
                self.law.admrul.key = Some(value.to_string());
            }
            "law.expc.key" => {
                self.law.expc.key = Some(value.to_string());
            }
            "cache.enabled" => {
                self.cache.enabled = value
                    .parse::<bool>()
                    .map_err(|_| WarpError::Config(format!("Invalid boolean value: {}", value)))?;
            }
            "cache.ttl_seconds" => {
                self.cache.ttl_seconds = value.parse::<u64>().map_err(|_| {
                    WarpError::Config(format!("Invalid TTL seconds value: {}", value))
                })?;
            }
            "cache.max_size_mb" => {
                self.cache.max_size_mb = value.parse::<u64>().map_err(|_| {
                    WarpError::Config(format!("Invalid cache size value: {}", value))
                })?;
            }
            "cache.cache_dir" => {
                self.cache.cache_dir = Some(PathBuf::from(value));
            }
            _ => {
                return Err(WarpError::Config(format!(
                    "Unknown configuration key: {}",
                    key
                )));
            }
        }

        self.save()?;
        Ok(())
    }

    /// Get a configuration value by key path
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "law.key" => self.law.key.clone(),
            "law.nlic.key" => self.law.nlic.key.clone(),
            "law.elis.key" => self.law.elis.key.clone(),
            "law.prec.key" => self.law.prec.key.clone(),
            "law.admrul.key" => self.law.admrul.key.clone(),
            "law.expc.key" => self.law.expc.key.clone(),
            "cache.enabled" => Some(self.cache.enabled.to_string()),
            "cache.ttl_seconds" => Some(self.cache.ttl_seconds.to_string()),
            "cache.max_size_mb" => Some(self.cache.max_size_mb.to_string()),
            "cache.cache_dir" => self
                .cache
                .cache_dir
                .as_ref()
                .map(|p| p.display().to_string()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();

        assert!(config.enabled);
        assert_eq!(config.ttl_seconds, 86400);
        assert_eq!(config.max_size_mb, 100);
        // cache_dir should be None when using default_cache_dir function in TOML
        // but the Default implementation sets it to Some(path) for direct usage
        assert!(config.cache_dir.is_some());
    }

    #[test]
    fn test_cache_config_from_toml() {
        let toml_str = r#"
[law]
key = "test-key"

[cache]
enabled = false
ttl_seconds = 3600
max_size_mb = 50
"#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert!(!config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 3600);
        assert_eq!(config.cache.max_size_mb, 50);
    }

    #[test]
    fn test_cache_config_with_custom_dir() {
        let toml_str = r#"
[law]
key = "test-key"

[cache]
enabled = true
ttl_seconds = 7200
max_size_mb = 200
cache_dir = "/custom/cache/path"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert!(config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 7200);
        assert_eq!(config.cache.max_size_mb, 200);
        assert_eq!(
            config.cache.cache_dir,
            Some(PathBuf::from("/custom/cache/path"))
        );
    }

    #[test]
    fn test_cache_config_conversion() {
        let config_cache = CacheConfig {
            enabled: true,
            ttl_seconds: 3600,
            max_size_mb: 50,
            cache_dir: Some(PathBuf::from("/tmp/test")),
        };

        let cache_config = config_cache.to_cache_config();

        assert_eq!(cache_config.max_size, 50 * 1024 * 1024); // 50MB in bytes
        assert_eq!(cache_config.default_ttl.num_seconds(), 3600);
    }

    #[test]
    fn test_config_get_set_cache_values() {
        let mut config = Config::default();

        // Test setting cache values
        config.set("cache.enabled", "false").unwrap();
        config.set("cache.ttl_seconds", "7200").unwrap();
        config.set("cache.max_size_mb", "250").unwrap();

        assert!(!config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 7200);
        assert_eq!(config.cache.max_size_mb, 250);

        // Test getting cache values
        assert_eq!(config.get("cache.enabled"), Some("false".to_string()));
        assert_eq!(config.get("cache.ttl_seconds"), Some("7200".to_string()));
        assert_eq!(config.get("cache.max_size_mb"), Some("250".to_string()));
    }

    #[test]
    fn test_config_backward_compatibility() {
        // Test that configs without cache section still work with defaults
        let toml_str = r#"
[law]
key = "legacy-key"

[law.nlic]
key = "nlic-key"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();

        // Cache should use default values
        assert!(config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 86400);
        assert_eq!(config.cache.max_size_mb, 100);

        // Law config should still work
        assert_eq!(config.law.key, Some("legacy-key".to_string()));
        assert_eq!(config.law.nlic.key, Some("nlic-key".to_string()));
    }
}
