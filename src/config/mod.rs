use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::error::{Result, WarpError};

const CONFIG_DIR_NAME: &str = ".pyhub/warp";
const CONFIG_FILE_NAME: &str = "config.yaml";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub law: LawConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LawConfig {
    /// Legacy API key (for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    
    /// NLIC configuration
    pub nlic: NlicConfig,
    
    /// ELIS configuration
    pub elis: ElisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NlicConfig {
    /// NLIC API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElisConfig {
    /// ELIS API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl Config {
    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| WarpError::Config("Could not determine home directory".to_string()))?;
        
        Ok(home_dir.join(CONFIG_DIR_NAME))
    }
    
    /// Get the configuration file full path
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_path()?.join(CONFIG_FILE_NAME))
    }
    
    /// Initialize configuration directory and file
    pub fn initialize() -> Result<()> {
        let config_dir = Self::config_path()?;
        
        // Create config directory with restricted permissions
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| WarpError::Config(format!("Failed to create config directory: {}", e)))?;
            
            // Set directory permissions to 0700 on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = fs::Permissions::from_mode(0o700);
                fs::set_permissions(&config_dir, permissions)
                    .map_err(|e| WarpError::Config(format!("Failed to set directory permissions: {}", e)))?;
            }
        }
        
        let config_file = Self::config_file_path()?;
        
        // Create default config file if it doesn't exist
        if !config_file.exists() {
            let default_config = Self::default();
            let yaml = serde_yaml::to_string(&default_config)
                .map_err(|e| WarpError::Config(format!("Failed to serialize config: {}", e)))?;
            
            fs::write(&config_file, yaml)
                .map_err(|e| WarpError::Config(format!("Failed to write config file: {}", e)))?;
            
            // Set file permissions to 0600 on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = fs::Permissions::from_mode(0o600);
                fs::set_permissions(&config_file, permissions)
                    .map_err(|e| WarpError::Config(format!("Failed to set file permissions: {}", e)))?;
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
        
        let config: Self = serde_yaml::from_str(&contents)
            .map_err(|e| WarpError::Config(format!("Failed to parse config file: {}", e)))?;
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        Self::initialize()?;
        
        let config_file = Self::config_file_path()?;
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| WarpError::Config(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&config_file, yaml)
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
        self.law.nlic.key.clone()
            .or_else(|| self.law.key.clone())
    }
    
    /// Get ELIS API key (with backward compatibility)
    pub fn get_elis_api_key(&self) -> Option<String> {
        self.law.elis.key.clone()
            .or_else(|| self.law.key.clone())
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
            _ => {
                return Err(WarpError::Config(format!("Unknown configuration key: {}", key)));
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
            _ => None,
        }
    }
}